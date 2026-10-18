use super::*;
use yamantaka_domain::pdf_text::PdfPageText;

#[derive(Default)]
pub(super) struct TextState {
    identity: Option<(String, u32)>,
    pending: Option<Cancellation>,
    page: Option<Arc<PdfPageText>>,
    error: Option<String>,
}

impl TextState {
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && (self.page.is_some() || self.error.is_some())
    }
    pub(super) fn capture_summary(&self) -> serde_json::Value {
        serde_json::json!({"ready": self.page.is_some(), "failed": self.error.is_some(), "page": self.page.as_ref().map(|page| page.page()), "characters": self.page.as_ref().map_or(0, |page| page.characters().len())})
    }
    pub(super) fn begin(&mut self, document: &str, page: u32) -> Option<Cancellation> {
        if self
            .identity
            .as_ref()
            .is_some_and(|(id, current)| id == document && *current == page)
        {
            return None;
        }
        self.clear();
        let cancel = Cancellation::default();
        self.identity = Some((document.into(), page));
        self.pending = Some(cancel.clone());
        Some(cancel)
    }
    pub(super) fn payload(&self) -> Option<&Arc<PdfPageText>> {
        self.page.as_ref()
    }
    pub(super) fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub(super) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    pub(super) fn clear(&mut self) {
        if let Some(cancel) = self.pending.take() {
            cancel.cancel();
        }
        self.identity = None;
        self.page = None;
        self.error = None;
    }
    pub(super) fn accept(
        &mut self,
        document: &str,
        page: u32,
        cancel: &Cancellation,
        result: std::result::Result<Arc<PdfPageText>, String>,
    ) {
        if cancel.is_cancelled()
            || !self
                .identity
                .as_ref()
                .is_some_and(|(id, current)| id == document && *current == page)
            || !self
                .pending
                .as_ref()
                .is_some_and(|current| current.same_request(cancel))
        {
            return;
        }
        self.pending = None;
        match result {
            Ok(payload) if payload.page() == page => self.page = Some(payload),
            Ok(_) => self.error = Some("PDF text belongs to a different page".into()),
            Err(error) => self.error = Some(error),
        }
    }
}
impl Drop for TextState {
    fn drop(&mut self) {
        self.clear();
    }
}

impl YamantakaApp {
    pub(super) fn request_page_text(&mut self, document: &str) {
        if self.selection.enabled {
            self.request_selection_pages(document, &[self.page]);
            return;
        }
        if let Some(cancel) = self.text.begin(document, self.page)
            && let Err(error) = self.runtime.submit(Action::TextGeometry(
                document.into(),
                self.page,
                cancel.clone(),
            ))
        {
            self.text
                .accept(document, self.page, &cancel, Err(error.to_string()));
        }
    }
    pub(super) fn text_panel_content(&mut self, ui: &mut egui::Ui, p: Palette) {
        let state = if self.selection.enabled {
            let Some(state) = self.selection.text_state(self.page) else {
                return;
            };
            state
        } else {
            &self.text
        };
        let compact = ui.ctx().content_rect().width() < 620.0;
        if compact {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(self.tr("Texto de la página", "Page text"))
                        .size(20.0)
                        .color(p.ink),
                );
                if let Some(page) = &state.page
                    && !page.text().trim().is_empty()
                    && theme::navigation(ui, self.tr("Copiar texto", "Copy text"), false, p)
                        .clicked()
                {
                    ui.ctx().copy_text(page.text().into());
                }
            });
        } else {
            ui.heading(self.tr("Texto de la página", "Page text"));
        }
        if state.pending.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new(self.tr("Extrayendo texto…", "Extracting text…"))
                        .color(p.secondary),
                );
            });
            return;
        }
        if let Some(error) = &state.error {
            ui.label(RichText::new(error).color(p.secondary));
            if theme::navigation(ui, self.tr("Reintentar", "Retry"), false, p).clicked() {
                if self.selection.enabled {
                    self.selection.retry_page(self.page);
                } else {
                    self.text.clear();
                }
            }
            return;
        }
        let Some(page) = &state.page else { return };
        if page.text().trim().is_empty() {
            ui.label(
                RichText::new(self.tr(
                    "Esta página no contiene texto extraíble.",
                    "This page contains no extractable text.",
                ))
                .color(p.secondary),
            );
            return;
        }
        if !compact
            && theme::navigation(ui, self.tr("Copiar texto", "Copy text"), false, p).clicked()
        {
            ui.ctx().copy_text(page.text().into());
        }
        ui.add_space(8.0);
        ui.add(egui::Label::new(RichText::new(page.text()).color(p.ink)).wrap());
    }
}
