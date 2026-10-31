use super::{revision::AnnotationRevision, *};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredNoteDraft {
    pub document: String,
    pub annotation: String,
    pub reference: bool,
    pub revision: AnnotationRevision,
    pub fingerprint: String,
    pub page: u32,
    pub rotation: u16,
    pub passage: String,
    pub kind: AnnotationKind,
    pub color: AnnotationColor,
    pub review: AnnotationReviewState,
    pub note: String,
    pub tags: String,
}
impl StoredNoteDraft {
    pub fn same_editor(&self, other: &Self) -> bool {
        self.document == other.document
            && self.annotation == other.annotation
            && self.reference == other.reference
    }
    pub fn validate(&self) -> Result<()> {
        PageRotation::from_degrees(self.rotation.into())?;
        if self.document.is_empty()
            || self.annotation.is_empty()
            || self.document.len() > 1024
            || self.annotation.len() > 1024
            || self.fingerprint.len() > 1024
            || self.passage.len() > 1024 * 1024
            || self.note.len() > 65536
            || self.tags.len() > 65536
        {
            return Err(Failure::Invalid(
                "Stored-note recovery draft exceeds its limits; original session is preserved"
                    .into(),
            ));
        }
        Ok(())
    }
}
