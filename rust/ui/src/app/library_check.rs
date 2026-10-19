use super::*;
use yamantaka_application::library_check::{
    LibraryCheckPhase, LibraryCheckProgress, LibraryCheckReply, LibraryCheckRequest,
};
#[derive(Default)]
pub(super) struct CheckState {
    pending: Option<Cancellation>,
    progress: LibraryCheckProgress,
    error: bool,
    preview: Option<u8>,
    pub(super) capture_mode: bool,
}
impl CheckState {
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && self.preview.is_none()
    }
    pub(super) fn cancel(&self) {
        if let Some(token) = &self.pending {
            token.cancel();
        }
    }
    pub(super) fn accept(&mut self, reply: LibraryCheckReply) {
        if self
            .pending
            .as_ref()
            .is_some_and(|token| token.same_request(&reply.cancel))
        {
            self.pending = None;
            self.progress = reply.progress;
            self.error = reply.error.is_some();
        }
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        let c = self.progress.counts;
        serde_json::json!({"pending":self.pending.is_some(),"phase":format!("{:?}",self.progress.phase),"checked":c.checked(),
            "available":c.available,"missing":c.missing,"changed":c.changed,"unverifiable":c.unverifiable,"failed":c.failed,"error":self.error})
    }
}
impl Drop for CheckState {
    fn drop(&mut self) {
        self.cancel();
    }
}
impl YamantakaApp {
    pub fn preview_library_check(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.library_check.preview = Some(mode);
            self.library_check.capture_mode = true;
            self.preview_tools_menu();
        }
    }
    fn start_library_check(&mut self) {
        if self.library_check.pending.is_some() {
            return;
        }
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::CheckLibrary(Box::new(LibraryCheckRequest {
                governor: self.governor.clone(),
                cancel: cancel.clone(),
            }))) {
            Ok(()) => {
                self.library_check.pending = Some(cancel);
                self.library_check.progress = LibraryCheckProgress {
                    phase: LibraryCheckPhase::Running,
                    ..Default::default()
                };
                self.library_check.error = false;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(super) fn update_library_check(&mut self) {
        if let Some(snapshot) = self.runtime.library_check_progress()
            && self
                .library_check
                .pending
                .as_ref()
                .is_some_and(|token| token.same_request(&snapshot.cancel))
        {
            self.library_check.progress = snapshot.progress;
        }
        if let Some(mode) = self.library_check.preview
            && !self.documents.is_empty()
        {
            self.library_check.preview = None;
            self.start_library_check();
            if mode == 1 {
                self.library_check.cancel();
                self.runtime.wake();
            }
            if mode == 2 {
                self.missing_only = true;
                self.modified_only = false;
            }
            if mode == 3 {
                self.modified_only = true;
                self.missing_only = false;
            }
        }
    }
    pub(super) fn library_check_menu(&mut self, ui: &mut egui::Ui, p: Palette) {
        if self.library_check.capture_mode {
            let submenu = egui::containers::menu::SubMenu::id_from_widget_id(ui.next_auto_id());
            egui::containers::menu::MenuState::from_id(ui.ctx(), submenu, |_| {});
            egui::containers::menu::MenuState::from_ui(ui, |state, _| {
                state.open_item = Some(submenu)
            });
        }
        ui.menu_button(self.tr("Comprobar archivos", "Check library files"), |ui| {
            ui.set_max_width(300.0_f32.min((ui.ctx().content_rect().width() - 24.0).max(120.0)));
            let height = (ui.ctx().content_rect().bottom() - ui.cursor().top() - 12.0).max(40.0);
            ui.set_max_height(height);
            egui::ScrollArea::vertical()
                .max_height(height)
                .show(ui, |ui| {
                    let phase = self.library_check.progress.phase;
                    let label = match phase {
                        LibraryCheckPhase::Idle => self.tr(
                            "Verifica los PDF de toda la biblioteca.",
                            "Verify PDFs throughout the library.",
                        ),
                        LibraryCheckPhase::Running
                            if self
                                .library_check
                                .pending
                                .as_ref()
                                .is_some_and(Cancellation::is_cancelled) =>
                        {
                            self.tr("Finalizando cancelación…", "Finishing cancellation…")
                        }
                        LibraryCheckPhase::Running => self.tr(
                            "Comprobación en segundo plano",
                            "Checking in the background",
                        ),
                        LibraryCheckPhase::Completed => {
                            self.tr("Comprobación terminada", "Check completed")
                        }
                        LibraryCheckPhase::Cancelled => {
                            self.tr("Comprobación cancelada", "Check cancelled")
                        }
                        LibraryCheckPhase::Failed => self.tr(
                            "No se pudo completar. Reintenta.",
                            "The check could not finish. Retry.",
                        ),
                    };
                    ui.add(egui::Label::new(RichText::new(label).color(p.ink)).wrap());
                    let c = self.library_check.progress.counts;
                    if phase != LibraryCheckPhase::Idle {
                        ui.label(format!(
                            "{}: {}",
                            self.tr("Comprobados", "Checked"),
                            c.checked()
                        ));
                        for (es, en, count) in [
                            ("Disponibles", "Available", c.available),
                            ("Ausentes", "Missing", c.missing),
                            ("Modificados", "Changed", c.changed),
                            ("Sin verificar", "Unverifiable", c.unverifiable),
                            ("Errores", "Errors", c.failed),
                        ] {
                            ui.label(
                                RichText::new(format!("{}: {count}", self.tr(es, en)))
                                    .color(p.secondary),
                            );
                        }
                    }
                    if self.library_check.pending.is_some() {
                        if theme::navigation(
                            ui,
                            self.tr("Cancelar comprobación", "Cancel check"),
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.library_check.cancel();
                            self.runtime.wake();
                        }
                    } else if theme::navigation(
                        ui,
                        self.tr("Comprobar ahora", "Check now"),
                        false,
                        p,
                    )
                    .clicked()
                    {
                        self.start_library_check();
                        ui.close();
                    }
                    if c.missing > 0
                        && theme::navigation(ui, self.tr("Ver ausentes", "Show missing"), false, p)
                            .clicked()
                    {
                        self.show_surface(Surface::Library);
                        self.query.clear();
                        self.favorites = false;
                        self.reading_state = LibraryReadingState::All;
                        self.missing_only = true;
                        self.modified_only = false;
                        self.offset = 0;
                        self.refresh();
                        ui.close();
                    }
                    if c.changed > 0
                        && theme::navigation(
                            ui,
                            self.tr("Ver modificados", "Show changed"),
                            false,
                            p,
                        )
                        .clicked()
                    {
                        self.show_surface(Surface::Library);
                        self.query.clear();
                        self.favorites = false;
                        self.reading_state = LibraryReadingState::All;
                        self.modified_only = true;
                        self.missing_only = false;
                        self.offset = 0;
                        self.refresh();
                        ui.close();
                    }
                });
        });
    }
}
