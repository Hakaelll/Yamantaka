use super::*;

impl YamantakaApp {
    pub(super) fn reading_shortcut(&mut self, ui: &mut egui::Ui, p: Palette) {
        let active = self.tabs.model.active().map(|tab| tab.document());
        let Some(document) = self
            .documents
            .iter()
            .find(|document| Some(document.id.as_str()) == active)
            .or_else(|| {
                self.documents
                    .iter()
                    .find(|document| document.reading_progress > 0.0)
            })
            .cloned()
        else {
            return;
        };
        let label = self.tr("Continuar leyendo", "Continue reading");
        let response = if ui.ctx().content_rect().width() >= 1100.0 {
            theme::control(ui, label, Some(theme::Icon::Right), false, false, p)
        } else {
            theme::icon_button(ui, label, theme::Icon::Right, p)
        };
        if response
            .on_hover_text(format!(
                "{}\n{} {} / {} · {:.0}%",
                document.display_title(),
                self.tr("Página", "Page"),
                document
                    .last_page_index
                    .saturating_add(1)
                    .min(document.page_count),
                document.page_count,
                document.reading_progress.clamp(0.0, 1.0) * 100.0,
            ))
            .clicked()
        {
            self.open(document);
        }
    }
}

impl YamantakaApp {
    pub(super) fn library_header(&mut self, ui: &mut egui::Ui, compact: bool, p: Palette) {
        let available = ui.available_width();
        let tiny = available < 420.0 && ui.ctx().content_rect().height() < 420.0;
        let search_width = if available > 700.0 {
            360.0
        } else {
            (available - 200.0).max(200.0)
        };
        let title_width = if available > 500.0 {
            (available - search_width - 18.0).max(150.0)
        } else {
            available
        };
        if !tiny {
            ui.horizontal_wrapped(|ui| {
                if available >= 420.0 {
                    ui.allocate_ui_with_layout(
                        egui::vec2(
                            title_width,
                            if available < 420.0 {
                                26.0
                            } else if compact {
                                52.0
                            } else {
                                64.0
                            },
                        ),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(title_width);
                            ui.label(
                                RichText::new(self.tr("Tu biblioteca", "Your library")).font(
                                    egui::FontId::new(
                                        if compact { 26.0 } else { 38.0 },
                                        if compact {
                                            egui::FontFamily::Proportional
                                        } else {
                                            egui::FontFamily::Name("Editorial".into())
                                        },
                                    ),
                                ),
                            );
                            if available >= 420.0 {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(format!(
                                            "{} {}",
                                            self.count,
                                            if self.count == 1 {
                                                self.tr(
                                                    "documento en tu colección",
                                                    "document in your collection",
                                                )
                                            } else {
                                                self.tr(
                                                    "documentos en tu colección",
                                                    "documents in your collection",
                                                )
                                            }
                                        ))
                                        .color(p.secondary)
                                        .size(13.0),
                                    )
                                    .wrap(),
                                );
                            }
                        },
                    );
                }
                let search_width = if available <= 500.0 {
                    available
                } else {
                    search_width
                };
                ui.allocate_ui_with_layout(
                    egui::vec2(search_width, 40.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(search_width);
                        self.library_search(ui, search_width, p);
                    },
                );
            });
        }
        ui.add_space(if compact { 4.0 } else { 12.0 });
        ui.horizontal_wrapped(|ui| {
            if tiny {
                self.library_search_menu(ui, p);
            }
            if available < 900.0 {
                let label = self.tr("Filtros y orden", "Filters and sorting");
                let response = if available >= 600.0 {
                    theme::control(
                        ui,
                        label,
                        Some(theme::Icon::Sort),
                        self.library_has_filters(),
                        false,
                        p,
                    )
                } else {
                    theme::icon_toggle(ui, label, theme::Icon::Sort, self.library_has_filters(), p)
                };
                let mut popup = egui::Popup::menu(&response);
                if self.smart_menu_preview {
                    popup = popup.open(true);
                }
                popup.show(|ui| {
                    let height =
                        (ui.ctx().content_rect().bottom() - ui.cursor().top() - 12.0).max(40.0);
                    ui.set_max_height(height);
                    egui::ScrollArea::vertical()
                        .id_salt("compact-library-filters")
                        .max_height(height)
                        .show(ui, |ui| self.library_filters(ui, p));
                });
            } else {
                self.library_filters(ui, p);
            }
            if available < 900.0
                && self.library_has_filters()
                && theme::icon_button(
                    ui,
                    self.tr("Restablecer filtros", "Reset filters"),
                    theme::Icon::Close,
                    p,
                )
                .clicked()
            {
                self.reset_library_filters();
            }
            if !compact {
                self.reading_shortcut(ui, p);
            }
            self.library_selection_control(ui, p);
            if available < 420.0 {
                self.library_view_menu(ui, p);
            } else {
                let labelled = ui.available_width() > 900.0;
                let view_width = if labelled { 300.0 } else { 140.0 };
                ui.add_space((ui.available_size_before_wrap().x - view_width - 10.0).max(0.0));
                ui.allocate_ui_with_layout(
                    egui::vec2(view_width, 40.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        let response = if labelled {
                            theme::control(
                                ui,
                                self.tr("Lista", "List"),
                                Some(theme::Icon::List),
                                self.list,
                                false,
                                p,
                            )
                        } else {
                            theme::icon_toggle(
                                ui,
                                self.tr("Lista", "List"),
                                theme::Icon::List,
                                self.list,
                                p,
                            )
                        };
                        if response.clicked() {
                            self.list = true;
                        }
                        let response = if labelled {
                            theme::control(
                                ui,
                                self.tr("Portadas", "Covers"),
                                Some(theme::Icon::Grid),
                                !self.list,
                                false,
                                p,
                            )
                        } else {
                            theme::icon_toggle(
                                ui,
                                self.tr("Portadas", "Covers"),
                                theme::Icon::Grid,
                                !self.list,
                                p,
                            )
                        };
                        if response.clicked() {
                            self.list = false;
                        }
                        let response = if labelled {
                            theme::control(
                                ui,
                                self.tr("Detalles", "Details"),
                                Some(theme::Icon::Panel),
                                self.inspector,
                                false,
                                p,
                            )
                        } else {
                            theme::icon_toggle(
                                ui,
                                self.tr("Detalles", "Details"),
                                theme::Icon::Panel,
                                self.inspector,
                                p,
                            )
                        };
                        if response.clicked() {
                            self.inspector = !self.inspector;
                        }
                    },
                );
            }
        });
        ui.add_space(if compact { 4.0 } else { 12.0 });
    }
}

impl YamantakaApp {
    fn library_view_menu(&mut self, ui: &mut egui::Ui, p: Palette) {
        let icon = if self.list {
            theme::Icon::List
        } else {
            theme::Icon::Grid
        };
        let response =
            theme::icon_button(ui, self.tr("Vista de biblioteca", "Library view"), icon, p);
        let mut popup = egui::Popup::menu(&response);
        if self.demo
            && self.capture.is_some()
            && ui.ctx().data(|data| {
                data.get_temp::<bool>(egui::Id::new("preview-library-view-menu"))
                    .unwrap_or(false)
            })
        {
            popup = popup.open(true);
        }
        popup.show(|ui| {
            ui.set_max_width((ui.ctx().content_rect().width() - 32.0).clamp(160.0, 300.0));
            for (list, es, en, icon) in [
                (false, "Portadas", "Covers", theme::Icon::Grid),
                (true, "Lista", "List", theme::Icon::List),
            ] {
                if theme::control(ui, self.tr(es, en), Some(icon), self.list == list, false, p)
                    .clicked()
                {
                    self.list = list;
                    ui.close();
                }
            }
            if theme::control(
                ui,
                self.tr("Detalles", "Details"),
                Some(theme::Icon::Panel),
                self.inspector,
                false,
                p,
            )
            .clicked()
            {
                self.inspector = !self.inspector;
                ui.close();
            }
        });
    }
}

impl YamantakaApp {
    fn library_search(
        &mut self,
        ui: &mut egui::Ui,
        search_width: f32,
        p: Palette,
    ) -> egui::Response {
        let hint = self
            .tr("Buscar por título o autor", "Search title or author")
            .to_owned();
        let response = ui
            .scope(|ui| {
                ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::new(
                    1.0,
                    p.line
                        .lerp_to_gamma(p.secondary, if self.dark { 0.5 } else { 0.65 }),
                );
                ui.add_sized(
                    [search_width, 40.0],
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text(hint.clone())
                        .background_color(p.surface)
                        .margin(egui::vec2(36.0, 10.0)),
                )
            })
            .inner;
        theme::icon(
            ui.painter(),
            egui::pos2(response.rect.left() + 18.0, response.rect.center().y),
            theme::Icon::Search,
            p.secondary,
        );
        theme::record_control_bounds(ui, &hint, response.rect);
        if response.changed() {
            self.offset = 0;
            self.refresh();
        }
        response
    }
    fn library_search_menu(&mut self, ui: &mut egui::Ui, p: Palette) {
        let response = theme::icon_toggle(
            ui,
            self.tr("Buscar en biblioteca", "Search library"),
            theme::Icon::Search,
            !self.query.trim().is_empty(),
            p,
        );
        let mut popup = egui::Popup::menu(&response);
        if self.demo
            && self.capture.is_some()
            && ui.ctx().data(|data| {
                data.get_temp::<bool>(egui::Id::new("preview-library-search-menu"))
                    .unwrap_or(false)
            })
        {
            popup = popup.open(true);
        }
        popup.show(|ui| {
            let width = (ui.ctx().content_rect().width() - 56.0).clamp(120.0, 640.0);
            let label = ui.label(self.tr("Buscar en biblioteca", "Search library"));
            let input = self.library_search(ui, width, p).labelled_by(label.id);
            if response.clicked() {
                input.request_focus();
            }
        });
    }
}
