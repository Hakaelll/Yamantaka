pub mod batch;
pub mod marks;
pub mod page_note_draft;
pub mod review;
pub mod revision;
pub mod selection_note_draft;
pub mod stored_note_draft;
use crate::{
    Failure, Result,
    memory::{ByteLease, Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    pdf_geometry::NormalizedPageRect,
    reader_preferences::PageRotation,
    text_selection::{GeometryCoverage, SelectedPage},
};
use serde::{Deserialize, Serialize};
pub mod repository;

pub const MAX_ANNOTATION_NOTE_UTF16: usize = 4000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnnotationKind {
    Highlight,
    Underline,
    Strikeout,
    TextNote,
    PageMarker,
    PageComment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnnotationColor {
    Yellow,
    Green,
    Blue,
    Rose,
    Violet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttachmentStatus {
    Attached,
    ReviewRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnnotationReviewState {
    Pending,
    Accepted,
    Resolved,
}

/// JSON keys remain compatible with Flutter's normalized-page-v1 geometry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RectangleFields")]
pub struct AnnotationRect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}
impl From<NormalizedPageRect> for AnnotationRect {
    fn from(region: NormalizedPageRect) -> Self {
        let [left, top] = region.min();
        let [right, bottom] = region.max();
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}
impl AnnotationRect {
    pub fn region(self) -> NormalizedPageRect {
        NormalizedPageRect::new([self.left, self.top], [self.right, self.bottom]).unwrap()
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RectangleFields {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}
impl TryFrom<RectangleFields> for AnnotationRect {
    type Error = Failure;
    fn try_from(value: RectangleFields) -> Result<Self> {
        Ok(NormalizedPageRect::new([value.left, value.top], [value.right, value.bottom])?.into())
    }
}

pub struct AnnotationSource<'a> {
    pub document_id: &'a str,
    pub fingerprint: &'a str,
    pub rotation: PageRotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationPlacement {
    Attached,
    /// The editor must offer a page note and obtain confirmation before saving.
    PageNoteFallback {
        requested: AnnotationKind,
        coverage: GeometryCoverage,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationDraftValues {
    pub document_id: String,
    pub page_index: u32,
    #[serde(rename = "type")]
    pub kind: AnnotationKind,
    pub color: AnnotationColor,
    pub note: Option<String>,
    pub selected_text: Option<String>,
    pub context_before: Option<String>,
    pub context_after: Option<String>,
    pub rects: Vec<AnnotationRect>,
    pub page_rotation: u16,
    pub document_fingerprint: String,
    pub status: AttachmentStatus,
    pub review_state: AnnotationReviewState,
}

/// A pending editor owns its copied text and geometry independently of PDF pages.
/// It does not mutate a PDF or database, nor invent bounds for missing glyphs.
pub struct AnnotationDraft {
    values: AnnotationDraftValues,
    placement: AnnotationPlacement,
    _lease: ByteLease,
}
impl AnnotationDraft {
    pub fn values(&self) -> &AnnotationDraftValues {
        &self.values
    }
    pub fn placement(&self) -> AnnotationPlacement {
        self.placement
    }
    pub fn page_note(
        source: AnnotationSource<'_>,
        page: u32,
        color: AnnotationColor,
        note: Option<&str>,
        governor: &MemoryGovernor,
    ) -> Result<Self> {
        validate_source_note(&source, note)?;
        let note = note.map(str::trim).filter(|note| !note.is_empty());
        let bytes = source
            .document_id
            .len()
            .checked_add(source.fingerprint.len())
            .and_then(|bytes| bytes.checked_add(note.map_or(0, str::len)))
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                bytes,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let values = AnnotationDraftValues {
            document_id: copy(source.document_id)?,
            page_index: page,
            kind: AnnotationKind::PageComment,
            color,
            note: note.map(copy).transpose()?,
            selected_text: None,
            context_before: None,
            context_after: None,
            rects: Vec::new(),
            page_rotation: source.rotation.degrees(),
            document_fingerprint: copy(source.fingerprint)?,
            status: AttachmentStatus::Attached,
            review_state: AnnotationReviewState::Pending,
        };
        Self::from_reserved(values, AnnotationPlacement::Attached, lease)
    }
    pub fn from_selection(
        source: AnnotationSource<'_>,
        selected: &SelectedPage<'_>,
        requested: AnnotationKind,
        color: AnnotationColor,
        note: Option<&str>,
        governor: &MemoryGovernor,
    ) -> Result<Self> {
        validate_source_note(&source, note)?;
        let coverage = selected.geometry_coverage();
        let meaningful = selected.ink_regions().all(|region| {
            region.max()[0] - region.min()[0] > 0.0001 && region.max()[1] - region.min()[1] > 0.0001
        });
        let placement = if requested != AnnotationKind::PageComment
            && (coverage != GeometryCoverage::Complete || !meaningful)
        {
            AnnotationPlacement::PageNoteFallback {
                requested,
                coverage,
            }
        } else {
            AnnotationPlacement::Attached
        };
        let kind = if matches!(placement, AnnotationPlacement::PageNoteFallback { .. }) {
            AnnotationKind::PageComment
        } else {
            requested
        };
        let selected_text = selected.text().trim();
        let (before, after) = selected.annotation_context();
        let before = before.trim();
        let after = after.trim();
        let note = note.map(str::trim).filter(|text| !text.is_empty());
        if selected_text.is_empty() && kind != AnnotationKind::PageComment {
            return Err(Failure::Invalid("Annotation passage is empty".into()));
        }
        let count = if kind == AnnotationKind::PageComment {
            0
        } else {
            selected.ink_regions().count()
        };
        let bytes = count
            .checked_mul(std::mem::size_of::<AnnotationRect>())
            .and_then(|value| value.checked_add(selected_text.len()))
            .and_then(|value| value.checked_add(before.len()))
            .and_then(|value| value.checked_add(after.len()))
            .and_then(|value| value.checked_add(note.map_or(0, str::len)))
            .and_then(|value| value.checked_add(source.document_id.len()))
            .and_then(|value| value.checked_add(source.fingerprint.len()))
            .and_then(|value| value.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                bytes,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let mut rects = Vec::new();
        rects
            .try_reserve_exact(count)
            .map_err(|_| Failure::MemoryBudget)?;
        if count > 0 {
            rects.extend(selected.ink_regions().map(AnnotationRect::from));
        }
        let values = AnnotationDraftValues {
            document_id: copy(source.document_id)?,
            page_index: selected.page(),
            kind,
            color,
            note: note.map(copy).transpose()?,
            selected_text: (!selected_text.is_empty())
                .then(|| copy(selected_text))
                .transpose()?,
            context_before: (!before.is_empty()).then(|| copy(before)).transpose()?,
            context_after: (!after.is_empty()).then(|| copy(after)).transpose()?,
            rects,
            page_rotation: source.rotation.degrees(),
            document_fingerprint: copy(source.fingerprint)?,
            status: AttachmentStatus::Attached,
            review_state: AnnotationReviewState::Pending,
        };
        Self::from_reserved(values, placement, lease)
    }
    fn from_reserved(
        values: AnnotationDraftValues,
        placement: AnnotationPlacement,
        mut lease: ByteLease,
    ) -> Result<Self> {
        let actual = values
            .rects
            .capacity()
            .checked_mul(std::mem::size_of::<AnnotationRect>())
            .and_then(|value| value.checked_add(values.document_id.capacity()))
            .and_then(|value| value.checked_add(values.document_fingerprint.capacity()))
            .and_then(|value| value.checked_add(values.note.as_ref().map_or(0, String::capacity)))
            .and_then(|value| {
                value.checked_add(values.selected_text.as_ref().map_or(0, String::capacity))
            })
            .and_then(|value| {
                value.checked_add(values.context_before.as_ref().map_or(0, String::capacity))
            })
            .and_then(|value| {
                value.checked_add(values.context_after.as_ref().map_or(0, String::capacity))
            })
            .and_then(|value| value.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        lease
            .try_resize(actual)
            .map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            values,
            placement,
            _lease: lease,
        })
    }
}
fn validate_source_note(source: &AnnotationSource<'_>, note: Option<&str>) -> Result<()> {
    if source.document_id.is_empty()
        || source.fingerprint.is_empty()
        || source.document_id.len() > 4096
        || source.fingerprint.len() > 4096
    {
        return Err(Failure::Invalid(
            "Annotation source identity is invalid".into(),
        ));
    }
    if note.is_some_and(|text| text.encode_utf16().count() > MAX_ANNOTATION_NOTE_UTF16) {
        return Err(Failure::Invalid(
            "Annotation note exceeds 4000 UTF-16 units".into(),
        ));
    }
    Ok(())
}
fn copy(text: &str) -> Result<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(text.len())
        .map_err(|_| Failure::MemoryBudget)?;
    value.push_str(text);
    Ok(value)
}
