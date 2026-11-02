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
pub(crate) fn cache<T>(
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
        if version > 2 {
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
        if version < 2 {
            let transaction = connection.transaction().map_err(cache_error)?;
            transaction.execute_batch("CREATE TABLE index_sources(document_id TEXT PRIMARY KEY,source_key BLOB NOT NULL CHECK(length(source_key)=32),digest BLOB NOT NULL CHECK(length(digest)=32),revision TEXT NOT NULL) WITHOUT ROWID; PRAGMA user_version=2;").map_err(cache_error)?;
            transaction.commit().map_err(cache_error)?;
        }
        *cached = Some(connection);
    }
    operation(cached.as_mut().unwrap())
}
pub(crate) fn source_key(file: &CataloguedFile) -> [u8; 32] {
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
pub(crate) fn read_file(connection: &Connection, document: &str) -> Result<CataloguedFile> {
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
    let validated = chrono::Utc::now().to_rfc3339();
    match &result {
        FileCheckResult::Missing => {
            transaction
                .execute(
                    "UPDATE documents SET is_missing=1,last_validated_at=?1 WHERE id=?2",
                    params![validated, expected.document],
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
            save_reference(
                &transaction,
                cached,
                &expected.document,
                &expected.file,
                &file,
                digest,
            )?;
            transaction
                .execute(
                    "UPDATE documents SET is_missing=0,is_modified=0,file_modified_at=?1,last_validated_at=?2 WHERE id=?3",
                    params![modified, validated, expected.document],
                )
                .map_err(storage)?;
        }
        FileCheckResult::Changed => {
            transaction
                .execute(
                    "UPDATE documents SET is_modified=1,last_validated_at=?1 WHERE id=?2",
                    params![validated, expected.document],
                )
                .map_err(storage)?;
        }
        FileCheckResult::Unverifiable => {}
    }
    cancel.check()?;
    transaction.commit().map_err(storage)?;
    Ok(result)
}

fn save_reference(
    main: &Connection,
    cached: &RefCell<Option<Connection>>,
    document: &str,
    previous: &CataloguedFile,
    current: &CataloguedFile,
    digest: &[u8; 32],
) -> Result<()> {
    let key = source_key(current);
    let previous = source_key(previous);
    cache(main, cached, |store| {
        let update = store.transaction().map_err(cache_error)?;
        update.execute("INSERT INTO file_hashes VALUES(?1,?2,?3) ON CONFLICT(document_id,source_key) DO UPDATE SET digest=excluded.digest",params![document,key.as_slice(),digest.as_slice()]).map_err(cache_error)?;
        // Preserve the old key if the independent main transaction fails.
        update
            .execute(
                "DELETE FROM file_hashes WHERE document_id=?1 AND source_key NOT IN (?2,?3)",
                params![document, key.as_slice(), previous.as_slice()],
            )
            .map_err(cache_error)?;
        update.commit().map_err(cache_error)?;
        Ok(())
    })
}
pub(crate) fn relocate(
    database: &mut Database,
    plan: &yamantaka_domain::relocation::RelocationPlan,
    cancel: &yamantaka_domain::Cancellation,
) -> Result<()> {
    cancel.check()?;
    let expected = &plan.expected;
    let candidate = &plan.candidate;
    if !expected.file.missing
        || candidate.missing
        || candidate.size != expected.file.size
        || candidate.page_count != expected.file.page_count
        || candidate.fingerprint != expected.file.fingerprint
        || expected.digest.is_some_and(|digest| digest != plan.digest)
        || !candidate.path.is_absolute()
        || candidate.path.to_str().is_none()
    {
        return Err(Failure::Invalid("Invalid relocation candidate".into()));
    }
    chrono::DateTime::parse_from_rfc3339(&candidate.modified)
        .map_err(|_| Failure::Invalid("Invalid relocation date".into()))?;
    let cached = &database.file_checks;
    let tx = database
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    if snapshot(&tx, cached, &expected.document)? != *expected {
        return Err(Failure::Invalid(
            "The source record changed; review again".into(),
        ));
    }
    let normalized = candidate.path.to_string_lossy().to_lowercase();
    let legacy = if let Some(unc) = normalized.strip_prefix(r"\\?\unc\") {
        format!(r"\\{unc}")
    } else {
        normalized
            .strip_prefix(r"\\?\")
            .unwrap_or(&normalized)
            .to_owned()
    };
    let occupied:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM documents WHERE id<>?1 AND (normalized_path IN (?2,?3) OR lower(path) IN (?2,?3)))",params![expected.document,normalized,legacy],|row|row.get(0)).map_err(storage)?;
    if occupied {
        return Err(Failure::DestinationOccupied);
    }
    save_reference(
        &tx,
        cached,
        &expected.document,
        &expected.file,
        candidate,
        &plan.digest,
    )?;
    let filename = candidate
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Failure::Invalid("Invalid file name".into()))?;
    tx.execute("UPDATE documents SET path=?1,normalized_path=?2,file_name=?3,file_modified_at=?4,is_missing=0,is_modified=0,last_validated_at=?5 WHERE id=?6",params![candidate.path.to_string_lossy(),normalized,filename,candidate.modified,chrono::Utc::now().to_rfc3339(),expected.document]).map_err(storage)?;
    crate::search::source_relocated(&tx, &expected.document)?;
    cancel.check()?;
    tx.commit().map_err(storage)?;
    Ok(())
}

pub(crate) fn save_import_reference(
    main: &Connection,
    cached: &RefCell<Option<Connection>>,
    id: &str,
    document: &yamantaka_domain::NewDocument,
    digest: &[u8; 32],
) -> Result<()> {
    let file = CataloguedFile {
        path: document.source.canonical_path.clone(),
        fingerprint: yamantaka_domain::source_revision::legacy_fingerprint(
            document.source.size,
            document.source.head_sha256,
        ),
        page_count: u32::try_from(document.info.pages.len()).map_err(storage)?,
        missing: false,
        size: document.source.size,
        modified: chrono::DateTime::<chrono::Utc>::from(document.source.modified).to_rfc3339(),
    };
    save_reference(main, cached, id, &file, &file, digest)
}
pub(crate) fn remove_import_reference(database: &Database, id: &str) -> Result<()> {
    let mut cached = database.file_checks.borrow_mut();
    if let Some(cached) = cached.as_mut() {
        cached
            .execute("DELETE FROM file_hashes WHERE document_id=?", [id])
            .map_err(cache_error)?;
        cached
            .execute("DELETE FROM index_sources WHERE document_id=?", [id])
            .map_err(cache_error)?;
    }
    Ok(())
}
