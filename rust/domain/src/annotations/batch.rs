use super::*;
use crate::{pdf_text::PdfPageText, text_selection::TextSelection};
use std::sync::Arc;

pub struct AnnotationDraftBatch {
    drafts: Vec<AnnotationDraft>,
    _lease: ByteLease,
}
impl AnnotationDraftBatch {
    pub fn drafts(&self) -> &[AnnotationDraft] {
        &self.drafts
    }
    pub fn requires_page_note_confirmation(&self) -> bool {
        self.drafts.iter().any(|draft| {
            matches!(
                draft.placement(),
                AnnotationPlacement::PageNoteFallback { .. }
            )
        })
    }
    pub fn from_selection(
        source: AnnotationSource<'_>,
        selection: TextSelection,
        pages: &[Arc<PdfPageText>],
        requested: AnnotationKind,
        color: AnnotationColor,
        note: Option<&str>,
        governor: &MemoryGovernor,
    ) -> Result<Self> {
        let count = u64::from(selection.end().page()) - u64::from(selection.start().page()) + 1;
        if selection.is_empty() || pages.len() as u64 != count {
            return Err(Failure::Invalid(
                "Annotation selection requires every source page".into(),
            ));
        }
        for (index, page) in pages.iter().enumerate() {
            if u64::from(page.page()) != u64::from(selection.start().page()) + index as u64 {
                return Err(Failure::Invalid(
                    "Annotation source pages are missing, duplicated or unordered".into(),
                ));
            }
        }
        let bytes = pages
            .len()
            .checked_mul(std::mem::size_of::<AnnotationDraft>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        let mut lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                bytes,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let mut drafts = Vec::new();
        drafts
            .try_reserve_exact(pages.len())
            .map_err(|_| Failure::MemoryBudget)?;
        for page in pages {
            let selected = selection.page(page)?.ok_or_else(|| {
                Failure::Invalid("Annotation source page is outside the selection".into())
            })?;
            if selected.text().trim().is_empty() {
                continue;
            }
            drafts.push(AnnotationDraft::from_selection(
                AnnotationSource {
                    document_id: source.document_id,
                    fingerprint: source.fingerprint,
                    rotation: source.rotation,
                },
                &selected,
                requested,
                color,
                note,
                governor,
            )?);
        }
        if drafts.is_empty() {
            return Err(Failure::Invalid("Annotation passage is empty".into()));
        }
        let actual = drafts
            .capacity()
            .checked_mul(std::mem::size_of::<AnnotationDraft>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        lease
            .try_resize(actual)
            .map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            drafts,
            _lease: lease,
        })
    }
}
