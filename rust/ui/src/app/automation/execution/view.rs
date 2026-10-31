use super::*;
fn status(status: ExecutionStatus, english: bool) -> &'static str {
    match (status, english) {
        (ExecutionStatus::Completed, true) => "Completed",
        (ExecutionStatus::Completed, false) => "Completada",
        (ExecutionStatus::Cancelled, true) => "Cancelled; changes reverted",
        (ExecutionStatus::Cancelled, false) => "Cancelada; cambios revertidos",
        (ExecutionStatus::RolledBack, true) => "Changes reverted",
        (ExecutionStatus::RolledBack, false) => "Cambios revertidos",
        (ExecutionStatus::Interrupted, true) => "Interrupted; review recovery",
        (ExecutionStatus::Interrupted, false) => "Interrumpida; revisa la recuperación",
        (ExecutionStatus::Failed, true) => "Failed; review the journal",
        (ExecutionStatus::Failed, false) => "Fallida; revisa el journal",
    }
}
impl YamantakaApp {
    pub(in crate::app::automation) fn rule_execution_view(
        &mut self,
        ui: &mut egui::Ui,
        p: Palette,
    ) {
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let state = &mut self.automation.execution;
        if let Some(error) = &state.error {
            ui.label(error);
            if theme::navigation(ui, tr("Recargar historial", "Reload history"), false, p).clicked()
            {
                self.load_rule_journals();
            }
            return;
        }
        if let Some(pending) = &state.pending {
            let label = if pending.undo || pending.cancel.is_cancelled() {
                tr("Revirtiendo cambios…", "Reverting changes…")
            } else {
                tr(
                    "Aplicando el plan confirmado",
                    "Applying the confirmed plan",
                )
            };
            ui.label(RichText::new(label).size(20.0).color(p.ink));
            if !pending.undo {
                ui.label(format!(
                    "{} {}",
                    pending.watch.completed(),
                    tr("documentos procesados", "documents processed")
                ));
            }
            if !pending.cancel.is_cancelled()
                && theme::navigation(ui, tr("Cancelar operación", "Cancel operation"), false, p)
                    .clicked()
            {
                pending.cancel.cancel();
            }
            ui.add_space(16.0);
        } else if let Some(receipt) = &state.receipt {
            ui.label(
                RichText::new(status(receipt.status, english))
                    .size(20.0)
                    .color(p.ink),
            );
            if receipt.reversible
                && matches!(
                    receipt.status,
                    ExecutionStatus::Failed | ExecutionStatus::Interrupted
                )
            {
                ui.label(tr(
                    "Quedan cambios por revertir. Revisa la ejecución en el historial.",
                    "Changes remain to be reverted. Review the execution in history.",
                ));
            }
            ui.add_space(16.0);
        }
    }
    pub(in crate::app::automation) fn rule_journal_view(&mut self, ui: &mut egui::Ui, p: Palette) {
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let busy = self.automation.busy();
        let state = &mut self.automation.execution;
        let Some(page) = state.page.clone() else {
            return;
        };
        if page.total == 0 {
            return;
        }
        ui.add_space(24.0);
        let caption = format!(
            "{} · {}",
            tr("Historial de ejecuciones", "Execution history"),
            page.total
        );
        if theme::navigation(ui, &caption, state.history_open, p).clicked() {
            state.history_open = !state.history_open;
        }
        if !state.history_open {
            ui.add_space(16.0);
            return;
        }
        let mut undo = None;
        for entry in &page.entries {
            let frame=egui::Frame::new().fill(p.surface).corner_radius(8).inner_margin(16).show(ui,|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new(&entry.rule_name).size(19.0).color(p.ink));
                ui.label(status(entry.status,english));
                ui.label(format!("{} / {} {}",entry.completed,entry.total,tr("documentos procesados","documents processed")));
                if let Some(code)=&entry.error_code {ui.label(RichText::new(code).size(12.0).color(p.secondary));}
                if entry.reversible&&ui.add_enabled_ui(!busy,|ui|theme::navigation(ui,tr("Revisar deshacer","Review undo"),false,p)).inner.clicked(){state.review=Some(entry.clone());}
                if state.review.as_ref().is_some_and(|review|review.revision.id==entry.revision.id) {
                    ui.label(tr("Sólo se revertirán los cambios cuyos valores actuales coincidan con el recibo. Las discrepancias se conservarán para revisión.","Only changes whose current values match the receipt will be reverted. Conflicts will be preserved for review."));
                    if ui.add_enabled_ui(!busy,|ui|theme::navigation(ui,tr("Confirmar deshacer","Confirm undo"),false,p)).inner.clicked(){undo=Some(entry.revision.clone());}
                    if theme::navigation(ui,tr("Cancelar","Cancel"),false,p).clicked(){state.review=None;}
                }
            });
            if self.automation.simulation.execution_preview_done() {
                frame.response.scroll_to_me(Some(egui::Align::Min));
            }
            ui.add_space(8.0);
        }
        let mut reload = false;
        if page.total > 80 || state.offset > 0 {
            if ui
                .add_enabled_ui(!busy && state.offset > 0, |ui| {
                    theme::navigation(
                        ui,
                        tr("Ejecuciones anteriores", "Previous executions"),
                        false,
                        p,
                    )
                })
                .inner
                .clicked()
            {
                state.offset = state.offset.saturating_sub(80);
                reload = true;
            }
            if ui
                .add_enabled_ui(!busy && (state.offset as u64 + 80) < page.total, |ui| {
                    theme::navigation(ui, tr("Más ejecuciones", "More executions"), false, p)
                })
                .inner
                .clicked()
            {
                state.offset = state.offset.saturating_add(80);
                reload = true;
            }
        }
        if reload {
            self.load_rule_journals();
        }
        if let Some(revision) = undo {
            self.start_rule_execution(ExecutionCommand::Undo(revision));
        }
    }
}
