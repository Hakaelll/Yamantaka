use super::*;

impl YamantakaApp {
    pub(super) fn refresh_resume_document(&mut self) {
        let cancel = Cancellation::default();
        if self
            .runtime
            .submit(Action::Query(
                LibraryQuery {
                    reading_state: LibraryReadingState::InProgress,
                    sort: LibrarySort::LastOpened,
                    sort_direction: SortDirection::Descending,
                    page_size: 1,
                    ..Default::default()
                },
                cancel.clone(),
            ))
            .is_ok()
            && let Some(previous) = self.resume_query.replace(cancel)
        {
            previous.cancel();
        }
    }

    fn reading_shortcut(&mut self, ui: &mut egui::Ui, p: Palette, icon: bool, contextual: bool) {
        let active = self.reader.active.as_ref().map(|(document, _)| document);
        let Some(document) = active.or(self.resume_document.as_ref()).cloned() else {
            return;
        };
        let in_session = active.is_some();
        let page = if in_session {
            self.reader.page
        } else {
            document.last_page_index
        };
        let context = format!(
            "{} · {} {} / {}",
            document.display_title(),
            self.tr("Pág.", "Page"),
            page.saturating_add(1).min(document.page_count.max(1)),
            document.page_count.max(1)
        );
        let action = self.tr("Continuar", "Continue");
        let button = if icon {
            theme::icon_button(ui, action, theme::Icon::Right, p)
        } else {
            let label = if contextual && ui.available_width() >= 360.0 {
                format!("{action} · {context}")
            } else {
                action.to_owned()
            };
            theme::bounded_navigation(ui, &label, p)
        };
        if button.on_hover_text(context).clicked() {
            if in_session {
                self.show_surface(Surface::Reader);
            } else {
                self.open(document);
            }
        }
    }

    pub(super) fn library_header(&mut self, ui: &mut egui::Ui, p: Palette) {
        let width = ui.available_width();
        let icons = width < 810.0;
        let compact_search = width < 540.0;
        let filtered = self.library_has_filters();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = if icons { 4.0 } else { 8.0 };
            if compact_search {
                self.library_search_menu(ui, p);
            } else {
                let reserve = if icons { 250.0 } else { 490.0 };
                self.library_search(ui, (width - reserve).clamp(140.0, 410.0), p);
            }
            let add_label = self.tr("Añadir PDF", "Add PDFs");
            let add = if icons {
                theme::primary_icon(ui, add_label, theme::Icon::Add, p)
            } else {
                theme::control(ui, add_label, Some(theme::Icon::Add), false, true, p)
            };
            if add.clicked()
                && let Some(paths) = yamantaka_windows::pick_pdfs()
            {
                self.import(paths);
            }
            let filters = self.tr("Filtros", "Filters");
            let response = if icons {
                theme::icon_toggle(ui, filters, theme::Icon::Sort, filtered, p)
            } else {
                theme::control(ui, filters, Some(theme::Icon::Sort), filtered, false, p)
            };
            let mut popup = egui::Popup::menu(&response);
            if self.smart_menu_preview {
                popup = popup.open(true);
            }
            popup.show(|ui| {
                egui::ScrollArea::vertical()
                    .max_height((ui.ctx().content_rect().height() - 82.0).max(58.0))
                    .show(ui, |ui| {
                        self.collection_menu(ui, p);
                        self.library_filters(ui, p);
                    });
            });
            self.library_view_menu(ui, icons, p);
            if self.library_selection.toolbar_visible() {
                self.library_selection_control(ui, p);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.tools_menu(ui, true, p);
                if !filtered {
                    self.reading_shortcut(ui, p, icons || ui.available_width() < 118.0, false);
                }
            });
        });
        if filtered {
            ui.horizontal(|ui| {
                let count = if self.count == 1 {
                    self.tr("resultado", "result")
                } else {
                    self.tr("resultados", "results")
                };
                ui.label(
                    RichText::new(format!("{} {count}", self.count))
                        .size(13.0)
                        .color(p.secondary),
                );
                let label = self.tr("Limpiar filtros", "Clear filters");
                let clear = if compact_search {
                    theme::icon_button(ui, label, theme::Icon::Close, p)
                } else {
                    theme::navigation(ui, label, false, p)
                };
                if clear.clicked() {
                    self.reset_library_filters();
                }
                self.reading_shortcut(ui, p, false, true);
                if let Some((_, collection)) = &self.collections.filter {
                    ui.add(
                        egui::Label::new(RichText::new(collection).size(13.0).color(p.secondary))
                            .truncate(),
                    )
                    .on_hover_text(collection);
                }
            });
        }
        ui.separator();
    }

    fn library_view_menu(&mut self, ui: &mut egui::Ui, compact: bool, p: Palette) {
        let icon = if self.list {
            theme::Icon::List
        } else {
            theme::Icon::Grid
        };
        let label = self.tr("Vista", "View");
        let response = if compact {
            theme::icon_button(ui, label, icon, p)
        } else {
            theme::control(ui, label, Some(icon), false, false, p)
        };
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
            ui.separator();
            self.library_sort_control(ui, p);
        });
    }

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
                    [search_width, 36.0],
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text(hint.clone())
                        .background_color(p.surface)
                        .margin(egui::vec2(36.0, 8.0)),
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
