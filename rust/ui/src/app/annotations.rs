use super::*;
use yamantaka_application::annotations::{
    AnnotationMutation, AnnotationOperation, AnnotationReply, AnnotationRequest,
};
use yamantaka_domain::{
    Failure,
    annotations::{repository::*, *},
    reader_preferences::PageRotation,
    text_selection::snapshot::TextSelectionSnapshot,
};
pub(super) mod paint;
mod panel;
#[cfg(test)]
mod tests;

pub(super) enum EditorSource {
    PageNote {
        page: u32,
        fingerprint: String,
        rotation: PageRotation,
    },
    Selection {
        snapshot: TextSelectionSnapshot,
        fingerprint: String,
        rotation: PageRotation,
    },
    Stored {
        page: Arc<AnnotationPage>,
        index: usize,
    },
}
pub(super) struct AnnotationEditor {
    source: EditorSource,
    kind: AnnotationKind,
    color: AnnotationColor,
    note: String,
    review: AnnotationReviewState,
    confirmed: bool,
}
impl AnnotationEditor {
    fn stored(&self) -> Option<&AnnotationRecordData> {
        if let EditorSource::Stored { page, index } = &self.source {
            Some(page.entries()[*index].data())
        } else {
            None
        }
    }
    fn passage(&self) -> &str {
        match &self.source {
            EditorSource::Stored { page, index } => page.entries()[*index]
                .data()
                .values
                .selected_text
                .as_deref()
                .unwrap_or(""),
            EditorSource::Selection { snapshot, .. } => snapshot.text(),
            EditorSource::PageNote { .. } => "",
        }
    }
}
struct Pending {
    document: String,
    cancel: Cancellation,
    writing: bool,
}
#[derive(Default)]
pub(super) struct AnnotationState {
    document: Option<String>,
    page: Option<Arc<AnnotationPage>>,
    pending: Option<Pending>,
    editor: Option<AnnotationEditor>,
    error: Option<String>,
    saved: bool,
    pub(super) revision: u64,
}
impl AnnotationState {
    pub(super) fn editing(&self) -> bool {
        self.editor.is_some()
    }
    pub(super) fn ready(&self) -> bool {
        self.page.is_some() && self.pending.is_none()
    }
    pub(super) fn bind(&mut self, document: &str) {
        if self.document.as_deref() == Some(document) {
            return;
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| !pending.writing)
        {
            self.pending.take().unwrap().cancel.cancel();
        }
        self.document = Some(document.into());
        self.page = None;
        self.editor = None;
        self.error = None;
        self.saved = false;
    }
    pub(super) fn accept(&mut self, reply: AnnotationReply) -> Option<String> {
        let pending = self.pending.as_ref()?;
        if reply.document_id.as_deref() != Some(&pending.document)
            || !pending.cancel.same_request(&reply.cancel)
        {
            return None;
        }
        let pending = self.pending.take().unwrap();
        if self.document.as_deref() != Some(&pending.document) {
            return reply
                .result
                .err()
                .filter(|_| pending.writing)
                .map(|error| error.to_string());
        }
        match reply.result {
            Ok(page) => {
                self.page = Some(Arc::new(page));
                self.error = None;
                self.saved = pending.writing;
                if pending.writing {
                    self.revision = self.revision.wrapping_add(1);
                    self.editor = None;
                }
            }
            Err(error) => self.error = Some(error.to_string()),
        }
        None
    }
    fn request(
        &mut self,
        governor: &MemoryGovernor,
        mutation: Option<AnnotationMutation>,
        offset: usize,
    ) -> Option<AnnotationRequest> {
        if self.pending.is_some() {
            return None;
        }
        let document = self.document.clone()?;
        let cancel = Cancellation::default();
        let writing = mutation.is_some();
        let operation = mutation.map_or(AnnotationOperation::Read, |mutation| {
            AnnotationOperation::Change {
                document_id: document.clone(),
                mutation,
            }
        });
        self.pending = Some(Pending {
            document: document.clone(),
            cancel: cancel.clone(),
            writing,
        });
        self.error = None;
        self.saved = false;
        Some(AnnotationRequest {
            operation,
            query: AnnotationQuery {
                document_id: Some(document),
                offset,
                ..Default::default()
            },
            governor: governor.clone(),
            cancel,
        })
    }
    fn mutation(&self, governor: &MemoryGovernor) -> yamantaka_domain::Result<AnnotationMutation> {
        let editor = self
            .editor
            .as_ref()
            .ok_or_else(|| Failure::Invalid("No annotation is being edited".into()))?;
        if let Some(record) = editor.stored() {
            return Ok(AnnotationMutation::Edit {
                id: record.id.clone(),
                kind: editor.kind,
                color: editor.color,
                note: Some(editor.note.clone()),
                review: editor.review,
            });
        }
        if let EditorSource::PageNote {
            page,
            fingerprint,
            rotation,
        } = &editor.source
        {
            let document = self
                .document
                .as_deref()
                .ok_or_else(|| Failure::Invalid("Annotation document is unavailable".into()))?;
            if editor.note.trim().is_empty() {
                return Err(Failure::Invalid("Write a note before saving".into()));
            }
            let draft = AnnotationDraft::page_note(
                AnnotationSource {
                    document_id: document,
                    fingerprint,
                    rotation: *rotation,
                },
                *page,
                editor.color,
                Some(&editor.note),
                governor,
            )?;
            return Ok(AnnotationMutation::Create {
                draft: Arc::new(draft),
                confirm_page_note: false,
                tags: Vec::new(),
            });
        }
        let EditorSource::Selection {
            snapshot,
            fingerprint,
            rotation,
        } = &editor.source
        else {
            unreachable!()
        };
        let document = self
            .document
            .as_deref()
            .ok_or_else(|| Failure::Invalid("Annotation document is unavailable".into()))?;
        let batch = yamantaka_domain::annotations::batch::AnnotationDraftBatch::from_selection(
            AnnotationSource {
                document_id: document,
                fingerprint,
                rotation: *rotation,
            },
            snapshot.selection(),
            snapshot.pages(),
            editor.kind,
            editor.color,
            Some(&editor.note),
            governor,
        )?;
        if batch.requires_page_note_confirmation() && !editor.confirmed {
            return Err(Failure::Invalid(
                "Confirm the page-note fallback before saving".into(),
            ));
        }
        Ok(AnnotationMutation::CreateBatch {
            batch: Arc::new(batch),
            confirm_page_note: editor.confirmed,
            tags: Vec::new(),
        })
    }
    fn fallback(&self) -> bool {
        self.editor.as_ref().is_some_and(|editor| {
            let EditorSource::Selection { snapshot, .. } = &editor.source else {
                return false;
            };
            editor.kind != AnnotationKind::PageComment
                && snapshot.pages().iter().any(|page| {
                    snapshot
                        .selection()
                        .page(page)
                        .ok()
                        .flatten()
                        .is_some_and(|selected| {
                            if selected.text().trim().is_empty() {
                                return false;
                            }
                            selected.geometry_coverage()
                                != yamantaka_domain::text_selection::GeometryCoverage::Complete
                                || !selected.ink_regions().all(|region| {
                                    region.max()[0] - region.min()[0] > 0.0001
                                        && region.max()[1] - region.min()[1] > 0.0001
                                })
                        })
                })
        })
    }
}
impl Drop for AnnotationState {
    fn drop(&mut self) {
        if let Some(pending) = &self.pending
            && !pending.writing
        {
            pending.cancel.cancel();
        }
    }
}
impl YamantakaApp {
    pub(super) fn begin_page_note(&mut self) {
        let Some((document, info)) = &self.active else {
            return;
        };
        if self.page as usize >= info.pages.len() {
            return;
        }
        self.annotations.bind(&document.id);
        self.annotations.editor = Some(AnnotationEditor {
            source: EditorSource::PageNote {
                page: self.page,
                fingerprint: document.fingerprint.clone(),
                rotation: self
                    .reader_preferences
                    .as_ref()
                    .map_or(PageRotation::Deg0, |prefs| prefs.page_rotation),
            },
            kind: AnnotationKind::PageComment,
            color: AnnotationColor::Violet,
            note: String::new(),
            review: AnnotationReviewState::Pending,
            confirmed: false,
        });
        self.annotations.error = None;
        self.panel = true;
        self.reader_panel_tab = outline::ReaderPanelTab::Annotations;
    }
    pub fn preview_annotation_list(&mut self) {
        if self.demo && self.capture.is_some() {
            self.panel = true;
            self.reader_panel_tab = outline::ReaderPanelTab::Annotations;
        }
    }
    pub fn preview_page_note(&mut self, save: bool) {
        if self.demo && self.capture.is_some() {
            self.annotation_preview = if save { 5 } else { 4 };
        }
    }

    pub fn preview_annotations(&mut self, save: bool, multi: bool) {
        if self.demo && self.capture.is_some() {
            self.preview_selection(false);
            self.annotation_multi_preview = multi;
            self.annotation_preview = if save { 2 } else { 1 };
        }
    }
    pub(super) fn annotation_capture_summary(&self) -> serde_json::Value {
        serde_json::json!({"resident":self.annotations.page.as_ref().map_or(0,|page|page.entries().len()),"total":self.annotations.page.as_ref().map_or(0,|page|page.total),"pending":self.annotations.pending.is_some(),"editing":self.annotations.editor.is_some(),"fallback":self.annotations.fallback(),"saved":self.annotations.saved})
    }
    pub(super) fn apply_annotation_preview(&mut self) {
        if self.annotation_preview == 0 {
            return;
        }
        if matches!(self.annotation_preview, 4 | 5) && self.annotations.editor.is_none() {
            self.begin_page_note();
            if let Some(editor) = &mut self.annotations.editor {
                editor.note = "An observation from this scanned page.".into();
            }
        }
        if matches!(self.annotation_preview, 1 | 2)
            && self.annotations.editor.is_none()
            && self.selection_preview == selection::SelectionPreview::Complete
            && self.selection.preview_ready()
        {
            if self.annotation_multi_preview && !self.selection.extend_preview(1) {
                return;
            }
            self.annotate_selection();
            if let Some(editor) = &mut self.annotations.editor {
                editor.note = "A passage to revisit during study.".into();
            }
        }
        if self.annotations.editor.is_some() && self.annotations.pending.is_none() {
            if matches!(self.annotation_preview, 1 | 4) {
                self.annotation_preview = 0;
            } else if matches!(self.annotation_preview, 2 | 5) {
                match self.annotations.mutation(&self.governor) {
                    Ok(change) => {
                        self.annotation_command(Some(change), 0);
                        self.annotation_preview = 3;
                    }
                    Err(error) => {
                        self.error = Some(error.to_string());
                        self.annotation_preview = 0;
                    }
                }
            }
        }
        if self.annotation_preview == 3
            && self.annotations.pending.is_none()
            && self.annotations.saved
        {
            self.selection.clear();
            self.selection_preview = selection::SelectionPreview::Off;
            self.annotation_preview = 0;
        }
    }
    fn annotation_command(&mut self, mutation: Option<AnnotationMutation>, offset: usize) {
        let Some(request) = self.annotations.request(&self.governor, mutation, offset) else {
            return;
        };
        let document_id = request.query.document_id.clone();
        let cancel = request.cancel.clone();
        if let Err(error) = self.runtime.submit(Action::Annotation(Box::new(request))) {
            self.annotations.accept(AnnotationReply {
                document_id,
                result: Err(error),
                cancel,
            });
        }
    }
    pub(super) fn annotate_selection(&mut self) {
        let Some((document, _)) = &self.active else {
            return;
        };
        match self.selection.annotation_source(&self.governor) {
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
                    kind: AnnotationKind::Highlight,
                    color: AnnotationColor::Violet,
                    note: String::new(),
                    review: AnnotationReviewState::Pending,
                    confirmed: false,
                });
                self.annotations.error = None;
                self.panel = true;
                self.reader_panel_tab = outline::ReaderPanelTab::Annotations;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}
