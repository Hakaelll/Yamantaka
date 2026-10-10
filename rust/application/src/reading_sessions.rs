use yamantaka_domain::reading_sessions::{
    ReaderSessionContext, ReadingSessionFinish, ReadingSessionRepository, ReadingSessionStart,
};
use yamantaka_domain::{Cancellation, Result};

pub enum ReadingSessionChange {
    Start {
        started_at: String,
        page: u32,
    },
    Finish {
        ended_at: String,
        page: u32,
        active_seconds: u64,
        context: Option<ReaderSessionContext>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadingSessionOperation {
    Start,
    Finish,
}

pub struct ReadingSessionRequest {
    pub id: String,
    pub document: String,
    pub change: ReadingSessionChange,
    pub cancel: Cancellation,
}

pub struct ReadingSessionReply {
    pub id: String,
    pub document: String,
    pub operation: ReadingSessionOperation,
    pub result: Result<()>,
    pub cancel: Cancellation,
}

impl ReadingSessionRequest {
    pub(crate) fn execute(
        self,
        repository: &mut impl ReadingSessionRepository,
    ) -> ReadingSessionReply {
        let operation = match &self.change {
            ReadingSessionChange::Start { .. } => ReadingSessionOperation::Start,
            ReadingSessionChange::Finish { .. } => ReadingSessionOperation::Finish,
        };
        let result = self.cancel.check().and_then(|()| match &self.change {
            ReadingSessionChange::Start { started_at, page } => repository.start_reading_session(
                &ReadingSessionStart {
                    id: &self.id,
                    document: &self.document,
                    started_at,
                    page: *page,
                },
                &self.cancel,
            ),
            ReadingSessionChange::Finish {
                ended_at,
                page,
                active_seconds,
                context,
            } => repository.finish_reading_session(
                &ReadingSessionFinish {
                    id: &self.id,
                    document: &self.document,
                    ended_at,
                    page: *page,
                    active_seconds: *active_seconds,
                    context: context.as_ref(),
                },
                &self.cancel,
            ),
        });
        ReadingSessionReply {
            id: self.id,
            document: self.document,
            operation,
            result,
            cancel: self.cancel,
        }
    }
}
