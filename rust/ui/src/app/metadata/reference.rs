use super::*;
use yamantaka_application::{
    cover_source::CoverSource,
    metadata_source::{MetadataSourceReply, MetadataSourceRequest},
};
use yamantaka_domain::{Failure, RenderedPage, source_revision::CataloguedFile};

#[derive(Clone, PartialEq, Eq)]
struct ReferenceScope {
    file: CataloguedFile,
    page: u32,
}
pub(super) struct MetadataReference {
    pub field: MetadataField,
    pub pending: Option<Cancellation>,
    pub image: Option<PageImage>,
    pub page: Option<u32>,
    pub aspect: Option<f32>,
    pub error: Option<Failure>,
    pub reveal: bool,
    generation: u64,
    scope: Option<ReferenceScope>,
}
impl Default for MetadataReference {
    fn default() -> Self {
        Self {
            field: MetadataField::Title,
            pending: None,
            image: None,
            page: None,
            aspect: None,
            error: None,
            reveal: false,
            generation: 0,
            scope: None,
        }
    }
}
impl MetadataReference {
    pub(super) fn retry(&mut self) {
        self.scope = None;
    }
}
impl Drop for MetadataReference {
    fn drop(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
    }
}

impl MetadataEditor {
    pub(super) fn ready(&self) -> bool {
        self.snapshot.is_some() && self.pending.is_none() && self.source_pending.is_none()
    }
    pub(super) fn source_request(&mut self, field: MetadataField) -> Option<MetadataSourceRequest> {
        if !self.ready() || !self.clean_draft() || self.document.missing || self.error.is_some() {
            return None;
        }
        let expected = self.snapshot.as_ref()?;
        expected.evidence(field)?;
        let cancel = Cancellation::default();
        self.source_pending = Some((field, cancel.clone()));
        Some(MetadataSourceRequest {
            expected: Box::new(expected.clone()),
            field,
            cancel,
        })
    }
}
impl YamantakaApp {
    pub(super) fn prepare_metadata_reference(&mut self) {
        let Some(editor) = self.metadata.as_mut() else {
            return;
        };
        let Some(snapshot) = &editor.snapshot else {
            return;
        };
        if snapshot.file.missing || snapshot.file.page_count == 0 {
            return;
        }
        let scope = ReferenceScope {
            file: snapshot.file.clone(),
            page: snapshot
                .evidence(editor.reference.field)
                .map_or(0, |e| e.page),
        };
        if editor.reference.scope.as_ref() == Some(&scope) {
            return;
        }
        if let Some(old) = editor.reference.pending.take() {
            old.cancel();
            self.runtime.wake();
        }
        if editor
            .reference
            .scope
            .as_ref()
            .is_some_and(|old| old.file != scope.file)
        {
            editor.reference.image = None;
            editor.reference.page = None;
        }
        let Some(generation) = editor.reference.generation.checked_add(1) else {
            return;
        };
        let cancel = Cancellation::default();
        let source = if snapshot.evidence(editor.reference.field).is_some() {
            CoverSource::metadata_evidence(&editor.document, snapshot, editor.reference.field)
        } else {
            CoverSource::capture(&editor.document)
        };
        let action = Action::PreviewCover(source, scope.page, generation, cancel.clone());
        editor.reference.scope = Some(scope);
        editor.reference.error = None;
        match self.runtime.submit(action) {
            Ok(()) => {
                editor.reference.generation = generation;
                editor.reference.pending = Some(cancel);
            }
            Err(error) => editor.reference.error = Some(error),
        }
    }
    pub(in crate::app) fn accept_metadata_reference(
        &mut self,
        _ctx: &egui::Context,
        id: &str,
        result: &yamantaka_domain::Result<Arc<RenderedPage>>,
        cancel: &Cancellation,
    ) -> bool {
        let Some(editor) = &mut self.metadata else {
            return false;
        };
        if editor.document.id != id
            || !editor
                .reference
                .pending
                .as_ref()
                .is_some_and(|token| token.same_request(cancel))
        {
            return false;
        }
        editor.reference.pending = None;
        if cancel.is_cancelled() {
            return true;
        }
        let result = (|| {
            let page = result.as_ref().map_err(Clone::clone)?;
            if page.key.document_id != id
                || page.key.generation != editor.reference.generation
                || editor
                    .reference
                    .scope
                    .as_ref()
                    .is_none_or(|scope| scope.page != page.key.page)
            {
                return Err(Failure::Invalid("Metadata preview scope changed".into()));
            }
            let presenter = self
                .presenter
                .as_mut()
                .ok_or_else(|| Failure::Unsupported("Metadata preview compositor".into()))?;
            let texture = presenter.upload(page)?;
            Ok((
                PageImage::Native(texture),
                page.key.page,
                page.width as f32 / page.height as f32,
            ))
        })();
        match result {
            Ok((image, page, aspect)) => {
                editor.reference.aspect = Some(aspect);
                editor.reference.image = Some(image);
                editor.reference.page = Some(page);
                editor.reference.error = None;
            }
            Err(error) => editor.reference.error = Some(error),
        }
        true
    }
    pub(super) fn request_metadata_source(&mut self, field: MetadataField) {
        let Some(request) = self
            .metadata
            .as_mut()
            .and_then(|editor| editor.source_request(field))
        else {
            return;
        };
        let document = request.expected.document.clone();
        let cancel = request.cancel.clone();
        if let Err(error) = self
            .runtime
            .submit(Action::MetadataSource(Box::new(request)))
        {
            self.accept_metadata_source(MetadataSourceReply {
                document,
                result: Err(error),
                cancel,
            });
        }
    }
    pub(in crate::app) fn accept_metadata_source(&mut self, reply: MetadataSourceReply) {
        let Some(editor) = self.metadata.as_mut() else {
            return;
        };
        let Some((field, cancel)) = &editor.source_pending else {
            return;
        };
        if editor.document.id != reply.document || !cancel.same_request(&reply.cancel) {
            return;
        }
        let field = *field;
        editor.source_pending = None;
        if reply.cancel.is_cancelled() {
            return;
        }
        let location = match reply.result {
            Ok(location) => location,
            Err(_) => {
                editor.error=Some(if !self.english{"La fuente ha cambiado o no está disponible. Vuelve a consultar los metadatos."}else{"The source changed or is unavailable. Reload metadata before opening it."}.into());
                return;
            }
        };
        if editor.snapshot.as_ref().is_none_or(|s| {
            s.evidence(field).is_none_or(|e| e.page != location.page)
                || s.document != location.document.id
                || s.file.path != location.document.path
                || s.file.fingerprint != location.document.fingerprint
                || s.file.page_count != location.document.page_count
        }) || location.document.missing
            || location.page >= location.document.page_count
        {
            return;
        }
        let id = location.document.id.clone();
        let inherited = self
            .reader
            .active
            .as_ref()
            .filter(|(document, _)| document.id == id)
            .and(self.reader.navigation.clone());
        self.sync_tab_context();
        self.tabs.cancel();
        if self.open_candidate(location.document) {
            self.open_intent.set_page(id.clone(), location.page);
            if let Some(navigation) = inherited {
                self.open_intent.inherit_navigation(id, navigation);
            }
            if let Some(editor) = self.metadata.as_mut() {
                editor.source_opened = true;
            }
            self.runtime.wake();
        }
    }
}
