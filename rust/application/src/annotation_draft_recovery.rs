use yamantaka_domain::{
    Cancellation, Failure, Result,
    annotations::{
        repository::{AnnotationRecord, AnnotationRepository},
        stored_note_draft::StoredNoteDraft,
    },
    memory::MemoryGovernor,
};

pub struct Request {
    pub draft: StoredNoteDraft,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub struct Reply {
    pub draft: StoredNoteDraft,
    pub result: Result<AnnotationRecord>,
    pub cancel: Cancellation,
}
impl Request {
    pub(crate) fn execute(self, repository: &impl AnnotationRepository) -> Reply {
        let result = (|| {
            self.cancel.check()?;
            self.draft.validate()?;
            let record = repository
                .annotation_for_edit(
                    &self.draft.document,
                    &self.draft.annotation,
                    &self.governor,
                    &self.cancel,
                )?
                .ok_or(Failure::StaleAnnotation)?;
            let values = &record.data().values;
            if record.revision() != &self.draft.revision
                || values.page_index != self.draft.page
                || values.page_rotation != self.draft.rotation
                || values.document_fingerprint != self.draft.fingerprint
                || values.selected_text.as_deref().unwrap_or("") != self.draft.passage
            {
                return Err(Failure::StaleAnnotation);
            }
            self.cancel.check()?;
            Ok(record)
        })();
        Reply {
            draft: self.draft,
            result,
            cancel: self.cancel,
        }
    }
}
