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
    fn begin(&mut self, document: &str, page: u32) -> Option<Cancellation> {
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
        let compact = ui.ctx().content_rect().width() < 620.0;
        if compact {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(self.tr("Texto de la página", "Page text"))
                        .size(20.0)
                        .color(p.ink),
                );
                if let Some(page) = &self.text.page
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
        if self.text.pending.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new(self.tr("Extrayendo texto…", "Extracting text…"))
                        .color(p.secondary),
                );
            });
            return;
        }
        if let Some(error) = &self.text.error {
            ui.label(RichText::new(error).color(p.secondary));
            if theme::navigation(ui, self.tr("Reintentar", "Retry"), false, p).clicked() {
                self.text.clear();
            }
            return;
        }
        let Some(page) = &self.text.page else { return };
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

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::memory::{Cost, MemoryCategory, MemoryRepresentation};

    fn payload(governor: &MemoryGovernor, page: u32) -> Arc<PdfPageText> {
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                4096,
            ))
            .unwrap();
        Arc::new(PdfPageText::from_reserved(page, String::new(), Vec::new(), lease).unwrap())
    }

    #[test]
    fn page_change_cancels_previous_request_and_discards_late_or_foreign_results() {
        let governor = MemoryGovernor::default();
        let mut state = TextState::default();
        let previous = state.begin("first", 0).unwrap();
        assert!(state.begin("first", 0).is_none());
        let current = state.begin("first", 1).unwrap();
        assert!(previous.is_cancelled());
        state.accept("first", 0, &previous, Ok(payload(&governor, 0)));
        state.accept("other", 1, &current, Ok(payload(&governor, 1)));
        state.accept(
            "first",
            1,
            &Cancellation::default(),
            Ok(payload(&governor, 1)),
        );
        assert!(state.page.is_none());
        assert_eq!(governor.usage().text, 0);
        state.accept("first", 1, &current, Ok(payload(&governor, 1)));
        assert_eq!(state.page.as_ref().unwrap().page(), 1);
        assert!(state.pending.is_none());
        state.accept("first", 1, &current, Err("duplicate late failure".into()));
        assert!(state.error.is_none());
        state.clear();
        assert_eq!(governor.usage().text, 0);
    }

    #[test]
    fn failed_and_cancelled_requests_do_not_install_text_and_drop_cancels_work() {
        let governor = MemoryGovernor::default();
        let mut state = TextState::default();
        let current = state.begin("first", 1).unwrap();
        state.accept("first", 1, &current, Ok(payload(&governor, 2)));
        assert!(state.error.is_some());
        assert!(state.begin("first", 1).is_none());
        assert_eq!(governor.usage().text, 0);
        state.clear();
        let current = state.begin("first", 1).unwrap();
        current.cancel();
        state.accept("first", 1, &current, Ok(payload(&governor, 1)));
        assert!(state.page.is_none());
        assert_eq!(governor.usage().text, 0);
        state.clear();
        let current = state.begin("second", 0).unwrap();
        drop(state);
        assert!(current.is_cancelled());
    }

    #[test]
    fn closing_or_changing_document_releases_the_retained_result() {
        let governor = MemoryGovernor::default();
        let mut state = TextState::default();
        let current = state.begin("first", 0).unwrap();
        state.accept("first", 0, &current, Ok(payload(&governor, 0)));
        assert!(governor.usage().text > 0);
        let current = state.begin("second", 0).unwrap();
        assert_eq!(governor.usage().text, 0);
        state.accept("second", 0, &current, Ok(payload(&governor, 0)));
        drop(state);
        assert_eq!(governor.usage().text, 0);
    }
}
