use super::*;
use yamantaka_domain::pdf_text::PdfPageText;
mod view;

#[derive(Default)]
pub(super) struct TextState {
    identity: Option<(String, u32)>,
    pending: Option<Cancellation>,
    page: Option<Arc<PdfPageText>>,
    error: Option<String>,
}

impl TextState {
    pub(super) fn is_for(&self, document: &str, page: u32) -> bool {
        self.identity
            .as_ref()
            .is_some_and(|(id, current)| id == document && *current == page)
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && (self.page.is_some() || self.error.is_some())
    }
    pub(super) fn capture_summary(&self) -> serde_json::Value {
        serde_json::json!({"document": self.identity.as_ref().map(|(id,_)| id), "ready": self.page.is_some(), "failed": self.error.is_some(), "page": self.page.as_ref().map(|page| page.page()), "characters": self.page.as_ref().map_or(0, |page| page.characters().len())})
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
    pub(super) fn owns(&self, document: &str, page: u32, cancel: &Cancellation) -> bool {
        !cancel.is_cancelled()
            && self
                .identity
                .as_ref()
                .is_some_and(|(id, current)| id == document && *current == page)
            && self
                .pending
                .as_ref()
                .is_some_and(|current| current.same_request(cancel))
    }

    pub(super) fn request(&mut self, runtime: &Runtime, document: &str, page: u32) {
        if let Some(cancel) = self.begin(document, page)
            && let Err(error) =
                runtime.submit(Action::TextGeometry(document.into(), page, cancel.clone()))
        {
            self.accept(document, page, &cancel, Err(error.to_string()));
        }
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
        if !self.owns(document, page, cancel) {
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
        if self.reader.selection.enabled {
            self.request_selection_pages(document, &[self.reader.page]);
            return;
        }
        self.reader
            .text
            .request(&self.runtime, document, self.reader.page);
    }
    pub(super) fn text_panel_content(&mut self, ui: &mut egui::Ui, p: Palette) {
        let state = if self.reader.selection.enabled {
            let Some(state) = self.reader.selection.text_state(self.reader.page) else {
                return;
            };
            state
        } else {
            &self.reader.text
        };
        let retry = state.show(ui, self.english, p);
        if retry {
            if self.reader.selection.enabled {
                self.reader.selection.retry_page(self.reader.page);
            } else {
                self.reader.text.clear();
            }
        }
    }
}
