use crate::{Cancellation, Result};
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub struct LibraryCounts {
    pub documents: u64,
    pub missing_documents: u64,
    pub annotations: u64,
    pub library_notes: u64,
    pub collections: u64,
    pub tags: u64,
    pub workspaces: u64,
    pub index_state_records: u64,
}
#[derive(Clone, Copy, Serialize)]
pub struct DatabaseDiagnostics {
    pub schema: u32,
    pub logical_database_bytes: u64,
    pub counts: LibraryCounts,
    pub quick_check_ok: bool,
    pub foreign_keys_ok: bool,
}
pub trait DiagnosticsRepository {
    fn diagnostics(&self, cancel: &Cancellation) -> Result<DatabaseDiagnostics>;
}
