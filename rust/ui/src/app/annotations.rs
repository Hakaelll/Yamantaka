use super::*;
use yamantaka_application::annotations::{
    AnnotationMutation, AnnotationOperation, AnnotationReply, AnnotationRequest,
};
use yamantaka_domain::{
    Failure,
    annotations::{repository::*, *},
    pdf_text::PdfPageText,
    reader_preferences::PageRotation,
    text_selection::TextSelection,
};
mod panel;
#[cfg(test)]
mod tests;

pub(super) enum EditorSource {
    Selection {
        page: Arc<PdfPageText>,
        range: TextSelection,
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
            EditorSource::Selection { page, range, .. } => page
                .selected_text(range.start().character()..range.end().character())
                .unwrap_or(""),
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
        let EditorSource::Selection {
            page,
            range,
            fingerprint,
            rotation,
        } = &editor.source
        else {
            unreachable!()
        };
        let selected = range
            .page(page)?
            .ok_or_else(|| Failure::Invalid("The selected passage is unavailable".into()))?;
        let document = self
            .document
            .as_deref()
            .ok_or_else(|| Failure::Invalid("Annotation document is unavailable".into()))?;
        let draft = AnnotationDraft::from_selection(
            AnnotationSource {
                document_id: document,
                fingerprint,
                rotation: *rotation,
            },
            &selected,
            editor.kind,
            editor.color,
            Some(&editor.note),
            governor,
        )?;
        if matches!(
            draft.placement(),
            AnnotationPlacement::PageNoteFallback { .. }
        ) && !editor.confirmed
        {
            return Err(Failure::Invalid(
                "Confirm the page-note fallback before saving".into(),
            ));
        }
        Ok(AnnotationMutation::Create {
            draft: Arc::new(draft),
            confirm_page_note: editor.confirmed,
            tags: Vec::new(),
        })
    }
    fn fallback(&self) -> bool {
        self.editor.as_ref().is_some_and(|editor| {
            let EditorSource::Selection { page, range, .. } = &editor.source else {
                return false;
            };
            editor.kind != AnnotationKind::PageComment
                && range.page(page).ok().flatten().is_some_and(|selected| {
                    selected.geometry_coverage()
                        != yamantaka_domain::text_selection::GeometryCoverage::Complete
                        || !selected.ink_regions().all(|region| {
                            region.max()[0] - region.min()[0] > 0.0001
                                && region.max()[1] - region.min()[1] > 0.0001
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
    pub fn preview_annotations(&mut self, save: bool) {
        if self.demo && self.capture.is_some() {
            self.preview_selection(false);
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
        if matches!(self.annotation_preview, 1 | 2)
            && self.annotations.editor.is_none()
            && self.selection_preview == selection::SelectionPreview::Complete
            && self.selection.preview_ready()
        {
            self.annotate_selection();
            if let Some(editor) = &mut self.annotations.editor {
                editor.note = "A passage to revisit during study.".into();
            }
        }
        if self.annotations.editor.is_some() && self.annotations.pending.is_none() {
            if self.annotation_preview == 1 {
                self.annotation_preview = 0;
            } else if self.annotation_preview == 2 {
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
        match self.selection.annotation_source() {
            Ok((range, page)) => {
                self.annotations.bind(&document.id);
                self.annotations.editor = Some(AnnotationEditor {
                    source: EditorSource::Selection {
                        page,
                        range,
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
