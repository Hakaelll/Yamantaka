use super::*;
use crate::app::outline::ReaderPanelTab;
use crate::app::thumbnails::ThumbnailContext;

impl ReferenceView {
    pub(super) fn navigator(
        &mut self,
        ui: &mut egui::Ui,
        tools: &mut ReferenceTools<'_>,
        notes_blocked: bool,
        english: bool,
        p: Palette,
    ) -> egui::Id {
        let runtime = tools.runtime;
        let governor = tools.governor;
        let tr = |es, en| if english { en } else { es };
        let button = theme::icon_button(
            ui,
            tr("Navegar por la referencia", "Navigate reference PDF"),
            theme::Icon::Panel,
            p,
        );
        if !matches!(
            self.pane.reader_panel_tab,
            ReaderPanelTab::Outline
                | ReaderPanelTab::Thumbnails
                | ReaderPanelTab::Find
                | ReaderPanelTab::Bookmarks
                | ReaderPanelTab::Text
                | ReaderPanelTab::Positions
                | ReaderPanelTab::Annotations
        ) {
            self.pane.reader_panel_tab = ReaderPanelTab::Outline;
        }
        if self.notes_preview {
            self.pane.reader_panel_tab = ReaderPanelTab::Annotations;
        } else if self.text_preview {
            self.pane.reader_panel_tab = ReaderPanelTab::Text;
        } else if self.bookmarks_preview {
            self.pane.reader_panel_tab = ReaderPanelTab::Bookmarks;
        } else if self.thumbnails_preview {
            self.pane.reader_panel_tab = ReaderPanelTab::Thumbnails;
        } else if self.search_preview {
            self.pane.reader_panel_tab = ReaderPanelTab::Find;
        } else if self.outline_preview {
            self.pane.reader_panel_tab = ReaderPanelTab::Outline;
        }
        let mut popup = egui::Popup::menu(&button)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        let requested = std::mem::take(&mut self.navigator_requested);
        if requested {
            popup = popup.open_memory(egui::SetOpenCommand::Bool(true));
        }
        if self.search_preview
            || self.outline_preview
            || self.thumbnails_preview
            || self.bookmarks_preview
            || self.text_preview
            || self.notes_preview
        {
            popup = popup.open(true);
        }
        let shown = popup.show(|ui| {
            self.input_layers.insert(ui.layer_id());
            ui.set_width((ui.ctx().content_rect().width() - 40.0).clamp(120.0, 420.0));
            let previous = self.pane.reader_panel_tab;
            let sections = [
                (ReaderPanelTab::Outline, tr("Índice", "Contents")),
                (ReaderPanelTab::Thumbnails, tr("Páginas", "Pages")),
                (ReaderPanelTab::Find, tr("Buscar", "Find")),
                (ReaderPanelTab::Bookmarks, tr("Marcadores", "Bookmarks")),
                (ReaderPanelTab::Text, tr("Texto", "Text")),
                (ReaderPanelTab::Annotations, tr("Notas", "Notes")),
                (ReaderPanelTab::Positions, tr("Posiciones", "Positions")),
            ];
            let selected = sections
                .iter()
                .find(|(tab, _)| *tab == self.pane.reader_panel_tab)
                .unwrap()
                .1;
            ui.horizontal(|ui| {
                let notes_tools = previous == ReaderPanelTab::Annotations
                    && !self.pane.annotations.editing()
                    && !self.pane.annotations.reviewing();
                let reserved = if notes_tools { 40.0 + ui.spacing().item_spacing.x } else { 0.0 };
                let response = egui::ComboBox::from_id_salt("reference-navigator-section")
                    .selected_text(selected)
                    .width((ui.available_width() - reserved).max(40.0))
                    .show_ui(ui, |ui| {
                        self.input_layers.insert(ui.layer_id());
                        for (tab, label) in sections {
                            ui.selectable_value(&mut self.pane.reader_panel_tab, tab, label);
                        }
                    })
                    .response;
                theme::record_control_bounds(ui, selected, response.rect);
                if notes_tools
                    && theme::icon_toggle(
                        ui,
                        tr("Herramientas de notas", "Note tools"),
                        theme::Icon::Settings,
                        self.notes_tools_open,
                        p,
                    ).clicked()
                {
                    self.notes_tools_open = !self.notes_tools_open;
                }
            });
            ui.add_space(8.0);
            if previous != self.pane.reader_panel_tab {
                self.stop_navigator_requests();
            }
            match self.pane.reader_panel_tab {
                ReaderPanelTab::Annotations => {
                    let Some((document, info)) = &self.pane.active else {
                        return;
                    };
                    self.pane
                        .annotations
                        .prepare(runtime, governor, &document.id);
                    let context = annotations::NotesContext {
                        english,
                        page: self.pane.page,
                        page_count: info.pages.len(),
                        busy: notes_blocked,
                        selectable: true,
                        reveal_actions: self.notes_scroll_preview,
                        governor,
                        marks_error: self.pane.annotation_paint.error().map(str::to_owned),
                    };
                    let height = (ui.ctx().content_rect().height() - 100.0).max(80.0);
                    let annotations = &mut self.pane.annotations;
                    let focused = annotations.focused_note();
                    if requested && focused {
                        self.notes_tools_open = false;
                    }
                    let mut scroll = egui::ScrollArea::vertical()
                        .id_salt(("reference-notes", annotations.editing(), annotations.reviewing(), self.notes_tools_open, focused))
                        .max_height(height)
                        .min_scrolled_height(height.min(360.0));
                    if requested && focused {
                        scroll = scroll.vertical_scroll_offset(0.0);
                    }
                    scroll.show(ui, |ui| {
                        if annotations.editing() {
                            self.notes_action = annotations.show_editor(ui, &context, p);
                            return;
                        }
                        if annotations.reviewing() {
                            self.notes_action = annotations.show_review(ui, &context, p);
                            return;
                        }
                        if self.notes_tools_open {
                            let filter = annotations.show_filters(ui, english, context.page, p);
                            if matches!(filter, Some(annotations::NotesAction::Read(_) | annotations::NotesAction::ClearFocus)) {
                                self.notes_tools_open = false;
                                self.notes_action = filter;
                                return;
                            }
                            let bulk = annotations.show_bulk(ui, english, notes_blocked, p);
                            let action = annotations.show_editor(ui, &context, p);
                            let scope = yamantaka_application::annotation_export::worker::AnnotationExportScope::Document(document.id.clone());
                            let export = tools.export.show(ui, english, &scope, annotations.ready(), p).map(annotations::NotesAction::Export);
                            self.notes_action = filter.or(bulk).or(action).or(export);
                            return;
                        }
                        let back = if focused {
                            annotations.show_filters(ui, english, context.page, p)
                        } else {
                            None
                        };
                        let issue = if annotations.study_write_error().is_some() || context.marks_error.is_some() {
                            annotations.show_editor(ui, &context, p)
                        } else {
                            if !annotations.ready() {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(tr("Actualizando notas…", "Updating notes…"));
                                });
                            }
                            None
                        };
                        self.notes_action = back.or(issue).or_else(|| annotations.show_list(ui, &context, p));
                    });
                }
                ReaderPanelTab::Positions => {
                    // Popup placement depends on its height, so do not derive height from its cursor.
                    let height = (ui.ctx().content_rect().height() - 100.0).max(80.0);
                    let mut action = None;
                    egui::ScrollArea::vertical()
                        .id_salt("reference-positions")
                        .max_height(height)
                        .min_scrolled_height(height.min(180.0))
                        .show(ui, |ui| {
                            if let Some(nav) = &self.pane.navigation {
                                action = navigation::positions(ui, nav, english, p);
                            }
                        });
                    if let Some(action) = action {
                        if action.apply(self.pane.navigation.as_mut().unwrap()) {
                            self.reference_navigation_changed();
                        }
                        ui.close();
                    }
                }
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
                ReaderPanelTab::Text => {
                    let Some((document, _)) = &self.pane.active else {
                        return;
                    };
                    if self.pane.selection.enabled {
                        self.pane
                            .selection
                            .request(runtime, &document.id, &[self.pane.page]);
                    } else {
                        self.pane
                            .text
                            .request(runtime, &document.id, self.pane.page);
                    }
                    let height = (ui.ctx().content_rect().height() - 160.0).max(80.0);
                    let mut retry = false;
                    egui::ScrollArea::vertical()
                        .id_salt("reference-text")
                        .max_height(height)
                        .min_scrolled_height(height.min(360.0))
                        .show(ui, |ui| {
                            if self.pane.selection.enabled {
                                if let Some(state) = self.pane.selection.text_state(self.pane.page)
                                {
                                    retry = state.show(ui, english, p);
                                }
                            } else {
                                retry = self.pane.text.show(ui, english, p);
                            }
                        });
                    if retry {
                        if self.pane.selection.enabled {
                            self.pane.selection.retry_page(self.pane.page);
                        } else {
                            self.pane.text.clear();
                        }
                    }
                }
                ReaderPanelTab::Bookmarks => {
                    let Some((document, info)) = &self.pane.active else {
                        return;
                    };
                    self.pane.bookmarks.prepare(runtime, &document.id);
                    let height = (ui.ctx().content_rect().height() - 160.0).max(80.0);
                    let mut jump = None;
                    egui::ScrollArea::vertical()
                        .id_salt("reference-bookmarks")
                        .max_height(height)
                        .min_scrolled_height(height.min(360.0))
                        .show(ui, |ui| {
                            jump = self.pane.bookmarks.show(
                                ui,
                                runtime,
                                self.pane.page,
                                info.pages.len(),
                                english,
                                p,
                            );
                        });
                    if let Some(page) = jump {
                        self.set_page(page);
                        ui.ctx().request_repaint();
                    }
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
        button.id
    }

    pub(in crate::app) fn stop_navigator_requests(&mut self) {
        self.pane.find.stop();
        self.pane.outline.cancel();
        self.pane.thumbnails.clear();
        self.pane.text.clear();
        self.pane.bookmarks.cancel_query();
        self.pane.annotations.cancel_query();
        self.notes_tools_open = false;
        self.search_visible = false;
    }
}
