use super::*;
use crate::text_selection::{SelectionPosition, TextSelection};

/// Native character boundaries survive restart; PDF geometry is always read again.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectionNoteDraft {
    pub document: String,
    pub reference: bool,
    pub fingerprint: String,
    pub rotation: u16,
    pub start: [u32; 2],
    pub end: [u32; 2],
    pub passage: String,
    pub kind: AnnotationKind,
    pub color: AnnotationColor,
    pub note: String,
    pub tags: String,
    pub confirmed: bool,
}
impl SelectionNoteDraft {
    pub fn same_editor(&self, other: &Self) -> bool {
        self.document == other.document && self.reference == other.reference
    }
    pub fn selection(&self) -> Result<TextSelection> {
        let start = SelectionPosition::new(self.start[0], self.start[1] as usize)?;
        let end = SelectionPosition::new(self.end[0], self.end[1] as usize)?;
        if start >= end {
            return Err(Failure::Invalid(
                "Recovered selection range is empty or inverted".into(),
            ));
        }
        Ok(TextSelection::new(start, end))
    }
    pub fn validate(&self) -> Result<()> {
        self.selection()?;
        PageRotation::from_degrees(self.rotation.into())?;
        if self.document.is_empty()
            || self.document.len() > 1024
            || self.fingerprint.len() > 1024
            || self.passage.len() > 1024 * 1024
            || self.note.len() > 65536
            || self.tags.len() > 65536
        {
            return Err(Failure::Invalid(
                "Selection-note recovery draft exceeds its limits; original session is preserved"
                    .into(),
            ));
        }
        Ok(())
    }
}
