use super::*;

impl YamantakaApp {
    pub(in crate::app) fn session_bar(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        if self.logo.is_none()
            && let Ok(icon) = image::load_from_memory(include_bytes!(
                "../../../../../assets/branding/yamantaka-logo-purple.png"
            ))
        {
            let icon = icon.to_rgba8();
            self.logo = Some(ui.ctx().load_texture(
                "Yamantaka brand",
                egui::ColorImage::from_rgba_unmultiplied(
                    [icon.width() as usize, icon.height() as usize],
                    icon.as_raw(),
                ),
                TextureOptions::LINEAR,
            ));
        }
        egui::Frame::new()
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(12, 2))
            .show(ui, |ui| {
                let compact = ui.available_width() < 500.0;
                ui.horizontal(|ui| {
                    if let Some(logo) = &self.logo {
                        ui.add(egui::Image::new(logo).fit_to_exact_size(egui::vec2(22.0, 22.0)));
                    }
                    self.library_return(ui, compact, p);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.session_menu(ui, compact, p);
                    });
                });
            });
    }

    pub(in crate::app) fn library_return(&mut self, ui: &mut egui::Ui, compact: bool, p: Palette) {
        let label = self.tr("Biblioteca", "Library");
        let button = if compact {
            theme::icon_button(ui, label, theme::Icon::Book, p)
        } else {
            theme::control(ui, label, Some(theme::Icon::Left), false, false, p)
        };
        let metadata = matches!(self.surface, Surface::Reader) && self.metadata_return_available();
        let button = if metadata {
            button.on_hover_text(self.tr("Volver a los metadatos", "Back to metadata"))
        } else {
            button
        };
        if button.clicked() {
            if self.inspector && !metadata {
                self.close_book_details();
            }
            self.show_surface(Surface::Library);
        }
    }

    pub(in crate::app) fn reader_switcher(&mut self, ui: &mut egui::Ui, width: f32, p: Palette) {
        let Some((document, _)) = &self.reader.active else {
            return;
        };
        let title = document.display_title().to_owned();
        let label = format!("{} · {title}", self.tr("Lecturas", "Readings"));
        ui.allocate_ui_with_layout(
            egui::vec2(width, 40.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_width(width);
                let response = theme::bounded_menu(ui, &title, p).on_hover_text(&label);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, response.enabled(), &label)
                });
                egui::Popup::menu(&response).show(|ui| {
                    egui::ScrollArea::vertical()
                        .max_height((ui.ctx().content_rect().height() - 72.0).max(64.0))
                        .show(ui, |ui| self.reader_documents_menu(ui));
                });
            },
        );
    }

    pub(in crate::app) fn session_menu(&mut self, ui: &mut egui::Ui, compact: bool, p: Palette) {
        let label = self.tr("Sesión", "Session");
        let response = if compact {
            theme::icon_button(ui, label, theme::Icon::Menu, p)
        } else {
            theme::control(ui, label, Some(theme::Icon::Menu), false, false, p)
        };
        egui::Popup::menu(&response).show(|ui| {
            egui::ScrollArea::vertical()
                .max_height((ui.ctx().content_rect().height() - 80.0).max(60.0))
                .show(ui, |ui| {
                    if !self.tabs.model.tabs().is_empty() {
                        ui.menu_button(self.tr("Lecturas abiertas", "Open readings"), |ui| {
                            self.reader_documents_menu(ui);
                        });
                    }
                    if !self.tabs.model.closed().is_empty()
                        && theme::bounded_navigation(
                            ui,
                            self.tr("Reabrir última lectura", "Reopen last reading"),
                            p,
                        )
                        .clicked()
                    {
                        self.reopen_reader_tab();
                        ui.close();
                    }
                    if theme::bounded_navigation(ui, self.tr("Cola de lectura", "Reading queue"), p)
                        .clicked()
                    {
                        self.show_reading_queue();
                        ui.close();
                    }
                    if theme::bounded_navigation(
                        ui,
                        self.tr("Espacios de trabajo", "Workspaces"),
                        p,
                    )
                    .clicked()
                    {
                        self.show_workspaces();
                        ui.close();
                    }
                    if theme::bounded_navigation(
                        ui,
                        self.tr("Diario de lectura", "Reading journal"),
                        p,
                    )
                    .clicked()
                    {
                        self.show_history();
                        ui.close();
                    }
                });
        });
    }

    pub(in crate::app) fn reader_documents_menu(&mut self, ui: &mut egui::Ui) {
        self.sync_tab_context();
        let entries: Vec<_> = self
            .tabs
            .model
            .tabs()
            .iter()
            .map(|tab| {
                (
                    tab.id().to_owned(),
                    self.tabs
                        .captions
                        .get(tab.document())
                        .cloned()
                        .unwrap_or_else(|| self.tr("Documento", "Document").into()),
                    tab.pinned(),
                )
            })
            .collect();
        if entries.is_empty() {
            return;
        }
        let active = self.tabs.model.active().map(|tab| tab.id().to_owned());
        let mut selected = None;
        let mut duplicate = false;
        let mut pin = None;
        let mut reorder = None;
        let mut close = None;
        let mut reopen = false;
        let mut bulk = None;
        ui.set_max_width(ui.available_width().min(340.0));
        ui.add_enabled_ui(
            !(self.library_selection.removing()
                || self.tabs.busy()
                || self.opening.busy()
                || self.reader.reader_preferences_loading),
            |ui| {
                for (id, title, pinned) in &entries {
                    if ui
                        .selectable_label(
                            Some(id) == active.as_ref(),
                            format!(
                                "{}{}",
                                title,
                                if *pinned {
                                    self.tr(" (fijado)", " (pinned)")
                                } else {
                                    ""
                                }
                            ),
                        )
                        .clicked()
                    {
                        selected = Some(id.clone());
                    }
                }
                ui.separator();
                if ui
                    .button(self.tr("Cerrar lectura", "Close reading"))
                    .clicked()
                {
                    close = active.clone();
                }
                if ui
                    .button(self.tr("Cerrar las demás", "Close other readings"))
                    .clicked()
                    && let Some(id) = active.clone()
                {
                    bulk = Some((id, false));
                }
                if ui
                    .button(self.tr("Cerrar las siguientes", "Close later readings"))
                    .clicked()
                    && let Some(id) = active.clone()
                {
                    bulk = Some((id, true));
                }
                if ui
                    .add_enabled(
                        !self.tabs.model.closed().is_empty(),
                        egui::Button::new(self.tr("Reabrir última lectura", "Reopen last reading")),
                    )
                    .clicked()
                {
                    reopen = true;
                }
                duplicate = ui
                    .button(self.tr("Duplicar lectura", "Duplicate reading"))
                    .clicked();
                if let Some(index) = entries
                    .iter()
                    .position(|(id, _, _)| Some(id) == active.as_ref())
                {
                    let (id, _, pinned) = &entries[index];
                    if ui
                        .button(if *pinned {
                            self.tr("Desfijar", "Unpin")
                        } else {
                            self.tr("Fijar", "Pin")
                        })
                        .clicked()
                    {
                        pin = Some(id.clone());
                    }
                    if ui
                        .add_enabled(
                            index > 0,
                            egui::Button::new(self.tr("Mover antes", "Move earlier")),
                        )
                        .clicked()
                    {
                        reorder = Some((index, index - 1));
                    }
                    if ui
                        .add_enabled(
                            index + 1 < entries.len(),
                            egui::Button::new(self.tr("Mover después", "Move later")),
                        )
                        .clicked()
                    {
                        reorder = Some((index, index + 1));
                    }
                }
            },
        );
        if let Some((id, to_right)) = bulk {
            self.close_other_reader_tabs(&id, to_right);
            return;
        }
        if let Some(id) = close {
            self.close_reader_tab(&id);
            return;
        }
        if reopen {
            self.reopen_reader_tab();
            return;
        }
        if duplicate {
            if let Some(id) = selected {
                self.duplicate_reader_tab_from(&id);
            } else {
                self.duplicate_reader_tab();
            }
        } else if let Some(id) = selected
            && (active.as_ref() != Some(&id) || !matches!(self.surface, Surface::Reader))
        {
            self.select_reader_tab(&id);
        }
        if let Some(id) = pin
            && !self.tabs.model.toggle_pinned(&id)
        {
            self.error = Some(
                self.tr(
                    "Puedes fijar hasta tres lecturas.",
                    "You can pin up to three readings.",
                )
                .into(),
            );
        }
        if let Some((from, to)) = reorder {
            self.tabs.model.reorder(from, to);
        }
    }
}
