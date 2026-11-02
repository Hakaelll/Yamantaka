pub(crate) mod aliases;
pub(crate) mod merge;
use crate::{Database, database::storage};
use std::sync::Arc;
use yamantaka_domain::{
    Cancellation, Failure, MAX_LIBRARY_PAGE, Result,
    duplicates::*,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
};

pub(crate) fn factory(database: &Database) -> Result<ReaderFactory> {
    let path = database.file_path().ok_or_else(|| {
        Failure::Unsupported("Duplicate discovery requires a file-backed library".into())
    })?;
    Ok(Box::new(move || {
        let database = Database::open_read_only(&path)?;
        database
            .connection
            .execute_batch("BEGIN DEFERRED")
            .map_err(storage)?;
        Ok(Box::new(database) as Box<dyn Reader>)
    }))
}
impl Reader for Database {
    fn page(
        &self,
        after: Option<&str>,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Vec<Arc<Source>>> {
        cancel.check()?;
        // The inherited fingerprint only narrows candidates; the worker hashes full files.
        let mut statement = self.connection.prepare("SELECT d.id,COALESCE(d.title_override,d.title,d.source_title,d.file_name) FROM documents d WHERE (?1 IS NULL OR d.id>?1) AND EXISTS(SELECT 1 FROM documents other WHERE other.fingerprint=d.fingerprint AND other.file_size=d.file_size AND other.id<>d.id) ORDER BY d.id LIMIT ?2").map_err(storage)?;
        let mut rows = statement
            .query(rusqlite::params![after, MAX_LIBRARY_PAGE as i64])
            .map_err(storage)?;
        let mut output = Vec::new();
        while let Some(row) = rows.next().map_err(storage)? {
            cancel.check()?;
            let id = row.get_ref(0).map_err(storage)?.as_str().map_err(storage)?;
            let title = row.get_ref(1).map_err(storage)?.as_str().map_err(storage)?;
            if id.len() > 262144 || title.len() > 262144 {
                return Err(Failure::Invalid(
                    "Duplicate candidate exceeds supported bounds".into(),
                ));
            }
            let transient = governor
                .try_reserve_one(Cost::new(
                    MemoryCategory::Other,
                    MemoryRepresentation::Cpu,
                    2 * 1024 * 1024,
                ))
                .map_err(|_| Failure::MemoryBudget)?;
            let file = crate::file_check::read_file(&self.connection, id)?;
            let bytes = id.len()
                + title.len()
                + file.path.as_os_str().as_encoded_bytes().len()
                + file.fingerprint.len()
                + file.modified.len();
            let lease = governor
                .try_reserve_one(Cost::new(
                    MemoryCategory::Other,
                    MemoryRepresentation::Cpu,
                    bytes * 2 + 512,
                ))
                .map_err(|_| Failure::MemoryBudget)?;
            drop(transient);
            output.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
            output.push(Arc::new(Source::new(id.into(), title.into(), file, lease)));
        }
        Ok(output)
    }
}
