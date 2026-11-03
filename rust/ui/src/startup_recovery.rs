//! Recovery does not need the failed library, a PDF engine, or saved preferences.
use crate::{
    restore_worker::{Command, Job, Outcome, Prepared, Review},
    theme,
};
use eframe::egui;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};
use yamantaka_storage::{Database, preferences::Preferences};

pub mod preview;

#[derive(Clone, Copy, Debug)]
pub enum FailedStage {
    Preferences,
    Library,
    InterruptedOperation,
    LibrarySelection,
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
    let handover: Rc<RefCell<Option<Prepared>>> = Default::default();
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
                store: store.clone(),
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
        let safety = preview.install(&install_store, &Default::default())?;
        if let Some(verification) = &verification_result {
            verification.verify_installed(&install_store, &safety)?;
        } else if !matches!(
            stage,
            FailedStage::InterruptedOperation | FailedStage::LibrarySelection
        ) {
            yamantaka_windows::show_restore_complete(&safety, english);
        }
    } else if let Some(verification) = &verification_result {
        verification.verify_unchanged(&install_store)?;
    }
    Ok(installed)
}

struct Recovery {
    store: PathBuf,
    stage: FailedStage,
    english: bool,
    job: Option<Job>,
    review: Option<Review>,
    failed: bool,
    handover: Rc<RefCell<Option<Prepared>>>,
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
        let interrupted = matches!(self.stage, FailedStage::InterruptedOperation);
        let selection = matches!(self.stage, FailedStage::LibrarySelection);
        let choose_label = if selection {
            self.tr("Revisar original", "Review original")
        } else if interrupted {
            self.tr("Revisar actual", "Review current")
        } else if compact {
            self.tr("Copia JSON…", "JSON backup…")
        } else {
            self.tr("Elegir copia JSON…", "Choose JSON backup…")
        };
        let restore_label = if interrupted || selection {
            self.tr("Confirmar y cerrar", "Confirm and close")
        } else if compact {
            self.tr("Restaurar", "Restore")
        } else {
            self.tr("Restaurar y cerrar", "Restore and close")
        };
        let exit_label = if compact {
            self.tr("Salir", "Exit")
        } else {
            self.tr("Salir sin restaurar", "Exit without restoring")
        };
        let enabled = self.job.is_none();
        let reviewed = self.review.is_some();
        let previous_label = if selection {
            self.tr("Importada…", "Imported library…")
        } else {
            self.tr("Copia anterior…", "Previous copy…")
        };
        let actions = egui::Panel::bottom("recovery-actions")
            .frame(egui::Frame::new().fill(p.surface).inner_margin(16.0))
            .show(ui, |ui| {
                let mut restore = false;
                let mut exit = false;
                let mut finish = |ui: &mut egui::Ui| {
                    if reviewed {
                        restore = ui
                            .add_enabled_ui(enabled, |ui| {
                                theme::bounded_navigation(ui, restore_label, p)
                            })
                            .inner
                            .clicked();
                    }
                    exit = theme::bounded_navigation(ui, exit_label, p).clicked();
                };
                let mut choose = false;
                let mut previous = false;
                ui.horizontal(|ui| {
                    choose = ui
                        .add_enabled_ui(enabled, |ui| {
                            theme::bounded_navigation(ui, choose_label, p)
                        })
                        .inner
                        .clicked();
                    previous = ui
                        .add_enabled_ui(enabled, |ui| {
                            theme::bounded_navigation(ui, previous_label, p)
                        })
                        .inner
                        .clicked();
                    if !compact {
                        finish(ui);
                    }
                });
                if compact {
                    ui.horizontal(finish);
                }
                (choose, previous, restore, exit)
            })
            .inner;
        if actions.0 && selection {
            self.review = None;
            self.start(
                Command::ReviewSelection {
                    root: self.store.clone(),
                    source: None,
                },
                ctx,
            );
        } else if actions.0 && interrupted {
            self.review = None;
            self.start(
                Command::ReviewPending {
                    active: self.store.clone(),
                    previous: None,
                },
                ctx,
            );
        } else if actions.0
            && let Some(path) = yamantaka_windows::pick_library_backup()
        {
            self.review = None;
            self.start(Command::Review(path), ctx);
        }
        let picker_root = if selection {
            Some(self.store.join("imports"))
        } else {
            self.store.parent().map(Path::to_path_buf)
        };
        if actions.1
            && let Some(parent) = picker_root
            && let Some(source) = yamantaka_windows::pick_library_safety_folder(&parent)
        {
            self.review = None;
            self.start(
                if selection {
                    Command::ReviewSelection {
                        root: self.store.clone(),
                        source: Some(source),
                    }
                } else if interrupted {
                    Command::ReviewPending {
                        active: self.store.clone(),
                        previous: Some(source),
                    }
                } else {
                    Command::ReviewRollback {
                        active: self.store.clone(),
                        source,
                    }
                },
                ctx,
            );
        }
        if actions.2
            && let Some(review) = self.review.clone()
        {
            self.start(Command::Prepare(review), ctx);
        }
        if actions.3 {
            self.handover.borrow_mut().take();
            if let Some(job) = &self.job {
                job.cancel();
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        egui::CentralPanel::default().frame(egui::Frame::new().fill(p.canvas).inner_margin(16.0)).show(ui, |ui| {
            egui::ScrollArea::vertical().id_salt("startup-recovery").show(ui, |ui| {
                if !compact {
                    ui.label(egui::RichText::new("YAMANTAKA").small().color(p.secondary));
                    ui.add_space(12.0);
                }
                theme::display(ui,self.tr("Recupera tu biblioteca","Recover your library"),if compact {20.0}else{28.0});
                ui.add_space(if compact {6.0}else{10.0});
                ui.label(self.tr(match self.stage {
                    FailedStage::Preferences => "No se pudieron leer las preferencias guardadas.",
                    FailedStage::Library => "No se pudo abrir la biblioteca guardada.",
                    FailedStage::InterruptedOperation => "Una operación interrumpida necesita que elijas qué biblioteca conservar activa.",
                    FailedStage::LibrarySelection => "No se pudo resolver el selector de biblioteca guardado.",
                }, match self.stage {
                    FailedStage::Preferences => "Saved preferences could not be read.",
                    FailedStage::Library => "The saved library could not be opened.",
                    FailedStage::InterruptedOperation => "An interrupted operation needs your choice of which library to keep active.",
                    FailedStage::LibrarySelection => "The saved library selection could not be resolved.",
                }));
                ui.add_space(10.0);
                ui.label(if selection {self.tr("Revisa la biblioteca original o elige una biblioteca de la carpeta imports. Se conservará el selector anterior antes de activar tu elección. Las bibliotecas y los PDF se mantienen separados, sin fusionarse ni modificarse.","Review the original library or choose a library from the imports folder. The previous selection record will be preserved before activating your choice. Libraries and PDFs stay separate, without merging or modification.")} else if interrupted { self.tr("Revisa la biblioteca actual o elige una copia anterior. Al confirmar se conservará el registro de la operación. Mantener la actual no mueve carpetas; volver a una anterior conserva la actual completa como respaldo. No se modifican los PDF.", "Review the current library or choose a previous copy. Confirmation preserves the operation record. Keeping the current library moves no folders; returning to a previous copy retains the whole current folder as a safety copy. PDFs are not changed.") } else { self.tr("Puedes revisar los archivos, restaurar una copia JSON o volver a una biblioteca anterior. La restauración conserva la carpeta actual completa como respaldo y no modifica tus PDF.", "You can inspect your files, restore a JSON backup, or return to a previous library. Restoration keeps the entire current folder as a safety copy and does not change your PDFs.") });
                ui.add_space(14.0);
                if self.failed {
                    ui.colored_label(p.accent, if selection {self.tr("No se pudo validar esta biblioteca o el selector cambió. Se conservan los archivos. Vuelve a revisar tu elección.","This library could not be validated or the selection record changed. Files are preserved. Review your choice again.")}else if interrupted {self.tr("No se pudo validar la biblioteca o la operación cambió. Se conservan los archivos. Vuelve a revisar o elige una copia anterior.","The library could not be validated or the operation changed. Files are preserved. Review again or choose a previous copy.")}else{self.tr("No se pudo validar esta copia. Los archivos actuales se conservan. Selecciona otra copia.", "This backup could not be validated. Current files are kept. Choose another backup.")});
                }
                if let Some(review) = &self.review {
                    egui::Frame::new().fill(p.surface).corner_radius(10).inner_margin(16.0).show(ui, |ui| {
                        ui.label(review.source().file_name().unwrap_or_default().to_string_lossy());
                        for (table, es, en) in [("documents", "Libros", "Books"), ("annotations", "Anotaciones", "Annotations"), ("bookmarks", "Marcadores", "Bookmarks")] {
                            ui.label(format!("{}: {}", self.tr(es, en), review.counts().get(table).copied().unwrap_or(0)));
                        }
                        if review.rollback() {ui.label(self.tr("La carpeta elegida pasará a ser la biblioteca activa. Cierra otras instancias que la estén utilizando.","The selected folder will become the active library. Close other instances using it."));}
                        else if selection {ui.label(self.tr("Al confirmar solo cambia el selector de biblioteca. Los archivos de ambas bibliotecas se conservan.","Confirmation changes only the library selection. Files in both libraries are kept."));}
                        else if interrupted {ui.label(self.tr("Se mantendrá la biblioteca actual. Las demás carpetas se conservan.","The current library will remain active. Other folders are preserved."));}
                        ui.label(if selection {self.tr("Al confirmar se cerrará la aplicación. Vuelve a abrirla para usar la biblioteca elegida.","Confirmation closes the application. Reopen it to use the chosen library.")}else{self.tr("Al confirmar, Yamantaka se cerrará para instalar la copia. Después podrás volver a abrir la aplicación.", "On confirmation, Yamantaka will close to install the backup. You can then reopen the application.")});
                    });
                }
                if self.job.is_some() { ui.label(self.tr("Comprobando la copia…", "Checking the backup…")); }
            });
        });
        self.capture_preview(ctx);
    }
}
