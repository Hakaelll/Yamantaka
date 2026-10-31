//! Recovery does not need the failed library, a PDF engine, or saved preferences.
use crate::{
    restore_worker::{Command, Job, Outcome},
    theme,
};
use eframe::egui;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};
use yamantaka_storage::{
    Database,
    backup::{self, RestorePreview, RestoreReview},
    preferences::Preferences,
};

pub mod preview;

#[derive(Clone, Copy, Debug)]
pub enum FailedStage {
    Preferences,
    Library,
}

pub fn load(store: &Path) -> std::result::Result<(Database, Preferences), FailedStage> {
    // Validate settings before opening a database which may need a schema upgrade.
    let preferences =
        Preferences::load(&store.join("preferences.json")).map_err(|_| FailedStage::Preferences)?;
    let database =
        Database::open(&store.join("yamantaka.sqlite3")).map_err(|_| FailedStage::Library)?;
    Ok((database, preferences))
}

pub fn run(
    store: PathBuf,
    stage: FailedStage,
    english: bool,
    software: bool,
    verification: Option<preview::Verification>,
) -> std::result::Result<bool, Box<dyn std::error::Error>> {
    let handover: Rc<RefCell<Option<RestorePreview>>> = Default::default();
    let output = handover.clone();
    let install_store = store.clone();
    let verification_result = verification.clone();
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Yamantaka · Recovery")
            .with_inner_size([760.0, 600.0])
            .with_min_inner_size([640.0, 480.0]),
        renderer: if software {
            eframe::Renderer::Glow
        } else {
            eframe::Renderer::Wgpu
        },
        ..Default::default()
    };
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup {
        setup.instance_descriptor.backends = eframe::wgpu::Backends::DX12;
    }
    if verification.is_some() {
        options.viewport = options.viewport.with_inner_size([640.0, 480.0]);
    }
    eframe::run_native(
        "Yamantaka",
        options,
        Box::new(move |cc| {
            theme::load_fonts(&cc.egui_ctx);
            theme::install(&cc.egui_ctx, true);
            let mut app = Recovery {
                stage,
                english,
                job: None,
                review: None,
                failed: false,
                handover: output,
                verification,
            };
            app.start_preview(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )?;
    let prepared = handover.borrow_mut().take();
    let installed = prepared.is_some();
    if let Some(preview) = prepared {
        let safety = backup::install_preview(preview, &install_store, &Default::default())?;
        if let Some(verification) = &verification_result {
            verification.verify_installed(&install_store, &safety)?;
        } else {
            yamantaka_windows::show_restore_complete(&safety, english);
        }
    } else if let Some(verification) = &verification_result {
        verification.verify_unchanged(&install_store)?;
    }
    Ok(installed)
}

struct Recovery {
    stage: FailedStage,
    english: bool,
    job: Option<Job>,
    review: Option<RestoreReview>,
    failed: bool,
    handover: Rc<RefCell<Option<RestorePreview>>>,
    verification: Option<preview::Verification>,
}
impl Recovery {
    fn tr<'a>(&self, es: &'a str, en: &'a str) -> &'a str {
        if self.english { en } else { es }
    }
    fn start(&mut self, command: Command, ctx: &egui::Context) {
        self.failed = false;
        let notify = ctx.clone();
        match Job::start(command, move || notify.request_repaint()) {
            Ok(job) => self.job = Some(job),
            Err(_) => self.failed = true,
        }
    }
}
impl eframe::App for Recovery {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let context = ui.ctx().clone();
        let ctx = &context;
        if ctx.input(|input| input.viewport().close_requested()) && self.handover.borrow().is_none()
        {
            if let Some(job) = &self.job {
                job.cancel();
            }
            return;
        }
        ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::new("capture-control-bounds"),
                self.verification.is_some(),
            );
            data.insert_temp(egui::Id::new("yamantaka-english"), self.english);
            data.insert_temp(
                egui::Id::new("capture-controls"),
                Vec::<theme::ControlBounds>::new(),
            )
        });
        if let Some(result) = self.job.as_mut().and_then(Job::poll) {
            self.job = None;
            match result {
                Ok(Outcome::Reviewed(review)) => self.review = Some(review),
                Ok(Outcome::Prepared(preview)) => {
                    *self.handover.borrow_mut() = Some(*preview);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Err(_) => {
                    self.failed = true;
                    self.review = None;
                }
            }
        }
        self.advance_preview(ctx);
        let p = theme::Palette::new(true);
        let compact = ctx.content_rect().width() < 440.0;
        let choose_label = if compact {
            self.tr("Copia JSON…", "JSON backup…")
        } else {
            self.tr("Elegir copia JSON…", "Choose JSON backup…")
        };
        let restore_label = if compact {
            self.tr("Restaurar", "Restore")
        } else {
            self.tr("Restaurar y cerrar", "Restore and close")
        };
        let exit_label = if compact {
            self.tr("Salir", "Exit")
        } else {
            self.tr("Salir sin restaurar", "Exit without restoring")
        };
        egui::Panel::bottom("recovery-actions")
            .frame(egui::Frame::new().fill(p.surface).inner_margin(16.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.add_enabled_ui(self.job.is_none(), |ui| {
                        if theme::bounded_navigation(ui, choose_label, p).clicked()
                            && let Some(path) = yamantaka_windows::pick_library_backup()
                        {
                            self.review = None;
                            self.start(Command::Review(path), ctx);
                        }
                        if let Some(review) = self.review.clone()
                            && theme::bounded_navigation(ui, restore_label, p).clicked()
                        {
                            self.start(Command::Prepare(review), ctx);
                        }
                    });
                    if theme::bounded_navigation(ui, exit_label, p).clicked() {
                        self.handover.borrow_mut().take();
                        if let Some(job) = &self.job {
                            job.cancel();
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(p.canvas).inner_margin(16.0)).show(ui, |ui| {
            ui.label(egui::RichText::new("YAMANTAKA").small().color(p.secondary));
            ui.add_space(12.0);
            theme::display(ui, self.tr("Recupera tu biblioteca", "Recover your library"), 28.0);
            egui::ScrollArea::vertical().id_salt("startup-recovery").show(ui, |ui| {
                ui.label(self.tr(match self.stage {
                    FailedStage::Preferences => "No se pudieron leer las preferencias guardadas.",
                    FailedStage::Library => "No se pudo abrir la biblioteca guardada.",
                }, match self.stage {
                    FailedStage::Preferences => "Saved preferences could not be read.",
                    FailedStage::Library => "The saved library could not be opened.",
                }));
                ui.add_space(10.0);
                ui.label(self.tr("Puedes salir y revisar los archivos, o restaurar una copia JSON. La restauración conserva la carpeta actual completa como respaldo y no modifica tus PDF.", "You can exit and inspect your files, or restore a JSON backup. Restoration keeps the entire current folder as a safety copy and does not change your PDFs."));
                ui.add_space(14.0);
                if self.failed {
                    ui.colored_label(p.accent, self.tr("No se pudo validar esta copia. Los archivos actuales se conservan. Selecciona otra copia.", "This backup could not be validated. Current files are kept. Choose another backup."));
                }
                if let Some(review) = &self.review {
                    egui::Frame::new().fill(p.surface).corner_radius(10).inner_margin(16.0).show(ui, |ui| {
                        ui.label(review.source().file_name().unwrap_or_default().to_string_lossy());
                        for (table, es, en) in [("documents", "Libros", "Books"), ("annotations", "Anotaciones", "Annotations"), ("bookmarks", "Marcadores", "Bookmarks")] {
                            ui.label(format!("{}: {}", self.tr(es, en), review.counts().get(table).copied().unwrap_or(0)));
                        }
                        ui.label(self.tr("Al confirmar, Yamantaka se cerrará para instalar la copia. Después podrás volver a abrir la aplicación.", "On confirmation, Yamantaka will close to install the backup. You can then reopen the application."));
                    });
                }
                if self.job.is_some() { ui.label(self.tr("Comprobando la copia…", "Checking the backup…")); }
            });
        });
        self.capture_preview(ctx);
    }
}
