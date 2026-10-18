use super::*;
mod form;
mod panel;
mod reference;
mod view;
use reference::MetadataReference;
use yamantaka_application::metadata_review::{MetadataChange, MetadataReply, MetadataRequest};
use yamantaka_domain::metadata_review::MetadataField;
use yamantaka_domain::metadata_review::{ManualMetadata, MetadataMutation, MetadataReview};

pub(super) struct MetadataEditor {
    document: Document,
    snapshot: Option<MetadataReview>,
    reference: MetadataReference,
    source_pending: Option<(MetadataField, Cancellation)>,
    pending: Option<(Cancellation, bool)>,
    title: String,
    author: String,
    error: Option<String>,
    confirm_reset: bool,
    saved: bool,
    analyzing: bool,
    analyzed: bool,
    source_opened: bool,
}
impl Drop for MetadataEditor {
    fn drop(&mut self) {
        if let Some((_, cancel)) = &self.source_pending {
            cancel.cancel();
        }
        if let Some((cancel, false)) = &self.pending {
            cancel.cancel();
        }
    }
}
impl MetadataEditor {
    fn clean_draft(&self) -> bool {
        self.snapshot
            .as_ref()
            .is_some_and(|s| self.title == s.title && self.author == s.author)
    }
    fn reanalyze(&mut self) -> Option<(Box<MetadataReview>, Cancellation)> {
        if self.pending.is_some()
            || self.source_pending.is_some()
            || !self.clean_draft()
            || self.document.missing
        {
            return None;
        }
        let expected = Box::new(self.snapshot.clone()?);
        let cancel = Cancellation::default();
        self.pending = Some((cancel.clone(), true));
        self.analyzing = true;
        self.analyzed = false;
        self.saved = false;
        self.error = None;
        Some((expected, cancel))
    }
    pub(super) fn capture_ready(&self) -> bool {
        self.snapshot.is_some()
            && self.pending.is_none()
            && self.source_pending.is_none()
            && self.reference.pending.is_none()
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"document":self.document.id,"title":self.title,"author":self.author,"snapshot":self.snapshot.as_ref().map(|s|serde_json::json!({"title":s.title,"author":s.author,"titleOverride":s.title_override,"authorOverride":s.author_override,"suggestedTitle":s.suggested_title,"suggestedAuthor":s.suggested_author})),"saved":self.saved,"analyzing":self.analyzing,"analyzed":self.analyzed,"error":self.error,"pending":self.pending.is_some(),"sourcePending":self.source_pending.is_some(),"sourceOpened":self.source_opened,"reference":{"page":self.reference.page,"pending":self.reference.pending.is_some(),"error":self.reference.error.is_some(),"evidencePage":self.snapshot.as_ref().and_then(|s|s.evidence(self.reference.field)).map(|e|e.page)}})
    }
    fn read(&mut self) -> MetadataRequest {
        if let Some((_, cancel)) = self.source_pending.take() {
            cancel.cancel();
        }
        if let Some((cancel, false)) = self.pending.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        self.pending = Some((cancel.clone(), false));
        self.error = None;
        self.saved = false;
        self.analyzed = false;
        MetadataRequest {
            document: self.document.id.clone(),
            change: MetadataChange::Read,
            cancel,
        }
    }
    fn apply(&mut self, mutation: MetadataMutation) -> Option<MetadataRequest> {
        if self.pending.is_some() || self.source_pending.is_some() {
            return None;
        }
        let expected = self.snapshot.clone()?;
        let cancel = Cancellation::default();
        self.pending = Some((cancel.clone(), true));
        self.error = None;
        self.saved = false;
        self.analyzed = false;
        Some(MetadataRequest {
            document: self.document.id.clone(),
            change: MetadataChange::Apply {
                expected: Box::new(expected),
                mutation,
            },
            cancel,
        })
    }
    fn accept(&mut self, reply: MetadataReply) -> Option<MetadataReview> {
        if self.document.id != reply.document
            || self.pending.as_ref().is_none_or(|(cancel, writing)| {
                !cancel.same_request(&reply.cancel) || *writing != reply.writing
            })
        {
            return None;
        }
        self.pending = None;
        let analyzed = std::mem::take(&mut self.analyzing);
        match reply.result {
            Ok(snapshot) => {
                self.document.path = snapshot.file.path.clone();
                self.document.fingerprint = snapshot.file.fingerprint.clone();
                self.document.page_count = snapshot.file.page_count;
                self.document.missing = snapshot.file.missing;
                self.title = snapshot.title.clone();
                self.author = snapshot.author.clone();
                self.snapshot = Some(snapshot.clone());
                self.confirm_reset = false;
                self.error = None;
                self.saved = reply.writing && !analyzed;
                self.analyzed = reply.writing && analyzed;
                reply.writing.then_some(snapshot)
            }
            Err(error) => {
                self.error = Some(error.to_string());
                None
            }
        }
    }
}
impl YamantakaApp {
    pub fn preview_metadata(&mut self, action: &'static str) {
        if self.demo && self.capture.is_some() {
            self.metadata_preview = Some((action, 0));
        }
    }
    pub(super) fn advance_metadata_preview(&mut self) {
        let Some((action, stage)) = self.metadata_preview else {
            return;
        };
        if stage == 0 {
            if let Some(document) = self.documents.iter().find(|d| d.id == "demo-0").cloned() {
                self.begin_metadata_editor(document);
                self.metadata_preview = Some((action, 1));
            }
        } else if stage == 1
            && self
                .metadata
                .as_ref()
                .is_some_and(MetadataEditor::capture_ready)
        {
            if action == "open" {
                self.metadata_preview = Some((action, 3));
                return;
            }
            if matches!(
                action,
                "reanalyze"
                    | "source"
                    | "source-return"
                    | "source-draft"
                    | "source-stale"
                    | "reference"
            ) {
                if let Some((expected, cancel)) =
                    self.metadata.as_mut().and_then(MetadataEditor::reanalyze)
                {
                    self.metadata_preview = Some((action, 2));
                    self.send_metadata_reanalysis(expected, cancel);
                }
                return;
            }
            let mutation = match action {
                "dismiss" => MetadataMutation::DismissSuggestions,
                "reset" => MetadataMutation::ResetOverrides,
                _ => MetadataMutation::Save(
                    ManualMetadata::new("Geometry of Light", "Ada Lovelace").unwrap(),
                ),
            };
            if matches!(mutation, MetadataMutation::Save(_))
                && let Some(editor) = self.metadata.as_mut()
            {
                editor.title = "Geometry of Light".into();
                editor.author = "Ada Lovelace".into();
            }
            if let Some(request) = self
                .metadata
                .as_mut()
                .and_then(|editor| editor.apply(mutation))
            {
                self.metadata_preview = Some((action, 2));
                self.send_metadata(request);
            }
        } else if stage == 2
            && self
                .metadata
                .as_ref()
                .is_some_and(MetadataEditor::capture_ready)
        {
            if matches!(action, "source" | "source-return" | "source-stale") {
                if action == "source-stale"
                    && let Some(snapshot) = self
                        .metadata
                        .as_ref()
                        .and_then(|editor| editor.snapshot.clone())
                {
                    self.send(Action::Metadata(Box::new(MetadataRequest {
                        document: snapshot.document.clone(),
                        change: MetadataChange::Apply {
                            expected: Box::new(snapshot),
                            mutation: MetadataMutation::Save(
                                ManualMetadata::new("Newer title", "Newer author").unwrap(),
                            ),
                        },
                        cancel: Cancellation::default(),
                    })));
                }
                self.request_metadata_source(MetadataField::Title);
                self.metadata_preview = Some((action, 4));
            } else {
                if action == "reference"
                    && let Some(editor) = self.metadata.as_mut()
                {
                    editor.reference.reveal = true;
                }
                if action == "source-draft"
                    && let Some(editor) = self.metadata.as_mut()
                {
                    editor.title = "My unsaved geometry".into();
                }
                self.metadata_preview = Some((action, 3));
            }
        } else if stage == 4
            && action == "source-stale"
            && self
                .metadata
                .as_ref()
                .is_some_and(|editor| editor.error.is_some() && editor.source_pending.is_none())
        {
            self.metadata_preview = Some((action, 3));
        } else if stage == 4
            && self.metadata.as_ref().is_some_and(|editor| {
                self.active
                    .as_ref()
                    .is_some_and(|(doc, _)| doc.id == editor.document.id)
                    && editor
                        .snapshot
                        .as_ref()
                        .and_then(|s| s.title_evidence.as_ref())
                        .is_some_and(|e| e.page == self.page)
            })
            && !self.loading
            && !self.opening.busy()
            && !self.reader_pages.is_empty()
        {
            if action == "source-return" {
                self.show_surface(Surface::Library);
            }
            self.metadata_preview = Some((action, 3));
        }
    }
    pub(super) fn metadata_return_available(&self) -> bool {
        self.metadata.as_ref().is_some_and(|editor| {
            editor.source_opened
                && self
                    .active
                    .as_ref()
                    .is_some_and(|(document, _)| document.id == editor.document.id)
        })
    }
    pub(super) fn metadata_source_context(&self) -> bool {
        self.metadata
            .as_ref()
            .is_some_and(|editor| editor.source_opened)
    }
    pub(super) fn begin_metadata_editor(&mut self, document: Document) {
        let mut editor = MetadataEditor {
            title: document.display_title().into(),
            author: document.author.clone().unwrap_or_default(),
            document,
            snapshot: None,
            reference: MetadataReference::default(),
            source_pending: None,
            pending: None,
            error: None,
            confirm_reset: false,
            saved: false,
            analyzing: false,
            analyzed: false,
            source_opened: false,
        };
        let request = editor.read();
        self.metadata = Some(editor);
        self.send_metadata(request);
    }
    fn send_metadata(&mut self, request: MetadataRequest) {
        let document = request.document.clone();
        let writing = matches!(request.change, MetadataChange::Apply { .. });
        let cancel = request.cancel.clone();
        if let Err(error) = self.runtime.submit(Action::Metadata(Box::new(request))) {
            self.accept_metadata(MetadataReply {
                document,
                writing,
                cancel,
                result: Err(error),
            });
        }
    }
    fn send_metadata_reanalysis(&mut self, expected: Box<MetadataReview>, cancel: Cancellation) {
        let document = expected.document.clone();
        if let Err(error) = self
            .runtime
            .submit(Action::ReanalyzeMetadata(expected, cancel.clone()))
        {
            self.accept_metadata(MetadataReply {
                document,
                writing: true,
                cancel,
                result: Err(error),
            });
        }
    }
    pub(super) fn accept_metadata(&mut self, reply: MetadataReply) {
        if let Some(saved) = self
            .metadata
            .as_mut()
            .and_then(|editor| editor.accept(reply))
        {
            for document in self
                .documents
                .iter_mut()
                .chain(self.active.iter_mut().map(|(document, _)| document))
            {
                if document.id == saved.document {
                    document.title = Some(saved.title.clone());
                    document.author = (!saved.author.is_empty()).then(|| saved.author.clone());
                }
            }
            self.refresh();
        }
    }
}
