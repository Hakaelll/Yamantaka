use super::*;

#[derive(Default)]
pub(in crate::app) struct NotesPreview {
    phase: u8,
    shared: bool,
    editor: bool,
    id: Option<String>,
}
impl NotesPreview {
    pub fn ready(&self) -> bool {
        matches!(self.phase, 0 | 10 | 12 | 16)
    }
    pub fn phase(&self) -> u8 {
        self.phase
    }
}
impl YamantakaApp {
    pub fn preview_reference_notes(&mut self, shared: bool, editor: bool) {
        if self.demo && self.capture.is_some() {
            self.reference_notes_preview = NotesPreview {
                phase: 1,
                shared,
                editor,
                id: None,
            };
        }
    }
    pub(in crate::app) fn advance_reference_notes(&mut self) {
        if self.reference_notes_preview.ready() {
            return;
        }
        if self.reference_notes_preview.phase == 13 {
            if self.reference.is_none()
                && self.retired_annotation_writes.is_empty()
                && self.reader.annotations.ready()
                && self
                    .reader
                    .annotations
                    .page
                    .as_ref()
                    .unwrap()
                    .entries()
                    .iter()
                    .any(|record| {
                        record.data().values.note.as_deref() == Some("Updated reference note")
                    })
            {
                assert_eq!(self.reader.annotations.page.as_ref().unwrap().total, 3);
                self.reference_notes_preview.phase = 16;
            }
            return;
        }
        let Some(view) = &mut self.reference else {
            return;
        };
        let document = if self.reference_notes_preview.shared {
            "demo-0"
        } else {
            "demo-1"
        };
        if view.busy()
            || view
                .pane
                .active
                .as_ref()
                .is_none_or(|(doc, _)| doc.id != document)
        {
            return;
        }
        match self.reference_notes_preview.phase {
            1 if self.reader.annotations.ready() => {
                assert_eq!(self.reader.annotations.page.as_ref().unwrap().total, 2);
                view.set_page(1);
                view.notes_preview = true;
                self.reference_notes_preview.phase = 2;
            }
            2 if view.pane.annotations.ready() => {
                view.pane.begin_page_note();
                let editor = view.pane.annotations.editor.as_mut().unwrap();
                editor.note = "Reference study note".into();
                editor.tags = "Reference study".into();
                if self.reference_notes_preview.editor {
                    view.notes_scroll_preview = true;
                    self.reference_notes_preview.phase = 10;
                } else {
                    let change = view.pane.annotations.mutation(&self.governor).unwrap();
                    self.apply_notes_action(true, NotesAction::Change(change, 0));
                    self.reference_notes_preview.phase = 3;
                }
            }
            3 if view.pane.annotations.ready()
                && view.pane.annotations.saved
                && self.reader.annotations.ready()
                && self.reader.annotations.page.as_ref().unwrap().total
                    == if self.reference_notes_preview.shared {
                        3
                    } else {
                        2
                    } =>
            {
                let page = view.pane.annotations.page.as_ref().unwrap();
                let index = page
                    .entries()
                    .iter()
                    .position(|record| {
                        record.data().values.note.as_deref() == Some("Reference study note")
                    })
                    .unwrap();
                self.reference_notes_preview.id = Some(page.entries()[index].data().id.clone());
                assert_eq!(page.entries()[index].data().values.document_id, document);
                assert_eq!(page.entries()[index].data().values.page_index, 1);
                view.pane.annotations.edit_entry(index);
                view.pane.annotations.editor.as_mut().unwrap().note =
                    "Updated reference note".into();
                let change = view.pane.annotations.mutation(&self.governor).unwrap();
                self.apply_notes_action(true, NotesAction::Change(change, 0));
                if self.reference_notes_preview.shared {
                    self.request_reference_close();
                    self.reference_notes_preview.phase = 13;
                } else {
                    self.reference_notes_preview.phase = 4;
                }
            }
            4 if view.pane.annotations.ready() && view.pane.annotations.saved => {
                assert!(
                    view.pane
                        .annotations
                        .page
                        .as_ref()
                        .unwrap()
                        .entries()
                        .iter()
                        .any(|record| record.data().values.note.as_deref()
                            == Some("Updated reference note"))
                );
                let id = self.reference_notes_preview.id.clone().unwrap();
                self.apply_notes_action(
                    true,
                    NotesAction::Change(AnnotationMutation::Remove { id }, 0),
                );
                self.reference_notes_preview.phase = 5;
            }
            5 if view.pane.annotations.ready()
                && view.pane.annotations.saved
                && view.pane.annotations.page.as_ref().unwrap().total == 0 =>
            {
                assert_eq!(self.reader.annotations.page.as_ref().unwrap().total, 2);
                view.set_page(0);
                view.pane.begin_page_note();
                view.pane.annotations.editor.as_mut().unwrap().note = "Final reference note".into();
                let change = view.pane.annotations.mutation(&self.governor).unwrap();
                self.apply_notes_action(true, NotesAction::Change(change, 0));
                self.reference_notes_preview.phase = 6;
            }
            6 if view.pane.annotations.ready()
                && view.pane.annotations.saved
                && view.pane.annotations.page.as_ref().unwrap().total == 1 =>
            {
                assert_eq!(
                    view.pane.annotations.page.as_ref().unwrap().entries()[0]
                        .data()
                        .values
                        .note
                        .as_deref(),
                    Some("Final reference note")
                );
                assert_eq!(self.reader.annotations.page.as_ref().unwrap().total, 2);
                assert_eq!(self.reader.page, 0);
                self.reference_notes_preview.phase = 12;
            }
            _ => {}
        }
    }
}
