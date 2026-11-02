use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository, Result, reader_continuity::ReaderContinuity,
};

pub enum ContinuityChange {
    Load,
    Save {
        revision: u64,
        tabs: Box<ReaderContinuity>,
    },
}
pub struct ContinuityRequest {
    pub change: ContinuityChange,
    pub cancel: Cancellation,
}
pub struct ContinuityReply {
    pub revision: Option<u64>,
    pub result: Result<Option<ReaderContinuity>>,
    pub cancel: Cancellation,
    pub reference_document: Option<yamantaka_domain::Document>,
    pub metadata_document: Option<yamantaka_domain::Document>,
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
        tabs: &ReaderContinuity,
    ) -> Result<()> {
        if self.revision.is_some_and(|previous| revision <= previous) {
            return Err(Failure::Cancelled);
        }
        let tabs = retain_catalogued(repository, tabs.clone())?;
        repository.save_reader_continuity(&tabs)?;
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
        let mut result = match request.change {
            ContinuityChange::Load => (|| {
                request.cancel.check()?;
                let tabs = repository
                    .reader_continuity()?
                    .map(|tabs| retain_catalogued(repository, tabs))
                    .transpose()?;
                request.cancel.check()?;
                Ok(tabs)
            })(),
            ContinuityChange::Save { revision, tabs } => {
                self.save(repository, revision, &tabs).map(|_| None)
            }
        };
        let reference_document = result
            .as_ref()
            .ok()
            .and_then(|saved| saved.as_ref())
            .and_then(|saved| saved.comparison())
            .map(|saved| repository.get(&saved.document))
            .transpose();
        let reference_document = match reference_document {
            Ok(document) => document.flatten(),
            Err(error) => {
                result = Err(error);
                None
            }
        };
        let metadata_document = result
            .as_ref()
            .ok()
            .and_then(|saved| saved.as_ref())
            .and_then(|saved| saved.metadata_drafts().last())
            .map(|draft| repository.get(&draft.document))
            .transpose();
        let metadata_document = match metadata_document {
            Ok(document) => document.flatten(),
            Err(error) => {
                result = Err(error);
                None
            }
        };
        ContinuityReply {
            revision,
            result,
            cancel: request.cancel,
            reference_document,
            metadata_document,
        }
    }
}

fn retain_catalogued(
    repository: &impl LibraryRepository,
    mut tabs: ReaderContinuity,
) -> Result<ReaderContinuity> {
    let mut ids = std::collections::BTreeSet::new();
    for tab in tabs.tabs().tabs().iter().chain(tabs.tabs().closed()) {
        ids.insert(tab.document().to_owned());
    }
    let tab_ids = ids.clone();
    if let Some(comparison) = tabs.comparison() {
        ids.insert(comparison.primary.clone());
        ids.insert(comparison.document.clone());
    }
    for draft in tabs.page_notes() {
        ids.insert(draft.document.clone());
    }
    for draft in tabs.metadata_drafts() {
        ids.insert(draft.document.clone());
    }
    for draft in tabs.annotation_edits() {
        ids.insert(draft.document.clone());
    }
    for draft in tabs.selection_notes() {
        ids.insert(draft.document.clone());
    }
    let mut absent = Vec::new();
    for id in ids {
        match repository.canonical_document_id(&id)? {
            Some(canonical) if canonical != id => tabs.remap_document(&id, &canonical)?,
            Some(_) => {}
            None if tab_ids.contains(&id) => absent.push(id),
            None => {}
        }
    }
    tabs.forget_documents(&absent);
    Ok(tabs)
}
