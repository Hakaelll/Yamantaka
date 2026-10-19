use crate::{Database, database::storage};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, path::PathBuf};
use yamantaka_domain::{
    Failure, Result,
    file_check::{FileCheckResult, FileSnapshot},
    source_revision::CataloguedFile,
};
fn cache_error(_: impl std::fmt::Display) -> Failure {
    Failure::Storage("File verification cache could not be read or saved".into())
}
fn cache<T>(
    main: &Connection,
    cached: &RefCell<Option<Connection>>,
    operation: impl FnOnce(&mut Connection) -> Result<T>,
) -> Result<T> {
    let mut cached = cached.borrow_mut();
    if cached.is_none() {
        let mut connection = match main
            .path()
            .filter(|path| !path.is_empty() && *path != ":memory:")
        {
            Some(path) => {
                let mut name = PathBuf::from(path).into_os_string();
                name.push(".file-hashes.sqlite3");
                Connection::open(PathBuf::from(name)).map_err(cache_error)?
            }
            None => Connection::open_in_memory().map_err(cache_error)?,
        };
        let version: u32 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(cache_error)?;
        if version > 1 {
            return Err(Failure::Invalid(
                "Unsupported file verification cache version; file preserved".into(),
            ));
        }
        if version == 0 {
            let tables: u32 = connection
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table'",
                    [],
                    |row| row.get(0),
                )
                .map_err(cache_error)?;
            if tables != 0 {
                return Err(Failure::Invalid(
                    "Unrecognized file verification cache; file preserved".into(),
                ));
            }
        }
        crate::database::configure(&connection).map_err(cache_error)?;
        if version == 0 {
            let transaction = connection.transaction().map_err(cache_error)?;
            transaction.execute_batch("CREATE TABLE file_hashes(document_id TEXT NOT NULL,source_key BLOB NOT NULL CHECK(length(source_key)=32),digest BLOB NOT NULL CHECK(length(digest)=32),PRIMARY KEY(document_id,source_key)) WITHOUT ROWID; PRAGMA user_version=1;").map_err(cache_error)?;
            transaction.commit().map_err(cache_error)?;
        }
        *cached = Some(connection);
    }
    operation(cached.as_mut().unwrap())
}
fn source_key(file: &CataloguedFile) -> [u8; 32] {
    Sha256::digest(
        serde_json::json!([
            file.path,
            file.fingerprint,
            file.page_count,
            file.size,
            file.modified
        ])
        .to_string()
        .as_bytes(),
    )
    .into()
}
fn read_file(connection: &Connection, document: &str) -> Result<CataloguedFile> {
    let encoded:Option<Option<String>>=connection.query_row("SELECT CASE WHEN length(CAST(record AS BLOB))<=262144 THEN record END FROM (SELECT json_object('path',path,'fingerprint',fingerprint,'pages',page_count,'size',file_size,'modified',file_modified_at,'missing',is_missing) AS record FROM documents WHERE id=?)",[document],|row|row.get(0)).optional().map_err(storage)?;
    let encoded = encoded
        .ok_or_else(|| Failure::Invalid("File document no longer exists".into()))?
        .ok_or_else(|| Failure::Invalid("File record exceeds supported bounds".into()))?;
    let value: serde_json::Value = serde_json::from_str(&encoded).map_err(storage)?;
    let text = |name: &str| {
        value[name]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| Failure::Invalid("Invalid file record".into()))
    };
    Ok(CataloguedFile {
        path: text("path")?.into(),
        fingerprint: text("fingerprint")?,
        page_count: u32::try_from(
            value["pages"]
                .as_u64()
                .ok_or_else(|| Failure::Invalid("Invalid page count".into()))?,
        )
        .map_err(storage)?,
        size: value["size"]
            .as_u64()
            .ok_or_else(|| Failure::Invalid("Invalid file size".into()))?,
        modified: text("modified")?,
        missing: value["missing"].as_i64() != Some(0),
    })
}
fn snapshot(
    main: &Connection,
    cached: &RefCell<Option<Connection>>,
    document: &str,
) -> Result<FileSnapshot> {
    let file = read_file(main, document)?;
    let key = source_key(&file);
    let digest = cache(main, cached, |store| {
        let bytes:Option<Option<Vec<u8>>>=store.query_row("SELECT CASE WHEN length(digest)=32 THEN digest END FROM file_hashes WHERE document_id=?1 AND source_key=?2",params![document,key.as_slice()],|row|row.get(0)).optional().map_err(cache_error)?;
        Ok(bytes.flatten().and_then(|bytes| bytes.try_into().ok()))
    })?;
    Ok(FileSnapshot {
        document: document.into(),
        file,
        digest,
    })
}
pub(crate) fn read(database: &Database, document: &str) -> Result<FileSnapshot> {
    snapshot(&database.connection, &database.file_checks, document)
}
pub(crate) fn apply(
    database: &mut Database,
    expected: &FileSnapshot,
    result: FileCheckResult,
    cancel: &yamantaka_domain::Cancellation,
) -> Result<FileCheckResult> {
    cancel.check()?;
    let cached = &database.file_checks;
    let transaction = database
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    if snapshot(&transaction, cached, &expected.document)? != *expected {
        return Err(Failure::Invalid(
            "File record changed; refresh and verify again".into(),
        ));
    }
    match &result {
        FileCheckResult::Missing => {
            transaction
                .execute(
                    "UPDATE documents SET is_missing=1 WHERE id=?",
                    [&expected.document],
                )
                .map_err(storage)?;
        }
        FileCheckResult::Available {
            digest, modified, ..
        } => {
            chrono::DateTime::parse_from_rfc3339(modified)
                .map_err(|_| Failure::Invalid("Invalid verified file date".into()))?;
            let mut file = expected.file.clone();
            file.modified = modified.clone();
            file.missing = false;
            let key = source_key(&file);
            // Keep current and previous keys across a failed main commit; prune older versions.
            cache(&transaction, cached, |store| {
                let previous = source_key(&expected.file);
                let update = store.transaction().map_err(cache_error)?;
                update.execute("INSERT INTO file_hashes VALUES(?1,?2,?3) ON CONFLICT(document_id,source_key) DO UPDATE SET digest=excluded.digest",params![expected.document,key.as_slice(),digest.as_slice()]).map_err(cache_error)?;
                update.execute("DELETE FROM file_hashes WHERE document_id=?1 AND source_key NOT IN (?2,?3)",params![expected.document,key.as_slice(),previous.as_slice()]).map_err(cache_error)?;
                update.commit().map_err(cache_error)?;
                Ok(())
            })?;
            transaction
                .execute(
                    "UPDATE documents SET is_missing=0,file_modified_at=?1 WHERE id=?2",
                    params![modified, expected.document],
                )
                .map_err(storage)?;
        }
        FileCheckResult::Changed | FileCheckResult::Unverifiable => {}
    }
    cancel.check()?;
    transaction.commit().map_err(storage)?;
    Ok(result)
}
