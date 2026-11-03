use super::*;
use crate::restore_worker::{Command, Job, Outcome, Prepared, Review};
use std::{cell::RefCell, rc::Rc};

pub struct RestoreCommit {
    pub preview: Prepared,
    pub store: PathBuf,
}
pub type RestoreHandover = Rc<RefCell<Option<RestoreCommit>>>;
#[derive(Default)]
pub(super) struct RestoreState {
    store: Option<PathBuf>,
    handover: Option<RestoreHandover>,
    job: Option<Job>,
    review: Option<Review>,
    error: Option<yamantaka_domain::Failure>,
    pub(super) open: bool,
    pub(super) close_after: bool,
    focus_cancel: bool,
    queued: bool,
    rollback: bool,
    preview: Option<bool>,
}
impl RestoreState {
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"open":self.open,"busy":self.busy(),"reviewed":self.review.is_some(),"books":self.review.as_ref().and_then(|review|review.counts().get("documents")).copied(),"queued":self.queued,"rollback":self.rollback})
    }
    pub(super) fn busy(&self) -> bool {
        self.job.is_some()
    }
    pub(super) fn ready(&self) -> bool {
        !self.busy() && self.preview.is_none()
    }
    pub(super) fn cancel(&self) {
        if let Some(job) = &self.job {
            job.cancel();
        }
    }
    fn start(&mut self, command: Command, ctx: &egui::Context) {
        if self.busy() || self.queued {
            return;
        }
        self.rollback = match &command {
            Command::ReviewRollback { .. } => true,
            Command::Prepare(r) => r.rollback(),
            Command::Review(_) | Command::ReviewExpected { .. } => false,
            Command::ReviewPending { previous, .. } => previous.is_some(),
            Command::ReviewSelection { .. } => false,
        };
        self.error = None;
        self.open = true;
        self.focus_cancel = true;
        let notify = ctx.clone();
        match Job::start(command, move || notify.request_repaint()) {
            Ok(job) => self.job = Some(job),
            Err(error) => self.error = Some(error),
        }
    }
}
impl YamantakaApp {
    pub(super) fn cancel_restore_handover(&mut self) {
        if self.backup_restore.queued {
            if let Some(handover) = &self.backup_restore.handover {
                handover.borrow_mut().take();
            }
            self.backup_restore.queued = false;
            self.backup_restore.open = false;
            self.backup_restore.close_after = false;
        }
    }
    pub fn configure_restore(&mut self, store: Option<PathBuf>, handover: RestoreHandover) {
        self.backup_restore.store = store;
        self.backup_restore.handover = Some(handover);
    }
    pub fn preview_restore(&mut self, ctx: &egui::Context, path: PathBuf, apply: bool) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        self.preview_storage(ctx);
        self.backup_restore.preview = Some(apply);
        self.backup_restore.start(Command::Review(path), ctx);
    }
    pub fn preview_rollback(&mut self, ctx: &egui::Context, source: PathBuf, apply: bool) {
        if self.demo && self.capture.is_some() {
            self.preview_storage(ctx);
            if let Some(active) = self.backup_restore.store.clone() {
                self.backup_restore.preview = Some(apply);
                self.backup_restore
                    .start(Command::ReviewRollback { active, source }, ctx);
            }
        }
    }
    pub(super) fn update_restore(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.backup_restore.job.as_mut().and_then(Job::poll) {
            self.backup_restore.job = None;
            match result {
                Ok(Outcome::Reviewed(review)) => {
                    if self.backup_restore.preview == Some(true) {
                        self.backup_restore
                            .start(Command::Prepare(review.clone()), ctx);
                    } else {
                        self.backup_restore.preview = None;
                    }
                    self.backup_restore.review = Some(review);
                    self.backup_restore.focus_cancel = true;
                }
                Ok(Outcome::Prepared(preview)) => {
                    if self.backup_restore.open
                        && let (Some(store), Some(handover)) =
                            (&self.backup_restore.store, &self.backup_restore.handover)
                    {
                        *handover.borrow_mut() = Some(RestoreCommit {
                            preview: *preview,
                            store: store.clone(),
                        });
                        self.backup_restore.queued = true;
                    }
                    self.backup_restore.preview = None;
                }
                Err(error) => {
                    self.backup_restore.error = Some(error);
                    self.backup_restore.preview = None;
                }
            }
            if self.backup_restore.close_after {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
        if self.backup_restore.queued {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
    pub(in crate::app) fn preview_apply_duplicate_restore(&mut self) {
        if self.demo && self.capture.is_some() && self.backup_restore.busy() {
            self.backup_restore.preview = Some(true);
        }
    }
    pub(in crate::app) fn review_duplicate_undo(
        &mut self,
        ctx: &egui::Context,
        path: PathBuf,
        sha256: String,
    ) {
        if self.backup_restore.store.is_some() && !self.backup_restore.busy() {
            self.backup_restore.review = None;
            self.backup_restore
                .start(Command::ReviewExpected { path, sha256 }, ctx);
        }
    }
    pub(super) fn restore_button(&mut self, ui: &mut egui::Ui, p: Palette) {
        if ui
            .add_enabled_ui(
                self.backup_restore.store.is_some()
                    && !self.backups.busy()
                    && !self.backup_restore.busy()
                    && !self.backup_restore.queued,
                |ui| {
                    theme::bounded_navigation(
                        ui,
                        self.tr("Restaurar copia JSON…", "Restore JSON backup…"),
                        p,
                    )
                },
            )
            .inner
            .clicked()
            && let Some(path) = yamantaka_windows::pick_library_backup()
        {
            self.backup_restore.review = None;
            self.backup_restore.start(Command::Review(path), ui.ctx());
        }
    }
    pub(super) fn rollback_button(&mut self, ui: &mut egui::Ui, p: Palette) {
        let enabled = self.backup_restore.store.is_some()
            && !self.backups.busy()
            && !self.backup_restore.busy()
            && !self.backup_restore.queued;
        if ui
            .add_enabled_ui(enabled, |ui| {
                theme::bounded_navigation(
                    ui,
                    self.tr("Biblioteca anterior…", "Previous library…"),
                    p,
                )
            })
            .inner
            .clicked()
            && let Some(active) = self.backup_restore.store.clone()
            && let Some(parent) = active.parent()
            && let Some(source) = yamantaka_windows::pick_library_safety_folder(parent)
        {
            self.backup_restore.review = None;
            self.backup_restore
                .start(Command::ReviewRollback { active, source }, ui.ctx());
        }
    }
    pub(super) fn restore_dialog(&mut self, ctx: &egui::Context) {
        if !self.backup_restore.open {
            return;
        }
        let mut state = std::mem::take(&mut self.backup_restore);
        let p = Palette::new(self.dark);
        let compact = ctx.content_rect().height() < 420.0;
        let mut close = false;
        let modal=egui::Modal::new(egui::Id::new("backup-restore-review")).frame(egui::Frame::new().fill(p.surface).stroke(egui::Stroke::new(1.0,p.line)).corner_radius(12).inner_margin(16)).show(ctx,|ui|{
            ui.set_width((ctx.content_rect().width()-56.0).clamp(180.0,520.0));
            let header_start=ui.cursor().top();
            theme::display(ui,if state.rollback {self.tr("Biblioteca anterior","Previous library")}else{self.tr("Restaurar biblioteca","Restore library")},22.0);
            if compact && state.review.is_some() {
                ui.label(RichText::new(self.tr("Se activará esta copia y se guardará la actual.","This copy will become active; the current one will be kept.")).size(12.0).color(p.secondary));
            }
            let header_height=ui.cursor().top()-header_start;
            egui::ScrollArea::vertical().max_height((ctx.content_rect().height()-64.0-header_height-56.0).clamp(32.0,440.0)).show(ui,|ui|{
                if state.queued {ui.label(self.tr("Cerrando la biblioteca para restaurar…","Closing the library to restore…"));}
                else if state.busy() {ui.label(self.tr("Comprobando la copia…","Checking the backup…"));}
                else if let Some(error)=&state.error {
                    ui.label(RichText::new(match error {
                        yamantaka_domain::Failure::BackupSourceChanged=>self.tr("La copia ha cambiado. Ciérrala y vuelve a seleccionarla para revisar.","The backup changed. Close this review and select it again."),
                        yamantaka_domain::Failure::Cancelled=>self.tr("Restauración cancelada. Tu biblioteca se conserva.","Restore cancelled. Your library is kept."),
                        _=>self.tr("No se pudo validar la copia. Tu biblioteca se conserva. Selecciona otra copia.","Could not validate this backup. Your library is kept. Choose another backup."),
                    }).color(p.accent));
                } else if let Some(review)=&state.review {
                    let source=format!("{} · v{}",review.source().file_name().unwrap_or_default().to_string_lossy(),review.version());
                    ui.add(if compact {egui::Label::new(source).truncate()}else{egui::Label::new(source).wrap()});
                    for (table,es,en) in [("documents","Libros","Books"),("annotations","Anotaciones","Annotations"),("bookmarks","Marcadores","Bookmarks"),("collections","Colecciones","Collections")] {ui.label(format!("{}: {}",self.tr(es,en),review.counts().get(table).copied().unwrap_or(0)));}
                    if state.rollback {ui.label(self.tr("Se activará esta carpeta completa y la biblioteca actual quedará en otra carpeta de seguridad. Cierra otras instancias que utilicen la copia elegida.","This entire folder will become active, and the current library will be kept in another safety folder. Close other instances using the selected copy."));}
                    ui.add_space(8.0);ui.label(self.tr("Yamantaka se cerrará para instalar esta copia. Se conservará la biblioteca actual en una carpeta de seguridad. Tus PDF no se modifican.","Yamantaka will close to install this backup. Your current library will be kept in a safety folder. Your PDFs are not changed."));
                }
            });
            ui.horizontal_wrapped(|ui|{
                ui.add_enabled_ui(!state.queued,|ui|{
                    let cancel=theme::navigation(ui,self.tr("Cancelar","Cancel"),false,p);if state.focus_cancel{cancel.request_focus();state.focus_cancel=false;}
                    if cancel.clicked(){state.cancel();close=true;}
                    if !state.busy()&&state.error.is_none()&&let Some(review)=state.review.clone()&&theme::bounded_navigation(ui,if state.rollback {self.tr("Volver y cerrar","Return and close")}else{self.tr("Restaurar y cerrar","Restore and close")},p).clicked(){state.start(Command::Prepare(review),ctx);}
                });
            });
        });
        if !state.queued && (close || modal.should_close()) {
            state.cancel();
            state.open = false;
        }
        self.backup_restore = state;
    }
}
