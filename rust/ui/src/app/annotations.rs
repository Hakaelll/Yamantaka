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
mod bulk;
pub(super) mod export;
mod filters;
pub(super) mod paint;
pub(super) mod panel;
mod reference_preview;
mod tags;
pub(super) use reference_preview::MarksPreview;

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
    tags: String,
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
    filters: filters::Filters,
    filter_draft: filters::Filters,
    filters_open: bool,
    filter_preview: Option<bool>,
    selected: bulk::BulkSelection,
    bulk_preview: Option<bool>,
    bulk_tag: String,
    bulk_tag_preview: Option<bool>,
}
impl AnnotationState {
    pub(in crate::app) fn writing(&self) -> bool {
        self.pending.as_ref().is_some_and(|p| p.writing)
    }
    pub(in crate::app) fn invalidate_records(&mut self) {
        if self.editing() || self.writing() {
            return;
        }
        if let Some(pending) = self.pending.take() {
            pending.cancel.cancel();
        }
        self.page = None;
        self.error = None;
        self.saved = false;
        self.selected = Default::default();
        self.bulk_tag.clear();
        self.revision = self.revision.wrapping_add(1);
    }
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
        self.selected = Default::default();
        self.bulk_tag.clear();
        self.filters = Default::default();
        self.filter_draft = Default::default();
        self.filters_open = false;
        self.page = None;
        self.editor = None;
        self.error = None;
        self.saved = false;
    }
    pub(super) fn writing_reply(&self, reply: &AnnotationReply) -> bool {
        self.pending.as_ref().is_some_and(|pending| {
            pending.writing
                && reply.document_id.as_deref() == Some(&pending.document)
                && pending.cancel.same_request(&reply.cancel)
        })
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
                self.selected = Default::default();
                self.bulk_tag.clear();
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
        if !writing {
            self.bulk_tag.clear();
            self.selected = Default::default();
        }
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
            query: self.filters.query(document, offset),
            governor: governor.clone(),
            cancel,
        })
    }
    fn mutation(&self, governor: &MemoryGovernor) -> yamantaka_domain::Result<AnnotationMutation> {
        let editor = self
            .editor
            .as_ref()
            .ok_or_else(|| Failure::Invalid("No annotation is being edited".into()))?;
        let tag_values = editor.tag_changes()?;
        if let Some(record) = editor.stored() {
            return Ok(AnnotationMutation::Edit {
                id: record.id.clone(),
                kind: editor.kind,
                color: editor.color,
                note: Some(editor.note.clone()),
                review: editor.review,
                tags: tag_values,
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
                tags: tag_values.unwrap_or_default(),
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
            tags: tag_values.unwrap_or_default(),
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
        if self.global_notes.writing() || self.library_tags.writing() {
            return;
        }
        let Some((document, info)) = &self.reader.active else {
            return;
        };
        if self.reader.page as usize >= info.pages.len() {
            return;
        }
        self.reader.annotations.bind(&document.id);
        self.reader.annotations.editor = Some(AnnotationEditor {
            source: EditorSource::PageNote {
                page: self.reader.page,
                fingerprint: document.fingerprint.clone(),
                rotation: self
                    .reader
                    .reader_preferences
                    .as_ref()
                    .map_or(PageRotation::Deg0, |prefs| prefs.page_rotation),
            },
            kind: AnnotationKind::PageComment,
            color: AnnotationColor::Violet,
            note: String::new(),
            tags: String::new(),
            review: AnnotationReviewState::Pending,
            confirmed: false,
        });
        self.reader.annotations.error = None;
        self.reader.panel = true;
        self.reader.reader_panel_tab = outline::ReaderPanelTab::Annotations;
    }
    pub fn preview_annotation_list(&mut self) {
        if self.demo && self.capture.is_some() {
            self.reader.panel = true;
            self.reader.reader_panel_tab = outline::ReaderPanelTab::Annotations;
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
        serde_json::json!({"studyTaggedResident":self.reader.annotations.page.as_ref().map_or(0,|page|page.entries().iter().filter(|record|record.data().tags.iter().any(|tag|tag=="Study")).count()),"selected":self.reader.annotations.selected.count(),"acceptedResident":self.reader.annotations.page.as_ref().map_or(0,|page|page.entries().iter().filter(|record|record.data().values.review_state==AnnotationReviewState::Accepted).count()),"filters":{"active":self.reader.annotations.filters.count(),"text":self.reader.annotations.filters.text,"tag":self.reader.annotations.filters.tag,"kind":self.reader.annotations.filters.kind,"review":self.reader.annotations.filters.review},"resident":self.reader.annotations.page.as_ref().map_or(0,|page|page.entries().len()),"total":self.reader.annotations.page.as_ref().map_or(0,|page|page.total),"pending":self.reader.annotations.pending.is_some(),"editing":self.reader.annotations.editor.is_some(),"fallback":self.reader.annotations.fallback(),"saved":self.reader.annotations.saved})
    }
    pub(super) fn apply_annotation_preview(&mut self) {
        if self.annotation_preview == 0 {
            return;
        }
        if matches!(self.annotation_preview, 4 | 5) && self.reader.annotations.editor.is_none() {
            self.begin_page_note();
            if let Some(editor) = &mut self.reader.annotations.editor {
                editor.note = "An observation from this scanned page.".into();
                editor.tags = "Study, Optics".into();
            }
        }
        if matches!(self.annotation_preview, 1 | 2)
            && self.reader.annotations.editor.is_none()
            && self.selection_preview == selection::SelectionPreview::Complete
            && self.reader.selection.preview_ready()
        {
            if self.annotation_multi_preview && !self.reader.selection.extend_preview(1) {
                return;
            }
            self.annotate_selection();
            if let Some(editor) = &mut self.reader.annotations.editor {
                editor.note = "A passage to revisit during study.".into();
                editor.tags = "Study, Optics".into();
            }
        }
        if self.reader.annotations.editor.is_some() && self.reader.annotations.pending.is_none() {
            if matches!(self.annotation_preview, 1 | 4) {
                self.annotation_preview = 0;
            } else if matches!(self.annotation_preview, 2 | 5) {
                match self.reader.annotations.mutation(&self.governor) {
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
            && self.reader.annotations.pending.is_none()
            && self.reader.annotations.saved
        {
            self.reader.selection.clear();
            self.selection_preview = selection::SelectionPreview::Off;
            self.annotation_preview = 0;
        }
    }
    fn annotation_command(&mut self, mutation: Option<AnnotationMutation>, offset: usize) {
        if mutation.is_some() && (self.global_notes.writing() || self.library_tags.writing()) {
            return;
        }
        let Some(request) = self
            .reader
            .annotations
            .request(&self.governor, mutation, offset)
        else {
            return;
        };
        let document_id = request.query.document_id.clone();
        let cancel = request.cancel.clone();
        if let Err(error) = self.runtime.submit(Action::Annotation(Box::new(request))) {
            self.reader.annotations.accept(AnnotationReply {
                document_id,
                result: Err(error),
                cancel,
            });
        }
    }
    pub(super) fn annotate_selection(&mut self) {
        if self.global_notes.writing() || self.library_tags.writing() {
            return;
        }
        let Some((document, _)) = &self.reader.active else {
            return;
        };
        match self.reader.selection.annotation_source(&self.governor) {
            Ok(snapshot) => {
                self.reader.annotations.bind(&document.id);
                self.reader.annotations.editor = Some(AnnotationEditor {
                    source: EditorSource::Selection {
                        snapshot,
                        fingerprint: document.fingerprint.clone(),
                        rotation: self
                            .reader
                            .reader_preferences
                            .as_ref()
                            .map_or(PageRotation::Deg0, |prefs| prefs.page_rotation),
                    },
                    kind: AnnotationKind::Highlight,
                    color: AnnotationColor::Violet,
                    note: String::new(),
                    tags: String::new(),
                    review: AnnotationReviewState::Pending,
                    confirmed: false,
                });
                self.reader.annotations.error = None;
                self.reader.panel = true;
                self.reader.reader_panel_tab = outline::ReaderPanelTab::Annotations;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}

impl YamantakaApp {
    pub(in crate::app) fn invalidate_annotation_marks(&mut self, document: Option<&str>) {
        self.reader.annotation_paint.invalidate_document(document);
        if let Some(reference) = &mut self.reference {
            reference
                .pane
                .annotation_paint
                .invalidate_document(document);
        }
    }
}
