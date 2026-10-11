use super::*;

impl YamantakaApp {
    pub(in crate::app) fn reader_tab_strip(&mut self, ui: &mut egui::Ui, palette: Palette) {
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
        egui::Frame::new()
            .fill(palette.surface)
            .inner_margin(egui::Margin::symmetric(18, 6))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                if self.tabs.pending.is_some()
                    || self.opening.busy()
                    || self.reader_preferences_loading
                {
                    ui.disable();
                }
                if ui.available_width() < 420.0 {
                    let caption = entries
                        .iter()
                        .find(|(id, _, _)| Some(id) == active.as_ref())
                        .map(|(_, title, _)| title.as_str())
                        .unwrap_or("");
                    egui::ComboBox::from_id_salt("reader-tabs-compact")
                        .selected_text(caption)
                        .width((ui.available_width() - 8.0).max(40.0))
                        .truncate()
                        .show_ui(ui, |ui| {
                            for (id, title, pinned) in &entries {
                                if ui
                                    .selectable_label(
                                        Some(id) == active.as_ref(),
                                        format!("{}{}", if *pinned { "• " } else { "" }, title),
                                    )
                                    .clicked()
                                {
                                    selected = Some(id.clone());
                                }
                            }
                            ui.separator();
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
                                        egui::Button::new(
                                            self.tr("Mover a la izquierda", "Move left"),
                                        ),
                                    )
                                    .clicked()
                                {
                                    reorder = Some((index, index - 1));
                                }
                                if ui
                                    .add_enabled(
                                        index + 1 < entries.len(),
                                        egui::Button::new(
                                            self.tr("Mover a la derecha", "Move right"),
                                        ),
                                    )
                                    .clicked()
                                {
                                    reorder = Some((index, index + 1));
                                }
                            }
                        });
                } else {
                    egui::ScrollArea::horizontal()
                        .id_salt("reader-tab-strip")
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                for (index, (id, title, pinned)) in entries.iter().enumerate() {
                                    ui.push_id(id, |ui| {
                                        let label = if *pinned {
                                            format!("• {title}")
                                        } else {
                                            title.clone()
                                        };
                                        let response = ui.add_sized(
                                            [200.0, 32.0],
                                            egui::Button::new(
                                                RichText::new(label).size(13.0).color(
                                                    if Some(id) == active.as_ref() {
                                                        palette.ink
                                                    } else {
                                                        palette.secondary
                                                    },
                                                ),
                                            )
                                            .selected(Some(id) == active.as_ref())
                                            .fill(if Some(id) == active.as_ref() {
                                                palette.raised
                                            } else {
                                                Color32::TRANSPARENT
                                            })
                                            .stroke(egui::Stroke::NONE)
                                            .corner_radius(6)
                                            .truncate(),
                                        );
                                        if Some(id) == active.as_ref() {
                                            ui.painter().line_segment(
                                                [
                                                    response.rect.left_bottom()
                                                        + egui::vec2(8.0, -1.0),
                                                    response.rect.right_bottom()
                                                        + egui::vec2(-8.0, -1.0),
                                                ],
                                                egui::Stroke::new(2.0, palette.accent),
                                            );
                                        }
                                        if response.clicked() {
                                            selected = Some(id.clone());
                                        }
                                        response.on_hover_text(title).context_menu(|ui| {
                                            if ui
                                                .button(
                                                    self.tr(
                                                        "Duplicar lectura",
                                                        "Duplicate reading",
                                                    ),
                                                )
                                                .clicked()
                                            {
                                                selected = Some(id.clone());
                                                duplicate = true;
                                                ui.close();
                                            }
                                            if ui
                                                .button(if *pinned {
                                                    self.tr("Desfijar", "Unpin")
                                                } else {
                                                    self.tr("Fijar", "Pin")
                                                })
                                                .clicked()
                                            {
                                                pin = Some(id.clone());
                                                ui.close();
                                            }
                                            if ui
                                                .add_enabled(
                                                    index > 0,
                                                    egui::Button::new(
                                                        self.tr(
                                                            "Mover a la izquierda",
                                                            "Move left",
                                                        ),
                                                    ),
                                                )
                                                .clicked()
                                            {
                                                reorder = Some((index, index - 1));
                                                ui.close();
                                            }
                                            if ui
                                                .add_enabled(
                                                    index + 1 < entries.len(),
                                                    egui::Button::new(
                                                        self.tr("Mover a la derecha", "Move right"),
                                                    ),
                                                )
                                                .clicked()
                                            {
                                                reorder = Some((index, index + 1));
                                                ui.close();
                                            }
                                        });
                                    });
                                }
                                if theme::icon_button(
                                    ui,
                                    self.tr("Duplicar lectura actual", "Duplicate current reading"),
                                    theme::Icon::Plus,
                                    palette,
                                )
                                .clicked()
                                {
                                    duplicate = true;
                                }
                            });
                        });
                }
            });
        if duplicate {
            if let Some(id) = selected {
                self.duplicate_reader_tab_from(&id);
            } else {
                self.duplicate_reader_tab();
            }
        } else if let Some(id) = selected {
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
