use super::*;

#[derive(Default)]
pub(in crate::app) struct NotesPreview {
    phase: u8,
    shared: bool,
    editor: bool,
    filters: Option<bool>,
    bulk: Option<bool>,
    id: Option<String>,
}
impl NotesPreview {
    pub fn ready(&self) -> bool {
        matches!(self.phase, 0 | 10 | 12 | 16 | 25 | 26 | 40 | 41)
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
                filters: None,
                bulk: None,
                id: None,
            };
        }
    }
    pub fn preview_reference_note_filters(&mut self, empty: bool) {
        if self.demo && self.capture.is_some() {
            self.reference_notes_preview.filters = Some(empty);
        }
    }
    pub fn preview_reference_note_bulk(&mut self, save: bool) {
        if self.demo && self.capture.is_some() {
            self.reference_notes_preview.bulk = Some(save);
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
                if let Some(save) = self.reference_notes_preview.bulk {
                    assert_eq!(view.pane.annotations.page.as_ref().unwrap().total, 82);
                    assert_eq!(
                        view.pane.annotations.page.as_ref().unwrap().entries().len(),
                        80
                    );
                    view.pane.annotations.selected.0.fill(true);
                    view.pane.annotations.bulk_tag = "Reference batch".into();
                    if save {
                        let change = view
                            .pane
                            .annotations
                            .review_mutation(AnnotationReviewState::Accepted)
                            .unwrap();
                        self.apply_notes_action(true, NotesAction::Change(change, 0));
                        self.reference_notes_preview.phase = 30;
                    } else {
                        self.reference_notes_preview.phase = 40;
                    }
                    return;
                }
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
                view.pane.annotations.editor.as_mut().unwrap().tags = "Reference study".into();
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
                if self.reference_notes_preview.filters.is_some() {
                    view.pane.annotations.filter_draft = filters::Filters {
                        focused: None,
                        page: None,
                        text: "Final reference".into(),
                        tag: "Reference study".into(),
                        kind: Some(AnnotationKind::PageComment),
                        review: Some(AnnotationReviewState::Pending),
                    };
                    assert!(view.pane.annotations.apply_filters());
                    self.apply_notes_action(true, NotesAction::Read(0));
                    self.reference_notes_preview.phase = 20;
                } else {
                    self.reference_notes_preview.phase = 12;
                }
            }
            20 if view.pane.annotations.ready() => {
                assert_eq!(view.pane.annotations.page.as_ref().unwrap().total, 1);
                assert_eq!(view.pane.annotations.filters.count(), 4);
                view.pane.annotations.filter_draft.text = "No matching reference note".into();
                assert!(view.pane.annotations.apply_filters());
                self.apply_notes_action(true, NotesAction::Read(0));
                self.reference_notes_preview.phase = 21;
            }
            21 if view.pane.annotations.ready() => {
                assert_eq!(view.pane.annotations.page.as_ref().unwrap().total, 0);
                view.pane.annotations.filter_draft = Default::default();
                assert!(view.pane.annotations.apply_filters());
                self.apply_notes_action(true, NotesAction::Read(0));
                self.reference_notes_preview.phase = 22;
            }
            22 if view.pane.annotations.ready() => {
                assert_eq!(view.pane.annotations.page.as_ref().unwrap().total, 1);
                assert_eq!(view.pane.annotations.filters.count(), 0);
                let empty = self.reference_notes_preview.filters.unwrap();
                view.pane.annotations.filter_draft = filters::Filters {
                    focused: None,
                    page: None,
                    text: if empty {
                        "No matching reference note"
                    } else {
                        "Final reference"
                    }
                    .into(),
                    tag: "Reference study".into(),
                    kind: Some(AnnotationKind::PageComment),
                    review: Some(AnnotationReviewState::Pending),
                };
                assert!(view.pane.annotations.apply_filters());
                self.apply_notes_action(true, NotesAction::Read(0));
                self.reference_notes_preview.phase = if empty { 24 } else { 23 };
            }
            23 | 24 if view.pane.annotations.ready() => {
                let empty = self.reference_notes_preview.filters.unwrap();
                assert_eq!(
                    view.pane.annotations.page.as_ref().unwrap().total,
                    if empty { 0 } else { 1 }
                );
                assert_eq!(self.reader.annotations.page.as_ref().unwrap().total, 2);
                assert_eq!(self.reader.annotations.filters.count(), 0);
                assert_eq!(self.reader.page, 0);
                self.reference_notes_preview.phase = if empty { 26 } else { 25 };
            }
            30 if view.pane.annotations.ready() && view.pane.annotations.saved => {
                let page = view.pane.annotations.page.as_ref().unwrap();
                assert_eq!(page.total, 82);
                assert!(page.entries().iter().all(
                    |entry| entry.data().values.review_state == AnnotationReviewState::Accepted
                ));
                view.pane.annotations.selected.0.fill(true);
                view.pane.annotations.bulk_tag = "Reference batch".into();
                let change = view.pane.annotations.tag_mutation().unwrap();
                self.apply_notes_action(true, NotesAction::Change(change, 0));
                self.reference_notes_preview.phase = 31;
            }
            31 if view.pane.annotations.ready() && view.pane.annotations.saved => {
                let page = view.pane.annotations.page.as_ref().unwrap();
                assert!(page.entries().iter().all(|entry| {
                    entry.data().tags.iter().any(|tag| tag == "Reference batch")
                        && entry.data().tags.iter().any(|tag| tag == "Existing study")
                }));
                self.apply_notes_action(true, NotesAction::Read(80));
                self.reference_notes_preview.phase = 32;
            }
            32 if view.pane.annotations.ready() => {
                let page = view.pane.annotations.page.as_ref().unwrap();
                assert_eq!(page.offset, 80);
                assert_eq!(page.entries().len(), 2);
                assert!(
                    page.entries()
                        .iter()
                        .all(|entry| entry.data().values.review_state
                            == AnnotationReviewState::Pending
                            && entry.data().tags == ["Existing study"])
                );
                assert_eq!(view.pane.annotations.selected.count(), 0);
                self.apply_notes_action(true, NotesAction::Read(0));
                self.reference_notes_preview.phase = 33;
            }
            33 if view.pane.annotations.ready() => {
                let page = view.pane.annotations.page.as_ref().unwrap();
                assert_eq!(page.total, 82);
                assert_eq!(page.offset, 0);
                assert_eq!(page.entries().len(), 80);
                assert!(
                    page.entries()
                        .iter()
                        .all(|entry| entry.data().values.review_state
                            == AnnotationReviewState::Accepted
                            && entry.data().tags.iter().any(|tag| tag == "Reference batch")
                            && entry.data().tags.iter().any(|tag| tag == "Existing study"))
                );
                assert_eq!(view.pane.annotations.selected.count(), 0);
                assert_eq!(self.reader.annotations.page.as_ref().unwrap().total, 2);
                assert!(
                    self.reader
                        .annotations
                        .page
                        .as_ref()
                        .unwrap()
                        .entries()
                        .iter()
                        .all(|entry| entry.data().values.review_state
                            == AnnotationReviewState::Pending
                            && entry.data().tags.is_empty())
                );
                self.reference_notes_preview.phase = 41;
            }
            _ => {}
        }
    }
}
