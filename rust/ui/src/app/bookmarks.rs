use super::*;
use yamantaka_domain::bookmarks::{BOOKMARK_PAGE_SIZE, Bookmark, BookmarkChange, BookmarkPage};

mod controller;
mod view;
pub(super) use controller::BookmarkState;

impl YamantakaApp {
    pub(super) fn reader_panel(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        info: &PdfInfo,
        p: Palette,
    ) {
        self.prepare_annotations(&document.id);
        if !self.reader.panel {
            self.reader.text.clear();
            self.reader.thumbnails.clear();
            return;
        }
        if self.reader.reader_panel_tab != outline::ReaderPanelTab::Thumbnails {
            self.reader.thumbnails.clear();
        }
        if self.reader.reader_panel_tab == outline::ReaderPanelTab::Bookmarks {
            self.reader.bookmarks.prepare(&self.runtime, &document.id);
        }
        if self.reader.reader_panel_tab == outline::ReaderPanelTab::Text {
            self.request_page_text(&document.id);
        } else {
            self.reader.text.clear();
        }
        if self.reader.reader_panel_tab == outline::ReaderPanelTab::Outline {
            self.reader.outline.request(&self.runtime, &document.id);
        }
        // Narrow windows use the same scrollable content as a full-width drawer,
        // rather than leaving a sliver of PDF alongside a fixed-width sidebar.
        if ui.ctx().content_rect().width() < 620.0 {
            egui::Frame::new()
                .fill(p.surface)
                .inner_margin(16)
                .show(ui, |ui| {
                    let height = ui.available_height().max(80.0);
                    egui::ScrollArea::vertical()
                        .id_salt("reader-drawer")
                        .max_height(height)
                        .show(ui, |ui| self.reader_panel_content(ui, info, p));
                });
        } else {
            if self.reader_panel_restore {
                ui.ctx().data_mut(|data| {
                    data.remove::<egui::containers::panel::PanelState>(egui::Id::new(
                        "reader-panel",
                    ))
                });
                self.reader_panel_restore = false;
            }
            let width = navigator_panel(ui, self.reader_panel_width, |ui| {
                self.reader_panel_content(ui, info, p)
            });
            if (self.reader_panel_width - width).abs() > 0.5
                && !ui.input(|input| input.pointer.primary_down())
            {
                self.reader_panel_width = width;
                self.persist_library_state();
            }
        }
    }

    fn reader_panel_content(&mut self, ui: &mut egui::Ui, info: &PdfInfo, p: Palette) {
        let previous_tab = self.reader.reader_panel_tab;
        let compact = ui.ctx().content_rect().width() < 620.0;
        let tabs = [
            (outline::ReaderPanelTab::Outline, "Índice", "Contents"),
            (outline::ReaderPanelTab::Thumbnails, "Páginas", "Pages"),
            (
                outline::ReaderPanelTab::Bookmarks,
                "Marcadores",
                "Bookmarks",
            ),
            (outline::ReaderPanelTab::Find, "Buscar", "Find"),
            (outline::ReaderPanelTab::Text, "Texto", "Text"),
            (outline::ReaderPanelTab::Annotations, "Notas", "Notes"),
            (
                outline::ReaderPanelTab::Positions,
                "Posiciones",
                "Positions",
            ),
        ];
        let selected = tabs
            .iter()
            .find(|(tab, _, _)| *tab == self.reader.reader_panel_tab)
            .unwrap();
        let selected_label = self.tr(selected.1, selected.2);
        ui.horizontal(|ui| {
            let response = egui::ComboBox::from_id_salt("reader-panel-section")
                .selected_text(selected_label)
                .width((ui.available_width() - 50.0).max(80.0))
                .show_ui(ui, |ui| {
                    for (tab, es, en) in tabs {
                        let label = self.tr(es, en);
                        ui.selectable_value(&mut self.reader.reader_panel_tab, tab, label);
                    }
                })
                .response;
            theme::record_control_bounds(ui, selected_label, response.rect);
            if theme::icon_button(
                ui,
                self.tr("Cerrar panel", "Close panel"),
                theme::Icon::Minus,
                p,
            )
            .clicked()
            {
                self.reader.panel = false;
                self.reader.find.stop();
            }
        });
        if previous_tab != self.reader.reader_panel_tab {
            self.reader.find.stop();
            self.reader.bookmarks.cancel_query();
        }
        ui.add_space(
            if compact && self.reader.reader_panel_tab == outline::ReaderPanelTab::Text {
                4.0
            } else if compact {
                8.0
            } else {
                18.0
            },
        );
        if self.reader.reader_panel_tab == outline::ReaderPanelTab::Positions {
            self.navigation_menu(ui, p);
            return;
        }
        if self.reader.reader_panel_tab == outline::ReaderPanelTab::Annotations {
            self.annotation_panel(ui, info, p);
            return;
        }
        if self.reader.reader_panel_tab == outline::ReaderPanelTab::Find {
            self.find_panel_content(ui, info, p);
            return;
        }
        if self.reader.reader_panel_tab == outline::ReaderPanelTab::Text {
            self.text_panel_content(ui, p);
            return;
        }
        self.reader.text.clear();
        if self.reader.reader_panel_tab == outline::ReaderPanelTab::Thumbnails {
            self.thumbnails_panel_content(ui, info, p);
            return;
        }
        if self.reader.reader_panel_tab == outline::ReaderPanelTab::Outline {
            self.outline_panel_content(ui, p);
            return;
        }
        if !compact {
            ui.label(
                RichText::new(self.tr("Tu recorrido de lectura", "Your reading trail")).size(22.0),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(self.tr(
                    "Guarda pasajes para volver a ellos.",
                    "Keep passages you want to revisit.",
                ))
                .color(p.secondary),
            );
            ui.add_space(14.0);
        }
        if let Some(page) = self.reader.bookmarks.show(
            ui,
            &self.runtime,
            self.reader.page,
            info.pages.len(),
            self.english,
            p,
        ) {
            self.navigate(page);
            if compact {
                self.reader.panel = false;
            }
        }
    }
}

fn navigator_panel(ui: &mut egui::Ui, width: f32, contents: impl FnOnce(&mut egui::Ui)) -> f32 {
    navigator_frame("reader-panel", width)
        .show(ui, |ui| navigator_content(ui, contents))
        .response
        .rect
        .width()
}

pub(super) fn navigator_content(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui)) {
    let rect = ui.available_rect_before_wrap();
    ui.set_min_width(rect.width());
    // Long or enlarged controls scroll inside the chosen panel width.
    let mut content = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("navigator-content")
            .max_rect(rect),
    );
    content.set_clip_rect(rect.intersect(ui.clip_rect()));
    content.add_space(14.0);
    egui::ScrollArea::both()
        .id_salt("reader-navigator")
        .auto_shrink([false, false])
        .show(&mut content, contents);
}

pub(super) fn navigator_frame(id: &'static str, width: f32) -> egui::Panel {
    egui::Panel::left(id)
        .default_size(width)
        .size_range(220.0..=500.0)
        .resizable(true)
}

impl YamantakaApp {
    pub(super) fn accept_bookmarks(
        &mut self,
        document: &str,
        cancel: &Cancellation,
        result: Result<BookmarkPage, String>,
        changed: bool,
    ) {
        if changed {
            self.refresh();
            self.reader.bookmarks.invalidate(document, cancel);
            if let Some(reference) = &mut self.reference {
                reference.pane.bookmarks.invalidate(document, cancel);
            }
        }
        if let Some(reference) = &mut self.reference
            && reference.pane.bookmarks.owns(document, cancel)
        {
            reference.pane.bookmarks.accept(document, cancel, result);
        } else if self.reader.bookmarks.owns(document, cancel) {
            self.reader.bookmarks.accept(document, cancel, result);
        } else if !cancel.is_cancelled()
            && let Err(error) = result
        {
            self.error = Some(format!(
                "{}: {error}",
                self.tr(
                    "No se pudieron guardar los marcadores",
                    "Could not save bookmarks",
                )
            ));
        }
    }
}
