use crate::source_revision::CataloguedFile;
use crate::{Document, Failure, Result};
pub mod draft;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataField {
    Title,
    Author,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataEvidenceKind {
    ProminentText,
    TitleWithByline,
    RepeatedTitleWithByline,
    ExplicitByline,
    ConflictingBylines,
    Corroborated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataEvidence {
    pub value: String,
    pub page: u32,
    pub kind: MetadataEvidenceKind,
}

pub struct MetadataLocation {
    pub document: Document,
    pub page: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataReview {
    pub document: String,
    pub revision: String,
    pub title: String,
    pub author: String,
    pub automatic_title: Option<String>,
    pub automatic_author: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
    pub title_override: Option<String>,
    pub author_override: Option<String>,
    pub suggested_title: Option<String>,
    pub suggested_author: Option<String>,
    pub file: CataloguedFile,
    pub title_evidence: Option<MetadataEvidence>,
    pub author_evidence: Option<MetadataEvidence>,
}

impl MetadataReview {
    pub fn evidence(&self, field: MetadataField) -> Option<&MetadataEvidence> {
        match field {
            MetadataField::Title => self.title_evidence.as_ref(),
            MetadataField::Author => self.author_evidence.as_ref(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManualMetadata {
    title: String,
    author: String,
}
impl ManualMetadata {
    pub fn new(title: &str, author: &str) -> Result<Self> {
        fn clean(value: &str) -> Result<String> {
            if value.chars().count() > 512
                || value.chars().any(|c| c.is_control() && !c.is_whitespace())
            {
                return Err(Failure::Invalid(
                    "Metadata requires at most 512 characters without control codes".into(),
                ));
            }
            Ok(value.split_whitespace().collect::<Vec<_>>().join(" "))
        }
        let title = clean(title)?;
        let author = clean(author)?;
        if title.is_empty() {
            return Err(Failure::Invalid(
                "Enter a title before saving metadata".into(),
            ));
        }
        Ok(Self { title, author })
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn author(&self) -> &str {
        &self.author
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataMutation {
    Save(ManualMetadata),
    ResetOverrides,
    DismissSuggestions,
}
