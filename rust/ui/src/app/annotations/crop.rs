use super::*;
use yamantaka_domain::{Priority, RenderedPage};

pub(in crate::app) struct Request {
    pub id: String,
    pub page: u32,
    pub fingerprint: String,
    pub points: [[f64; 2]; 2],
}

#[derive(Default)]
pub(in crate::app) struct Preview {
    id: Option<String>,
    source: Option<(String, u32)>,
    pending: Option<Cancellation>,
    image: Option<(egui::ColorImage, egui::TextureHandle)>,
    failed: bool,
}

impl Preview {
    pub(in crate::app) fn clear(&mut self) {
        if let Some(cancel) = self.pending.take() {
            cancel.cancel();
        }
        *self = Self::default();
    }

    pub(super) fn request(
        &mut self,
        runtime: &Runtime,
        document: &str,
        title: &str,
        size: [f32; 2],
        request: Request,
    ) {
        self.clear();
        self.id = Some(request.id);
        self.source = Some((title.to_owned(), request.page));
        let [a, b] = request.points;
        let width = size[0] * (a[0] - b[0]).abs() as f32;
        let height = size[1] * (a[1] - b[1]).abs() as f32;
        let scale = (638.0 / width.max(height)).min(8192.0 / size[0].max(size[1]));
        let canvas = size.map(|dimension| (dimension * scale).ceil().max(1.0) as u32);
        let bounds = |axis: usize| {
            let size = f64::from(canvas[axis]);
            let first = (a[axis].min(b[axis]) * size).floor() as u32;
            let last = (a[axis].max(b[axis]) * size).ceil() as u32;
            (first.min(canvas[axis] - 1), last)
        };
        let ((left, right), (top, bottom)) = (bounds(0), bounds(1));
        let key = RenderKey {
            document_id: document.into(),
            page: request.page,
            region: Some([left, top, (right - left).max(1), (bottom - top).max(1)]),
            physical_width: canvas[0],
            physical_height: canvas[1],
            rotation: 0,
            generation: 0,
        };
        let cancel = Cancellation::default();
        if runtime
            .submit(Action::Render(key, Priority::Thumbnail, cancel.clone()))
            .is_ok()
        {
            self.pending = Some(cancel);
        } else {
            self.failed = true;
        }
    }

    pub(in crate::app) fn accept(
        &mut self,
        ctx: &egui::Context,
        page: &RenderedPage,
        cancel: &Cancellation,
    ) -> bool {
        if !self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.same_request(cancel))
        {
            return false;
        }
        self.pending = None;
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [page.width as usize, page.height as usize],
            page.pixels(),
        );
        let texture = ctx.load_texture("study-crop", image.clone(), egui::TextureOptions::LINEAR);
        self.image = Some((image, texture));
        true
    }

    pub(in crate::app) fn fail(&mut self, cancel: &Cancellation) -> bool {
        if !self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.same_request(cancel))
        {
            return false;
        }
        self.pending = None;
        self.failed = true;
        true
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, id: &str, english: bool, p: Palette) {
        if self.id.as_deref() != Some(id) {
            return;
        }
        let tr = |es, en| if english { en } else { es };
        if let Some((image, texture)) = &self.image {
            let size = texture.size_vec2();
            let fit = (ui.available_width() / size.x).min(1.0);
            ui.add(
                egui::Image::new((texture.id(), size * fit))
                    .alt_text(tr("Recorte del PDF", "PDF crop")),
            );
            ui.horizontal_wrapped(|ui| {
                if theme::navigation(ui, tr("Copiar imagen", "Copy image"), false, p).clicked() {
                    ui.ctx().copy_image(image.clone());
                }
                if let Some((title, page)) = &self.source
                    && theme::navigation(ui, tr("Copiar cita", "Copy citation"), false, p).clicked()
                {
                    ui.ctx().copy_text(format!(
                        "{title} · {} {}",
                        tr("pág.", "p."),
                        u64::from(*page) + 1
                    ));
                }
                if theme::navigation(ui, tr("Guardar PNG", "Save PNG"), false, p).clicked()
                    && let Some(path) = yamantaka_windows::save_crop_image()
                {
                    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                        let parent = path.parent().ok_or("Missing output directory")?;
                        let mut file = tempfile::NamedTempFile::new_in(parent)?;
                        let mut writer = std::io::BufWriter::new(file.as_file_mut());
                        let rgba: Vec<_> = image
                            .pixels
                            .iter()
                            .flat_map(|color| color.to_array())
                            .collect();
                        image::write_buffer_with_format(
                            &mut writer,
                            &rgba,
                            image.size[0] as u32,
                            image.size[1] as u32,
                            image::ColorType::Rgba8,
                            image::ImageFormat::Png,
                        )?;
                        std::io::Write::flush(&mut writer)?;
                        drop(writer);
                        file.persist_noclobber(path)?;
                        Ok(())
                    })();
                    self.failed = result.is_err();
                }
            });
        } else if self.pending.is_some() {
            ui.spinner();
        }
        if self.failed {
            ui.label(tr(
                "No se pudo abrir o guardar el recorte.",
                "The crop could not be opened or saved.",
            ));
        }
    }
}

impl Drop for Preview {
    fn drop(&mut self) {
        if let Some(cancel) = self.pending.take() {
            cancel.cancel();
        }
    }
}
