use super::*;
use yamantaka_domain::RenderedPage;

impl ReaderPage {
    pub(super) fn accepts(&self, page: &RenderedPage, cancel: &Cancellation) -> bool {
        !cancel.is_cancelled()
            && self.cancel.same_request(cancel)
            && self.requested.as_ref() == Some(&page.key)
    }
}

impl YamantakaApp {
    pub(super) fn accept_rendered_page(
        &mut self,
        ctx: &egui::Context,
        page: &RenderedPage,
        cancel: &Cancellation,
    ) {
        let reader = self.reader.viewport.accepts(page, cancel);
        let reference = self
            .reference
            .as_ref()
            .is_some_and(|reference| reference.pane.viewport.accepts(page, cancel));
        let thumbnail = self.reader.thumbnails.accepts(page, cancel);
        if !reader && !thumbnail && !reference {
            return;
        }
        let replacement = if let Some(presenter) = &mut self.presenter {
            presenter.upload(page).map(PageImage::Native)
        } else {
            // The alternate compositor's GPU lifetime accounting remains pending.
            Ok(PageImage::Managed(ctx.load_texture(
                "pdf-page",
                egui::ColorImage::from_rgba_unmultiplied(
                    [page.width as usize, page.height as usize],
                    page.pixels(),
                ),
                TextureOptions::LINEAR,
            )))
        };
        if reference {
            if let Some(reference) = &mut self.reference {
                match replacement {
                    Ok(image) => {
                        reference.pane.viewport.install(page, cancel, image);
                    }
                    Err(error) => {
                        reference.fail_render(cancel, &error);
                    }
                }
            }
        } else if thumbnail {
            self.reader.thumbnails.install(page, cancel, replacement);
        } else {
            match replacement {
                Ok(image) => {
                    if let Some(replaced) = self.reader.viewport.install(page, cancel, image) {
                        if self.capture.is_some() && self.capture_render_events.len() < 128 {
                            self.capture_render_events.push(serde_json::json!({
                                "event":"installed","document":page.key.document_id,
                                "page":page.key.page,"generation":page.key.generation,
                                "rotation":page.key.rotation,"pixels":[page.width,page.height],
                                "replacedImage":replaced,
                            }));
                        }
                    }
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    self.reader.viewport.retry = true;
                }
            }
            self.reader.loading = false;
        }
    }
}
