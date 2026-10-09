use super::*;
use crate::{Cancellation, search::IndexAccess};
pub const ANNOTATION_PAGE_SIZE: usize = 80;
#[derive(Default)]
pub struct AnnotationQuery {
    pub document_id: Option<String>,
    pub page: Option<u32>,
    pub kind: Option<AnnotationKind>,
    pub color: Option<AnnotationColor>,
    pub review: Option<AnnotationReviewState>,
    pub tag: Option<String>,
    pub text: String,
    pub newest_first: bool,
    pub updated_after: Option<String>,
    pub updated_before: Option<String>,
    pub offset: usize,
}
pub struct AnnotationRecordData {
    pub id: String,
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
    _lease: ByteLease,
}
impl AnnotationRecord {
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
            .ok_or(Failure::MemoryBudget)?;
        let bytes = bytes
            .checked_add(extension_bytes)
            .ok_or(Failure::MemoryBudget)?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            data,
            _lease: lease,
        })
    }
}
pub struct AnnotationPage {
    entries: Vec<AnnotationRecord>,
    pub total: u64,
    pub offset: usize,
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
            _lease: lease,
        })
    }
}
pub enum AnnotationChange<'a> {
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
    Edit {
        id: &'a str,
        kind: AnnotationKind,
        color: AnnotationColor,
        note: Option<&'a str>,
        review: AnnotationReviewState,
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
pub trait AnnotationRepository {
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
