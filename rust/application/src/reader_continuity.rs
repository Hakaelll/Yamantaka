use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository, Result, reader_tabs::DocumentTabs,
};

pub enum ContinuityChange {
    Load,
    Save {
        revision: u64,
        tabs: Box<DocumentTabs>,
    },
}
pub struct ContinuityRequest {
    pub change: ContinuityChange,
    pub cancel: Cancellation,
}
pub struct ContinuityReply {
    pub revision: Option<u64>,
    pub result: Result<Option<DocumentTabs>>,
    pub cancel: Cancellation,
}
#[derive(Default)]
pub(crate) struct ContinuityWriter {
    revision: Option<u64>,
}
impl ContinuityWriter {
    pub(crate) fn save(
        &mut self,
        repository: &mut impl LibraryRepository,
        revision: u64,
        tabs: &DocumentTabs,
    ) -> Result<()> {
        if self.revision.is_some_and(|previous| revision <= previous) {
            return Err(Failure::Cancelled);
        }
        repository.save_reader_continuity(tabs)?;
        self.revision = Some(revision);
        Ok(())
    }
    pub(crate) fn execute(
        &mut self,
        repository: &mut impl LibraryRepository,
        request: ContinuityRequest,
    ) -> ContinuityReply {
        let revision = match &request.change {
            ContinuityChange::Load => None,
            ContinuityChange::Save { revision, .. } => Some(*revision),
        };
        let result = match request.change {
            ContinuityChange::Load => (|| {
                request.cancel.check()?;
                let tabs = repository.reader_continuity()?;
                request.cancel.check()?;
                Ok(tabs)
            })(),
            ContinuityChange::Save { revision, tabs } => {
                self.save(repository, revision, &tabs).map(|_| None)
            }
        };
        ContinuityReply {
            revision,
            result,
            cancel: request.cancel,
        }
    }
}
