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
mod conflict_preview;
pub(super) use conflict_preview::ConflictPreview;
mod editor_view;
mod page_note_recovery;
mod pane;
mod reference_editor_preview;
pub(super) use reference_editor_preview::NotesPreview;
pub(super) mod export;
mod filters;
mod history;
mod history_preview;
pub(super) mod paint;
pub(super) mod panel;
mod reference_preview;
mod selection_note_recovery;
mod stored_note_recovery;
mod tags;
pub(super) use editor_view::{NotesAction, NotesContext};
pub(super) use reference_preview::MarksPreview;

pub(super) enum RecoveredDraft {
    Stored(yamantaka_domain::annotations::stored_note_draft::StoredNoteDraft),
    Selection(yamantaka_domain::annotations::selection_note_draft::SelectionNoteDraft),
}
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
    RecoveredStored {
        record: Arc<AnnotationRecord>,
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
    fn stored_record(&self) -> Option<&AnnotationRecord> {
        match &self.source {
            EditorSource::Stored { page, index } => Some(&page.entries()[*index]),
            EditorSource::RecoveredStored { record } => Some(record),
            _ => None,
        }
    }
    fn stored(&self) -> Option<&AnnotationRecordData> {
        self.stored_record().map(AnnotationRecord::data)
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
            EditorSource::RecoveredStored { record } => {
                record.data().values.selected_text.as_deref().unwrap_or("")
            }
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
    recovered: Option<RecoveredDraft>,
    error: Option<String>,
    saved: bool,
    stale: bool,
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
    pub(in crate::app) fn history_available(&self, redo: bool) -> bool {
        !self.stale
            && !self.editing()
            && self.pending.is_none()
            && self.page.as_ref().is_some_and(|page| {
                if redo {
                    page.history.redo > 0
                } else {
                    page.history.undo > 0
                }
            })
    }
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
        self.editor.is_some() || self.recovered.is_some()
    }
    pub(in crate::app) fn discard_editor(&mut self) {
        self.editor = None;
        self.recovered = None;
    }
    pub(in crate::app) fn unsaved_editor(&self) -> bool {
        self.recovered.is_some()
            || self.editor.as_ref().is_some_and(|editor| {
                editor.stored().is_none_or(|record| {
                    editor.kind != record.values.kind
                        || editor.color != record.values.color
                        || editor.review != record.values.review_state
                        || editor.note != record.values.note.as_deref().unwrap_or("")
                        || editor.tags != record.tags.join(", ")
                })
            })
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
        self.stale = false;
        self.document = Some(document.into());
        self.recovered = None;
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
    pub(in crate::app) fn owns(&self, reply: &AnnotationReply) -> bool {
        self.pending.as_ref().is_some_and(|pending| {
            reply.document_id.as_deref() == Some(&pending.document)
                && pending.cancel.same_request(&reply.cancel)
        })
    }
    pub(super) fn writing_reply(&self, reply: &AnnotationReply) -> bool {
        self.owns(reply) && self.writing()
    }
    pub(in crate::app) fn cancel_query(&mut self) {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| !pending.writing)
        {
            self.pending.take().unwrap().cancel.cancel();
            self.stale = true;
        }
    }
    pub(in crate::app) fn retire_write(&mut self) -> Option<(String, Cancellation)> {
        self.cancel_query();
        let pending = self.pending.take()?;
        Some((pending.document, pending.cancel))
    }
    pub(in crate::app) fn refresh(
        &mut self,
        document: Option<&str>,
        origin: Option<&Cancellation>,
    ) {
        if document.is_some_and(|document| self.document.as_deref() != Some(document))
            || origin.is_some_and(|origin| {
                self.pending
                    .as_ref()
                    .is_some_and(|pending| pending.cancel.same_request(origin))
            })
        {
            return;
        }
        self.cancel_query();
        self.stale = true;
    }
    pub(in crate::app) fn prepare(
        &mut self,
        runtime: &Runtime,
        governor: &MemoryGovernor,
        document: &str,
    ) {
        self.bind(document);
        if (self.page.is_none() || self.stale && !self.editing())
            && self.pending.is_none()
            && self.error.is_none()
        {
            let offset = self.page.as_ref().map_or(0, |page| page.offset);
            self.command(runtime, governor, None, offset);
        }
    }
    pub(in crate::app) fn command(
        &mut self,
        runtime: &Runtime,
        governor: &MemoryGovernor,
        change: Option<AnnotationMutation>,
        offset: usize,
    ) {
        let Some(request) = self.request(governor, change, offset) else {
            return;
        };
        let document_id = request.query.document_id.clone();
        let cancel = request.cancel.clone();
        if let Err(error) = runtime.submit(Action::Annotation(Box::new(request))) {
            self.accept(AnnotationReply {
                document_id,
                result: Err(error),
                cancel,
            });
        }
    }
    pub(in crate::app) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"history":self.page.as_ref().map(|page|serde_json::json!({"undo":page.history.undo,"redo":page.history.redo})),"document":self.document,"pending":self.pending.is_some(),"writing":self.writing(),"editing":self.editing(),"saved":self.saved,"stale":self.stale,"filterCount":self.filters.count(),"selected":self.selected.count(),"offset":self.page.as_ref().map(|page|page.offset),"total":self.page.as_ref().map(|page|page.total),"entries":self.page.as_ref().map(|page|page.entries().iter().map(|record|serde_json::json!({"id":record.data().id,"page":record.data().values.page_index,"note":record.data().values.note,"tags":record.data().tags,"review":record.data().values.review_state,"kind":record.data().values.kind})).collect::<Vec<_>>())})
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
                self.stale = !pending.writing
                    && self
                        .editor
                        .as_ref()
                        .and_then(AnnotationEditor::stored_record)
                        .is_some_and(|old| {
                            page.entries()
                                .iter()
                                .find(|record| record.data().id == old.data().id)
                                .map_or(self.stale, |record| old.revision() != record.revision())
                        });
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
            Err(error) => {
                if matches!(error, Failure::StaleAnnotation) {
                    self.stale = true;
                }
                self.error = Some(error.to_string());
            }
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
            self.stale = self.stale
                && self
                    .editor
                    .as_ref()
                    .is_some_and(|editor| editor.stored().is_some());
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
        if let Some(record) = editor.stored_record() {
            return Ok(AnnotationMutation::Edit {
                id: record.data().id.clone(),
                expected: record.revision().clone(),
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
        self.reader.begin_page_note();
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
        if mutation.is_some()
            && (self.global_notes.writing()
                || self.library_tags.writing()
                || self
                    .reference
                    .as_ref()
                    .is_some_and(|reference| reference.pane.annotations.writing()))
        {
            return;
        }
        self.reader
            .annotations
            .command(&self.runtime, &self.governor, mutation, offset);
    }
    pub(super) fn annotate_selection(&mut self) {
        if self.global_notes.writing() || self.library_tags.writing() {
            return;
        }
        self.reader.annotate_selection(&self.governor);
        if let Some(error) = &self.reader.annotations.error {
            self.error = Some(error.clone());
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
