use super::*;
use crate::startup_repair_worker::{Job, Operation, Outcome};
use yamantaka_storage::startup::repair::Review;

#[derive(Default)]
pub(super) struct State {
    job: Option<Job>,
    review: Option<Review>,
    preserved: Option<PathBuf>,
    error: Option<String>,
    preview: Option<&'static str>,
    verified: bool,
}
impl YamantakaApp {
    fn startup_record_job(&mut self, operation: Operation, ctx: &egui::Context) {
        let state = &mut self.startup_repair;
        state.review = None;
        state.error = None;
        let notify = ctx.clone();
        match Job::start(operation, move || notify.request_repaint()) {
            Ok(job) => state.job = Some(job),
            Err(error) => state.error = Some(error.to_string()),
        }
    }
    pub(super) fn poll_startup_record_repair(&mut self, ctx: &egui::Context) {
        let Some(result) = self.startup_repair.job.as_mut().and_then(Job::poll) else {
            return;
        };
        self.startup_repair.job = None;
        match result {
            Ok(Outcome::Reviewed(review)) => match self.startup_repair.preview {
                Some("apply") => self.startup_record_job(Operation::Apply(review), ctx),
                Some("stale") => {
                    std::fs::write(
                        self.store.join("rust-startup-v1.json"),
                        b"changed synthetic startup record",
                    )
                    .unwrap();
                    self.startup_record_job(Operation::Apply(review), ctx);
                }
                Some("cancel") => {
                    self.startup_repair.preview = None;
                    self.startup_repair.verified = true;
                }
                _ => {
                    self.startup_repair.review = Some(review);
                    if self.startup_repair.preview == Some("review") {
                        self.startup_repair.preview = None;
                        self.startup_repair.verified = true;
                    }
                }
            },
            Ok(Outcome::Repaired(repaired)) => {
                self.startup_repair.preserved = Some(repaired.preserved);
                if self.startup_repair.preview == Some("apply") {
                    let next = yamantaka_storage::startup::StartupRun::begin(&self.store, false);
                    assert!(!next.mode().is_safe());
                    next.finish().unwrap();
                    self.startup_repair.preview = None;
                    self.startup_repair.verified = true;
                }
            }
            Err(error) => {
                self.startup_repair.error = Some(error.to_string());
                if self.startup_repair.preview == Some("stale") {
                    assert_eq!(
                        std::fs::read(self.store.join("rust-startup-v1.json")).unwrap(),
                        b"changed synthetic startup record"
                    );
                    self.startup_repair.preview = None;
                    self.startup_repair.verified = true;
                } else if self.startup_repair.preview.is_some() {
                    panic!("Synthetic startup record operation failed: {error}");
                }
            }
        }
    }
    pub(crate) fn preview_startup_record_repair(
        &mut self,
        mode: &'static str,
        ctx: &egui::Context,
    ) {
        if self.demo && self.capture.is_some() {
            self.startup_repair.preview = Some(mode);
            self.startup_record_job(Operation::Review(self.store.clone()), ctx);
        }
    }
    pub(super) fn startup_record_repair_ready(&self) -> bool {
        self.startup_repair.job.is_none() && self.startup_repair.preview.is_none()
    }
    pub(super) fn startup_record_repair_summary(&self) -> serde_json::Value {
        serde_json::json!({"reviewed":self.startup_repair.review.is_some(),"repaired":self.startup_repair.preserved.is_some(),"error":self.startup_repair.error.is_some(),"verified":self.startup_repair.verified})
    }
    pub(super) fn startup_record_repair_controls(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.add_space(12.0);
        ui.separator();
        if let Some(path) = self.startup_repair.preserved.clone() {
            ui.label(self.tr("Registro recuperado. El archivo anterior se ha conservado. Cierra la aplicación y vuelve a abrirla para salir del modo seguro.", "Startup record recovered. The previous file was preserved. Close and reopen the app to leave safe mode."));
            if theme::navigation(
                ui,
                self.tr("Ver archivo conservado", "Show preserved file"),
                false,
                p,
            )
            .clicked()
                && let Err(error) = yamantaka_windows::reveal(&path)
            {
                self.startup_repair.error = Some(error.to_string());
            }
        } else if let Some(review) = self.startup_repair.review.clone() {
            ui.label(self.tr("Se conservará el registro actual con otro nombre antes de crear uno nuevo. La biblioteca, las preferencias y los PDF se mantienen.", "The current record will be preserved under another name before creating a new one. Your library, preferences and PDFs are kept."));
            ui.label(format!(
                "{} {}",
                review.bytes,
                self.tr("bytes conservados", "bytes to preserve")
            ));
            ui.horizontal_wrapped(|ui| {
                if theme::navigation(
                    ui,
                    self.tr("Conservar y recuperar", "Preserve and recover"),
                    false,
                    p,
                )
                .clicked()
                {
                    self.startup_record_job(Operation::Apply(review), ui.ctx());
                }
                if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked() {
                    self.startup_repair.review = None;
                }
            });
        } else {
            ui.add_enabled_ui(self.startup_repair.job.is_none(), |ui| {
                if theme::navigation(
                    ui,
                    self.tr("Revisar registro de inicio", "Review startup record"),
                    false,
                    p,
                )
                .clicked()
                {
                    self.startup_record_job(Operation::Review(self.store.clone()), ui.ctx());
                }
            });
        }
        if self.startup_repair.error.is_some() {
            Self::file_status_label(ui, self.tr("No se pudo completar la operación. Se conservan los archivos. Comprueba los permisos y vuelve a revisar el registro.", "The operation could not finish. Files are preserved. Check permissions and review the record again."), p);
        }
    }
}
