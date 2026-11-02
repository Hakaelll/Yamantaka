use super::*;

#[derive(Default)]
pub(super) struct BackgroundImport {
    pub(super) context_pending: Option<Cancellation>,
    pub(super) last_batch: Option<Arc<yamantaka_domain::automation::imported::ImportedDocuments>>,
    pending: Option<Cancellation>,
    catalogue_revision: u64,
    recovery: Vec<yamantaka_domain::import_recovery::PendingImport>,
    recovery_error: bool,
    recovery_loaded: bool,
    preview_recovery: Option<u8>,
}

impl BackgroundImport {
    pub(super) fn accept_recovery(
        &mut self,
        result: yamantaka_domain::Result<Vec<yamantaka_domain::import_recovery::PendingImport>>,
    ) {
        self.recovery_loaded = true;
        match result {
            Ok(jobs) => {
                self.recovery = jobs;
                self.recovery_error = false;
            }
            Err(_) => {
                self.recovery_error = true;
            }
        }
    }
    pub(super) fn recovery_ready(&self) -> bool {
        self.recovery_loaded && self.preview_recovery.is_none_or(|mode| mode == 0)
    }
    pub(super) fn recovery_summary(&self) -> serde_json::Value {
        serde_json::json!({"pendingJobs":self.recovery.len(),"loaded":self.recovery_loaded,"unavailable":self.recovery_error,
            "completed":self.recovery.iter().map(|job|job.completed).sum::<u64>()})
    }
    pub(super) fn mark_refreshed(&mut self, revision: u64) {
        self.catalogue_revision = revision;
    }
    pub(super) fn busy(&self) -> bool {
        self.pending.is_some()
    }

    pub(super) fn begin(&mut self) -> Cancellation {
        if let Some(cancel) = self.context_pending.take() {
            cancel.cancel();
        }
        self.last_batch = None;
        self.cancel();
        let token = Cancellation::default();
        self.pending = Some(token.clone());
        token
    }

    pub(super) fn submission_failed(&mut self, token: &Cancellation) {
        if self.finish(token) {
            self.recovery_loaded = true;
        }
    }
    pub(super) fn finish(&mut self, token: &Cancellation) -> bool {
        if !self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.same_request(token))
        {
            return false;
        }
        self.pending = None;
        self.recovery_loaded = false;
        true
    }

    fn cancel(&mut self) {
        self.recovery_loaded = false;
        if let Some(token) = self.pending.take() {
            token.cancel();
        }
    }
}

impl Drop for BackgroundImport {
    fn drop(&mut self) {
        self.cancel();
    }
}

impl YamantakaApp {
    pub(in crate::app) fn request_import_rule_context(&mut self, clear: bool) {
        if let Some(cancel) = self.importing.context_pending.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::ImportRuleContext {
            clear,
            cancel: cancel.clone(),
        }) {
            Ok(()) => self.importing.context_pending = Some(cancel),
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub(super) fn poll_import_progress(&mut self, ctx: &egui::Context) {
        if self.runtime.import_revision() == self.importing.catalogue_revision {
            return;
        }
        match self.try_refresh() {
            Ok(()) => {}
            Err(yamantaka_domain::Failure::QueueFull) => {
                ctx.request_repaint_after(Duration::from_millis(100))
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub fn preview_import_recovery(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.importing.preview_recovery = Some(mode.min(2));
            self.preview_tools_menu();
        }
    }
    pub(super) fn apply_import_recovery_preview(&mut self) {
        if self.importing.recovery_loaded
            && let Some(mode @ (1 | 2)) = self.importing.preview_recovery
            && let Some(job) = self.importing.recovery.first().cloned()
        {
            self.importing.preview_recovery = None;
            if mode == 1 {
                self.resume_import_job(job.id);
            } else {
                self.discard_import_job(job.id);
            }
        }
    }
    fn discard_import_job(&mut self, id: String) {
        self.importing.recovery_loaded = false;
        if let Err(error) = self.runtime.submit(Action::DiscardImport(id)) {
            self.importing.recovery_loaded = true;
            self.error = Some(error.to_string());
        }
    }
    fn resume_import_job(&mut self, id: String) {
        self.tabs.continuity.overridden = true;
        self.open_intent.leave_destination();
        let cancel = self.importing.begin();
        if let Err(error) = self
            .runtime
            .submit(Action::ResumeImport(id, cancel.clone()))
        {
            self.importing.submission_failed(&cancel);
            self.error = Some(error.to_string());
        }
    }
    fn import_recovery_menu(&mut self, ui: &mut egui::Ui, p: Palette) {
        if !self.importing.recovery_loaded {
            return;
        }
        if self.importing.recovery_error {
            ui.label(self.tr(
                "Recuperación de importación no disponible",
                "Import recovery unavailable",
            ));
            if theme::navigation(ui, self.tr("Reintentar", "Retry"), false, p).clicked() {
                self.send(Action::PendingImports);
            }
        }
        if self.importing.recovery.is_empty() {
            return;
        }
        let label = format!(
            "{} ({})",
            self.tr("Importaciones pendientes", "Pending imports"),
            self.importing.recovery.len()
        );
        if self.importing.preview_recovery == Some(0) {
            let submenu = egui::containers::menu::SubMenu::id_from_widget_id(ui.next_auto_id());
            egui::containers::menu::MenuState::from_id(ui.ctx(), submenu, |_| {});
            egui::containers::menu::MenuState::from_ui(ui, |state, _| {
                state.open_item = Some(submenu)
            });
        }
        ui.menu_button(label, |ui| {
            ui.set_max_width(280.0_f32.min((ui.ctx().content_rect().width()-24.0).max(120.0)));
            let height=(ui.ctx().content_rect().bottom()-ui.cursor().top()-12.0).max(40.0);
            egui::ScrollArea::vertical().max_height(height).show(ui, |ui| {
                for job in self.importing.recovery.clone() {
                    ui.add(egui::Label::new(RichText::new(&job.label).color(p.ink)).truncate());
                    ui.label(RichText::new(format!("{} {}",job.completed,self.tr("confirmados", "confirmed"))).size(12.0).color(p.secondary));
                    ui.horizontal_wrapped(|ui| {
                        if theme::navigation(ui,self.tr("Reanudar", "Resume"),false,p).clicked() {
                            self.resume_import_job(job.id.clone());ui.close();
                        }
                        if theme::navigation(ui,self.tr("Descartar", "Discard"),false,p)
                            .on_hover_text(self.tr("Conservar los libros guardados y olvidar solo este trabajo pendiente.",
                                "Keep saved books and forget only this pending job.")).clicked() {
                            self.discard_import_job(job.id);ui.close();
                        }
                    });
                    ui.separator();
                }
            });
        });
    }
    pub(super) fn operation_actions(&mut self, ui: &mut egui::Ui, p: Palette) {
        if theme::navigation(
            ui,
            self.tr("Recuperación de documentos", "Document recovery"),
            false,
            p,
        )
        .clicked()
        {
            self.show_quarantine();
            ui.close();
        }
        if self.startup_mode.is_safe()
            && theme::navigation(ui, self.tr("Modo seguro", "Safe mode"), true, p).clicked()
        {
            self.safe_notice = true;
            ui.close();
        }
        if theme::navigation(
            ui,
            &format!(
                "{} · {}",
                self.tr("Comandos", "Commands"),
                shortcuts::bindings::caption(
                    self.shortcut_bindings
                        .token(shortcuts::Shortcut::CommandPalette)
                )
            ),
            false,
            p,
        )
        .clicked()
        {
            self.open_command_palette(ui.ctx());
            ui.close();
        }

        if !self.importing.busy() {
            if self
                .importing
                .last_batch
                .as_ref()
                .is_some_and(|batch| !batch.is_empty())
                && theme::navigation(
                    ui,
                    self.tr(
                        "Reglas guardadas: lote disponible",
                        "Saved rules: batch available",
                    ),
                    false,
                    p,
                )
                .clicked()
            {
                self.show_automation_rules();
                ui.close();
            }
            self.import_recovery_menu(ui, p);
        }
        if self.importing.pending.is_some()
            && theme::navigation(
                ui,
                self.tr("Cancelar importación", "Cancel import"),
                false,
                p,
            )
            .clicked()
        {
            self.importing.cancel();
            self.runtime.wake();
            self.send(Action::PendingImports);
            self.refresh();
            ui.close();
        }
        if !self.tabs.closing.busy()
            && (self.opening.busy() || self.tabs.busy())
            && theme::navigation(ui, self.tr("Cancelar apertura", "Cancel opening"), false, p)
                .clicked()
        {
            self.cancel_reader_opening();
            ui.close();
        }
    }
}
