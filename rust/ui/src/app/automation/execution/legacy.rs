use super::*;
use yamantaka_application::automation_plan::{PlanReply, PlanRequest, PreparedPlan};
use yamantaka_domain::automation::legacy::*;
impl YamantakaApp {
    pub(in crate::app::automation) fn prepare_legacy_review(&mut self, journal: JournalRevision) {
        if self.automation.busy() {
            return;
        }
        let state = &mut self.automation.execution;
        state.legacy_plan = None;
        state.error = None;
        state.legacy_offset = 0;
        state.history_open = false;
        let request = PlanRequest::new_legacy(journal, self.governor.clone());
        let cancel = request.cancel.clone();
        match self.runtime.submit(Action::PrepareRule(Box::new(request))) {
            Ok(()) => state.legacy_pending = Some(cancel),
            Err(error) => state.error = Some(error.to_string()),
        }
    }
    pub(in crate::app::automation) fn legacy_review_matches(&self, cancel: &Cancellation) -> bool {
        self.automation
            .execution
            .legacy_pending
            .as_ref()
            .is_some_and(|pending| pending.same_request(cancel))
    }
    pub(in crate::app::automation) fn accept_legacy_review(&mut self, reply: PlanReply) {
        let state = &mut self.automation.execution;
        state.legacy_pending = None;
        if reply.cancel.is_cancelled() || !matches!(self.surface, Surface::Automation) {
            return;
        }
        match reply.result {
            Ok(PreparedPlan::Legacy(plan)) => state.legacy_plan = Some(plan),
            Ok(PreparedPlan::Rule(_)) => {
                state.error = Some("Unexpected rule review response".into())
            }
            Err(Failure::Cancelled) => {}
            Err(error) => {
                state.history_open = true;
                state.error = Some(error.to_string());
            }
        }
    }
    pub(in crate::app::automation) fn confirm_legacy_recovery(&mut self) {
        if self.automation.busy() {
            return;
        }
        if let Some(plan) = self.automation.execution.legacy_plan.clone() {
            self.start_rule_execution(ExecutionCommand::RecoverInherited(plan));
        }
    }
    pub(in crate::app::automation) fn legacy_recovery_view(
        &mut self,
        ui: &mut egui::Ui,
        p: Palette,
    ) -> bool {
        let busy = self.automation.busy();
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let state = &mut self.automation.execution;
        if let Some(cancel) = &state.legacy_pending {
            ui.label(tr(
                "Preparando la revisión de recibos heredados…",
                "Preparing inherited receipt review…",
            ));
            if theme::navigation(ui, tr("Cancelar revisión", "Cancel review"), false, p).clicked()
            {
                cancel.cancel();
            }
            return true;
        }
        let Some(plan) = state.legacy_plan.clone() else {
            return false;
        };
        let mut confirm = false;
        let mut close = false;
        egui::Frame::new().fill(p.raised).corner_radius(8).inner_margin(16).show(ui,|ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(tr("Recuperar recibos de Flutter","Recover Flutter receipts")).size(24.0).color(p.ink));
            ui.label(&plan.journal.rule_name);
            ui.label(format!("{} {} · {} {}",plan.documents.len(),tr("documentos","documents"),plan.skipped,tr("recibos sin cambios","unchanged receipts")));
            ui.label(RichText::new(tr("Estos recibos no guardan todos los valores posteriores. Revisa el estado actual y lo que quieres restaurar. Sólo se recuperarán las acciones persistidas.","These receipts do not store every later value. Review the current state and what you want to restore. Only persisted actions will be recovered.")).color(p.secondary));
            ui.add_space(12.0);
            for document in plan.documents.iter().skip(state.legacy_offset).take(80) {
                if document.revision.is_none(){ui.label(RichText::new(tr("Documento no disponible","Unavailable document")).size(19.0).color(p.ink));ui.label(document.id.chars().take(80).collect::<String>());}else{ui.label(RichText::new(&document.title).size(19.0).color(p.ink));}
                if document.revision.is_none(){ui.label(tr("Documento ausente; no se modificará ni se recreará.","Document absent; it will not be modified or recreated."));}
                for change in &document.changes {
                    match change {
                        LegacyChange::RestoreFavorite {current,value}=>{let flag=|value:bool|if value {tr("Sí","Yes")}else{tr("No","No")};ui.label(format!("{}: {} → {}",tr("Favorito","Favorite"),flag(*current),flag(*value)));}
                        LegacyChange::RestoreProgress {current,page,page_count,mode,theme}=>{
                            if *page_count==0 {ui.label(tr("Sin páginas; progreso 0.","No pages; progress 0."));}else{ui.label(format!("{}: {} → {}",tr("Página","Page"),current.page.saturating_add(1),page.saturating_add(1)));}
                            ui.label(format!("{}: {} → {}",tr("Modo","Mode"),current.mode,mode.persisted_name()));
                            ui.label(format!("{}: {} → {}",tr("Tema","Theme"),current.theme,theme.persisted_name()));
                        }
                        LegacyChange::RemoveMembership {target,revision}=>{
                            let target=&plan.targets[*target as usize];let action=match target.kind {LegacyTargetKind::Tag=>tr("Quitar etiqueta","Remove tag"),LegacyTargetKind::Collection=>tr("Quitar de colección","Remove from collection"),LegacyTargetKind::Workspace=>tr("Quitar de cola","Remove from queue")};
                            ui.label(format!("{action}: {}",target.name));if revision.is_none(){ui.label(RichText::new(tr("Ya ausente; sin cambios.","Already absent; unchanged.")).size(12.0).color(p.secondary));}
                        }
                    }
                }
                ui.add_space(12.0);
            }
            if plan.documents.len()>80 {
                if ui.add_enabled_ui(state.legacy_offset>0,|ui|theme::navigation(ui,tr("Documentos anteriores","Previous documents"),false,p)).inner.clicked(){state.legacy_offset=state.legacy_offset.saturating_sub(80);}
                if ui.add_enabled_ui(state.legacy_offset+80<plan.documents.len(),|ui|theme::navigation(ui,tr("Más documentos","More documents"),false,p)).inner.clicked(){state.legacy_offset=state.legacy_offset.saturating_add(80);}
            }
            let button=ui.add_enabled_ui(!busy,|ui|theme::navigation(ui,tr("Confirmar recuperación","Confirm recovery"),false,p)).inner;confirm=button.clicked();
            if state.inherited_preview==10&&state.inherited_mode==9{button.scroll_to_me(Some(egui::Align::Center));}
            close=theme::navigation(ui,tr("Cerrar revisión","Close review"),false,p).clicked();
        });
        if close {
            state.legacy_plan = None;
            state.history_open = true;
        }
        if confirm {
            self.confirm_legacy_recovery();
        }
        ui.add_space(16.0);
        true
    }
}
