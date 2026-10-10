use yamantaka_domain::{Cancellation, Result};

pub struct ReaderCloseRequest {
    pub document: String,
    pub cancel: Cancellation,
}
pub struct ReaderCloseReply {
    pub document: String,
    pub cancel: Cancellation,
    pub result: Result<()>,
}
impl ReaderCloseRequest {
    /// After native admission, cancellation cannot undo a completed close.
    pub(crate) fn reply(self, result: Result<()>) -> ReaderCloseReply {
        ReaderCloseReply {
            document: self.document,
            cancel: self.cancel,
            result,
        }
    }
}
