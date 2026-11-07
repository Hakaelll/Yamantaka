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

    fn reading_shortcut(&mut self, ui: &mut egui::Ui, p: Palette) {
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
        let position = format!(
            "{} {} / {}",
            self.tr("Pág.", "Page"),
            page.saturating_add(1).min(document.page_count.max(1)),
            document.page_count.max(1)
        );
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 7.0;
            let compact = ui.available_width() < 500.0;
            let cover = ui
                .push_id(("resume-cover", &document.id), |ui| {
                    book::miniature(
                        ui,
                        &document,
                        if compact { 26.0 } else { 36.0 },
                        false,
                        true,
                        p,
                    )
                })
                .inner;
            let title_width =
                (ui.available_width() - if compact { 110.0 } else { 135.0 }).clamp(40.0, 370.0);
            ui.allocate_ui_with_layout(
                egui::vec2(title_width, 32.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(title_width);
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.add(
                        egui::Label::new(RichText::new(document.display_title()).strong())
                            .truncate(),
                    )
                    .on_hover_text(document.display_title());
                    ui.label(RichText::new(&position).size(12.0).color(p.secondary));
                },
            );
            let button = theme::control(
                ui,
                self.tr("Continuar", "Continue"),
                (!compact).then_some(theme::Icon::Right),
                false,
                false,
                p,
            )
            .on_hover_text(format!("{} · {position}", document.display_title()));
            if button.clicked() || cover.clicked() {
                if in_session {
                    self.show_surface(Surface::Reader);
                } else {
                    self.open(document);
                }
            }
        });
    }

    pub(super) fn library_header(&mut self, ui: &mut egui::Ui, p: Palette) {
        let width = ui.available_width();
        let compact = width < 940.0;
        let small = width < 525.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = if small { 4.0 } else { 8.0 };
            let add_label = self.tr("Añadir PDF", "Add PDFs");
            let add = if compact {
                theme::primary_icon(ui, add_label, theme::Icon::Add, p)
            } else {
                theme::control(ui, add_label, Some(theme::Icon::Add), false, true, p)
            };
            if add.clicked()
                && let Some(paths) = yamantaka_windows::pick_pdfs()
            {
                self.import(paths);
            }
            if small {
                self.library_search_menu(ui, p);
            } else {
                let search_width =
                    (width - if compact { 320.0 } else { 735.0 }).clamp(126.0, 430.0);
                self.library_search(ui, search_width, p);
            }
            if !compact {
                self.collection_menu(ui, p);
            }
            let filters = self.tr("Filtros", "Filters");
            let active = self.library_has_filters();
            let response = if compact {
                theme::icon_toggle(ui, filters, theme::Icon::Sort, active, p)
            } else {
                theme::control(ui, filters, Some(theme::Icon::Sort), active, false, p)
            };
            let mut popup = egui::Popup::menu(&response);
            if self.smart_menu_preview {
                popup = popup.open(true);
            }
            popup.show(|ui| {
                egui::ScrollArea::vertical()
                    .max_height((ui.ctx().content_rect().height() - 96.0).max(58.0))
                    .show(ui, |ui| {
                        if compact {
                            self.collection_menu(ui, p);
                        }
                        self.library_filters(ui, p);
                    });
            });
            self.library_view_menu(ui, compact, p);
            self.tools_menu(ui, compact, p);
            self.library_selection_control(ui, p);
        });

        let filtered = self.library_has_filters();
        let count = match (self.count, filtered) {
            (1, true) => self.tr("resultado", "result"),
            (1, false) => self.tr("libro", "book"),
            (_, true) => self.tr("resultados", "results"),
            (_, false) => self.tr("libros", "books"),
        };
        let count = format!("{} {count}", self.count);
        if filtered {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&count).size(13.0).color(p.secondary));
                let label = self.tr("Limpiar filtros", "Clear filters");
                let clear = if small {
                    theme::icon_button(ui, label, theme::Icon::Close, p)
                } else {
                    theme::navigation(ui, label, false, p)
                };
                if clear.clicked() {
                    self.reset_library_filters();
                }
                if let Some((_, collection)) = &self.collections.filter {
                    ui.add(
                        egui::Label::new(RichText::new(collection).size(13.0).color(p.secondary))
                            .truncate(),
                    )
                    .on_hover_text(collection);
                }
            });
        } else if small {
            self.reading_shortcut(ui, p);
        } else {
            ui.horizontal(|ui| {
                self.reading_shortcut(ui, p);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(&count).size(13.0).color(p.secondary));
                });
            });
        }
        ui.add_space(if small { 0.0 } else { 6.0 });
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
