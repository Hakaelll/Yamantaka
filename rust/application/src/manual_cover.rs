use yamantaka_domain::{Cancellation, LibraryRepository, Result, cover_selection::ManualCover};

pub struct ManualCoverRequest {
    pub choice: ManualCover,
    pub cancel: Cancellation,
}
pub struct ManualCoverReply {
    pub document: String,
    pub result: Result<()>,
    pub cancel: Cancellation,
}
impl ManualCoverRequest {
    pub fn execute(self, repository: &mut impl LibraryRepository) -> ManualCoverReply {
        let result = self
            .cancel
            .check()
            .and_then(|_| repository.save_manual_cover(&self.choice));
        // An accepted atomic write keeps its real outcome even after presentation cancellation.
        ManualCoverReply {
            document: self.choice.document().into(),
            result,
            cancel: self.cancel,
        }
    }
}
