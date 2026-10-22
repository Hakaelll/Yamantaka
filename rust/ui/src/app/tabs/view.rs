use super::*;

impl YamantakaApp {
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
