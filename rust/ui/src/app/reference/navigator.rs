use super::*;
use crate::app::outline::ReaderPanelTab;
use crate::app::thumbnails::ThumbnailContext;

impl ReferenceView {
    pub(super) fn navigator(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        english: bool,
        p: Palette,
    ) {
        let tr = |es, en| if english { en } else { es };
        let button = theme::icon_button(
            ui,
            tr("Navegar por la referencia", "Navigate reference PDF"),
            theme::Icon::Panel,
            p,
        );
        if !matches!(
            self.pane.reader_panel_tab,
            ReaderPanelTab::Outline | ReaderPanelTab::Thumbnails | ReaderPanelTab::Find
        ) {
            self.pane.reader_panel_tab = ReaderPanelTab::Outline;
        }
        if self.thumbnails_preview {
            self.pane.reader_panel_tab = ReaderPanelTab::Thumbnails;
        } else if self.search_preview {
            self.pane.reader_panel_tab = ReaderPanelTab::Find;
        } else if self.outline_preview {
            self.pane.reader_panel_tab = ReaderPanelTab::Outline;
        }
        let mut popup = egui::Popup::menu(&button)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        if self.search_preview || self.outline_preview || self.thumbnails_preview {
            popup = popup.open(true);
        }
        let shown = popup.show(|ui| {
            ui.set_width((ui.ctx().content_rect().width() - 40.0).clamp(120.0, 420.0));
            let previous = self.pane.reader_panel_tab;
            ui.horizontal_wrapped(|ui| {
                for (tab, label) in [
                    (ReaderPanelTab::Outline, tr("Índice", "Contents")),
                    (ReaderPanelTab::Thumbnails, tr("Páginas", "Pages")),
                    (ReaderPanelTab::Find, tr("Buscar", "Find")),
                ] {
                    if theme::navigation(ui, label, self.pane.reader_panel_tab == tab, p).clicked()
                    {
                        self.pane.reader_panel_tab = tab;
                    }
                }
            });
            ui.add_space(8.0);
            if previous != self.pane.reader_panel_tab {
                self.stop_navigator_requests();
            }
            match self.pane.reader_panel_tab {
                ReaderPanelTab::Find => {
                    let height = (ui.ctx().content_rect().height() - 160.0).max(80.0);
                    egui::ScrollArea::vertical()
                        .id_salt("reference-find")
                        .max_height(height)
                        .min_scrolled_height(height.min(360.0))
                        .show(ui, |ui| {
                            self.reference_search_contents(ui, runtime, english, p)
                        });
                }
                ReaderPanelTab::Thumbnails => {
                    let Some((document, info)) = &self.pane.active else {
                        return;
                    };
                    let rotation = self
                        .pane
                        .reader_preferences
                        .as_ref()
                        .map_or(0, |prefs| prefs.page_rotation.degrees());
                    let height = (ui.ctx().content_rect().height() - 160.0).max(80.0);
                    let reduced_motion = ui
                        .ctx()
                        .data(|data| {
                            data.get_temp::<bool>(egui::Id::new("yamantaka-reduced-motion"))
                        })
                        .unwrap_or(false);
                    let jump = self.pane.thumbnails.show(
                        ui,
                        runtime,
                        ThumbnailContext {
                            document: &document.id,
                            info,
                            page: self.pane.page,
                            rotation,
                            compact: ui.ctx().content_rect().width() < 620.0,
                            english,
                            reduced_motion,
                            max_height: Some(height),
                        },
                        p,
                    );
                    if let Some(page) = jump {
                        self.set_page(page);
                        ui.ctx().request_repaint();
                    }
                }
                _ => self.reference_outline_contents(ui, runtime, english, p),
            }
            self.search_visible = self.pane.reader_panel_tab == ReaderPanelTab::Find;
        });
        if self.pane.panel && shown.is_none() {
            self.stop_navigator_requests();
        }
        self.pane.panel = shown.is_some();
    }

    pub(super) fn stop_navigator_requests(&mut self) {
        self.pane.find.stop();
        self.pane.outline.cancel();
        self.pane.thumbnails.clear();
        self.search_visible = false;
    }
}
