use super::*;
use crate::legacy_import_worker::{Command, Job, Outcome, Verification};
use yamantaka_domain::Failure;
use yamantaka_storage::{
    managed_imports::ImportTarget,
    migration::{LegacySource, MigrationProgress, MigrationReceipt, MigrationReview},
};
mod preview;
mod view;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pending {
    Review,
    Copy,
    Activate,
    Original,
}
#[derive(Default)]
pub(super) struct ImportState {
    root: Option<PathBuf>,
    source: Option<LegacySource>,
    review: Option<MigrationReview>,
    target: Option<ImportTarget>,
    receipt: Option<MigrationReceipt>,
    job: Option<Job>,
    pending: Option<Pending>,
    error: Option<Failure>,
    pub(super) open: bool,
    focus_cancel: bool,
    selected_next: bool,
    original_next: bool,
    pub(super) close_after: bool,
    verification: Option<Verification>,
    preview: Option<u8>,
    verified: bool,
}
impl ImportState {
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
    fn begin(&mut self, command: Command, pending: Pending, ctx: &egui::Context) {
        if self.busy() {
            return;
        }
        let notify = ctx.clone();
        match Job::start(command, move || notify.request_repaint(), self.verification) {
            Ok(job) => {
                self.job = Some(job);
                self.pending = Some(pending);
                self.error = None;
                self.focus_cancel = true;
            }
            Err(error) => self.error = Some(error),
        }
    }
    fn inspect(&mut self, ctx: &egui::Context) {
        if let (Some(source), Some(root)) = (&self.source, &self.root) {
            let source = source.clone();
            let target = ImportTarget::new(root.clone());
            self.review = None;
            self.receipt = None;
            self.target = Some(target);
            self.selected_next = false;
            self.original_next = false;
            self.begin(Command::Review(source), Pending::Review, ctx);
        }
    }
    fn copy(&mut self, ctx: &egui::Context) {
        if let (Some(review), Some(target)) = (&self.review, &self.target) {
            self.begin(
                Command::Copy(review.clone(), target.clone()),
                Pending::Copy,
                ctx,
            );
        }
    }
    fn activate(&mut self, ctx: &egui::Context) {
        if self.receipt.is_some()
            && let Some(target) = &self.target
        {
            self.begin(Command::Activate(target.clone()), Pending::Activate, ctx);
        }
    }
    fn reset_review(&mut self) {
        self.review = None;
        self.target = None;
        self.receipt = None;
        self.error = None;
        self.selected_next = false;
        self.original_next = false;
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"open":self.open,"busy":self.busy(),"reviewed":self.review.is_some(),"copied":self.receipt.is_some(),"selectedNext":self.selected_next,"originalNext":self.original_next,"failed":self.error.is_some(),"verified":self.verified,"books":self.review.as_ref().map(|review|review.imported_rows().get("documents").copied().unwrap_or(0))})
    }
}
impl YamantakaApp {
    pub fn configure_legacy_import(&mut self, root: PathBuf, source: LegacySource) {
        self.legacy_import.root = Some(root);
        self.legacy_import.source = Some(source);
    }
    pub(super) fn update_legacy_import(&mut self, ctx: &egui::Context) {
        if let Some(result) = self.legacy_import.job.as_mut().and_then(Job::poll) {
            let pending = self.legacy_import.pending.take();
            self.legacy_import.job = None;
            match result {
                Ok(Outcome::Reviewed(review)) => {
                    self.legacy_import.source = Some(review.source().clone());
                    self.legacy_import.review = Some(review);
                    self.legacy_import.focus_cancel = true;
                }
                Ok(Outcome::Copied(receipt)) => {
                    self.legacy_import.receipt = Some(receipt);
                    self.legacy_import.focus_cancel = true;
                }
                Ok(Outcome::Activated) => self.legacy_import.selected_next = true,
                Ok(Outcome::Original) => {
                    self.legacy_import.selected_next = false;
                    self.legacy_import.original_next = true;
                }
                Err(error) => {
                    if matches!(pending, Some(Pending::Review | Pending::Copy)) {
                        self.legacy_import.review = None;
                    }
                    self.legacy_import.error = Some(error);
                }
            }
            if self.legacy_import.close_after {
                self.legacy_import.close_after = false;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
        self.drive_legacy_import_preview(ctx);
    }
    pub(super) fn legacy_import_settings(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.add_space(24.0);
        ui.separator();
        ui.add_space(16.0);
        ui.label(
            RichText::new(self.tr("Tu biblioteca de Flutter", "Your Flutter library"))
                .size(18.0)
                .strong(),
        );
        ui.label(self.tr("Crea una copia independiente de tus libros, notas y ajustes. Conserva la biblioteca original.","Make an independent copy of your books, notes and settings. Keep the original library."));
        if theme::navigation(
            ui,
            self.tr("Importar desde Flutter", "Import from Flutter"),
            false,
            p,
        )
        .clicked()
        {
            self.legacy_import.open = true;
            self.legacy_import.focus_cancel = true;
        }
        if self
            .legacy_import
            .root
            .as_ref()
            .is_some_and(|root| root != &self.store)
            || self.legacy_import.selected_next
        {
            ui.label(if self.legacy_import.selected_next {
                self.tr(
                    "La copia importada se usará al volver a abrir.",
                    "The imported copy will be used next launch.",
                )
            } else {
                self.tr(
                    "Estás usando una biblioteca importada.",
                    "You are using an imported library.",
                )
            });
            ui.add_enabled_ui(!self.legacy_import.busy(), |ui| {
                if theme::bounded_navigation(
                    ui,
                    self.tr(
                        "Usar biblioteca Rust original al abrir",
                        "Use original Rust library next launch",
                    ),
                    p,
                )
                .clicked()
                    && let Some(root) = self.legacy_import.root.clone()
                {
                    self.legacy_import.open = true;
                    self.legacy_import
                        .begin(Command::Original(root), Pending::Original, ui.ctx());
                }
            });
        }
    }
}
