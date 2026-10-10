use super::*;
use repository::{ANNOTATION_PAGE_SIZE, AnnotationPage};

pub struct AnnotationReviewMetadata {
    pub document: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub progress: f64,
}

pub struct AnnotationReviewSnapshot {
    page: AnnotationPage,
    documents: Vec<AnnotationReviewMetadata>,
    _lease: ByteLease,
}
impl AnnotationReviewSnapshot {
    pub fn page(&self) -> &AnnotationPage {
        &self.page
    }
    pub fn documents(&self) -> &[AnnotationReviewMetadata] {
        &self.documents
    }
    pub fn from_reserved(
        page: AnnotationPage,
        documents: Vec<AnnotationReviewMetadata>,
        mut lease: ByteLease,
    ) -> Result<Self> {
        if documents.len() > ANNOTATION_PAGE_SIZE || page.entries().len() > ANNOTATION_PAGE_SIZE {
            return Err(Failure::Invalid(
                "Review snapshot exceeds one annotation page".into(),
            ));
        }
        for (index, metadata) in documents.iter().enumerate() {
            if metadata.document.is_empty()
                || !metadata.progress.is_finite()
                || documents[..index]
                    .iter()
                    .any(|m| m.document == metadata.document)
                || !page
                    .entries()
                    .iter()
                    .any(|r| r.data().values.document_id == metadata.document)
            {
                return Err(Failure::Invalid(
                    "Review snapshot metadata is invalid".into(),
                ));
            }
        }
        if page.entries().iter().any(|r| {
            !documents
                .iter()
                .any(|m| m.document == r.data().values.document_id)
        }) {
            return Err(Failure::Invalid(
                "Review snapshot metadata is incomplete".into(),
            ));
        }
        let bytes = documents.iter().try_fold(
            documents
                .capacity()
                .checked_mul(std::mem::size_of::<AnnotationReviewMetadata>())
                .and_then(|n| n.checked_add(std::mem::size_of::<Self>()))
                .ok_or(Failure::MemoryBudget)?,
            |n, m| {
                n.checked_add(m.document.capacity())
                    .and_then(|n| n.checked_add(m.title.as_ref().map_or(0, String::capacity)))
                    .and_then(|n| n.checked_add(m.author.as_ref().map_or(0, String::capacity)))
                    .ok_or(Failure::MemoryBudget)
            },
        )?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            page,
            documents,
            _lease: lease,
        })
    }
}
