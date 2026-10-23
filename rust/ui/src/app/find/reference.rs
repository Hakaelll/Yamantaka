use super::*;
use crate::app::reference::ReferenceView;

impl ReferenceView {
    pub(in crate::app) fn reference_search(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        english: bool,
        p: Palette,
    ) {
        let button = theme::icon_button(
            ui,
            if english {
                "Search reference PDF"
            } else {
                "Buscar en referencia"
            },
            theme::Icon::Search,
            p,
        );
        let mut popup = egui::Popup::menu(&button)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        if self.search_preview {
            popup = popup.open(true);
        }
        let shown = popup.show(|ui| {
            ui.set_width((ui.ctx().content_rect().width() - 40.0).clamp(120.0, 420.0));
            let height = (ui.ctx().content_rect().height() - 100.0).max(80.0);
            egui::ScrollArea::vertical()
                .max_height(height)
                .min_scrolled_height(height.min(360.0))
                .show(ui, |ui| {
                    self.reference_search_contents(ui, runtime, english, p)
                });
        });
        if self.search_visible && shown.is_none() {
            self.pane.find.stop();
        }
        self.search_visible = shown.is_some();
    }

    fn reference_search_contents(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        english: bool,
        p: Palette,
    ) {
        let tr = |es, en| if english { en } else { es };
        let Some((document, info)) = self.pane.active.clone() else {
            return;
        };
        if let Some(query) = self.pane.find.preview.take() {
            self.pane.find.editor = query;
            self.pane.find.start(runtime, &document, &info);
        }
        ui.label(RichText::new(tr("Buscar en referencia", "Search reference PDF")).size(18.0));
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.pane.find.editor)
                .char_limit(1024)
                .hint_text(tr("Palabra o frase", "Word or phrase"))
                .desired_width(ui.available_width()),
        );
        if !self.search_visible {
            response.request_focus();
        }
        let enter = submit_on_enter(ui, &response);
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::TextEdit,
                ui.is_enabled(),
                tr("Buscar en referencia", "Search reference PDF"),
            )
        });
        let mut changed = response.changed();
        ui.horizontal_wrapped(|ui| {
            changed |= ui
                .checkbox(
                    &mut self.pane.find.match_case,
                    tr("Mayúsculas", "Match case"),
                )
                .changed();
            changed |= ui
                .checkbox(
                    &mut self.pane.find.whole_words,
                    tr("Palabras completas", "Whole words"),
                )
                .changed();
        });
        if changed {
            self.pane.find.stop();
            self.pane.find.phase = FindPhase::Idle;
            self.pane.find.query = None;
        }
        let search = theme::navigation(ui, tr("Buscar", "Find"), false, p).clicked();
        if search || enter {
            self.pane.find.start(runtime, &document, &info);
        }
        let mut choose = None;
        let mut step = None;
        match &self.pane.find.phase {
            FindPhase::Idle => {
                ui.label(tr(
                    "Busca un pasaje de este PDF.",
                    "Find a passage in this PDF.",
                ));
            }
            FindPhase::Stopped => {
                ui.label(tr("Búsqueda cancelada.", "Search cancelled."));
            }
            FindPhase::Failed(error) => {
                ui.label(tr(
                    "No se pudo completar la búsqueda.",
                    "Search could not be completed.",
                ));
                ui.collapsing(tr("Detalles", "Details"), |ui| {
                    ui.label(error);
                });
            }
            FindPhase::Running { progress, .. } => {
                ui.label(format!(
                    "{} {} / {}",
                    tr("Páginas revisadas", "Pages searched"),
                    progress.scanned_pages(),
                    info.pages.len()
                ));
                if theme::navigation(ui, tr("Cancelar búsqueda", "Cancel search"), false, p)
                    .clicked()
                {
                    self.pane.find.stop();
                }
                ui.ctx().request_repaint_after(Duration::from_millis(100));
            }
            FindPhase::Ready { batch, selected } => {
                let count = batch.matches().count();
                if count == 0 {
                    ui.label(
                        if batch.text_presence == DocumentTextPresence::Unavailable {
                            tr(
                                "Este PDF no contiene texto extraíble.",
                                "This PDF contains no extractable text.",
                            )
                        } else {
                            tr("No se encontraron coincidencias.", "No matches found.")
                        },
                    );
                } else {
                    ui.label(format!(
                        "{} / {} {}",
                        selected.map_or(0, |index| index + 1),
                        count,
                        tr("en este grupo", "in this batch")
                    ));
                    ui.horizontal_wrapped(|ui| {
                        if theme::navigation(ui, tr("Anterior", "Previous"), false, p).clicked() {
                            step = Some(false);
                        }
                        if theme::navigation(ui, tr("Siguiente", "Next"), false, p).clicked() {
                            step = Some(true);
                        }
                    });
                    for (index, (page, item)) in batch.matches().enumerate() {
                        let label =
                            format!("{} {} · {}", tr("Página", "Page"), page + 1, item.snippet);
                        let response = ui.add(
                            egui::Button::new(&label)
                                .wrap()
                                .selected(*selected == Some(index)),
                        );
                        theme::record_control_bounds(ui, &label, response.rect);
                        if response.clicked() {
                            choose = Some(index);
                        }
                    }
                }
            }
        }
        let target = if let Some(index) = choose {
            self.pane.find.choose(index)
        } else if let Some(forward) = step {
            self.pane.find.step(runtime, &document, &info, forward)
        } else {
            None
        };
        if let Some(target) = target {
            self.apply_reference_find(target);
            ui.ctx().request_repaint();
        }
    }

    pub(in crate::app) fn accept_reference_find(
        &mut self,
        runtime: &Runtime,
        cancel: &Cancellation,
        result: Result<FindBatch, String>,
    ) {
        let Some((document, info)) = &self.pane.active else {
            return;
        };
        if let Some(target) = self
            .pane
            .find
            .accept_result(runtime, document, info, cancel, result)
        {
            self.apply_reference_find(target);
        }
    }

    pub(in crate::app) fn preview_reference_find(&mut self, runtime: &Runtime) {
        let Some((document, info)) = &self.pane.active else {
            return;
        };
        self.pane.find.editor = "Rust".into();
        self.pane.find.start(runtime, document, info);
        self.pane.find.stop();
        self.pane.find.preview = Some("Mixed".into());
        self.search_preview = true;
    }
}
