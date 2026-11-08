use super::*;

impl reader_pane::ReaderPane {
    pub(in crate::app) fn begin_page_note(&mut self) {
        self.begin_page_note_at(self.page, AnnotationColor::Violet);
    }

    pub(in crate::app) fn begin_page_note_at(&mut self, page: u32, color: AnnotationColor) {
        if self.annotations.editing() || self.annotations.writing() {
            return;
        }
        let Some((document, info)) = &self.active else {
            return;
        };
        if page as usize >= info.pages.len() {
            return;
        }
        self.annotations.bind(&document.id);
        self.annotations.editor = Some(AnnotationEditor {
            source: EditorSource::PageNote {
                page,
                fingerprint: document.fingerprint.clone(),
                rotation: self
                    .reader_preferences
                    .as_ref()
                    .map_or(PageRotation::Deg0, |prefs| prefs.page_rotation),
            },
            kind: AnnotationKind::PageComment,
            color,
            note: String::new(),
            tags: String::new(),
            review: AnnotationReviewState::Pending,
            confirmed: false,
        });
        self.annotations.error = None;
        self.annotations.focus_editor = true;
        self.panel = true;
        self.reader_panel_tab = outline::ReaderPanelTab::Annotations;
    }
}

impl reader_pane::ReaderPane {
    pub(in crate::app) fn annotate_selection(&mut self, governor: &MemoryGovernor) {
        self.annotate_selection_as(governor, AnnotationKind::Highlight, AnnotationColor::Violet);
    }

    pub(in crate::app) fn annotate_selection_as(
        &mut self,
        governor: &MemoryGovernor,
        kind: AnnotationKind,
        color: AnnotationColor,
    ) {
        if self.annotations.editing() || self.annotations.writing() {
            return;
        }
        let Some((document, _)) = &self.active else {
            return;
        };
        match self.selection.annotation_source(governor) {
            Ok(snapshot) => {
                self.annotations.bind(&document.id);
                self.annotations.editor = Some(AnnotationEditor {
                    source: EditorSource::Selection {
                        snapshot,
                        fingerprint: document.fingerprint.clone(),
                        rotation: self
                            .reader_preferences
                            .as_ref()
                            .map_or(PageRotation::Deg0, |prefs| prefs.page_rotation),
                    },
                    kind,
                    color,
                    note: String::new(),
                    tags: String::new(),
                    review: AnnotationReviewState::Pending,
                    confirmed: false,
                });
                self.annotations.error = None;
                self.annotations.focus_editor = true;
                self.panel = true;
                self.reader_panel_tab = outline::ReaderPanelTab::Annotations;
            }
            Err(error) => self.annotations.error = Some(error.to_string()),
        }
    }
}

impl YamantakaApp {
    pub(in crate::app) fn apply_notes_action(&mut self, reference: bool, action: NotesAction) {
        if !reference {
            match action {
                NotesAction::RetryDraft => self.retry_annotation_draft(false),
                NotesAction::Export(format) => {
                    self.pick_annotation_export(export::ExportTarget::Primary, format)
                }
                NotesAction::Read(offset) => self.annotation_command(None, offset),
                NotesAction::Change(change, offset) => {
                    self.annotation_command(Some(change), offset)
                }
                NotesAction::PageNote => self.begin_page_note(),
                NotesAction::Selection => self.annotate_selection(),
                NotesAction::Focus(id) => {
                    self.reader
                        .focus_annotation(Some(id), &self.runtime, &self.governor)
                }
                NotesAction::ClearFocus => {
                    self.reader
                        .focus_annotation(None, &self.runtime, &self.governor)
                }
                NotesAction::Navigate(position, id, close) => {
                    self.reader.annotation_paint.focused = Some(id);
                    if self
                        .reader
                        .navigation
                        .as_mut()
                        .is_some_and(|nav| nav.navigate_to(position))
                    {
                        self.apply_navigation();
                    }
                    if close {
                        self.reader.panel = false;
                    }
                }
                NotesAction::ReloadMarks => self.reader.annotation_paint.invalidate_document(None),
            }
            return;
        }
        if matches!(&action, NotesAction::RetryDraft) {
            self.retry_annotation_draft(true);
            return;
        }
        if let NotesAction::Export(format) = action {
            self.pick_annotation_export(export::ExportTarget::Reference, format);
            return;
        }
        let blocked = self.global_notes.writing()
            || self.library_tags.writing()
            || self.reader.annotations.writing();
        let Some(view) = &mut self.reference else {
            return;
        };
        match action {
            NotesAction::Read(offset) => {
                view.pane
                    .annotations
                    .command(&self.runtime, &self.governor, None, offset)
            }
            NotesAction::Change(change, offset) if !blocked => {
                view.pane
                    .annotations
                    .command(&self.runtime, &self.governor, Some(change), offset)
            }
            NotesAction::PageNote if !blocked => view.pane.begin_page_note(),
            NotesAction::Selection if !blocked => view.pane.annotate_selection(&self.governor),
            NotesAction::Focus(id) if !blocked => {
                view.pane
                    .focus_annotation(Some(id), &self.runtime, &self.governor);
                self.reading_focus.target = shortcuts::ReadingTarget::Reference;
                view.navigator_requested = true;
            }
            NotesAction::ClearFocus if !blocked => {
                view.pane
                    .focus_annotation(None, &self.runtime, &self.governor)
            }
            NotesAction::Navigate(position, id, _) => {
                view.pane.annotation_paint.focused = Some(id);
                if view
                    .pane
                    .navigation
                    .as_mut()
                    .is_some_and(|nav| nav.navigate_to(position))
                {
                    view.reference_navigation_changed();
                }
            }
            NotesAction::ReloadMarks => view.pane.annotation_paint.invalidate_document(None),
            _ => {
                view.pane.annotations.error = Some(
                    if self.english {
                        "Wait for the other note operation to finish."
                    } else {
                        "Espera a que termine la otra operación de notas."
                    }
                    .into(),
                )
            }
        }
        if view.pane.annotations.editing() || view.pane.annotations.error.is_some() {
            view.pane.reader_panel_tab = outline::ReaderPanelTab::Annotations;
            view.navigator_requested = true;
        }
    }
}

impl reader_pane::ReaderPane {
    fn focus_annotation(
        &mut self,
        id: Option<String>,
        runtime: &Runtime,
        governor: &MemoryGovernor,
    ) {
        if self.annotations.editing() || self.annotations.writing() {
            return;
        }
        let Some((document, _)) = &self.active else {
            return;
        };
        self.annotations.bind(&document.id);
        self.annotations.cancel_query();
        if self.annotations.reviewing() {
            self.annotations.set_reviewing(false);
        }
        self.annotation_paint.focused = id.clone();
        self.annotations.filters.focused = id;
        self.annotations.filters_open = false;
        self.reader_panel_tab = outline::ReaderPanelTab::Annotations;
        self.panel = true;
        self.annotations.command(runtime, governor, None, 0);
    }
}

impl YamantakaApp {
    pub(in crate::app) fn accept_annotation_reply(&mut self, reply: AnnotationReply) {
        if self.global_notes.matches(&reply) {
            self.accept_global_note_reply(reply);
            return;
        }
        let document = reply.document_id.clone();
        if let Some(index) = self
            .retired_annotation_writes
            .iter()
            .position(|(document, cancel)| {
                reply.document_id.as_deref() == Some(document) && cancel.same_request(&reply.cancel)
            })
        {
            self.retired_annotation_writes.remove(index);
            if let Err(error) = reply.result {
                self.error = Some(error.to_string());
            }
            self.reader.annotations.refresh(document.as_deref(), None);
            if let Some(view) = &mut self.reference {
                view.pane.annotations.refresh(document.as_deref(), None);
            }
            self.invalidate_annotation_marks(document.as_deref());
            return;
        }
        let reference = self
            .reference
            .as_ref()
            .is_some_and(|view| view.pane.annotations.owns(&reply));
        let writing = if reference {
            self.reference
                .as_ref()
                .unwrap()
                .pane
                .annotations
                .writing_reply(&reply)
        } else {
            self.reader.annotations.writing_reply(&reply)
        };
        if writing {
            self.reader
                .annotations
                .refresh(document.as_deref(), Some(&reply.cancel));
            if let Some(view) = &mut self.reference {
                view.pane
                    .annotations
                    .refresh(document.as_deref(), Some(&reply.cancel));
            }
        }
        let error = if reference {
            self.reference
                .as_mut()
                .unwrap()
                .pane
                .annotations
                .accept(reply)
        } else {
            self.reader.annotations.accept(reply)
        };
        if let Some(error) = error {
            self.error = Some(error);
        }
        if writing {
            self.invalidate_annotation_marks(document.as_deref());
        }
    }
}
