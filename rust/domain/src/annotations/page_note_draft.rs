use super::*;

/// Unsaved editor input, deliberately separate from a committed annotation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageNoteDraft {
    pub document: String,
    pub fingerprint: String,
    pub reference: bool,
    pub page: u32,
    pub rotation: u16,
    pub color: AnnotationColor,
    pub note: String,
    pub tags: String,
}
impl PageNoteDraft {
    pub fn validate(&self) -> Result<()> {
        PageRotation::from_degrees(self.rotation.into())?;
        if self.document.is_empty()
            || self.document.len() > 1024
            || self.fingerprint.len() > 1024
            || self.note.len() > 64 * 1024
            || self.tags.len() > 64 * 1024
        {
            return Err(Failure::Invalid(
                "Page-note draft exceeds its recovery limits; original session is preserved".into(),
            ));
        }
        Ok(())
    }
}
