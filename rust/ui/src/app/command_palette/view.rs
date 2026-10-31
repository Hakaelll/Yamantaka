use super::*;
impl YamantakaApp {
    pub(in crate::app) fn command_palette_dialog(&mut self, ctx: &egui::Context) {
        if !self.command_palette.open {
            return;
        }
        if self.command_palette.context.is_some() && self.palette_document().is_none() {
            self.close_command_palette(ctx);
            return;
        }
        let p = Palette::new(self.dark);
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let mut close = false;
        let mut chosen = None;
        let search_id = egui::Id::new("command-palette-search");
        let (down, up, activate) = if !self.command_palette.info
            && ctx.memory(|memory| memory.focused() == Some(search_id))
        {
            ctx.input_mut(|input| {
                (
                    input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                    input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                    input.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                )
            })
        } else {
            (false, false, false)
        };
        let modal = egui::Modal::new(egui::Id::new("command-palette")).show(ctx, |ui| {
            ui.set_width((ctx.content_rect().width() - 48.0).clamp(120.0, 640.0));
            let height = (ctx.content_rect().height() - 48.0).clamp(80.0, 560.0);
            ui.set_max_height(height);
            egui::ScrollArea::vertical()
                .id_salt("command-palette-window")
                .max_height((ctx.content_rect().height() - 48.0).max(80.0))
                .animated(!self.reduced_motion)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let heading_width = (ui.available_width() - 80.0).max(40.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(heading_width, 0.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                theme::display(
                                    ui,
                                    if self.command_palette.info {
                                        tr("Información del documento", "Document information")
                                    } else {
                                        tr("Comandos", "Commands")
                                    },
                                    22.0,
                                );
                            },
                        );
                        if theme::navigation(ui, tr("Cerrar", "Close"), false, p)
                            .on_hover_text("Esc")
                            .clicked()
                        {
                            close = true;
                        }
                    });
                    ui.add_space(8.0);
                    if !self.command_palette.info
                        && let Some(document) = self.palette_document()
                    {
                        let target = if self
                            .command_palette
                            .context
                            .as_ref()
                            .is_some_and(|(target, _)| *target == ReadingTarget::Reference)
                        {
                            tr("Referencia", "Reference")
                        } else {
                            tr("Lectura principal", "Main reading")
                        };
                        ui.label(
                            RichText::new(format!(
                                "{target} · {}",
                                document
                                    .display_title()
                                    .chars()
                                    .take(80)
                                    .collect::<String>()
                            ))
                            .size(12.0)
                            .color(p.secondary),
                        );
                        ui.add_space(8.0);
                    }
                    if self.command_palette.info {
                        if let Some(document) = self.palette_document() {
                            for value in [
                                document.display_title(),
                                document.author.as_deref().unwrap_or("—"),
                                &document.file_name,
                            ] {
                                ui.label(value.chars().take(240).collect::<String>());
                            }
                            let count = if self
                                .command_palette
                                .context
                                .as_ref()
                                .is_some_and(|(target, _)| *target == ReadingTarget::Reference)
                            {
                                self.reference
                                    .as_ref()
                                    .and_then(|reference| reference.pane.active.as_ref())
                                    .map(|(_, info)| info.pages.len())
                            } else {
                                self.reader
                                    .active
                                    .as_ref()
                                    .map(|(_, info)| info.pages.len())
                            };
                            ui.label(format!("{} {}", count.unwrap_or(0), tr("páginas", "pages")));
                        }
                        return;
                    }
                    let search = ui.add(
                        egui::TextEdit::singleline(&mut self.command_palette.query)
                            .id(search_id)
                            .hint_text(tr(
                                "Buscar una acción o un documento…",
                                "Find an action or document…",
                            ))
                            .char_limit(160)
                            .desired_width(f32::INFINITY),
                    );
                    search.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::TextEdit,
                            ui.is_enabled(),
                            tr("Buscar comandos", "Find commands"),
                        )
                    });
                    if self.command_palette.focus {
                        search.request_focus();
                        self.command_palette.focus = false;
                    }
                    if search.changed() {
                        self.command_palette.selected = 0;
                    }
                    let entries = self.palette_entries();
                    let reveal = search.changed() || down || up;
                    if down {
                        self.command_palette.selected = self
                            .command_palette
                            .selected
                            .saturating_add(1)
                            .min(entries.len().saturating_sub(1));
                    }
                    if up {
                        self.command_palette.selected =
                            self.command_palette.selected.saturating_sub(1);
                    }
                    if reveal {
                        ctx.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
                    }
                    self.command_palette.selected = self
                        .command_palette
                        .selected
                        .min(entries.len().saturating_sub(1));
                    ui.add_space(8.0);
                    if entries.is_empty() {
                        ui.label(tr(
                            "No se encontraron acciones ni documentos recientes.",
                            "No matching actions or recent documents.",
                        ));
                    }
                    let wide = ui.available_width() > 460.0;
                    egui::ScrollArea::vertical()
                        .id_salt("command-palette-results")
                        .max_height((height - 190.0).clamp(48.0, 360.0))
                        .animated(!self.reduced_motion)
                        .show(ui, |ui| {
                            for (index, entry) in entries.iter().enumerate() {
                                let selected = index == self.command_palette.selected;
                                let mut button =
                                    egui::Button::new(RichText::new(&entry.label).color(p.ink))
                                        .selected(selected)
                                        .fill(if selected {
                                            p.raised
                                        } else {
                                            Color32::TRANSPARENT
                                        })
                                        .stroke(if selected {
                                            egui::Stroke::new(1.0, p.accent)
                                        } else {
                                            egui::Stroke::NONE
                                        })
                                        .wrap()
                                        .min_size(egui::vec2(ui.available_width(), 40.0));
                                if wide {
                                    button = button.shortcut_text(&entry.hint);
                                }
                                let response = ui
                                    .add_enabled(entry.enabled, button)
                                    .on_hover_text(&entry.hint);
                                theme::record_control_bounds(ui, &entry.label, response.rect);
                                if response.hovered()
                                    && ui.input(|input| input.pointer.delta() != egui::Vec2::ZERO)
                                    || response.has_focus()
                                {
                                    self.command_palette.selected = index;
                                }
                                if selected && reveal {
                                    response.scroll_to_me(Some(egui::Align::Center));
                                }
                                if response.clicked() {
                                    chosen = Some((entry.command, entry.id));
                                }
                            }
                        });
                    if activate
                        && let Some(entry) = entries
                            .get(self.command_palette.selected)
                            .filter(|entry| entry.enabled)
                    {
                        chosen = Some((entry.command, entry.id));
                    }
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("↑ ↓ · Enter · Esc")
                            .size(12.0)
                            .color(p.secondary),
                    );
                    if self.command_palette.error {
                        ui.label(tr(
                            "Los documentos recientes no están disponibles.",
                            "Recent documents are unavailable.",
                        ));
                    }
                });
        });
        if close || modal.should_close() {
            self.close_command_palette(ctx);
        } else if let Some((command, id)) = chosen {
            self.run_palette_command(ctx, command, id);
        }
    }
}
