mod merge_flow;
mod merge_view;
use super::*;
use std::sync::Arc;
use yamantaka_application::duplicates::{Reply, Request};
use yamantaka_domain::duplicates::{Report, Watch};

#[derive(Default)]
pub(super) struct State {
    merge: merge_flow::MergeState,
    pending: Option<Cancellation>,
    watch: Arc<Watch>,
    report: Option<Arc<Report>>,
    failed: bool,
    cancelled: bool,
    selected: Option<usize>,
    offset: usize,
    preview: Option<bool>,
}
impl State {
    pub(super) fn cancel(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
        self.report = None;
        self.merge.cancel();
    }
    pub(super) fn merge_open(&self) -> bool {
        self.merge.open()
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && self.preview.is_none() && self.merge.ready()
    }
    pub(super) fn accept(&mut self, reply: Reply) {
        if !self
            .pending
            .as_ref()
            .is_some_and(|token| token.same_request(&reply.cancel))
        {
            return;
        }
        self.pending = None;
        self.cancelled = reply.cancel.is_cancelled();
        if self.cancelled {
            return;
        }
        match reply.result {
            Ok(report) => {
                self.report = Some(report);
                self.failed = false;
            }
            Err(_) => self.failed = true,
        }
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        let progress = self.watch.snapshot();
        serde_json::json!({"pending":self.pending.is_some(),"failed":self.failed,"cancelled":self.cancelled,"groups":self.report.as_ref().map(|report|report.groups.len()),"checked":progress.checked,"matched":progress.matched,"unverified":progress.unverified,"detail":self.selected.is_some(),"merge":self.merge.summary()})
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.cancel();
    }
}
impl YamantakaApp {
    pub fn preview_duplicates(&mut self, members: bool) {
        if self.demo && self.capture.is_some() {
            self.show_surface(Surface::Duplicates);
            self.duplicates.preview = Some(members);
            self.start_duplicate_discovery();
        }
    }
    fn start_duplicate_discovery(&mut self) {
        if self.duplicates.pending.is_some() {
            return;
        }
        let cancel = Cancellation::default();
        let watch = Arc::new(Watch::default());
        let request = Request {
            governor: self.governor.clone(),
            cancel: cancel.clone(),
            watch: watch.clone(),
        };
        match self
            .runtime
            .submit(Action::DiscoverDuplicates(Box::new(request)))
        {
            Ok(()) => {
                self.duplicates.report = None;
                self.duplicates.pending = Some(cancel);
                self.duplicates.watch = watch;
                self.duplicates.failed = false;
                self.duplicates.cancelled = false;
                self.duplicates.selected = None;
                self.duplicates.offset = 0;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(super) fn duplicates_view(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        if self.duplicates.pending.is_some() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(250));
        }
        if self.duplicates.pending.is_none()
            && let Some(members) = self.duplicates.preview.take()
            && members
            && self
                .duplicates
                .report
                .as_ref()
                .is_some_and(|report| !report.groups.is_empty())
        {
            self.duplicates.selected = Some(0);
        }
        egui::ScrollArea::vertical().id_salt("duplicate-review").animated(!self.reduced_motion).show(ui, |ui| {
            let compact = ui.available_width() < 700.0;
            egui::Frame::new().inner_margin(egui::Margin::same(if compact {16} else {40})).show(ui, |ui| {
                ui.set_max_width(ui.available_width().min(1100.0));
                ui.horizontal_wrapped(|ui| {
                    if compact && theme::icon_button(ui, self.tr("Volver a la biblioteca", "Back to library"), theme::Icon::Left, p).clicked() { self.show_surface(Surface::Library); }
                    theme::display(ui, self.tr("Copias idénticas", "Identical copies"), if compact {20.0} else {38.0});
                    if !compact && theme::navigation(ui, self.tr("Volver a la biblioteca", "Back to library"), false, p).clicked() { self.show_surface(Surface::Library); }
                });
                ui.add_space(8.0);
                if !compact { ui.label(RichText::new(self.tr("Compara el contenido de tus PDF para revisar copias de la misma lectura.", "Compare your PDFs to review copies of the same reading.")).color(p.secondary)); }
                ui.add_space(16.0);
                ui.horizontal_wrapped(|ui| {
                    if self.duplicates.pending.is_some() {
                        let progress = self.duplicates.watch.snapshot();
                        ui.label(format!("{} {}", progress.checked, self.tr("candidatos comprobados", "candidates checked")));
                        if theme::navigation(ui, self.tr("Cancelar búsqueda", "Cancel search"), false, p).clicked() { self.duplicates.cancel(); }
                    } else if theme::control(ui, self.tr("Buscar copias", "Find copies"), Some(theme::Icon::Search), false, true, p).clicked() { self.start_duplicate_discovery(); }
                });
                if self.duplicates.failed { ui.label(self.tr("No se pudo terminar. Comprueba el acceso y la memoria disponible y vuelve a intentarlo.", "Could not finish. Check file access and available memory, then try again.")); }
                if self.duplicates.cancelled { ui.label(self.tr("Búsqueda cancelada. Puedes volver a iniciarla.", "Search cancelled. You can start it again.")); }
                if let Some(notice)=&self.duplicates.merge.notice {ui.label(notice);}
                if self.duplicates.merge.receipt.is_some() {
                    ui.horizontal_wrapped(|ui|{
                        if theme::navigation(ui,self.tr("Deshacer con el backup","Undo with backup"),false,p).clicked(){self.undo_duplicate_merge(ui.ctx());}
                    });
                    ui.label(RichText::new(self.tr("Deshacer restaura toda la biblioteca anterior y reemplaza los cambios posteriores.","Undo restores the whole previous library and replaces later changes.")).size(13.0).color(p.secondary));
                }
                self.duplicate_history_view(ui);
                let Some(report) = self.duplicates.report.clone() else { return; };
                ui.add_space(12.0);
                ui.label(RichText::new(format!("{} {} · {} {}", report.groups.len(), if report.groups.len() == 1 {self.tr("grupo", "group")} else {self.tr("grupos", "groups")}, report.progress.checked, self.tr("candidatos comprobados", "candidates checked"))).color(p.secondary));
                ui.label(RichText::new(if compact {self.tr("SHA-256 completo · PDF intactos", "Full SHA-256 · PDFs unchanged")} else {self.tr("Coincidencias por SHA-256 completo en el momento de la búsqueda. Los archivos permanecen intactos.", "Full SHA-256 matches at the time of this search. Files remain unchanged.")}).size(13.0).color(p.secondary));
                if report.progress.unverified > 0 { ui.label(format!("{} {}", report.progress.unverified, self.tr("archivos no se pudieron verificar; comprueba sus ubicaciones.", "files could not be verified; check their locations."))); }
                ui.add_space(20.0);
                if let Some(index) = self.duplicates.selected && let Some(group) = report.groups.get(index) {
                    if theme::navigation(ui, self.tr("Todos los grupos", "All groups"), false, p).clicked() { self.duplicates.selected = None; self.duplicates.offset = 0; }
                    ui.heading(&group.sources[0].catalogue.title);
                    ui.label(format!("{} {} · {:.1} MiB", group.sources.len(), self.tr("copias idénticas", "identical copies"), group.size as f64 / 1048576.0));
                    let kept=self.duplicates.merge.kept.clone().filter(|id|group.sources.iter().any(|source|source.catalogue.document==*id)).unwrap_or_else(||group.sources[0].catalogue.document.clone());
                    for source in group.sources.iter().skip(self.duplicates.offset).take(80) {
                        ui.add_space(12.0); ui.separator();
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(&source.catalogue.title).size(18.0));
                            if theme::navigation(ui, self.tr("Abrir lectura", "Open reading"), false, p).clicked() {
                                let id = source.catalogue.document.clone();
                                let cancel = self.open_intent.lookup(id.clone(), None);
                                self.send(Action::GetDocument(id, cancel));
                            }
                        });
                        ui.horizontal_wrapped(|ui|{
                            let chosen=source.catalogue.document==kept;
                            if theme::navigation(ui,self.tr("Conservar este registro","Keep this record"),chosen,p).clicked(){self.duplicates.merge.kept=Some(source.catalogue.document.clone());}
                            if !chosen && theme::navigation(ui,self.tr("Revisar consolidación","Review consolidation"),false,p).clicked(){self.review_duplicate_pair(index,&kept,&source.catalogue.document);}
                        });
                        let path = if self.demo && self.capture.is_some() { format!("Study library / {}", source.catalogue.file.path.file_name().unwrap_or_default().to_string_lossy()) } else { source.catalogue.file.path.to_string_lossy().into_owned() };
                        ui.label(RichText::new(path).size(13.0).color(p.secondary));
                    }
                    duplicate_paging(ui, &mut self.duplicates.offset, group.sources.len(), 80, self.dark, self.english);
                } else {
                    if report.groups.is_empty() { ui.label(self.tr("No se encontraron copias idénticas entre los candidatos verificados.", "No identical copies found among verified candidates.")); }
                    for (index, group) in report.groups.iter().enumerate().skip(self.duplicates.offset).take(20) {
                        ui.separator(); ui.add_space(12.0);
                        ui.label(RichText::new(&group.sources[0].catalogue.title).size(20.0));
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(format!("{} {} · {:.1} MiB", group.sources.len(), self.tr("copias", "copies"), group.size as f64 / 1048576.0)).color(p.secondary));
                            if theme::navigation(ui, self.tr("Revisar grupo", "Review group"), false, p).clicked() { self.duplicates.selected = Some(index); self.duplicates.offset = 0; }
                        });
                        ui.add_space(12.0);
                    }
                    duplicate_paging(ui, &mut self.duplicates.offset, report.groups.len(), 20, self.dark, self.english);
                }
            });
        });
    }
}
fn duplicate_paging(
    ui: &mut egui::Ui,
    offset: &mut usize,
    total: usize,
    page: usize,
    dark: bool,
    english: bool,
) {
    if total <= page {
        return;
    }
    ui.add_space(16.0);
    ui.horizontal_wrapped(|ui| {
        let p = Palette::new(dark);
        if ui
            .add_enabled_ui(*offset > 0, |ui| {
                theme::navigation(
                    ui,
                    if english {
                        "Previous page"
                    } else {
                        "Página anterior"
                    },
                    false,
                    p,
                )
            })
            .inner
            .clicked()
        {
            *offset = offset.saturating_sub(page);
        }
        ui.label(format!(
            "{}–{} / {}",
            *offset + 1,
            (*offset + page).min(total),
            total
        ));
        if ui
            .add_enabled_ui(*offset + page < total, |ui| {
                theme::navigation(
                    ui,
                    if english {
                        "Next page"
                    } else {
                        "Página siguiente"
                    },
                    false,
                    p,
                )
            })
            .inner
            .clicked()
        {
            *offset += page;
        }
    });
}
