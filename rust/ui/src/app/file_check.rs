use super::*;
use yamantaka_application::file_check::{FileCheckReply, FileCheckRequest};
use yamantaka_domain::file_check::FileCheckResult;
#[derive(Default)]
pub(super) struct CheckState {
    pending: Option<(String, Cancellation)>,
    outcome: Option<FileCheckReply>,
    preview: bool,
    import_preview: bool,
    capture_mode: bool,
}
impl CheckState {
    pub(super) fn cancel(&mut self) {
        if let Some((_, cancel)) = self.pending.take() {
            cancel.cancel();
        }
        self.outcome = None;
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && !self.preview && !self.import_preview
    }
    pub(super) fn accept(&mut self, reply: FileCheckReply) {
        if self.pending.as_ref().is_some_and(|(id, cancel)| {
            id == &reply.document.id && cancel.same_request(&reply.cancel) && !cancel.is_cancelled()
        }) {
            self.pending = None;
            self.outcome = Some(reply);
        }
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        let state = self.outcome.as_ref().map(|reply| match &reply.result {
            Ok(FileCheckResult::Available {
                baseline_created: true,
                ..
            }) => "baseline",
            Ok(FileCheckResult::Available { .. }) => "available",
            Ok(FileCheckResult::Missing) => "missing",
            Ok(FileCheckResult::Changed) => "changed",
            Ok(FileCheckResult::Unverifiable) => "unverifiable",
            Err(_) => "failed",
        });
        serde_json::json!({"pending":self.pending.is_some(),"state":state})
    }
}
impl YamantakaApp {
    pub fn preview_import_reference(&mut self) {
        if self.demo && self.capture.is_some() {
            self.file_check.import_preview = true;
            self.file_check.capture_mode = true;
        }
    }
    pub fn preview_file_check(&mut self) {
        if self.demo && self.capture.is_some() {
            self.inspector = true;
            self.selected = Some("demo-0".into());
            self.file_check.preview = true;
            self.file_check.capture_mode = true;
        }
    }

    pub(super) fn update_file_check(&mut self) {
        if self.file_check.pending.as_ref().is_some_and(|(id, _)| {
            !self.inspector
                || self.selected.as_ref() != Some(id)
                || !matches!(self.surface, Surface::Library)
        }) {
            self.file_check.cancel();
        }
        if self.file_check.preview
            && let Some(document) = self
                .documents
                .iter()
                .find(|document| document.id == "demo-0")
                .cloned()
        {
            self.file_check.preview = false;
            self.start_file_check(&document);
        }
        if self.file_check.import_preview
            && !self.importing.busy()
            && let Some(document) = self
                .documents
                .iter()
                .find(|document| {
                    document.file_name == "first.pdf" && !document.id.starts_with("demo-")
                })
                .cloned()
        {
            self.file_check.import_preview = false;
            self.inspector = true;
            self.selected = Some(document.id.clone());
            self.start_file_check(&document);
        }
    }
    fn start_file_check(&mut self, document: &Document) {
        self.file_check.cancel();
        let cancel = Cancellation::default();
        let request = FileCheckRequest {
            document: document.clone(),
            governor: self.governor.clone(),
            cancel: cancel.clone(),
        };
        match self.runtime.submit(Action::CheckFile(Box::new(request))) {
            Ok(()) => self.file_check.pending = Some((document.id.clone(), cancel)),
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(super) fn file_check_details(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        p: Palette,
    ) {
        ui.horizontal_wrapped(|ui| {
            if self.file_check.pending.is_some() {
                ui.label(self.tr("Comprobando archivo…", "Checking file…"));
                if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked() {
                    self.file_check.cancel();
                }
            } else if theme::navigation(ui, self.tr("Comprobar archivo", "Check file"), false, p)
                .clicked()
            {
                self.start_file_check(document);
            }
        });
        if let Some(reply) = &self.file_check.outcome
            && reply.document.id == document.id
            && reply.document.path == document.path
            && reply.document.fingerprint == document.fingerprint
        {
            let message=match &reply.result {
                Ok(FileCheckResult::Available{baseline_created:true,..})=>self.tr("Archivo disponible. Referencia de contenido guardada.", "File available. Content reference saved."),
                Ok(FileCheckResult::Available{..})=>self.tr("El contenido coincide con la última comprobación completa.", "Contents match the previous full verification."),
                Ok(FileCheckResult::Missing)=>self.tr("No se encuentra el PDF en su ubicación guardada.", "The PDF was not found at its saved location."),
                Ok(FileCheckResult::Changed)=>self.tr("El PDF ha cambiado. Revisa el archivo antes de continuar.", "The PDF has changed. Review the file before continuing."),
                Ok(FileCheckResult::Unverifiable)=>self.tr("El archivo está disponible, pero no se puede verificar su fecha guardada.", "The file is available, but its saved date cannot be verified."),
                Err(_)=>self.tr("No se pudo completar la comprobación. Reintenta o revisa el acceso al archivo.", "Verification could not finish. Retry or check file access."),
            };
            let response = Self::file_status_label(ui, message, p);
            if self.file_check.capture_mode {
                ui.scroll_to_rect(response.rect, Some(egui::Align::Center));
            }
        }
    }
}
