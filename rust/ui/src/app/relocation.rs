use super::*;
use yamantaka_application::relocation::{
    RelocationAction, RelocationOutcome, RelocationReply, RelocationRequest,
};
use yamantaka_domain::Failure;
use yamantaka_domain::relocation::RelocationPlan;
#[derive(Default)]
pub(super) struct RelocationState {
    pending: Option<(String, Cancellation)>,
    review: Option<Box<RelocationPlan>>,
    outcome: Option<RelocationReply>,
    preview: Option<(PathBuf, bool)>,
    auto_confirm: bool,
    capture_mode: bool,
}
impl RelocationState {
    pub(super) fn cancel(&mut self) {
        if let Some((_, token)) = self.pending.take() {
            token.cancel();
        }
        self.review = None;
        self.outcome = None;
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && self.preview.is_none() && !self.auto_confirm
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"pending":self.pending.is_some(),"review":self.review.is_some(),"exact":self.review.as_ref().map(|plan|plan.exact()),
            "applied":self.outcome.as_ref().is_some_and(|reply|matches!(reply.result,Ok(RelocationOutcome::Applied))),
            "failed":self.outcome.as_ref().is_some_and(|reply|reply.result.is_err())})
    }
    pub(super) fn accept(&mut self, reply: RelocationReply) {
        if !self.pending.as_ref().is_some_and(|(id, token)| {
            id == &reply.document && token.same_request(&reply.cancel) && !token.is_cancelled()
        }) {
            return;
        }
        self.pending = None;
        match reply.result {
            Ok(RelocationOutcome::Review(plan)) => self.review = Some(plan),
            result => {
                self.review = None;
                self.auto_confirm = false;
                self.outcome = Some(RelocationReply { result, ..reply });
            }
        }
    }
}
impl YamantakaApp {
    pub fn preview_relocation(&mut self, candidate: PathBuf, confirm: bool) {
        if self.demo && self.capture.is_some() {
            self.inspector = true;
            self.selected = Some("demo-0".into());
            self.relocation.preview = Some((candidate, confirm));
            self.relocation.capture_mode = true;
        }
    }
    fn relocation_open(&self, id: &str) -> bool {
        self.tabs
            .model
            .tabs()
            .iter()
            .any(|tab| tab.document() == id)
            || !self.opening.ready()
    }
    pub(super) fn update_relocation(&mut self) {
        let bound = self
            .relocation
            .pending
            .as_ref()
            .map(|(id, _)| id.as_str())
            .or_else(|| {
                self.relocation
                    .review
                    .as_ref()
                    .map(|plan| plan.expected.document.as_str())
            });
        if bound.is_some_and(|id| {
            !self.inspector
                || self.selected.as_deref() != Some(id)
                || !matches!(self.surface, Surface::Library)
                || self.relocation_open(id)
        }) {
            self.relocation.cancel();
            self.relocation.auto_confirm = false;
        }
        if self.relocation.preview.is_some()
            && let Some(doc) = self
                .documents
                .iter()
                .find(|doc| doc.id == "demo-0")
                .cloned()
        {
            let (candidate, confirm) = self.relocation.preview.take().unwrap();
            self.start_relocation(RelocationAction::Inspect {
                document: Box::new(doc),
                candidate,
            });
            self.relocation.auto_confirm = confirm;
        }
        if self.relocation.auto_confirm && self.relocation.review.is_some() {
            self.relocation.auto_confirm = false;
            let plan = self.relocation.review.take().unwrap();
            self.start_relocation(RelocationAction::Confirm(plan));
        }
    }
    fn start_relocation(&mut self, action: RelocationAction) {
        self.relocation.cancel();
        let cancel = Cancellation::default();
        let request = RelocationRequest {
            action,
            governor: self.governor.clone(),
            cancel: cancel.clone(),
        };
        let id = request.document().to_owned();
        match self.runtime.submit(Action::RelocateFile(Box::new(request))) {
            Ok(()) => self.relocation.pending = Some((id, cancel)),
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(super) fn relocation_details(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        p: Palette,
    ) {
        if self.relocation.pending.is_some() {
            ui.horizontal_wrapped(|ui| {
                ui.label(self.tr("Verificando nueva ubicación…", "Verifying new location…"));
                if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked() {
                    self.relocation.cancel();
                }
            });
        } else if document.missing && self.relocation.review.is_none() {
            let closed = !self.relocation_open(&document.id);
            if ui
                .add_enabled_ui(closed, |ui| {
                    theme::navigation(ui, self.tr("Localizar PDF…", "Locate PDF…"), false, p)
                })
                .inner
                .on_hover_text(self.tr(
                    "Cierra las lecturas de este libro antes de cambiar su ubicación.",
                    "Close this book's readings before changing its location.",
                ))
                .clicked()
                && let Some(candidate) = yamantaka_windows::pick_pdf()
            {
                self.start_relocation(RelocationAction::Inspect {
                    document: Box::new(document.clone()),
                    candidate,
                });
            }
        }
        if let Some(plan) = &self.relocation.review {
            let message = if plan.exact() {
                self.tr("El contenido coincide con la referencia completa guardada. Confirma la nueva ubicación.","Contents match the saved full reference. Confirm the new location.")
            } else {
                self.tr("Solo coincide la huella parcial. Igualdad completa sin verificar. Confirma que es el mismo libro.","Partial fingerprint match only. Full equality is unverified. Confirm this is the same book.")
            };
            let response = Self::file_status_label(ui, message, p);
            let path = plan.candidate.path.to_string_lossy();
            let name = plan
                .candidate
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            ui.add(egui::Label::new(name.as_ref()).truncate())
                .on_hover_text(path.as_ref());
            let actions = ui.horizontal_wrapped(|ui| {
                if theme::control(
                    ui,
                    self.tr("Actualizar ubicación", "Update location"),
                    None,
                    false,
                    true,
                    p,
                )
                .clicked()
                {
                    let plan = self.relocation.review.take().unwrap();
                    self.start_relocation(RelocationAction::Confirm(plan));
                }
                if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked() {
                    self.relocation.cancel();
                }
            });
            if self.relocation.capture_mode {
                ui.scroll_to_rect(
                    response.rect.union(actions.response.rect),
                    Some(egui::Align::Center),
                );
            }
        }
        if let Some(reply) = &self.relocation.outcome
            && reply.document == document.id
        {
            let message=match &reply.result {
                Ok(RelocationOutcome::Applied)=>self.tr("Ubicación actualizada. Tu lectura y tus anotaciones se han conservado.","Location updated. Your reading position and annotations were preserved."),
                Err(Failure::DestinationOccupied)=>self.tr("Ese archivo ya pertenece a otro documento. Elige otra ubicación.","That file already belongs to another document. Choose another location."),
                Err(Failure::Invalid(_))=>self.tr("El archivo no coincide o la revisión ha caducado. Selecciona y verifica el PDF de nuevo.","The file does not match, or the review is outdated. Select and verify the PDF again."),
                _=>self.tr("No se pudo actualizar la ubicación. Comprueba el acceso y vuelve a intentarlo.","The location could not be updated. Check access and try again."),
            };
            let response = Self::file_status_label(ui, message, p);
            if self.relocation.capture_mode {
                ui.scroll_to_rect(response.rect, Some(egui::Align::Center));
            }
        }
    }
}
