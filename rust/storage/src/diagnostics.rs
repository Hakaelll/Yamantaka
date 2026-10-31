use crate::{Database, database::storage};
pub use rusqlite::InterruptHandle;
use yamantaka_domain::{Cancellation, Failure, Result, diagnostics::*};

impl DiagnosticsRepository for Database {
    fn diagnostics(&self, cancel: &Cancellation) -> Result<DatabaseDiagnostics> {
        cancel.check()?;
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let count = |sql: &str| -> Result<u64> {
            cancel.check()?;
            u64::try_from(
                tx.query_row(sql, [], |row| row.get::<_, i64>(0))
                    .map_err(storage)?,
            )
            .map_err(storage)
        };
        let counts = LibraryCounts {
            documents: count("SELECT count(*) FROM documents")?,
            missing_documents: count("SELECT count(*) FROM documents WHERE is_missing=1")?,
            annotations: count("SELECT count(*) FROM annotations")?,
            library_notes: count("SELECT count(*) FROM document_notes")?,
            collections: count("SELECT count(*) FROM collections")?,
            tags: count("SELECT count(*) FROM tags")?,
            workspaces: count("SELECT count(*) FROM workspaces")?,
            index_state_records: count("SELECT count(*) FROM search_index_state")?,
        };
        let logical_database_bytes = count("PRAGMA page_count")?
            .checked_mul(count("PRAGMA page_size")?)
            .ok_or(Failure::MemoryBudget)?;
        let schema = u32::try_from(count("PRAGMA user_version")?).map_err(storage)?;
        cancel.check()?;
        let check: String = tx
            .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
            .map_err(storage)?;
        cancel.check()?;
        let foreign_keys_ok = tx
            .prepare("PRAGMA foreign_key_check")
            .map_err(storage)?
            .query([])
            .map_err(storage)?
            .next()
            .map_err(storage)?
            .is_none();
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(DatabaseDiagnostics {
            schema,
            logical_database_bytes,
            counts,
            quick_check_ok: check == "ok",
            foreign_keys_ok,
        })
    }
}
impl Database {
    pub fn diagnostics_interrupt(&self) -> rusqlite::InterruptHandle {
        self.connection.get_interrupt_handle()
    }
}
