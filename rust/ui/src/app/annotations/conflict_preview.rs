use super::*;

#[derive(Default)]
pub(in crate::app) struct ConflictPreview {
    pub phase: u8,
    editor: Option<AnnotationEditor>,
}
impl YamantakaApp {
    pub fn preview_annotation_conflict(&mut self) {
        if self.demo && self.capture.is_some() {
            self.annotation_edit_conflict.phase = 1;
        }
    }
    pub(in crate::app) fn advance_annotation_conflict(&mut self) {
        if self.annotation_preview != 0 || !self.reader.annotations.ready() {
            return;
        }
        match self.annotation_edit_conflict.phase {
            1 if !self
                .reader
                .annotations
                .page
                .as_ref()
                .unwrap()
                .entries()
                .is_empty() =>
            {
                self.reader.annotations.edit_entry(0);
                self.reader.annotations.editor.as_mut().unwrap().note =
                    "Newer annotation text".into();
                let change = self.reader.annotations.mutation(&self.governor).unwrap();
                let mut older = self.reader.annotations.editor.take().unwrap();
                older.note = "Uncommitted older draft".into();
                self.annotation_edit_conflict.editor = Some(older);
                self.annotation_command(Some(change), 0);
                self.annotation_edit_conflict.phase = 2;
            }
            2 if self.reader.annotations.saved => {
                self.reader.annotations.editor = self.annotation_edit_conflict.editor.take();
                let change = self.reader.annotations.mutation(&self.governor).unwrap();
                self.annotation_command(Some(change), 0);
                self.annotation_edit_conflict.phase = 3;
            }
            3 if self.reader.annotations.error.is_some() => {
                self.annotation_command(None, 0);
                self.annotation_edit_conflict.phase = 4;
            }
            4 if self.reader.annotations.pending.is_none() => {
                assert!(self.reader.annotations.stale);
                assert_eq!(
                    self.reader.annotations.editor.as_ref().unwrap().note,
                    "Uncommitted older draft"
                );
                assert_eq!(
                    self.reader.annotations.page.as_ref().unwrap().entries()[0]
                        .data()
                        .values
                        .note
                        .as_deref(),
                    Some("Newer annotation text")
                );
                self.annotation_edit_conflict.phase = 5;
            }
            _ => {}
        }
    }
    pub(in crate::app) fn annotation_conflict_summary(&self) -> serde_json::Value {
        serde_json::json!({"phase":self.annotation_edit_conflict.phase,"draft":self.reader.annotations.editor.as_ref().map(|editor|&editor.note)})
    }
}
