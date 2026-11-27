use super::*;

pub(super) fn paint_library_cloth(ui: &egui::Ui, p: Palette, dark: bool) {
    const TILE: usize = 128;
    let id = egui::Id::new(("library-cloth", dark));
    let texture = ui
        .ctx()
        .data(|data| data.get_temp::<egui::TextureHandle>(id));
    let texture = texture.unwrap_or_else(|| {
        let base = if dark {
            p.surface.lerp_to_gamma(p.raised, 0.30)
        } else {
            p.canvas
        };
        let pixels = (0..TILE * TILE)
            .map(|index| {
                let (x, y) = (index % TILE, index / TILE);
                let warp = [2, -1, -2, 0][x % 4];
                let weft = [1, 2, -1, -2][y % 4];
                let thread = (((x / 4 * 37) ^ (y / 4 * 21)) % 5) as i32 - 2;
                let shade = (warp + weft + thread) / if dark { 1 } else { 2 };
                let tint = |channel| (i32::from(channel) + shade).clamp(0, 255) as u8;
                Color32::from_rgb(tint(base.r()), tint(base.g()), tint(base.b()))
            })
            .collect();
        let texture = ui.ctx().load_texture(
            "Library cloth",
            egui::ColorImage::new([TILE, TILE], pixels),
            egui::TextureOptions::LINEAR_REPEAT,
        );
        ui.ctx()
            .data_mut(|data| data.insert_temp(id, texture.clone()));
        texture
    });
    let area = ui.available_rect_before_wrap();
    let tile_count = area.size() / TILE as f32;
    ui.painter().image(
        texture.id(),
        area,
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(tile_count.x, tile_count.y)),
        Color32::WHITE,
    );
}

impl YamantakaApp {
    pub(super) fn library_sidebar(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.label(theme::editorial("Yamantaka", 28.0).color(p.ink));
        ui.add_space(14.0);
        ui.spacing_mut().item_spacing.y = 4.0;
        egui::ScrollArea::vertical()
            .id_salt("library-sidebar-scroll")
            .show(ui, |ui| {
                let scopes = [
                    (
                        false,
                        LibraryReadingState::All,
                        self.tr("Todos los libros", "All books"),
                        !self.library_has_filters(),
                    ),
                    (
                        true,
                        LibraryReadingState::All,
                        self.tr("Favoritos", "Favorites"),
                        self.favorites && self.collections.filter.is_none(),
                    ),
                    (
                        false,
                        LibraryReadingState::InProgress,
                        self.tr("En lectura", "In progress"),
                        self.reading_state == LibraryReadingState::InProgress
                            && self.collections.filter.is_none(),
                    ),
                ];
                for (favorites, state, label, selected) in scopes {
                    let count = (selected
                        && !favorites
                        && state == LibraryReadingState::All
                        && self.library_loaded)
                        .then_some(self.count);
                    if theme::library_link(ui, label, count, selected, p).clicked() {
                        self.clear_library_filters();
                        self.favorites = favorites;
                        self.reading_state = state;
                        self.refresh();
                    }
                }
                if theme::library_link(
                    ui,
                    self.tr("Notas de la biblioteca", "Library notes"),
                    None,
                    false,
                    p,
                )
                .clicked()
                {
                    self.show_global_notes();
                    return;
                }
                ui.add_space(14.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.tr("Colecciones", "Collections")).color(p.secondary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::icon_button(
                            ui,
                            self.tr("Gestionar colecciones", "Manage collections"),
                            theme::Icon::Folder,
                            p,
                        )
                        .clicked()
                        {
                            self.show_collections(None);
                        }
                    });
                });
                if matches!(self.surface, Surface::Library) {
                    self.library_collection_links(ui, p);
                }
            });
    }

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
        let wide = width >= 840.0;
        let compact_search = width < 540.0;
        let filtered = self.library_has_filters();
        if width >= 460.0 {
            ui.horizontal(|ui| {
                let reserve = if wide {
                    395.0
                } else if width >= 660.0 {
                    220.0
                } else {
                    0.0
                };
                ui.allocate_ui_with_layout(
                    egui::vec2(width - reserve, 46.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let title = if let Some((_, name)) = &self.collections.filter {
                            name.as_str()
                        } else if self.favorites {
                            self.tr("Favoritos", "Favorites")
                        } else if self.reading_state == LibraryReadingState::InProgress {
                            self.tr("En lectura", "In progress")
                        } else {
                            self.tr("Biblioteca", "Library")
                        };
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.add(
                            egui::Label::new(theme::editorial(title, 32.0).color(p.ink)).truncate(),
                        )
                        .on_hover_text(title);
                        if self.library_loaded {
                            ui.label(
                                RichText::new(if self.count == 1 {
                                    self.tr("1 documento", "1 document").to_owned()
                                } else {
                                    format!("{} {}", self.count, self.tr("documentos", "documents"))
                                })
                                .size(13.0)
                                .color(p.secondary),
                            );
                        }
                    },
                );
                if reserve > 0.0 {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        if wide {
                            self.library_search_menu(ui, p);
                        }
                        self.library_display_options(ui, true, p);
                        if wide && !self.list {
                            self.library_size_control(ui, p);
                        }
                    });
                }
            });
            ui.add_space(5.0);
        }
        let show_groups =
            width >= 620.0 && !self.list && self.library_loaded && !self.documents.is_empty();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if wide {
                if show_groups {
                    self.library_grouping_tabs(ui, p);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.tools_menu(ui, true, p);
                    if !filtered {
                        self.reading_shortcut(ui, p, true, false);
                    }
                    self.library_actions(ui, filtered, p);
                });
            } else {
                if compact_search {
                    self.library_search_menu(ui, p);
                } else {
                    self.library_search(ui, (width - 250.0).clamp(140.0, 390.0), p);
                }
                self.library_actions(ui, filtered, p);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.tools_menu(ui, true, p);
                    if !filtered {
                        self.reading_shortcut(ui, p, true, false);
                    }
                });
            }
        });
        if !wide && show_groups {
            ui.horizontal_wrapped(|ui| self.library_grouping_tabs(ui, p));
        }
        if filtered {
            ui.horizontal(|ui| {
                if width < 460.0 {
                    ui.label(
                        RichText::new(format!(
                            "{} {}",
                            self.count,
                            if self.count == 1 {
                                self.tr("resultado", "result")
                            } else {
                                self.tr("resultados", "results")
                            },
                        ))
                        .size(13.0)
                        .color(p.secondary),
                    );
                }
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
                if wide && !self.query.trim().is_empty() {
                    let query = self.query.trim();
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("“{query}”"))
                                .size(13.0)
                                .color(p.secondary),
                        )
                        .truncate(),
                    )
                    .on_hover_text(query);
                } else if width < 460.0
                    && let Some((_, collection)) = &self.collections.filter
                {
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

    fn library_size_control(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.scope(|ui| {
            ui.spacing_mut().slider_width = 88.0;
            ui.spacing_mut().interact_size.y = 18.0;
            ui.spacing_mut().slider_rail_height = 4.0;
            let visuals = ui.visuals_mut();
            visuals.selection.bg_fill = p.accent;
            visuals.widgets.inactive.bg_fill = p.line;
            visuals.widgets.hovered.bg_fill = p.surface;
            visuals.widgets.active.bg_fill = p.surface;
            for widget in [
                &mut visuals.widgets.inactive,
                &mut visuals.widgets.hovered,
                &mut visuals.widgets.active,
            ] {
                widget.expansion = 0.0;
                widget.corner_radius = egui::CornerRadius::same(2);
                widget.fg_stroke = egui::Stroke::new(1.5, p.accent);
            }
            let label = self.tr("Tamaño de las portadas", "Cover size");
            let response = ui.add(
                egui::Slider::new(&mut self.library_cover_scale, 0.78..=1.24)
                    .step_by(0.01)
                    .show_value(false)
                    .handle_shape(egui::style::HandleShape::Circle)
                    .trailing_fill(true),
            );
            response.widget_info(|| {
                egui::WidgetInfo::slider(ui.is_enabled(), self.library_cover_scale.into(), label)
            });
            theme::record_control_bounds(ui, label, response.rect);
            response.on_hover_text(label);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 40.0), egui::Sense::hover());
            theme::icon(ui.painter(), rect.center(), theme::Icon::Book, p.secondary);
        });
    }

    fn library_actions(&mut self, ui: &mut egui::Ui, filtered: bool, p: Palette) {
        let add_label = self.tr("Añadir PDF", "Add PDFs");
        let add = theme::primary_icon(ui, add_label, theme::Icon::Add, p);
        if add.clicked()
            && let Some(paths) = yamantaka_windows::pick_pdfs()
        {
            self.import(paths);
        }
        let response = theme::icon_toggle(
            ui,
            self.tr("Filtros", "Filters"),
            theme::Icon::Sort,
            filtered,
            p,
        );
        let (mut popup, size) = theme::dropdown(&response, 240.0);
        if self.smart_menu_preview {
            popup = popup.open(true);
        }
        popup.show(|ui| {
            size.show(ui, |ui| {
                self.collection_menu(ui, p);
                self.library_filters(ui, p);
            });
        });
        self.library_view_menu(ui, true, p);
        if self.library_selection.toolbar_visible() {
            self.library_selection_control(ui, p);
        }
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
        let (mut popup, size) = theme::dropdown(&response, 212.0);
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
            size.show(ui, |ui| {
                self.library_display_options(ui, false, p);
                ui.separator();
                self.library_sort_control(ui, p);
            });
        });
    }

    fn library_display_options(&mut self, ui: &mut egui::Ui, direct: bool, p: Palette) {
        let volume = self.tr("Portadas 3D", "3D covers");
        let flat = self.tr("Portadas 2D", "2D covers");
        let list = self.tr("Lista", "List");
        let options = [
            (false, false, volume, "3D", theme::Icon::Book),
            (false, true, flat, "2D", theme::Icon::Grid),
            (true, false, list, list, theme::Icon::List),
        ];
        let mut render = |ui: &mut egui::Ui, (show_list, show_flat, label, short, icon)| {
            let selected = self.list == show_list && (show_list || self.flat_covers == show_flat);
            let response = if direct {
                theme::navigation(ui, short, selected, p)
            } else {
                theme::menu_choice(ui, label, icon, selected, p)
            };
            if response.on_hover_text(label).clicked() {
                self.list = show_list;
                if !show_list {
                    self.flat_covers = show_flat;
                }
                if !direct {
                    ui.close();
                }
            }
        };
        if direct {
            egui::Frame::new()
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.line))
                .corner_radius(12)
                .inner_margin(egui::Margin::same(2))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 1.0;
                    for option in options.into_iter().rev() {
                        render(ui, option);
                    }
                });
        } else {
            for option in options {
                render(ui, option);
            }
        }
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
        let (mut popup, size) = theme::dropdown(&response, 420.0);
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
            size.show(ui, |ui| {
                let label = ui.label(self.tr("Buscar en biblioteca", "Search library"));
                let input = self
                    .library_search(ui, ui.available_width(), p)
                    .labelled_by(label.id);
                if response.clicked() {
                    input.request_focus();
                }
            });
        });
    }
}
