use super::settings::quiet;
use super::*;
use yamantaka_windows::folder_watch::{Status, Watch};

const MAX_ROOTS: usize = 8;

#[derive(Default)]
pub(super) struct State {
    roots: Vec<Watch>,
    paused: bool,
    refresh_pending: bool,
}

impl YamantakaApp {
    pub(super) fn poll_folder_watches(&mut self, ctx: &egui::Context) {
        for root in &self.folder_watches.roots {
            self.folder_watches.refresh_pending |= root.take_changed();
        }
        if self.folder_watches.paused || !self.folder_watches.refresh_pending {
            return;
        }
        match self.try_refresh() {
            Ok(()) => self.folder_watches.refresh_pending = false,
            Err(yamantaka_domain::Failure::QueueFull) => {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
            Err(error) => {
                self.folder_watches.refresh_pending = false;
                self.error = Some(error.to_string());
            }
        }
    }

    pub(super) fn folder_watch_settings(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.add_space(20.0);
        settings::section_title(ui, self.tr("Carpetas autorizadas", "Authorized folders"));
        ui.add_space(6.0);
        quiet(ui, self.tr(
            "Actualiza la biblioteca cuando cambia una carpeta elegida. El permiso dura hasta cerrar la aplicación; no se añaden ni eliminan libros automáticamente.",
            "Refresh the library when a chosen folder changes. Permission lasts until the app closes; books are never added or removed automatically.",
        ), p);
        if self.startup_mode.is_safe() {
            quiet(
                ui,
                self.tr(
                    "La observación está desactivada en modo seguro.",
                    "Folder observation is disabled in safe mode.",
                ),
                p,
            );
            return;
        }
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            let can_add = self.folder_watches.roots.len() < MAX_ROOTS;
            if ui
                .add_enabled_ui(can_add, |ui| {
                    theme::bounded_navigation(
                        ui,
                        self.tr("Autorizar carpeta…", "Authorize folder…"),
                        p,
                    )
                })
                .inner
                .clicked()
                && let Some(root) = yamantaka_windows::pick_pdf_folder()
                && !self
                    .folder_watches
                    .roots
                    .iter()
                    .any(|watch| watch.root() == root)
            {
                self.add_folder_watch(root, ui.ctx());
            }
            if !self.folder_watches.roots.is_empty() {
                let label = if self.folder_watches.paused {
                    self.tr("Reanudar", "Resume")
                } else {
                    self.tr("Pausar", "Pause")
                };
                if theme::bounded_navigation(ui, label, p).clicked() {
                    self.folder_watches.paused = !self.folder_watches.paused;
                    self.folder_watches.refresh_pending = !self.folder_watches.paused;
                    for watch in &self.folder_watches.roots {
                        watch.set_paused(self.folder_watches.paused);
                    }
                }
            }
        });
        if self.folder_watches.roots.len() == MAX_ROOTS {
            quiet(
                ui,
                self.tr(
                    "Máximo de ocho carpetas por sesión.",
                    "Up to eight folders per session.",
                ),
                p,
            );
        }
        let mut remove = None;
        let mut retry = None;
        for (index, watch) in self.folder_watches.roots.iter().enumerate() {
            ui.push_id(("authorized-folder", index), |ui| {
                ui.add_space(10.0);
                ui.add(egui::Label::new(watch.root().to_string_lossy()).wrap());
                ui.horizontal_wrapped(|ui| {
                    let status = match watch.status() {
                        Status::Unavailable => self.tr("No disponible", "Unavailable"),
                        _ if self.folder_watches.paused => self.tr("En pausa", "Paused"),
                        Status::Starting => self.tr("Conectando…", "Connecting…"),
                        Status::Active => self.tr("Observando cambios", "Watching for changes"),
                    };
                    quiet(ui, status, p);
                    if watch.status() == Status::Unavailable
                        && theme::bounded_navigation(ui, self.tr("Reintentar", "Retry"), p)
                            .clicked()
                    {
                        retry = Some(index);
                    }
                    if theme::bounded_navigation(
                        ui,
                        self.tr("Retirar permiso", "Revoke permission"),
                        p,
                    )
                    .clicked()
                    {
                        remove = Some(index);
                    }
                });
            });
        }
        if let Some(index) = remove {
            self.folder_watches.roots.remove(index);
        } else if let Some(index) = retry {
            let root = self.folder_watches.roots.remove(index).root().to_owned();
            self.add_folder_watch(root, ui.ctx());
        }
    }

    fn add_folder_watch(&mut self, root: PathBuf, ctx: &egui::Context) {
        let wake = ctx.clone();
        match Watch::start(root, move || wake.request_repaint()) {
            Ok(watch) => {
                watch.set_paused(self.folder_watches.paused);
                self.folder_watches.roots.push(watch);
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}
