use std::sync::Arc;
use yamantaka_domain::{
    Cancellation, Result,
    annotations::{
        AnnotationColor, AnnotationDraft, AnnotationKind, AnnotationReviewState,
        repository::{
            AnnotationChange, AnnotationPage, AnnotationQuery, AnnotationRecord,
            AnnotationRepository,
        },
    },
    memory::MemoryGovernor,
    search::IndexAccess,
};

pub enum AnnotationMutation {
    CreateBatch {
        batch: Arc<yamantaka_domain::annotations::batch::AnnotationDraftBatch>,
        confirm_page_note: bool,
        tags: Vec<String>,
    },
    Create {
        draft: Arc<AnnotationDraft>,
        confirm_page_note: bool,
        tags: Vec<String>,
    },
    Save {
        record: Arc<AnnotationRecord>,
    },
    Remove {
        id: String,
    },
    Edit {
        id: String,
        kind: AnnotationKind,
        color: AnnotationColor,
        note: Option<String>,
        review: AnnotationReviewState,
    },
    Review {
        ids: Vec<String>,
        state: AnnotationReviewState,
    },
    AddTag {
        ids: Vec<String>,
        tag: String,
    },
    CheckAttachment {
        fingerprint: String,
    },
}
impl AnnotationMutation {
    fn borrowed(&self) -> AnnotationChange<'_> {
        match self {
            Self::CreateBatch {
                batch,
                confirm_page_note,
                tags,
            } => AnnotationChange::CreateBatch {
                batch,
                confirm_page_note: *confirm_page_note,
                tags,
            },
            Self::Create {
                draft,
                confirm_page_note,
                tags,
            } => AnnotationChange::Create {
                draft,
                confirm_page_note: *confirm_page_note,
                tags,
            },
            Self::Save { record } => AnnotationChange::Save { record },
            Self::Remove { id } => AnnotationChange::Remove { id },
            Self::Edit {
                id,
                kind,
                color,
                note,
                review,
            } => AnnotationChange::Edit {
                id,
                kind: *kind,
                color: *color,
                note: note.as_deref(),
                review: *review,
            },
            Self::Review { ids, state } => AnnotationChange::Review { ids, state: *state },
            Self::AddTag { ids, tag } => AnnotationChange::AddTag { ids, tag },
            Self::CheckAttachment { fingerprint } => {
                AnnotationChange::CheckAttachment { fingerprint }
            }
        }
    }
}
pub enum AnnotationOperation {
    Read,
    Change {
        document_id: String,
        mutation: AnnotationMutation,
    },
}
pub struct AnnotationRequest {
    pub operation: AnnotationOperation,
    pub query: AnnotationQuery,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub struct AnnotationReply {
    pub document_id: Option<String>,
    pub result: Result<AnnotationPage>,
    pub cancel: Cancellation,
}
impl AnnotationRequest {
    pub(crate) fn execute(
        self,
        repository: &mut impl AnnotationRepository,
        access: IndexAccess,
    ) -> AnnotationReply {
        let document_id = match &self.operation {
            AnnotationOperation::Read => self.query.document_id.clone(),
            AnnotationOperation::Change { document_id, .. } => Some(document_id.clone()),
        };
        let result = self.cancel.check().and_then(|()| match &self.operation {
            AnnotationOperation::Read => {
                repository.annotations(&self.query, &self.governor, &self.cancel)
            }
            AnnotationOperation::Change {
                document_id,
                mutation,
            } => repository.change_annotation(
                document_id,
                mutation.borrowed(),
                &self.query,
                access,
                &self.governor,
                &self.cancel,
            ),
        });
        AnnotationReply {
            document_id,
            result,
            cancel: self.cancel,
        }
    }
}

pub struct AnnotationMarksRequest {
    pub document: String,
    pub page: u32,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub struct AnnotationMarksReply {
    pub document: String,
    pub page: u32,
    pub result: Result<yamantaka_domain::annotations::marks::AnnotationMarks>,
    pub cancel: Cancellation,
}
impl AnnotationMarksRequest {
    pub(crate) fn execute(self, repository: &impl AnnotationRepository) -> AnnotationMarksReply {
        let result = self.cancel.check().and_then(|()| {
            repository.annotation_marks(&self.document, self.page, &self.governor, &self.cancel)
        });
        AnnotationMarksReply {
            document: self.document,
            page: self.page,
            result,
            cancel: self.cancel,
        }
    }
}
