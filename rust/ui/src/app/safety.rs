use super::*;
use yamantaka_domain::startup::StartupMode;

impl YamantakaApp {
    pub(crate) fn preview_safe_dismiss(&mut self, dismiss: bool) {
        if self.demo && self.capture.is_some() && dismiss {
            self.safe_notice = false;
        }
    }
    pub(super) fn safe_summary(&self) -> serde_json::Value {
        let index = self.runtime.index_progress();
        serde_json::json!({"notice":self.safe_notice,"indexPhase":format!("{:?}",index.phase),"indexEnabled":index.enabled,"continuityLoading":self.continuity_loading()})
    }
    pub(super) fn safe_mode_dialog(&mut self, ctx: &egui::Context) {
        if !self.safe_notice {
            return;
        }
        let p = Palette::new(self.dark);
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::new("safe-startup")).show(ctx, |ui| {
            ui.set_width((ctx.content_rect().width() - 48.0).clamp(120.0, 580.0));
            egui::ScrollArea::vertical().id_salt("safe-startup-body").max_height((ctx.content_rect().height() - 48.0).max(80.0)).animated(!self.reduced_motion).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    theme::display(ui, self.tr("Modo seguro", "Safe mode"), 24.0);
                    close = theme::navigation(ui, self.tr("Cerrar", "Close"), false, p).clicked();
                });
                ui.add_space(12.0);
                ui.label(match self.startup_mode {
                    StartupMode::RepeatedUnclean { .. } => self.tr("Los últimos tres inicios terminaron sin un cierre limpio.", "The last three starts ended without a clean shutdown."),
                    StartupMode::StateUnavailable => self.tr("No se pudo actualizar el registro de inicio. Se ha conservado el archivo existente.", "The startup record could not be updated. Its existing file was preserved."),
                    _ => self.tr("Has iniciado la aplicación en modo seguro.", "You started the application in safe mode."),
                });
                ui.add_space(10.0);
                ui.label(self.tr("Puedes abrir un libro y seguir leyendo. Las pestañas anteriores no se restauran y las portadas automáticas y la indexación están detenidas.", "You can open a book and keep reading. Previous tabs are not restored, and automatic covers and indexing are stopped."));
                ui.add_space(10.0);
                ui.label(self.tr("La sesión guardada se conserva y no se registra tiempo de lectura. Las copias y el diagnóstico siguen disponibles en Ajustes. Este modo no repara ni modifica tus PDF.", "Your saved session is preserved and reading time is not recorded. Backups and diagnostics remain available in Settings. This mode does not repair or modify your PDFs."));
                ui.add_space(10.0);
                ui.label(if self.startup_mode == StartupMode::StateUnavailable { self.tr("Se mantendrá este modo mientras el registro de inicio no pueda recuperarse o actualizarse.", "This mode remains active while the startup record cannot be recovered or updated.") } else { self.tr("Después de un cierre normal, vuelve a abrir la aplicación desde tu acceso habitual para regresar al inicio normal.", "After a normal shutdown, reopen the application from your usual shortcut to return to normal startup.") });
                ui.add_space(14.0);
                if theme::control(ui, self.tr("Continuar", "Continue"), Some(theme::Icon::Right), false, true, p).clicked() { close = true; }
            });
        });
        if close || modal.should_close() {
            self.safe_notice = false;
        }
    }
}
