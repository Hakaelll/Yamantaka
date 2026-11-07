use super::*;
use outline::ReaderPanelTab;
use yamantaka_domain::bookmarks::{BOOKMARK_PAGE_SIZE, Bookmark, BookmarkChange, BookmarkPage};

mod controller;
mod view;
pub(super) use controller::BookmarkState;

pub(super) const STUDY_TRAY_ID: &str = "reader-study-tray";

const NAVIGATION_TABS: [(ReaderPanelTab, &str, &str); 4] = [
    (ReaderPanelTab::Outline, "Índice", "Contents"),
    (ReaderPanelTab::Thumbnails, "Páginas", "Pages"),
    (ReaderPanelTab::Bookmarks, "Marcadores", "Bookmarks"),
    (ReaderPanelTab::Positions, "Posiciones", "Positions"),
];

const NOTES_TABS: [(ReaderPanelTab, &str, &str); 2] = [
    (ReaderPanelTab::Annotations, "Anotaciones", "Annotations"),
    (ReaderPanelTab::Text, "Texto de la página", "Page text"),
];

impl YamantakaApp {
    pub(super) fn close_study_tray(&mut self) {
        self.reader.panel = false;
    }

    pub(super) fn select_study_tab(&mut self, tab: ReaderPanelTab) {
        if self.reader.reader_panel_tab != tab {
            self.reader.bookmarks.cancel_query();
            self.reader.reader_panel_tab = tab;
        }
    }

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
        if compact_study(ui.ctx()) {
            study_frame(p).show(ui, |ui| {
                study_body(ui, |ui| {
                    self.reader_panel_content(ui, document, info, p, true)
                });
            });
            return;
        }

        if self.reader_panel_restore {
            ui.ctx().data_mut(|data| {
                data.remove::<egui::containers::panel::PanelState>(egui::Id::new(STUDY_TRAY_ID))
            });
            self.reader_panel_restore = false;
        }
        let viewport_height = ui.available_height();
        let preferred_height = self.reader_panel_width;
        let height = study_tray(STUDY_TRAY_ID, viewport_height, preferred_height, p)
            .show(ui, |ui| {
                study_body(ui, |ui| {
                    self.reader_panel_content(ui, document, info, p, false)
                });
            })
            .response
            .rect
            .height();
        let expected_height = preferred_height.min(study_tray_limit(viewport_height));
        if (expected_height - height).abs() > 0.5
            && height >= 220.0
            && !ui.input(|input| input.pointer.primary_down())
        {
            self.reader_panel_width = height.clamp(220.0, 500.0);
            self.persist_library_state();
        }
    }

    fn reader_panel_content(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        info: &PdfInfo,
        p: Palette,
        compact: bool,
    ) {
        self.study_header(ui, p, compact);
        if !self.reader.panel {
            return;
        }
        let tab = self.reader.reader_panel_tab;
        if tab != ReaderPanelTab::Thumbnails {
            self.reader.thumbnails.clear();
        }
        if tab == ReaderPanelTab::Text {
            self.request_page_text(&document.id);
        } else {
            self.reader.text.clear();
        }
        match tab {
            ReaderPanelTab::Thumbnails => self.thumbnails_panel_content(ui, info, p),
            ReaderPanelTab::Find => self.find_panel_content(ui, info, p, compact),
            _ => {
                egui::ScrollArea::vertical()
                    .id_salt(("reader-study-content", tab))
                    .min_scrolled_height(0.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| match tab {
                        ReaderPanelTab::Positions => {
                            if self.navigation_menu(ui, p) && compact {
                                self.close_study_tray();
                            }
                        }
                        ReaderPanelTab::Annotations => {
                            ui.set_max_width(ui.available_width().min(760.0));
                            self.annotation_panel(ui, info, p);
                        }
                        ReaderPanelTab::Text => self.text_panel_content(ui, p),
                        ReaderPanelTab::Outline => {
                            self.reader.outline.request(&self.runtime, &document.id);
                            self.outline_panel_content(ui, p);
                        }
                        ReaderPanelTab::Bookmarks => {
                            self.reader.bookmarks.prepare(&self.runtime, &document.id);
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
                                    self.close_study_tray();
                                }
                            }
                        }
                        ReaderPanelTab::Find | ReaderPanelTab::Thumbnails => unreachable!(),
                    });
            }
        }
    }

    fn study_header(&mut self, ui: &mut egui::Ui, p: Palette, compact: bool) {
        let selected = self.reader.reader_panel_tab;
        let tabs: &[_] = match selected.study_group() {
            ReaderPanelTab::Outline => &NAVIGATION_TABS,
            ReaderPanelTab::Annotations => &NOTES_TABS,
            _ => &[],
        };
        ui.horizontal(|ui| {
            if compact {
                let back = theme::control(
                    ui,
                    self.tr("Volver al PDF", "Back to PDF"),
                    Some(theme::Icon::Left),
                    false,
                    false,
                    p,
                );
                if back.clicked() {
                    self.close_study_tray();
                    if back.has_focus() {
                        self.reader.chrome.focus_controls();
                    }
                    return;
                }
            }
            if tabs.is_empty() {
                ui.label(RichText::new(self.tr("Buscar", "Find")).strong());
                if compact && let Some(counter) = self.reader.find.counter() {
                    ui.label(RichText::new(counter).color(p.secondary))
                        .on_hover_text(
                            self.tr("Coincidencias en este grupo", "Matches in this batch"),
                        );
                }
            } else if ui.available_width() < 450.0 {
                let (_, es, en) = tabs.iter().find(|(tab, ..)| *tab == selected).unwrap();
                let label = self.tr(es, en);
                let mut next = selected;
                let response = egui::ComboBox::from_id_salt("reader-panel-section")
                    .selected_text(label)
                    .width((ui.available_width() - 14.0).clamp(60.0, 172.0))
                    .show_ui(ui, |ui| {
                        for &(tab, es, en) in tabs {
                            ui.selectable_value(&mut next, tab, self.tr(es, en));
                        }
                    })
                    .response;
                theme::record_control_bounds(ui, label, response.rect);
                self.select_study_tab(next);
            } else {
                for &(tab, es, en) in tabs {
                    if theme::navigation(ui, self.tr(es, en), selected == tab, p).clicked() {
                        self.select_study_tab(tab);
                    }
                }
            }
            if !compact {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::icon_button(
                        ui,
                        self.tr("Cerrar bandeja", "Close tray"),
                        theme::Icon::Close,
                        p,
                    )
                    .clicked()
                    {
                        self.close_study_tray();
                    }
                });
            }
        });
        ui.separator();
    }
}

pub(super) fn compact_study(ctx: &egui::Context) -> bool {
    let rect = ctx.content_rect();
    rect.width() < 620.0 || rect.height() < 540.0
}

fn study_frame(p: Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(p.surface)
        .inner_margin(egui::Margin::symmetric(12, 8))
}

fn study_body(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui)) {
    let rect = ui.available_rect_before_wrap();
    ui.set_min_size(rect.size());
    let mut body = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("reader-study-body")
            .max_rect(rect),
    );
    body.set_clip_rect(body.clip_rect().intersect(rect));
    body.spacing_mut().item_spacing.y = 6.0;
    contents(&mut body);
}

fn study_tray_limit(height: f32) -> f32 {
    (height * 0.48).clamp(96.0, 500.0)
}

pub(super) fn study_tray(
    id: &'static str,
    viewport_height: f32,
    preferred: f32,
    p: Palette,
) -> egui::Panel {
    let limit = study_tray_limit(viewport_height);
    egui::Panel::bottom(id)
        .frame(study_frame(p))
        .default_size(preferred.min(limit))
        .size_range(limit.min(220.0)..=limit)
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
