use super::*;
use yamantaka_application::automation_simulation::{SimulationReply, SimulationRequest};
use yamantaka_domain::automation::simulation::*;
mod preview;
struct Pending {
    rule: Arc<SavedRule>,
    cancel: Cancellation,
    watch: Arc<SimulationWatch>,
}
#[derive(Default)]
pub(super) struct SimulationState {
    pending: Option<Pending>,
    outcome: Option<SimulationOutcome>,
    rule: Option<Arc<SavedRule>>,
    offset: usize,
    close_after: bool,
    preview: u8,
    preview_mode: u8,
}
impl SimulationState {
    pub(super) fn busy(&self) -> bool {
        self.pending.is_some()
    }
    pub(super) fn ready(&self) -> bool {
        !self.busy() && matches!(self.preview, 0 | 10)
    }
    pub(super) fn leave(&mut self) {
        if let Some(pending) = &self.pending {
            pending.cancel.cancel();
        }
        self.outcome = None;
        self.rule = None;
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        let (state, progress) = match &self.outcome {
            Some(SimulationOutcome::Complete(report)) => ("complete", report.progress),
            Some(SimulationOutcome::Cancelled(progress)) => ("cancelled", *progress),
            Some(SimulationOutcome::LimitReached(progress)) => ("limited", *progress),
            Some(SimulationOutcome::Failed { progress, .. }) => ("failed", *progress),
            None => {
                if let Some(pending) = &self.pending {
                    ("running", pending.watch.snapshot())
                } else {
                    ("idle", Default::default())
                }
            }
        };
        let ids = match &self.outcome {
            Some(SimulationOutcome::Complete(report)) => report
                .matches
                .iter()
                .take(80)
                .map(|matched| matched.document.as_str())
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        serde_json::json!({"phase":self.preview,"pending":self.busy(),"state":state,"scanned":progress.scanned,"matched":progress.matched,"unverified":progress.unverified,"ids":ids})
    }
}
impl Drop for SimulationState {
    fn drop(&mut self) {
        self.leave();
    }
}
impl YamantakaApp {
    pub(super) fn simulate_automation_rule(&mut self, rule: Arc<SavedRule>) {
        if self.automation.busy() {
            return;
        }
        let request = SimulationRequest::new(rule.clone(), self.governor.clone());
        let pending = Pending {
            rule,
            cancel: request.cancel.clone(),
            watch: request.watch.clone(),
        };
        self.automation.simulation.outcome = None;
        self.automation.simulation.offset = 0;
        match self.runtime.submit(Action::SimulateRule(Box::new(request))) {
            Ok(()) => self.automation.simulation.pending = Some(pending),
            Err(error) => {
                self.automation.simulation.outcome = Some(SimulationOutcome::Failed {
                    progress: Default::default(),
                    error,
                })
            }
        }
    }
    pub(in crate::app) fn accept_rule_simulation(&mut self, reply: SimulationReply) {
        let state = &mut self.automation.simulation;
        if !state.pending.as_ref().is_some_and(|pending| {
            pending.rule.revision.id == reply.rule && pending.cancel.same_request(&reply.cancel)
        }) {
            return;
        }
        let pending = state.pending.take().unwrap();
        if matches!(self.surface, Surface::Automation) {
            state.rule = Some(pending.rule);
            state.outcome = Some(reply.outcome);
        }
    }
    pub(in crate::app) fn poll_rule_simulation(&mut self, ctx: &egui::Context) {
        let state = &mut self.automation.simulation;
        if let Some(pending) = &state.pending {
            ctx.request_repaint_after(Duration::from_millis(100));
            if ctx.input(|input| input.viewport().close_requested()) {
                state.close_after = true;
                pending.cancel.cancel();
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            }
        } else if state.close_after {
            state.close_after = false;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
    pub(super) fn rule_simulation_view(&mut self, ui: &mut egui::Ui, p: Palette) {
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let state = &mut self.automation.simulation;
        if let Some(pending) = &state.pending {
            let progress = pending.watch.snapshot();
            ui.label(RichText::new(&pending.rule.name).size(20.0).color(p.ink));
            ui.label(if english {
                format!(
                    "{} scanned · {} matches",
                    progress.scanned, progress.matched
                )
            } else {
                format!(
                    "{} revisados · {} coincidencias",
                    progress.scanned, progress.matched
                )
            });
            if theme::navigation(ui, tr("Cancelar simulación", "Cancel simulation"), false, p)
                .clicked()
            {
                pending.cancel.cancel();
            }
            ui.add_space(16.0);
        }
        let Some(outcome) = &state.outcome else {
            return;
        };
        let mut clear = false;
        egui::Frame::new().fill(p.raised).corner_radius(8).inner_margin(16).show(ui,|ui| {
            ui.label(RichText::new(tr("Vista previa de coincidencias","Matching document preview")).size(21.0).color(p.ink));
            match outcome {
                SimulationOutcome::Complete(report)=>{
                    ui.label(RichText::new(&report.rule.name).color(p.secondary));
                    ui.label(if english {format!("{} matches · {} documents scanned",report.progress.matched,report.progress.scanned)}else{format!("{} coincidencias · {} documentos revisados",report.progress.matched,report.progress.scanned)});
                    if report.rule.definition().is_some_and(|definition|definition.trigger.kind==TriggerKind::DocumentImported) {ui.label(tr("Esta simulación manual no incluye un lote de importación.","This manual simulation has no import batch context."));}
                    if report.progress.unverified>0 {ui.label(if english {format!("{} files could not be verified.",report.progress.unverified)}else{format!("No se pudieron verificar {} archivos.",report.progress.unverified)});}
                    egui::ScrollArea::vertical().id_salt("rule-simulation-matches").max_height(240.0).show(ui,|ui| {
                        for matched in report.matches.iter().skip(state.offset).take(80) {ui.label(&matched.title);}
                    });
                    if report.matches.len()>80 {
                        ui.vertical(|ui| {
                            if ui.add_enabled_ui(state.offset>0,|ui|theme::navigation(ui,tr("Coincidencias anteriores","Previous matches"),false,p)).inner.clicked(){state.offset=state.offset.saturating_sub(80);}
                            if ui.add_enabled_ui(state.offset+80<report.matches.len(),|ui|theme::navigation(ui,tr("Más coincidencias","More matches"),false,p)).inner.clicked(){state.offset=state.offset.saturating_add(80);}
                        });
                    }
                }
                SimulationOutcome::Cancelled(progress)=>{ui.label(if english {format!("Simulation cancelled after {} documents.",progress.scanned)}else{format!("Simulación cancelada tras {} documentos.",progress.scanned)});}
                SimulationOutcome::LimitReached(progress)=>{ui.label(if english {format!("The scan limit was reached after {} documents. Run a narrower rule.",progress.scanned)}else{format!("Se alcanzó el límite tras {} documentos. Revisa una regla más acotada.",progress.scanned)});}
                SimulationOutcome::Failed {error,..}=>{ui.label(error.to_string());}
            }
            let close=theme::navigation(ui,tr("Cerrar vista previa","Close preview"),false,p);
            if state.preview==10 && state.preview_mode==3 {close.scroll_to_me(Some(egui::Align::Center));}
            if close.clicked(){clear=true;}
        });
        if clear {
            state.rule = None;
            state.outcome = None;
        }
        ui.add_space(16.0);
    }
}
