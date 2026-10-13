use crate::{Cancellation, RenderedPage, Result, memory::MemoryGovernor};
use std::{path::Path, sync::Arc};

pub struct CoverCacheRequest<'a> {
    pub identity: [u8; 32],
    pub document: &'a str,
    pub page: u32,
    pub generation: u64,
}

/// Recreatable pixel storage. It never owns or modifies the source PDF.
pub trait CoverImageCache: Send + Sync {
    fn identity(&self, path: &Path, page: u32, cancel: &Cancellation) -> Result<[u8; 32]>;
    fn load(
        &self,
        request: CoverCacheRequest<'_>,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Option<Arc<RenderedPage>>>;
    fn save(&self, identity: [u8; 32], image: &RenderedPage, cancel: &Cancellation) -> Result<()>;
}
