use super::*;
use crate::{Cancellation, search::IndexAccess};
pub const ANNOTATION_PAGE_SIZE: usize = 80;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnnotationGrouping {
    Document,
    Page,
    Review,
}
#[derive(Default)]
pub struct AnnotationQuery {
    pub document_id: Option<String>,
    pub annotation_id: Option<String>,
    pub page: Option<u32>,
    pub kind: Option<AnnotationKind>,
    pub color: Option<AnnotationColor>,
    pub review: Option<AnnotationReviewState>,
    pub tag: Option<String>,
    pub text: String,
    pub newest_first: bool,
    pub grouping: Option<AnnotationGrouping>,
    pub updated_after: Option<String>,
    pub updated_before: Option<String>,
    pub offset: usize,
    pub limit: Option<std::num::NonZeroUsize>,
}
pub struct AnnotationRecordData {
    pub id: String,
    /// Current catalogue title for presentation; not persisted with the annotation.
    pub document_title: Option<String>,
    pub values: AnnotationDraftValues,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub origin_document_id: Option<String>,
    pub origin_fingerprint: Option<String>,
    pub extensions: Vec<(String, AnnotationColumnValue)>,
}
pub enum AnnotationColumnValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}
pub struct AnnotationRecord {
    data: AnnotationRecordData,
    revision: super::revision::AnnotationRevision,
    _lease: ByteLease,
}
impl AnnotationRecord {
    pub fn revision(&self) -> &super::revision::AnnotationRevision {
        &self.revision
    }
    pub fn data(&self) -> &AnnotationRecordData {
        &self.data
    }
    pub fn from_reserved(data: AnnotationRecordData, mut lease: ByteLease) -> Result<Self> {
        let values = &data.values;
        let strings = [
            &data.id,
            &data.created_at,
            &data.updated_at,
            &values.document_id,
            &values.document_fingerprint,
        ];
        let optional = [
            &data.document_title,
            &data.origin_document_id,
            &data.origin_fingerprint,
            &values.note,
            &values.selected_text,
            &values.context_before,
            &values.context_after,
        ];
        let extension_bytes = data
            .extensions
            .iter()
            .try_fold(0usize, |bytes, (name, value)| {
                let length = match value {
                    AnnotationColumnValue::Text(value) => value.capacity(),
                    AnnotationColumnValue::Blob(value) => value.capacity(),
                    _ => 0,
                };
                bytes.checked_add(name.capacity())?.checked_add(length)
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    data.extensions
                        .capacity()
                        .checked_mul(std::mem::size_of::<(String, AnnotationColumnValue)>())?,
                )
            })
            .ok_or(Failure::MemoryBudget)?;
        let bytes = strings
            .into_iter()
            .map(String::capacity)
            .chain(
                optional
                    .into_iter()
                    .map(|text| text.as_ref().map_or(0, String::capacity)),
            )
            .chain(data.tags.iter().map(String::capacity))
            .try_fold(std::mem::size_of::<Self>(), usize::checked_add)
            .and_then(|bytes| {
                bytes.checked_add(
                    data.tags
                        .capacity()
                        .checked_mul(std::mem::size_of::<String>())?,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    values
                        .rects
                        .capacity()
                        .checked_mul(std::mem::size_of::<AnnotationRect>())?,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    values
                        .drawing
                        .as_ref()
                        .map_or(0, StudyGeometry::allocated_bytes),
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    values
                        .drawing
                        .as_ref()
                        .map_or(0, StudyGeometry::allocated_bytes),
                )
            })
            .ok_or(Failure::MemoryBudget)?;
        let bytes = bytes
            .checked_add(extension_bytes)
            .ok_or(Failure::MemoryBudget)?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        let revision = data.calculate_revision()?;
        Ok(Self {
            data,
            revision,
            _lease: lease,
        })
    }
}
#[derive(Clone, Copy, Default)]
pub struct AnnotationHistory {
    pub undo: usize,
    pub redo: usize,
}
pub struct AnnotationPage {
    entries: Vec<AnnotationRecord>,
    pub total: u64,
    pub offset: usize,
    pub history: AnnotationHistory,
    _lease: ByteLease,
}
impl AnnotationPage {
    pub fn entries(&self) -> &[AnnotationRecord] {
        &self.entries
    }
    pub fn into_entries(self) -> Vec<AnnotationRecord> {
        self.entries
    }
    pub fn from_reserved(
        entries: Vec<AnnotationRecord>,
        total: u64,
        offset: usize,
        mut lease: ByteLease,
    ) -> Result<Self> {
        let bytes = entries
            .capacity()
            .checked_mul(std::mem::size_of::<AnnotationRecord>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            entries,
            total,
            offset,
            history: AnnotationHistory::default(),
            _lease: lease,
        })
    }
}
pub enum AnnotationChange<'a> {
    Undo,
    Redo,
    CreateBatch {
        batch: &'a super::batch::AnnotationDraftBatch,
        confirm_page_note: bool,
        tags: &'a [String],
    },
    Create {
        draft: &'a AnnotationDraft,
        confirm_page_note: bool,
        tags: &'a [String],
    },
    Save {
        record: &'a AnnotationRecord,
    },
    Remove {
        id: &'a str,
    },
    EraseMark {
        id: &'a str,
    },
    Edit {
        id: &'a str,
        expected: &'a super::revision::AnnotationRevision,
        kind: AnnotationKind,
        color: AnnotationColor,
        note: Option<&'a str>,
        review: AnnotationReviewState,
        /// None preserves links; Some replaces them, including an empty list.
        tags: Option<&'a [String]>,
    },
    Review {
        ids: &'a [String],
        state: AnnotationReviewState,
    },
    AddTag {
        ids: &'a [String],
        tag: &'a str,
    },
    CheckAttachment {
        fingerprint: &'a str,
    },
}
/// A frozen identity includes its document so moved or stale records are rejected.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnnotationTarget {
    pub document_id: String,
    pub annotation_id: String,
}
pub enum AnnotationBatchChange<'a> {
    Review {
        targets: &'a [AnnotationTarget],
        state: AnnotationReviewState,
    },
    AddTag {
        targets: &'a [AnnotationTarget],
        tag: &'a str,
    },
}
pub trait AnnotationRepository {
    fn annotation_for_edit(
        &self,
        _document: &str,
        _annotation: &str,
        _governor: &MemoryGovernor,
        _cancel: &Cancellation,
    ) -> Result<Option<AnnotationRecord>> {
        Err(Failure::Unsupported(
            "Reading an annotation editor source".into(),
        ))
    }
    /// Captures one filtered review page and its document metadata in one read transaction.
    fn annotation_review_snapshot(
        &self,
        _query: &AnnotationQuery,
        _governor: &MemoryGovernor,
        _cancel: &Cancellation,
    ) -> Result<super::review::AnnotationReviewSnapshot> {
        Err(Failure::Unsupported(
            "Reading annotation review snapshots".into(),
        ))
    }
    /// Applies a cross-document batch atomically, including the resulting query and index refresh.
    fn change_annotation_batch(
        &mut self,
        _change: AnnotationBatchChange<'_>,
        _query: &AnnotationQuery,
        _access: IndexAccess,
        _governor: &MemoryGovernor,
        _cancel: &Cancellation,
    ) -> Result<AnnotationPage> {
        Err(Failure::Unsupported(
            "Changing annotations across documents".into(),
        ))
    }
    /// Opens an independent reader for export work without sharing the actor's connection.
    fn export_reader(&self) -> Result<Box<dyn AnnotationRepository + Send>> {
        Err(Failure::Unsupported("Independent annotation export".into()))
    }

    /// Publishes only successful output to a new destination, without replacing existing files.
    fn publish_annotation_export(
        &self,
        _destination: &std::path::Path,
        _cancel: &Cancellation,
        _write: &mut dyn FnMut(&mut dyn std::io::Write) -> Result<u64>,
    ) -> Result<u64> {
        Err(Failure::Unsupported("Publishing annotation exports".into()))
    }

    fn visit_annotations(
        &self,
        _document: &str,
        _governor: &MemoryGovernor,
        _cancel: &Cancellation,
        _visitor: &mut dyn FnMut(&AnnotationRecord) -> Result<()>,
    ) -> Result<u64> {
        Err(Failure::Unsupported("Visiting annotations".into()))
    }

    fn annotation_marks(
        &self,
        _document: &str,
        _page: u32,
        _governor: &MemoryGovernor,
        _cancel: &Cancellation,
    ) -> Result<super::marks::AnnotationMarks> {
        Err(Failure::Unsupported("Reading annotation marks".into()))
    }

    fn annotations(
        &self,
        _query: &AnnotationQuery,
        _governor: &MemoryGovernor,
        _cancel: &Cancellation,
    ) -> Result<AnnotationPage> {
        Err(Failure::Unsupported("Reading annotations".into()))
    }
    fn change_annotation(
        &mut self,
        _document_id: &str,
        _change: AnnotationChange<'_>,
        _query: &AnnotationQuery,
        _access: IndexAccess,
        _governor: &MemoryGovernor,
        _cancel: &Cancellation,
    ) -> Result<AnnotationPage> {
        Err(Failure::Unsupported("Changing annotations".into()))
    }
}
