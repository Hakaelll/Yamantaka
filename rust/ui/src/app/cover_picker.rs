use super::*;
use yamantaka_application::cover_source::CoverSource;
use yamantaka_application::manual_cover::{ManualCoverReply, ManualCoverRequest};
use yamantaka_domain::{Failure, RenderedPage, cover_selection::ManualCover};

pub(super) struct CoverPicker {
    pub(super) document: Document,
    pub(super) page: u32,
    pub(super) generation: u64,
    pub(super) loaded: bool,
    pub(super) save_failed: bool,
    pub(super) restore_failed: bool,
    pub(super) restore_committed: bool,
    pub(super) restoring: bool,
    pub(super) loading: Option<Cancellation>,
    pub(super) preview: Option<Cancellation>,
    pub(super) saving: Option<Cancellation>,
    pub(super) image: Option<PageImage>,
    pub(super) image_page: Option<u32>,
    pub(super) aspect: Option<f64>,
    pub(super) error: Option<Failure>,
}
impl Drop for CoverPicker {
    fn drop(&mut self) {
        if let Some(token) = &self.loading {
            token.cancel();
        }
        if let Some(token) = &self.preview {
            token.cancel();
        }
        if self.restoring
            && let Some(token) = &self.saving
        {
            token.cancel();
        }
    }
}
impl CoverPicker {
    pub(super) fn can_close(&self) -> bool {
        self.saving.is_none() || self.restoring
    }
    pub(super) fn ready(&self) -> bool {
        self.loading.is_none()
            && self.preview.is_none()
            && self.saving.is_none()
            && self.image_page == Some(self.page)
            && self.aspect.is_some()
            && self.error.is_none()
    }
    pub(super) fn capture_ready(&self) -> bool {
        self.loading.is_none()
            && self.preview.is_none()
            && self.saving.is_none()
            && (self.image.is_some() || self.error.is_some())
    }
}

impl YamantakaApp {
    pub(super) fn begin_cover_picker(&mut self, document: Document) {
        if self.library_selection.removing_document(&document.id) {
            return;
        }
        if self
            .cover_picker
            .as_ref()
            .is_some_and(|picker| picker.saving.is_some())
        {
            return;
        }
        let token = Cancellation::default();
        let result = self
            .runtime
            .submit(Action::CoverSourcePage(document.id.clone(), token.clone()));
        self.cover_picker = Some(CoverPicker {
            document,
            page: 0,
            generation: 0,
            loading: result.is_ok().then_some(token),
            preview: None,
            saving: None,
            loaded: false,
            save_failed: false,
            restore_failed: false,
            restore_committed: false,
            restoring: false,
            image: None,
            image_page: None,
            aspect: None,
            error: result.err(),
        });
        self.runtime.wake();
        self.surface = Surface::Library;
    }
    pub(super) fn preview_cover_picker(&mut self) {
        let Some(picker) = &mut self.cover_picker else {
            return;
        };
        if picker.saving.is_some() || !picker.loaded {
            return;
        }
        if let Some(token) = picker.preview.take() {
            token.cancel();
        }
        picker.error = None;
        picker.save_failed = false;
        picker.restore_failed = false;
        picker.restore_committed = false;
        let Some(generation) = picker.generation.checked_add(1) else {
            return;
        };
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::PreviewCover(
            CoverSource::capture(&picker.document),
            picker.page,
            generation,
            cancel.clone(),
        )) {
            Ok(()) => {
                picker.generation = generation;
                picker.preview = Some(cancel);
            }
            Err(error) => picker.error = Some(error),
        }
    }
    pub(super) fn accept_cover_source_page(
        &mut self,
        id: &str,
        result: yamantaka_domain::Result<Option<u32>>,
        cancel: &Cancellation,
    ) {
        let Some(picker) = &mut self.cover_picker else {
            return;
        };
        if picker.document.id != id
            || cancel.is_cancelled()
            || !picker
                .loading
                .as_ref()
                .is_some_and(|token| token.same_request(cancel))
        {
            return;
        }
        picker.loading = None;
        match result {
            Ok(page) => {
                picker.loaded = true;
                picker.page = page.unwrap_or(0);
                self.preview_cover_picker();
            }
            Err(error) => picker.error = Some(error),
        }
    }
    pub(super) fn accept_cover_picker_image(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        result: &yamantaka_domain::Result<Arc<RenderedPage>>,
        cancel: &Cancellation,
    ) -> bool {
        if self.accept_restored_cover(ctx, id, result, cancel) {
            return true;
        }
        let Some(picker) = &mut self.cover_picker else {
            return false;
        };
        if picker.document.id != id
            || !picker
                .preview
                .as_ref()
                .is_some_and(|token| token.same_request(cancel))
        {
            return false;
        }
        if cancel.is_cancelled() {
            return true;
        }
        picker.preview = None;
        let page = match result {
            Ok(page) => page,
            Err(error) => {
                picker.error = Some(error.clone());
                return true;
            }
        };
        if page.key.document_id != id
            || page.key.generation != picker.generation
            || page.key.page != picker.page
        {
            return true;
        }
        match render_updates::upload_image(ctx, &mut self.presenter, &mut self.managed_pages, page)
        {
            Ok(texture) => {
                picker.image = Some(texture);
                picker.image_page = Some(picker.page);
                picker.aspect = Some(f64::from(page.width) / f64::from(page.height));
                picker.error = None;
            }
            Err(error) => picker.error = Some(error),
        }
        true
    }
    pub(super) fn save_cover_picker(&mut self) {
        let Some(picker) = &mut self.cover_picker else {
            return;
        };
        if !picker.ready() {
            return;
        }
        let choice = match ManualCover::new(&picker.document, picker.page, picker.aspect.unwrap()) {
            Ok(choice) => choice,
            Err(error) => {
                picker.error = Some(error);
                picker.save_failed = true;
                return;
            }
        };
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::SaveManualCover(Box::new(ManualCoverRequest {
                choice,
                cancel: cancel.clone(),
            }))) {
            Ok(()) => picker.saving = Some(cancel),
            Err(error) => {
                picker.error = Some(error);
                picker.save_failed = true;
            }
        }
    }
    pub(super) fn accept_manual_cover(&mut self, ctx: &egui::Context, reply: ManualCoverReply) {
        let Some(picker) = &mut self.cover_picker else {
            return;
        };
        if picker.document.id != reply.document
            || !picker
                .saving
                .as_ref()
                .is_some_and(|token| token.same_request(&reply.cancel))
        {
            return;
        }
        picker.saving = None;
        match reply.result {
            Ok(()) => {
                self.cover_picker_demo.saved = true;
                let mut picker = self.cover_picker.take().unwrap();
                if let (Some(image), Some(aspect)) = (picker.image.take(), picker.aspect) {
                    self.install_selected_cover(ctx, &picker.document, picker.page, aspect, image);
                }
            }
            Err(error) => {
                picker.error = Some(error);
                picker.save_failed = true;
            }
        }
    }
    pub(super) fn cover_picker_summary(&self) -> serde_json::Value {
        match &self.cover_picker {
            Some(picker) => {
                serde_json::json!({"open":true,"document":picker.document.id,"page":picker.page,"previewPage":picker.image_page,"ready":picker.ready(),"saving":picker.saving.is_some(),"error":picker.error.is_some()})
            }
            None => serde_json::json!({"open":false}),
        }
    }
}
