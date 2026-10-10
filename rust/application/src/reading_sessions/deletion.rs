use yamantaka_domain::{Cancellation, Result, reading_sessions::ReadingSessionRepository};

pub struct ReadingSessionDeleteRequest {
    pub id: String,
    pub document: String,
    pub cancel: Cancellation,
}
pub struct ReadingSessionDeleteReply {
    pub id: String,
    pub document: String,
    pub cancel: Cancellation,
    pub result: Result<bool>,
}
impl ReadingSessionDeleteRequest {
    pub(crate) fn execute(
        self,
        repository: &mut impl ReadingSessionRepository,
    ) -> ReadingSessionDeleteReply {
        let result = self.cancel.check().and_then(|()| {
            repository.delete_reading_session(&self.id, &self.document, &self.cancel)
        });
        ReadingSessionDeleteReply {
            id: self.id,
            document: self.document,
            cancel: self.cancel,
            result,
        }
    }
}
