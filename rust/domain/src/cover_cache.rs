use crate::{Cancellation, RenderedPage, Result, memory::MemoryGovernor};
use std::{path::Path, sync::Arc};

pub struct CoverCacheRequest<'a> {
    pub identity: [u8; 32],
    pub document: &'a str,
    pub page: u32,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverCacheOperation {
    Inspect,
    Clear,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CoverCacheInventory {
    pub files: u64,
    pub bytes: u64,
    pub preserved: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CoverCacheReport {
    pub inventory: Option<CoverCacheInventory>,
    pub removed_files: u64,
    pub removed_bytes: u64,
    pub failed_files: u64,
    pub cancelled: bool,
}

/// Recreatable pixel storage. It never owns or modifies the source PDF.
pub trait CoverImageCache: Send + Sync {
    fn maintain(
        &self,
        _operation: CoverCacheOperation,
        _cancel: &Cancellation,
    ) -> Result<CoverCacheReport> {
        Err(crate::Failure::Unsupported(
            "Cover cache maintenance".into(),
        ))
    }

    fn identity(&self, path: &Path, page: u32, cancel: &Cancellation) -> Result<[u8; 32]>;
    fn load(
        &self,
        request: CoverCacheRequest<'_>,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Option<Arc<RenderedPage>>>;
    fn save(&self, identity: [u8; 32], image: &RenderedPage, cancel: &Cancellation) -> Result<()>;
}
