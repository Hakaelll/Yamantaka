use super::*;
use crate::backup_worker::{Job, Request};
use yamantaka_domain::{Failure, Result};

#[derive(Default)]
pub(super) struct BackupState {
    source: Option<(PathBuf, PathBuf)>,
    job: Option<Job>,
    outcome: Option<Result<PathBuf>>,
    pub(super) close_after: bool,
    preview: bool,
    verified: bool,
}
impl BackupState {
    pub(super) fn capture_active(&self) -> bool {
        self.preview || self.verified
    }
    pub(super) fn busy(&self) -> bool {
        self.job.is_some()
    }
    pub(super) fn cancel(&self) {
        if let Some(job) = &self.job {
            job.cancel();
        }
    }
    pub(super) fn ready(&self) -> bool {
        !self.busy() && !self.preview
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"busy":self.busy(),"saved":matches!(&self.outcome,Some(Ok(_))),"cancelled":matches!(&self.outcome,Some(Err(Failure::Cancelled))),"failed":matches!(&self.outcome,Some(Err(_))),"verified":self.verified})
    }
}
impl YamantakaApp {
    pub fn configure_backup(&mut self, database: Option<PathBuf>, root: PathBuf) {
        self.backups.source = database.map(|database| (database, root));
    }
    fn start_backup(
        &mut self,
        ctx: &egui::Context,
        destination: PathBuf,
        verification_cancel: bool,
    ) {
        if self.backups.busy() {
            return;
        }
        let Some((database, root)) = &self.backups.source else {
            return;
        };
        let request = Request {
            database: database.clone(),
            protected_root: root.clone(),
            destination,
            preferences: self.preferences.clone(),
            cancel: Cancellation::default(),
            verification_cancel,
        };
        let notify = ctx.clone();
        self.backups.outcome = None;
        match Job::start(request, move || notify.request_repaint()) {
            Ok(job) => self.backups.job = Some(job),
            Err(error) => self.backups.outcome = Some(Err(error)),
        }
    }
    pub(super) fn update_backup(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.backups.job.as_mut().and_then(Job::poll) {
            self.backups.job = None;
            if self.backups.preview {
                if let Ok(path) = &result {
                    let preview =
                        yamantaka_storage::backup::preview(path, &Cancellation::default()).unwrap();
                    assert_eq!(preview.counts["documents"], 10);
                    assert_eq!(preview.backup_version, 9);
                } else if matches!(result, Err(Failure::Cancelled)) {
                    assert_eq!(
                        std::fs::read(self.capture.as_ref().unwrap().with_extension("backup.json"))
                            .unwrap(),
                        b"previous complete backup"
                    );
                }
                self.backups.preview = false;
                self.backups.verified = true;
            }
            self.backups.outcome = Some(result);
            if self.backups.close_after {
                self.backups.close_after = false;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
    }
    pub fn preview_backup(&mut self, ctx: &egui::Context, mode: &str) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        self.preview_storage(ctx);
        let destination = if mode == "blocked" {
            self.store.join("preferences.json")
        } else {
            self.capture.as_ref().unwrap().with_extension("backup.json")
        };
        if mode == "cancel" {
            std::fs::write(&destination, b"previous complete backup").unwrap();
        }
        self.backups.preview = true;
        self.start_backup(ctx, destination, mode == "cancel");
    }
    pub(super) fn backup_settings(&mut self, ui: &mut egui::Ui, p: Palette) {
        let compact = ui.available_width() < 420.0;
        if compact {
            theme::display(ui, self.tr("Copias de seguridad", "Backups"), 20.0);
        } else {
            super::settings::section_title(ui, self.tr("Copias de seguridad", "Backups"));
        }
        ui.label(if compact { self.tr("Copia JSON de tus datos. PDF no incluidos.","JSON copy of your data. PDFs not included.") } else { self.tr("Guarda tus libros, notas y ajustes en JSON. Los PDF permanecen en sus ubicaciones.","Save your books, notes and settings as JSON. PDFs stay in their existing locations.") });
        ui.add_space(8.0);
        if let Some(job) = &self.backups.job {
            ui.label(self.tr(
                "Guardando copia… Puedes seguir leyendo.",
                "Saving backup… You can keep reading.",
            ));
            if let Some(progress) = job.progress() {
                ui.label(format!(
                    "{} {}",
                    progress.rows,
                    self.tr("registros en esta sección", "records in this section")
                ));
            }
            if theme::bounded_navigation(ui, self.tr("Cancelar copia", "Cancel backup"), p)
                .clicked()
            {
                job.cancel();
            }
        } else {
            if let Some(outcome) = &self.backups.outcome {
                match outcome {
                    Ok(path) => {
                        ui.label(RichText::new(self.tr("Copia guardada", "Backup saved")).strong());
                        if !compact {
                            ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                        }
                    }
                    Err(error) => {
                        ui.label(RichText::new(match error {
                        Failure::Cancelled=>self.tr("Copia cancelada. El archivo anterior se conserva.","Backup cancelled. The previous file is kept."),
                        Failure::Invalid(_)=>self.tr("Elige un archivo JSON fuera de la carpeta de la biblioteca. Comprueba también el tamaño de los datos.","Choose a JSON file outside the library folder. Also check the data size."),
                        _=>self.tr("No se pudo guardar. Comprueba el acceso o elige otra carpeta.","Could not save. Check access or choose another folder."),
                    }).color(p.accent));
                    }
                }
            }
            let action = ui
                .add_enabled_ui(self.backups.source.is_some(), |ui| {
                    theme::bounded_navigation(
                        ui,
                        self.tr("Exportar copia JSON", "Export JSON backup"),
                        p,
                    )
                })
                .inner;
            if compact && self.demo && self.capture.is_some() && self.backups.capture_active() {
                ui.scroll_to_rect(action.rect, Some(egui::Align::BOTTOM));
            }
            if action.clicked()
                && let Some(path) = yamantaka_windows::save_library_backup()
            {
                self.start_backup(ui.ctx(), path, false);
            }
            if self.backups.source.is_none() {
                ui.label(self.tr(
                    "Esta vista previa está en memoria. Abre la vista de escritorio para exportar.",
                    "This preview is in memory. Open the desktop preview to export.",
                ));
            }
            if let Some(Ok(path)) = &self.backups.outcome
                && theme::bounded_navigation(ui, self.tr("Mostrar en carpeta", "Show in folder"), p)
                    .clicked()
            {
                let _ = yamantaka_windows::reveal(path);
            }
        }
        ui.add_space(18.0);
        ui.separator();
        ui.add_space(18.0);
    }
}
