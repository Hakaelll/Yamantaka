//! Nonblocking, one-consumer PDF replies. Native handles never leave their owner.
use crate::{Cancellation, PdfEngine, PdfInfo, Priority, RenderKey, RenderedPage, Result};
use crossbeam_channel::{Receiver, Select, Sender, TryRecvError, bounded};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct PdfTicket<T> {
    receiver: Receiver<Result<T>>,
    pub consumer_id: u64,
    pub epoch: u64,
    pub cancellation: Cancellation,
    lifetime: Mutex<Option<Arc<dyn Send + Sync>>>,
}
impl<T> PdfTicket<T> {
    pub fn channel(
        consumer_id: u64,
        epoch: u64,
        cancellation: Cancellation,
    ) -> (Sender<Result<T>>, Self) {
        let (sender, receiver) = bounded(1);
        (
            sender,
            Self {
                receiver,
                consumer_id,
                epoch,
                cancellation,
                lifetime: Mutex::new(None),
            },
        )
    }
    pub fn retain(mut self, lifetime: Arc<dyn Send + Sync>) -> Self {
        *self.lifetime.get_mut().unwrap() = Some(lifetime);
        self
    }
    pub fn try_recv(&self) -> std::result::Result<Result<T>, TryRecvError> {
        let result = self.receiver.try_recv();
        if result.is_ok() {
            self.lifetime.lock().unwrap().take();
        }
        result
    }
    pub fn wait(self) -> Result<T> {
        self.receiver
            .recv()
            .unwrap_or_else(|_| Err(crate::Failure::Pdf("PDF worker stopped".into())))
    }
    /// Register readiness in a coordinator's blocking select; this never waits.
    pub fn readiness(&self) -> &Receiver<Result<T>> {
        &self.receiver
    }
    /// Disconnection is ready as well: coordinators must publish worker failure,
    /// rather than spin forever on an empty but closed response channel.
    pub fn is_ready(&self) -> bool {
        let mut select = Select::new();
        select.recv(&self.receiver);
        select.try_ready().is_ok()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfOpenPurpose {
    Reader,
    Background,
}

pub trait AsyncPdfEngine: PdfEngine {
    fn cover_cache_handle(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::cover_cache::CoverImageCache>> {
        None
    }

    /// Reconsider cancelled requests that are waiting for native capacity.
    /// Implementations with no dormant native queue need no additional wakeup.
    fn wake_cancelled(&self) {}
    fn memory_governor(&self) -> Option<crate::memory::MemoryGovernor> {
        None
    }
    fn try_source_revision(
        &self,
        _id: String,
        _cancel: Cancellation,
    ) -> Result<PdfTicket<crate::source_revision::SourceRevision>> {
        Err(crate::Failure::Unsupported("PDF source revision".into()))
    }
    /// Capture while the importer retains its source read guard; credentials never escape the native owner.
    fn try_capture_import_cover(
        &self,
        _id: String,
        _source: crate::source_revision::SourceRevision,
        _cancel: Cancellation,
    ) -> Result<PdfTicket<crate::cover_policy::DetectedCover>> {
        Err(crate::Failure::Unsupported(
            "Unlocked import cover capture".into(),
        ))
    }
    /// Render a chosen cover page while the caller holds its source read guard.
    fn try_capture_import_cover_page(
        &self,
        _id: String,
        _source: crate::source_revision::SourceRevision,
        _page: u32,
        _cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        Err(crate::Failure::Unsupported(
            "Guarded selected cover capture".into(),
        ))
    }
    fn try_detect_cover(
        &self,
        _id: String,
        _path: PathBuf,
        _generation: u64,
        _cancel: Cancellation,
    ) -> Result<PdfTicket<crate::cover_policy::DetectedCover>> {
        Err(crate::Failure::Unsupported(
            "Automatic PDF cover selection".into(),
        ))
    }
    fn try_cover(
        &self,
        _id: String,
        _path: PathBuf,
        _page: u32,
        _generation: u64,
        _cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        Err(crate::Failure::Unsupported("PDF cover".into()))
    }
    /// Atomically replace the visible reader set (at most two distinct IDs).
    /// This only changes scheduler metadata: it never waits for native work.
    /// Protected handles must not be evicted by background opens.
    fn set_visible_documents(&self, ids: &[String]) -> Result<()>;
    fn try_open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<crate::PdfPassword>,
        purpose: PdfOpenPurpose,
        cancel: Cancellation,
    ) -> Result<PdfTicket<PdfInfo>>;
    fn try_render(
        &self,
        key: RenderKey,
        priority: Priority,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>>;
    fn try_text(&self, id: String, page: u32, cancel: Cancellation) -> Result<PdfTicket<String>>;
    fn try_outline(
        &self,
        _id: String,
        _cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<crate::outline::PdfOutline>>> {
        Err(crate::Failure::Unsupported("PDF outline".into()))
    }
    fn try_close(&self, id: String) -> Result<PdfTicket<()>>;
    fn try_text_geometry(
        &self,
        _id: String,
        _page: u32,
        _purpose: crate::pdf_text::TextGeometryPurpose,
        _cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<crate::pdf_text::PdfPageText>>> {
        Err(crate::Failure::Unsupported("PDF text geometry".into()))
    }
    fn try_page_links(
        &self,
        _id: String,
        _page: u32,
        _cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<crate::pdf_links::PdfPageLinks>>> {
        Err(crate::Failure::Unsupported("PDF page links".into()))
    }
    fn try_find_page(
        &self,
        _id: String,
        _page: u32,
        _query: crate::pdf_find::PdfFindQuery,
        _offset: u32,
        _cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<crate::pdf_find::PdfPageMatches>>> {
        Err(crate::Failure::Unsupported("PDF page search".into()))
    }
}
