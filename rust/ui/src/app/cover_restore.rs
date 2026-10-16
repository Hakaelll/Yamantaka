use super::*;
use yamantaka_application::cover_source::CoverSource;
use yamantaka_domain::{Failure, RenderedPage};

impl YamantakaApp {
    pub(super) fn restore_cover_picker(&mut self) {
        let Some(picker) = &mut self.cover_picker else {
            return;
        };
        if !picker.ready() {
            return;
        }
        let Some(generation) = picker.generation.checked_add(1) else {
            return;
        };
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::RestoreAutomaticCover(
            CoverSource::capture(&picker.document),
            generation,
            cancel.clone(),
        )) {
            Ok(()) => {
                picker.generation = generation;
                picker.saving = Some(cancel);
                picker.restoring = true;
                picker.save_failed = false;
                picker.restore_failed = false;
                picker.restore_committed = false;
            }
            Err(error) => {
                picker.error = Some(error);
                picker.restore_failed = true;
                picker.restore_committed = false;
            }
        }
    }
    pub(super) fn accept_restored_cover(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        result: &yamantaka_domain::Result<Arc<RenderedPage>>,
        cancel: &Cancellation,
    ) -> bool {
        let Some(picker) = &mut self.cover_picker else {
            return false;
        };
        if picker.document.id != id
            || !picker
                .saving
                .as_ref()
                .is_some_and(|token| token.same_request(cancel))
        {
            return false;
        }
        picker.saving = None;
        picker.restoring = false;
        let image = match result {
            Ok(image) => {
                picker.restore_committed = true;
                self.cover_picker_demo.saved = true;
                image
            }
            Err(error) => {
                picker.error = Some(error.clone());
                picker.restore_failed = true;
                return true;
            }
        };
        if image.key.document_id != id || image.key.generation != picker.generation {
            picker.error = Some(Failure::Invalid("Restored cover scope mismatch".into()));
            picker.restore_failed = true;
            self.invalidate_library_cover(ctx, id);
            return true;
        }
        let uploaded = self
            .presenter
            .as_mut()
            .ok_or_else(|| Failure::Unsupported("Cover preview compositor".into()))
            .and_then(|presenter| presenter.upload(image));
        match uploaded {
            Ok(texture) => {
                let picker = self.cover_picker.take().unwrap();
                self.install_selected_cover(
                    ctx,
                    &picker.document,
                    image.key.page,
                    f64::from(image.width) / f64::from(image.height),
                    PageImage::Native(texture),
                );
            }
            Err(error) => {
                let picker = self.cover_picker.as_mut().unwrap();
                picker.error = Some(error);
                picker.restore_failed = true;
                self.invalidate_library_cover(ctx, id);
            }
        }
        true
    }
}
