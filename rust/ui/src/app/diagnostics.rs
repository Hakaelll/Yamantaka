use super::*;
use crate::diagnostics_worker::{Job, Operation, Outcome, Snapshot};
use yamantaka_application::diagnostics::Renderer;
use yamantaka_domain::Failure;
mod preview;

#[derive(Default)]
pub(super) struct DiagnosticsState {
    source: Option<(PathBuf, PathBuf)>,
    job: Option<Job>,
    snapshot: Option<Snapshot>,
    error: Option<Failure>,
    saved: bool,
    copied: bool,
    pub(super) open: bool,
    preview: u8,
    preview_mode: u8,
    verified: bool,
}
impl DiagnosticsState {
    pub fn ready(&self) -> bool {
        self.job.is_none() && matches!(self.preview, 0 | 90)
    }
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({"open":self.open,"busy":self.job.is_some(),"collected":self.snapshot.is_some(),"saved":self.saved,"copied":self.copied,"cancelled":matches!(self.error,Some(Failure::Cancelled)),"occupied":matches!(self.error,Some(Failure::DestinationOccupied)),"failed":self.error.is_some(),"verified":self.verified})
    }
}
impl YamantakaApp {
    pub fn configure_diagnostics(&mut self, source: Option<PathBuf>, root: PathBuf) {
        self.diagnostics.source = source.map(|source| (source, root));
    }
    fn start_diagnostics(&mut self, ctx: &egui::Context, save: Option<PathBuf>) {
        if self.diagnostics.job.is_some() {
            return;
        }
        let Some((source, root)) = &self.diagnostics.source else {
            return;
        };
        let operation = if let Some(destination) = save {
            let Some(snapshot) = &self.diagnostics.snapshot else {
                return;
            };
            Operation::Save {
                root: root.clone(),
                destination,
                json: snapshot.json.clone(),
            }
        } else {
            Operation::Collect {
                renderer: if self.presenter.is_some() {
                    Renderer::Wgpu
                } else {
                    Renderer::Glow
                },
                governor: self.governor.clone(),
            }
        };
        let ctx = ctx.clone();
        match Job::start(source.clone(), operation, move || ctx.request_repaint()) {
            Ok(job) => {
                self.diagnostics.job = Some(job);
                self.diagnostics.error = None;
                self.diagnostics.saved = false;
                self.diagnostics.copied = false;
            }
            Err(error) => self.diagnostics.error = Some(error),
        }
    }
    pub(super) fn update_diagnostics(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.diagnostics.job.as_mut().and_then(Job::poll) {
            self.diagnostics.job = None;
            match result {
                Ok(Outcome::Collected(snapshot)) => {
                    self.diagnostics.snapshot = Some(*snapshot);
                    self.diagnostics.error = None;
                }
                Ok(Outcome::Saved) => self.diagnostics.saved = true,
                Err(error) => self.diagnostics.error = Some(error),
            }
        }
        self.advance_diagnostics_preview(ctx);
    }
    pub(super) fn diagnostics_settings(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(16.0);
        ui.label(
            RichText::new(self.tr("Diagnóstico local", "Local diagnostics"))
                .size(20.0)
                .strong(),
        );
        ui.label(self.tr("Revisa versiones, recuentos y memoria sin incluir rutas, títulos ni contenido de tus documentos.", "Review versions, counts and memory without document paths, titles or content."));
        ui.add_space(12.0);
        let label = self.tr("Revisar diagnóstico", "Review diagnostics");
        if ui
            .add_enabled_ui(self.diagnostics.source.is_some(), |ui| {
                theme::navigation(ui, label, false, p)
            })
            .inner
            .clicked()
        {
            self.diagnostics.open = true;
            if self.diagnostics.snapshot.is_none() {
                self.start_diagnostics(ui.ctx(), None);
            }
        }
    }
    pub(super) fn diagnostics_dialog(&mut self, ctx: &egui::Context) {
        if !self.diagnostics.open {
            return;
        }
        let p = Palette::new(self.dark);
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let mut close = false;
        let mut refresh = false;
        let mut save = false;
        let modal = egui::Modal::new(egui::Id::new("local-diagnostics")).show(ctx, |ui| {
            ui.set_width((ctx.content_rect().width() - 48.0).clamp(120.0, 620.0));
            ui.set_max_height((ctx.content_rect().height() - 48.0).max(80.0));
            egui::ScrollArea::vertical().id_salt("diagnostics-review").max_height((ctx.content_rect().height() - 48.0).max(80.0)).animated(!self.reduced_motion).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    theme::display(ui, tr("Diagnóstico local", "Local diagnostics"), 24.0);
                    close = theme::navigation(ui, tr("Cerrar", "Close"), false, p).clicked();
                });
                ui.add_space(10.0);
                ui.label(tr("Este informe no incluye rutas, títulos, búsquedas, notas ni contraseñas. Solo se guarda o copia cuando lo eliges.", "This report excludes paths, titles, searches, notes and passwords. It is saved or copied only when you choose."));
                ui.add_space(14.0);
                if let Some(snapshot) = &self.diagnostics.snapshot {
                    ui.label(tr("Última instantánea. Actualiza el informe para recoger cambios nuevos.", "Last snapshot. Refresh the report to collect newer changes."));
                    let data = &snapshot.report.database;
                    ui.label(format!("{} · SQLite v{}", env!("CARGO_PKG_VERSION"), data.schema));
                    ui.horizontal_wrapped(|ui| {
                        for (count, label) in [(data.counts.documents, tr("documentos", "documents")), (data.counts.annotations, tr("anotaciones", "annotations")), (data.counts.missing_documents, tr("archivos ausentes", "missing files"))] { ui.label(format!("{count} {label}")); }
                    });
                    ui.label(if data.quick_check_ok && data.foreign_keys_ok { tr("Comprobación rápida y referencias: correctas.", "Quick check and references: OK.") } else { tr("Se detectaron problemas de integridad. Conserva una copia antes de recuperar datos.", "Integrity issues were found. Keep a copy before recovering data.") });
                    ui.collapsing(tr("Ver informe completo", "View full report"), |ui| { ui.add(egui::Label::new(RichText::new(snapshot.json.as_ref()).monospace().size(11.0)).wrap().selectable(true)); });
                }
                if let Some(error) = &self.diagnostics.error {
                    ui.add_space(10.0);
                    ui.label(match error { Failure::Cancelled => tr("Operación cancelada. Los archivos existentes se conservan.", "Operation cancelled. Existing files are preserved."), Failure::DestinationOccupied => tr("Ese archivo ya existe. Elige otro nombre para conservarlo.", "That file already exists. Choose another name to preserve it."), _ => tr("No se pudo completar el diagnóstico. Puedes volver a intentarlo.", "Diagnostics could not finish. You can retry.") });
                }
                if self.diagnostics.saved { ui.label(tr("Informe guardado.", "Report saved.")); }
                if self.diagnostics.copied { ui.label(tr("Informe copiado.", "Report copied.")); }
                ui.add_space(14.0);
                if let Some(job) = &self.diagnostics.job {
                    ui.label(tr("Recogiendo o guardando el informe…", "Collecting or saving the report…"));
                    if theme::navigation(ui, tr("Cancelar", "Cancel"), false, p).clicked() { job.cancel(); }
                } else {
                    ui.horizontal_wrapped(|ui| {
                        refresh = theme::navigation(ui, tr("Actualizar", "Refresh"), false, p).clicked();
                        ui.add_enabled_ui(self.diagnostics.snapshot.is_some(), |ui| {
                            if theme::navigation(ui, tr("Copiar informe", "Copy report"), false, p).clicked() && let Some(snapshot) = &self.diagnostics.snapshot { ctx.copy_text(snapshot.json.to_string()); self.diagnostics.copied = true; }
                            save = theme::control(ui, tr("Guardar JSON", "Save JSON"), Some(theme::Icon::Right), false, true, p).clicked();
                        });
                    });
                }
            });
        });
        if modal.should_close() || close {
            self.diagnostics.open = false;
            if let Some(job) = &self.diagnostics.job {
                job.cancel();
            }
        } else if refresh {
            self.start_diagnostics(ctx, None);
        } else if save && let Some(path) = yamantaka_windows::save_diagnostics_report() {
            self.start_diagnostics(ctx, Some(path));
        }
    }
}
