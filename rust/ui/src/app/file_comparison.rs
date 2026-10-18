use super::*;
use yamantaka_application::file_comparison::{ComparisonReply, ComparisonRequest};

#[derive(Default)]
pub(super) struct ComparisonState {
    pending: Option<(String, Cancellation)>,
    preview: Option<bool>,
    capture_mode: bool,
    outcome: Option<ComparisonReply>,
}
impl ComparisonState {
    pub(super) fn ready(&self) -> bool {
        self.preview.is_none() && self.pending.is_none()
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"pending":self.pending.is_some(),
            "equal":self.outcome.as_ref().and_then(|reply|reply.result.as_ref().ok()).copied(),
            "failed":self.outcome.as_ref().is_some_and(|reply|reply.result.is_err())})
    }
    pub(super) fn cancel(&mut self) {
        if let Some((_, cancel)) = self.pending.take() {
            cancel.cancel();
        }
        self.outcome = None;
    }
    pub(super) fn accept(&mut self, reply: ComparisonReply) {
        if self.pending.as_ref().is_some_and(|(id, cancel)| {
            id == &reply.document.id && cancel.same_request(&reply.cancel) && !cancel.is_cancelled()
        }) {
            self.pending = None;
            self.outcome = Some(reply);
        }
    }
}
impl YamantakaApp {
    pub fn preview_file_comparison(&mut self, different: bool) {
        if self.demo && self.capture.is_some() {
            self.inspector = true;
            self.selected = Some("demo-0".into());
            self.file_comparison.preview = Some(different);
            self.file_comparison.capture_mode = true;
        }
    }
    pub(super) fn update_file_comparison(&mut self) {
        if self
            .file_comparison
            .pending
            .as_ref()
            .is_some_and(|(id, _)| {
                !self.inspector
                    || self.selected.as_ref() != Some(id)
                    || !matches!(self.surface, Surface::Library)
            })
        {
            self.file_comparison.cancel();
        }
        if let Some(different) = self.file_comparison.preview
            && let Some(document) = self
                .documents
                .iter()
                .find(|document| document.id == "demo-0")
                .cloned()
            && let Some(capture) = &self.capture
        {
            self.file_comparison.preview = None;
            let candidate = capture.with_extension("candidate.pdf");
            let result = std::fs::read(&document.path).and_then(|mut bytes| {
                if different {
                    bytes.extend_from_slice(b"\n% synthetic comparison difference\n");
                }
                std::fs::write(&candidate, bytes)
            });
            if result.is_ok() {
                self.start_file_comparison(&document, candidate);
            } else {
                self.error = Some("Synthetic comparison fixture could not be prepared".into());
            }
        }
    }

    fn start_file_comparison(&mut self, document: &Document, candidate: PathBuf) {
        self.file_comparison.cancel();
        let cancel = Cancellation::default();
        let request = ComparisonRequest {
            document: document.clone(),
            candidate,
            governor: self.governor.clone(),
            cancel: cancel.clone(),
        };
        match self.runtime.submit(Action::CompareFiles(Box::new(request))) {
            Ok(()) => self.file_comparison.pending = Some((document.id.clone(), cancel)),
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(super) fn file_comparison_details(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        p: Palette,
    ) {
        if self
            .file_comparison
            .pending
            .as_ref()
            .is_some_and(|(id, _)| id != &document.id)
        {
            self.file_comparison.cancel();
        }
        ui.horizontal_wrapped(|ui| {
            if self.file_comparison.pending.is_some() {
                ui.label(self.tr("Comparando contenido…", "Comparing contents…"));
                if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked() {
                    self.file_comparison.cancel();
                }
            } else if ui
                .add_enabled_ui(!document.missing, |ui| {
                    theme::navigation(
                        ui,
                        self.tr("Comparar con otro PDF", "Compare with another PDF"),
                        false,
                        p,
                    )
                })
                .inner
                .clicked()
                && let Some(candidate) = yamantaka_windows::pick_pdf()
            {
                self.start_file_comparison(document, candidate);
            }
        });
        if let Some(reply) = &self.file_comparison.outcome
            && reply.document.id == document.id
            && reply.document.path == document.path
            && reply.document.fingerprint == document.fingerprint
        {
            let message = match &reply.result {
                Ok(true) => self.tr(
                    "Contenido idéntico verificado con SHA-256 completo.",
                    "Identical contents verified with full SHA-256.",
                ),
                Ok(false) => self.tr(
                    "El contenido de los archivos es diferente.",
                    "The files have different contents.",
                ),
                Err(yamantaka_domain::Failure::Invalid(_)) => self.tr(
                    "El archivo ha cambiado. Actualiza la biblioteca y vuelve a comparar.",
                    "The source changed. Refresh the library and compare again.",
                ),
                Err(yamantaka_domain::Failure::QueueFull) => self.tr(
                    "La comparación anterior está terminando. Vuelve a intentarlo.",
                    "The previous comparison is finishing. Try again.",
                ),
                Err(yamantaka_domain::Failure::MemoryBudget) => self.tr(
                    "Sin memoria disponible para comparar. Cierra una lectura y reintenta.",
                    "No memory available for comparison. Close a reading and retry.",
                ),
                Err(_) => self.tr(
                    "No se pudo verificar. Comprueba el acceso y vuelve a comparar.",
                    "Could not verify. Check file access and compare again.",
                ),
            };
            let response = ui.label(RichText::new(message).color(p.secondary));
            if self.file_comparison.capture_mode {
                ui.scroll_to_rect(response.rect, Some(egui::Align::Center));
            }
            if let Some(name) = reply.candidate.file_name() {
                ui.label(
                    RichText::new(name.to_string_lossy())
                        .size(12.0)
                        .color(p.secondary),
                );
            }
        }
    }
}
