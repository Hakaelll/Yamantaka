use crate::{Cancellation, Document, Failure, Result};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuarantineReason {
    PdfOpenFailed,
    InheritedOrUnknown,
}
impl QuarantineReason {
    pub fn from_failure(error: &Failure) -> Option<Self> {
        matches!(error, Failure::Pdf(_)).then_some(Self::PdfOpenFailed)
    }
}
#[derive(Clone)]
pub struct QuarantineRevision {
    pub document: String,
    pub state: [u8; 32],
}
pub struct QuarantinedDocument {
    pub document: Document,
    pub reason: QuarantineReason,
    pub revision: QuarantineRevision,
}
pub struct QuarantinePage {
    pub entries: Vec<QuarantinedDocument>,
    pub total: u64,
    pub offset: usize,
}
pub enum ReaderHealth {
    Opened,
    Failed(QuarantineReason),
}
pub trait QuarantineRepository {
    fn quarantined_documents(&self, offset: usize, cancel: &Cancellation)
    -> Result<QuarantinePage>;
    fn automatic_restore_allowed(&self, document: &str, cancel: &Cancellation) -> Result<bool>;
    fn record_reader_health(
        &mut self,
        document: &Document,
        health: ReaderHealth,
        cancel: &Cancellation,
    ) -> Result<()>;
    fn release_quarantine(
        &mut self,
        revision: &QuarantineRevision,
        offset: usize,
        cancel: &Cancellation,
    ) -> Result<QuarantinePage>;
}
