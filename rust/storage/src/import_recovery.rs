//! Pending operations belong to a separate versioned SQLite store, not schema v11.
use crate::Database;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;
use yamantaka_domain::{
    Failure, LibraryRepository, Result,
    import_recovery::{ImportSource, PendingImport},
};

fn error(_: impl std::fmt::Display) -> Failure {
    Failure::Storage("Import recovery store could not be read or saved".into())
}
fn store<T>(
    database: &Database,
    operation: impl FnOnce(&mut Connection) -> Result<T>,
) -> Result<T> {
    let mut cached = database.import_journal.borrow_mut();
    if cached.is_none() {
        let mut connection = match database
            .connection
            .path()
            .filter(|p| !p.is_empty() && *p != ":memory:")
        {
            Some(path) => {
                let mut sidecar = Path::new(path).as_os_str().to_owned();
                sidecar.push(".imports.sqlite3");
                Connection::open(std::path::PathBuf::from(sidecar)).map_err(error)?
            }
            None => Connection::open_in_memory().map_err(error)?,
        };
        let version: u32 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(error)?;
        if version > 2 {
            return Err(Failure::Invalid(
                "Unsupported import recovery version; store preserved".into(),
            ));
        }
        if version == 0 {
            let tables: u32 = connection
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table'",
                    [],
                    |row| row.get(0),
                )
                .map_err(error)?;
            if tables != 0 {
                return Err(Failure::Invalid(
                    "Unrecognized import recovery store; file preserved".into(),
                ));
            }
        }
        crate::database::configure(&connection).map_err(error)?;
        if version == 0 {
            let transaction = connection.transaction().map_err(error)?;
            transaction.execute_batch("CREATE TABLE import_jobs(id TEXT PRIMARY KEY,source TEXT NOT NULL,label TEXT NOT NULL,created_at TEXT NOT NULL); CREATE TABLE import_completed(job_id TEXT NOT NULL REFERENCES import_jobs(id) ON DELETE CASCADE,path TEXT NOT NULL,document_id TEXT NOT NULL,fingerprint TEXT NOT NULL,pages INTEGER NOT NULL,PRIMARY KEY(job_id,path)); PRAGMA user_version=1;").map_err(error)?;
            transaction.commit().map_err(error)?;
        }
        if version <= 1 {
            let transaction = connection.transaction().map_err(error)?;
            transaction.execute_batch("ALTER TABLE import_completed ADD COLUMN newly_imported INTEGER CHECK(newly_imported IS NULL OR newly_imported IN (0,1)); PRAGMA user_version=2;").map_err(error)?;
            transaction.commit().map_err(error)?;
        }
        *cached = Some(connection);
    }
    operation(cached.as_mut().unwrap())
}
pub(crate) fn key(path: &Path) -> String {
    let value = path.to_string_lossy().replace('/', "\\").to_lowercase();
    if let Some(rest) = value.strip_prefix("\\\\?\\unc\\") {
        format!("\\\\{rest}")
    } else {
        value.strip_prefix("\\\\?\\").unwrap_or(&value).to_owned()
    }
}
pub(crate) fn pending(database: &Database) -> Result<Vec<PendingImport>> {
    store(database, |store| {
        let (count,ids,labels):(u32,Option<u32>,Option<u32>) = store.query_row(
            "SELECT count(*),max(length(CAST(id AS BLOB))),max(length(CAST(label AS BLOB))) FROM import_jobs",[],
            |row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).map_err(error)?;
        if count > 80 || ids.unwrap_or(0) > 128 || labels.unwrap_or(0) > 512 {
            return Err(Failure::Invalid(
                "Import recovery records exceed supported bounds; store preserved".into(),
            ));
        }
        let mut statement = store.prepare("SELECT j.id,j.label,count(c.path) FROM import_jobs j LEFT JOIN import_completed c ON c.job_id=j.id GROUP BY j.id ORDER BY j.created_at,j.id LIMIT 80").map_err(error)?;
        statement
            .query_map([], |row| {
                Ok(PendingImport {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    completed: row.get::<_, u32>(2)?.into(),
                })
            })
            .map_err(error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(error)
    })
}
pub(crate) fn queue(database: &Database, source: &ImportSource) -> Result<String> {
    source.validate()?;
    let serialized = serde_json::to_string(source).map_err(error)?;
    if serialized.len() > 10 * 1024 * 1024 {
        return Err(Failure::Invalid(
            "Import recovery source is too large".into(),
        ));
    }
    store(database, |store| {
        let transaction = store.transaction().map_err(error)?;
        let count: u32 = transaction
            .query_row("SELECT count(*) FROM import_jobs", [], |row| row.get(0))
            .map_err(error)?;
        if count >= 80 {
            return Err(Failure::QueueFull);
        }
        let id = uuid::Uuid::new_v4().to_string();
        transaction
            .execute(
                "INSERT INTO import_jobs VALUES(?1,?2,?3,?4)",
                params![
                    id,
                    serialized,
                    source.label(),
                    chrono::Utc::now().to_rfc3339()
                ],
            )
            .map_err(error)?;
        transaction.commit().map_err(error)?;
        Ok(id)
    })
}
pub(crate) fn resume(database: &Database, id: &str) -> Result<ImportSource> {
    store(database, |store| {
        let size: u32 = store
            .query_row(
                "SELECT length(CAST(source AS BLOB)) FROM import_jobs WHERE id=?",
                [id],
                |row| row.get(0),
            )
            .optional()
            .map_err(error)?
            .ok_or_else(|| Failure::Invalid("Pending import no longer exists".into()))?;
        if size > 10 * 1024 * 1024 {
            return Err(Failure::Invalid(
                "Import recovery source is too large; store preserved".into(),
            ));
        }
        let value: String = store
            .query_row("SELECT source FROM import_jobs WHERE id=?", [id], |row| {
                row.get(0)
            })
            .map_err(error)?;
        let source: ImportSource = serde_json::from_str(&value).map_err(error)?;
        source.validate()?;
        Ok(source)
    })
}
pub(crate) fn committed(
    database: &Database,
    job: &str,
    path: &Path,
) -> Result<Option<yamantaka_domain::import_recovery::ImportCheckpoint>> {
    store(database, |store| {
        let row = store.query_row("SELECT CASE WHEN length(CAST(document_id AS BLOB))<=1024 THEN document_id END,CASE WHEN length(CAST(fingerprint AS BLOB))<=256 THEN fingerprint END,pages,newly_imported FROM import_completed WHERE job_id=?1 AND path=?2",params![job,key(path)],|row|Ok((row.get::<_,Option<String>>(0)?,row.get::<_,Option<String>>(1)?,row.get::<_,u32>(2)?,row.get::<_,Option<bool>>(3)?))).optional().map_err(error)?;
        let Some((id, fingerprint, pages, imported)) = row else {
            let Some(id) = database.registered_import_path(path)? else {
                return Ok(None);
            };
            let Some(document) = database.get(&id)? else {
                return Ok(None);
            };
            let owned:bool=database.connection.query_row("SELECT EXISTS(SELECT 1 FROM operation_journal WHERE id=?1 AND operation_kind='rustImportMember.v1' AND compatibility_hash='rust-import-member-v1' AND status='completed' AND json_extract(CASE WHEN json_valid(summary_json) THEN summary_json ELSE '{}' END,'$.version')=1 AND json_extract(summary_json,'$.job')=?2 AND json_extract(summary_json,'$.document')=?3 AND json_extract(summary_json,'$.path')=?4 AND json_extract(summary_json,'$.fingerprint')=?5 AND json_extract(summary_json,'$.pages')=?6)",params![format!("rust-import-member-{id}"),job,id,key(path),document.fingerprint,document.page_count],|row|row.get(0)).map_err(error)?;
            return Ok(
                owned.then_some(yamantaka_domain::import_recovery::ImportCheckpoint {
                    document: id,
                    origin: yamantaka_domain::import_recovery::CheckpointOrigin::Imported,
                }),
            );
        };
        let (Some(id), Some(fingerprint)) = (id, fingerprint) else {
            return Err(Failure::Invalid(
                "Import recovery checkpoint exceeds supported bounds; store preserved".into(),
            ));
        };
        let valid = database.get(&id)?.is_some_and(|document| {
            !document.missing
                && key(&document.path) == key(path)
                && document.fingerprint == fingerprint
                && document.page_count == pages
        });
        use yamantaka_domain::import_recovery::{CheckpointOrigin, ImportCheckpoint};
        Ok(valid.then_some(ImportCheckpoint {
            document: id,
            origin: match imported {
                Some(true) => CheckpointOrigin::Imported,
                Some(false) => CheckpointOrigin::Registered,
                None => CheckpointOrigin::Unknown,
            },
        }))
    })
}
pub(crate) fn checkpoint(
    database: &Database,
    job: &str,
    path: &Path,
    document: &str,
    newly_imported: bool,
) -> Result<()> {
    let document = database
        .get(document)?
        .ok_or_else(|| Failure::Invalid("Imported document no longer exists".into()))?;
    if document.missing
        || document.id.len() > 1024
        || document.fingerprint.len() > 256
        || key(&document.path) != key(path)
    {
        return Err(Failure::Invalid(
            "Imported source changed before its recovery checkpoint".into(),
        ));
    }
    store(database, |store| {
        store.execute("INSERT INTO import_completed(job_id,path,document_id,fingerprint,pages,newly_imported) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(job_id,path) DO UPDATE SET document_id=excluded.document_id,fingerprint=excluded.fingerprint,pages=excluded.pages,newly_imported=excluded.newly_imported",params![job,key(path),document.id,document.fingerprint,document.page_count,newly_imported]).map_err(error)?;
        Ok(())
    })
}
pub(crate) fn remove(database: &Database, id: &str) -> Result<()> {
    store(database, |store| {
        store
            .execute("DELETE FROM import_jobs WHERE id=?", [id])
            .map_err(error)?;
        Ok(())
    })
}
