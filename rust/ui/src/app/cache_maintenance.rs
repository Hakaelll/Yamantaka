use super::settings::{quiet, section_title};
use super::*;
use yamantaka_application::cache_maintenance::{CacheMaintenanceReply, CacheMaintenanceRequest};
use yamantaka_domain::cover_cache::{CoverCacheInventory, CoverCacheOperation};

#[derive(Default)]
pub(super) struct CacheState {
    pending: Option<(CoverCacheOperation, Cancellation)>,
    outcome: Option<CacheMaintenanceReply>,
    preview: Option<bool>,
}
impl CacheState {
    pub(super) fn cancel(&mut self) {
        if let Some((_, cancel)) = &self.pending {
            cancel.cancel();
        }
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && self.preview.is_none()
    }
    pub(super) fn accept(&mut self, reply: CacheMaintenanceReply) {
        if self.pending.as_ref().is_some_and(|(operation, cancel)| {
            *operation == reply.operation && cancel.same_request(&reply.cancel)
        }) {
            self.pending = None;
            self.outcome = Some(reply);
        }
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        let report = self
            .outcome
            .as_ref()
            .and_then(|reply| reply.result.as_ref().ok());
        serde_json::json!({"pending":self.pending.is_some(),"failed":self.outcome.as_ref().is_some_and(|reply|reply.result.is_err()),
            "files":report.and_then(|r|r.inventory).map(|i|i.files),"bytes":report.and_then(|r|r.inventory).map(|i|i.bytes),
            "preserved":report.and_then(|r|r.inventory).map(|i|i.preserved),"removedFiles":report.map(|r|r.removed_files),
            "removedBytes":report.map(|r|r.removed_bytes),"failedFiles":report.map(|r|r.failed_files),"cancelled":report.map(|r|r.cancelled)})
    }
}
impl Drop for CacheState {
    fn drop(&mut self) {
        self.cancel();
    }
}
fn size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}
impl YamantakaApp {
    pub fn preview_cover_cache(&mut self, ctx: &egui::Context, clear: bool) {
        if self.demo && self.capture.is_some() {
            self.preview_storage(ctx);
            self.cache_maintenance.preview = Some(clear);
            self.start_cache_maintenance(CoverCacheOperation::Inspect);
        }
    }
    pub(super) fn update_cache_maintenance(&mut self) {
        if self.cache_maintenance.pending.is_none()
            && let Some(clear) = self.cache_maintenance.preview.take()
            && clear
            && self
                .cache_maintenance
                .outcome
                .as_ref()
                .is_some_and(|r| r.result.is_ok())
        {
            self.start_cache_maintenance(CoverCacheOperation::Clear);
        }
    }
    fn start_cache_maintenance(&mut self, operation: CoverCacheOperation) {
        if self.cache_maintenance.pending.is_some() {
            return;
        }
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::MaintainCoverCache(Box::new(
            CacheMaintenanceRequest {
                operation,
                cancel: cancel.clone(),
            },
        ))) {
            Ok(()) => {
                self.cache_maintenance.outcome = None;
                self.cache_maintenance.pending = Some((operation, cancel));
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(super) fn cache_maintenance_settings(&mut self, ui: &mut egui::Ui, p: Palette) {
        let cramped = ui.available_width() < 420.0;
        if cramped {
            ui.label(
                RichText::new(self.tr("Caché de portadas", "Cover cache"))
                    .size(18.0)
                    .strong(),
            );
        } else {
            section_title(ui, self.tr("Caché de portadas", "Cover cache"));
        }
        ui.add_space(8.0);
        if !cramped {
            ui.label(self.tr(
                "Las imágenes se guardan para abrir la biblioteca con menos trabajo.",
                "Saved images reduce the work needed to open your library.",
            ));
        }
        ui.add_space(if cramped { 6.0 } else { 20.0 });
        let report = self
            .cache_maintenance
            .outcome
            .as_ref()
            .and_then(|reply| reply.result.as_ref().ok())
            .copied();
        let inventory = report.and_then(|r| r.inventory);
        let label = inventory
            .map(|i| size(i.bytes))
            .unwrap_or_else(|| self.tr("Sin calcular", "Not measured").into());
        let amount = ui.label(
            RichText::new(label)
                .size(if cramped { 22.0 } else { 28.0 })
                .color(p.ink),
        );
        if let Some(CoverCacheInventory { files, .. }) = inventory {
            quiet(
                ui,
                &format!(
                    "{files} {}",
                    if files == 1 {
                        self.tr("imagen regenerable", "regeneratable image")
                    } else {
                        self.tr("imágenes regenerables", "regeneratable images")
                    }
                ),
                p,
            );
        }
        ui.add_space(16.0);
        let actions = ui.horizontal_wrapped(|ui| {
            if let Some((operation, cancel)) = &self.cache_maintenance.pending {
                let message = if *operation == CoverCacheOperation::Inspect {
                    self.tr("Calculando tamaño…", "Measuring size…")
                } else {
                    self.tr("Liberando caché…", "Freeing cache…")
                };
                ui.label(message);
                if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked() {
                    cancel.cancel();
                    self.runtime.wake();
                }
            } else {
                if theme::bounded_navigation(ui, self.tr("Calcular tamaño", "Calculate size"), p)
                    .clicked()
                {
                    self.start_cache_maintenance(CoverCacheOperation::Inspect);
                }
                ui.add_enabled_ui(inventory.is_some_and(|i| i.files > 0), |ui| {
                    if theme::bounded_navigation(
                        ui,
                        self.tr("Liberar caché de portadas", "Free cover cache"),
                        p,
                    )
                    .clicked()
                    {
                        self.start_cache_maintenance(CoverCacheOperation::Clear);
                    }
                });
            }
        });
        if cramped && self.demo && self.capture.is_some() {
            ui.scroll_to_rect(
                amount.rect.union(actions.response.rect),
                Some(egui::Align::Center),
            );
        }
        ui.add_space(16.0);
        quiet(ui,self.tr("Se conservan tus PDF y las páginas que elegiste como portada. Las imágenes se regeneran cuando hacen falta.","Your PDFs and chosen cover pages are kept. Images regenerate when needed."),p);
        if let Some(report) = report {
            if report.removed_files > 0 {
                ui.add_space(14.0);
                ui.label(format!(
                    "{} {} · {} {}",
                    size(report.removed_bytes),
                    self.tr("liberados", "freed"),
                    report.removed_files,
                    if report.removed_files == 1 {
                        self.tr("imagen eliminada", "image removed")
                    } else {
                        self.tr("imágenes eliminadas", "images removed")
                    }
                ));
            }
            if report.cancelled {
                Self::file_status_label(
                    ui,
                    self.tr(
                        "Operación cancelada. Vuelve a calcular el tamaño restante.",
                        "Operation cancelled. Measure again for the remaining size.",
                    ),
                    p,
                );
            }
            if report.failed_files > 0 {
                Self::file_status_label(ui,self.tr("Algunos archivos no se pudieron consultar o eliminar. Revisa el acceso y reintenta.","Some files could not be inspected or removed. Check access and retry."),p);
            }
            if report.inventory.is_some_and(|i| i.preserved > 0) {
                quiet(
                    ui,
                    self.tr(
                        "Se han conservado otros archivos de la carpeta.",
                        "Other files in the folder were kept.",
                    ),
                    p,
                );
            }
        } else if self
            .cache_maintenance
            .outcome
            .as_ref()
            .is_some_and(|reply| reply.result.is_err())
        {
            Self::file_status_label(
                ui,
                self.tr(
                    "No se pudo consultar la caché. Revisa el acceso y reintenta.",
                    "The cache could not be inspected. Check access and retry.",
                ),
                p,
            );
        }
    }
}
