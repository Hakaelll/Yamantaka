use super::*;
use yamantaka_application::automation_plan::{PlanReply, PlanRequest};
use yamantaka_domain::Failure;
use yamantaka_domain::automation::{plan::*, simulation::SimulationReport};

#[derive(Default)]
pub(super) struct ReviewState {
    pending: Option<Cancellation>,
    plan: Option<Arc<RulePlan>>,
    error: Option<String>,
    close_after: bool,
}
impl ReviewState {
    pub(super) fn plan(&self) -> Option<Arc<RulePlan>> {
        self.plan.clone()
    }
    pub(super) fn busy(&self) -> bool {
        self.pending.is_some()
    }
    pub(super) fn leave(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
        self.plan = None;
        self.error = None;
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"pending":self.busy(),"failed":self.error.is_some(),"documents":self.plan.as_ref().map(|plan|plan.document_revisions().len()),"actions":self.plan.as_ref().map(|plan|plan.actions().len())})
    }
}
impl Drop for ReviewState {
    fn drop(&mut self) {
        self.leave();
    }
}
impl YamantakaApp {
    pub(super) fn prepare_rule_review(&mut self, report: Arc<SimulationReport>) {
        if self.automation.busy() {
            return;
        }
        self.automation.review.leave();
        self.automation.execution.collapse_history();
        let request = PlanRequest::new(report, self.governor.clone());
        let cancel = request.cancel.clone();
        match self.runtime.submit(Action::PrepareRule(Box::new(request))) {
            Ok(()) => self.automation.review.pending = Some(cancel),
            Err(error) => self.automation.review.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_rule_review(&mut self, reply: PlanReply) {
        let state = &mut self.automation.review;
        if !state
            .pending
            .as_ref()
            .is_some_and(|cancel| cancel.same_request(&reply.cancel))
        {
            return;
        }
        state.pending = None;
        if reply.cancel.is_cancelled() || !matches!(self.surface, Surface::Automation) {
            return;
        }
        match reply.result {
            Ok(plan) => state.plan = Some(plan),
            Err(Failure::Cancelled) => {}
            Err(error) => state.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn poll_rule_review(&mut self, ctx: &egui::Context) {
        let state = &mut self.automation.review;
        if let Some(cancel) = &state.pending {
            ctx.request_repaint_after(Duration::from_millis(100));
            if ctx.input(|input| input.viewport().close_requested()) {
                state.close_after = true;
                cancel.cancel();
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            }
        } else if state.close_after {
            state.close_after = false;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
    pub(super) fn rule_review_view(&mut self, ui: &mut egui::Ui, p: Palette) -> bool {
        let can_execute = !self.automation.busy();
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let state = &mut self.automation.review;
        if let Some(error) = &state.error {
            ui.label(RichText::new(error).color(p.ink));
            ui.add_space(12.0);
        }
        if let Some(cancel) = &state.pending {
            ui.label(tr(
                "Preparando revisión de acciones",
                "Preparing action review",
            ));
            if theme::navigation(ui, tr("Cancelar revisión", "Cancel review"), false, p).clicked()
            {
                cancel.cancel();
            }
            return true;
        }
        let Some(plan) = &state.plan else {
            return false;
        };
        let mut close = false;
        let mut execute = false;
        egui::Frame::new().fill(p.raised).corner_radius(8).inner_margin(16).show(ui,|ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(tr("Revisar acciones","Review actions")).size(24.0).color(p.ink));
            ui.label(RichText::new(&plan.report().rule.name).color(p.secondary));
            ui.label(if english {format!("{} documents · {} actions per document",plan.document_revisions().len(),plan.actions().len())}else{format!("{} documentos · {} acciones por documento",plan.document_revisions().len(),plan.actions().len())});
            ui.add_space(12.0);
            for action in plan.actions() {
                let (label,target)=match action {
                    PlannedAction::AddTag(target)=>(tr("Añadir etiqueta","Add tag"),Some(target)),
                    PlannedAction::AddCollection(target)=>(tr("Añadir a colección","Add to collection"),Some(target)),
                    PlannedAction::AddWorkspace(target)=>(tr("Añadir a espacio","Add to workspace"),Some(target)),
                    PlannedAction::AddReadingQueue(target)=>(tr("Añadir a cola de lectura","Add to reading queue"),Some(target)),
                    PlannedAction::SetReadingState {finished}=>(if *finished {tr("Marcar como terminado","Mark as finished")}else{tr("Restablecer progreso","Reset progress")},None),
                    PlannedAction::SetFavorite(value)=>(if *value {tr("Añadir a favoritos","Add to favorites")}else{tr("Quitar de favoritos","Remove from favorites")},None),
                };
                if let Some(target)=target {
                    ui.label(format!("{label}: {}",target.name));
                    if target.will_create() {ui.label(RichText::new(tr("Se creará al confirmar.","Will be created on confirmation.")).size(12.0).color(p.secondary));}
                } else {ui.label(label);}
                ui.add_space(6.0);
            }
            ui.add_space(12.0);
            ui.label(RichText::new(tr("Confirma para aplicar este plan. Cada cambio tendrá un recibo persistente para deshacerlo.","Confirm to apply this plan. Each change will have a persistent undo receipt.")).color(p.secondary));
            execute=ui.add_enabled_ui(can_execute,|ui|theme::navigation(ui,tr("Confirmar ejecución","Confirm execution"),false,p)).inner.clicked();
            let response=theme::navigation(ui,tr("Cerrar revisión","Close review"),false,p);
            if self.automation.simulation.compact_review_preview() {response.scroll_to_me(Some(egui::Align::Center));}
            close=response.clicked();
        });
        if close {
            state.plan = None;
        }
        ui.add_space(16.0);
        if execute {
            self.execute_reviewed_rule();
        }
        true
    }
}
