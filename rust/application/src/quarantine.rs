use yamantaka_domain::{Cancellation, Document, Result, quarantine::*};
pub enum Operation {
    Read,
    Health {
        document: Box<Document>,
        health: ReaderHealth,
    },
    Release(QuarantineRevision),
}
pub struct Request {
    pub operation: Operation,
    pub offset: usize,
    pub cancel: Cancellation,
}
pub struct Reply {
    pub health: bool,
    pub change: Option<(String, bool)>,
    pub result: Result<Option<QuarantinePage>>,
    pub cancel: Cancellation,
}
impl Request {
    pub(crate) fn execute(self, repository: &mut impl QuarantineRepository) -> Reply {
        let health = matches!(self.operation, Operation::Health { .. });
        let change = match &self.operation {
            Operation::Read => None,
            Operation::Release(revision) => Some((revision.document.clone(), false)),
            Operation::Health { document, health } => Some((
                document.id.clone(),
                matches!(health, ReaderHealth::Failed(_)),
            )),
        };
        let result = self.cancel.check().and_then(|()| match self.operation {
            Operation::Read => repository
                .quarantined_documents(self.offset, &self.cancel)
                .map(Some),
            Operation::Release(revision) => repository
                .release_quarantine(&revision, self.offset, &self.cancel)
                .map(Some),
            Operation::Health { document, health } => repository
                .record_reader_health(&document, health, &self.cancel)
                .map(|()| None),
        });
        Reply {
            health,
            change,
            result,
            cancel: self.cancel,
        }
    }
}
