use super::*;
impl YamantakaApp {
    pub(super) fn library_document_page(
        &mut self,
        ui: &mut egui::Ui,
        documents: Vec<Document>,
        compact: bool,
        p: Palette,
    ) {
        let compact = compact
            || self.library_density == yamantaka_domain::workspaces::WorkspaceDensity::Compact;
        if self.list {
            self.library_selection.columns = 1;
            self.list_documents(ui, documents, p);
        } else if ui.ctx().content_rect().width() < 420.0 {
            self.compact_covers(ui, documents, p);
        } else {
            let columns = (ui.available_width() / if compact { 260.0 } else { 240.0 })
                .floor()
                .max(1.0) as usize;
            self.library_selection.columns = columns;
            let gap = 36.0;
            let slot =
                ((ui.available_width() - gap * (columns - 1) as f32) / columns as f32).max(100.0);
            let width = if compact {
                slot.min(
                    ((ui.available_height() - book::CAPTION_HEIGHT - 5.0) / 1.34)
                        .clamp(110.0, 180.0),
                )
            } else {
                slot.min(228.0)
            };
            for row in documents.chunks(columns) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for document in row {
                        let response = ui
                            .allocate_ui_with_layout(
                                egui::vec2(slot, width * 1.34 + book::CAPTION_HEIGHT),
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    ui.push_id(&document.id, |ui| {
                                        book::cover(
                                            ui,
                                            document,
                                            width,
                                            self.library_selection.model.contains(&document.id),
                                            self.reduced_motion,
                                            p,
                                            if self.preview_tilt && document.id == "demo-0" {
                                                Some(egui::vec2(0.95, 0.75))
                                            } else {
                                                None
                                            },
                                        )
                                    })
                                    .inner
                                },
                            )
                            .inner;
                        self.library_book_response(ui, &response, document);
                        response.context_menu(|ui| {
                            if ui
                                .button(self.tr("Abrir documento", "Open document"))
                                .clicked()
                            {
                                self.open(document.clone());
                                ui.close();
                            }
                            if ui
                                .button(self.tr("Mostrar en Explorer", "Show in Explorer"))
                                .clicked()
                            {
                                if let Err(error) = yamantaka_windows::reveal(&document.path) {
                                    self.error = Some(error.to_string());
                                }
                                ui.close();
                            }
                            if ui
                                .add_enabled(
                                    !self.library_selection.writing(),
                                    egui::Button::new(
                                        self.tr("Cambiar favorito", "Toggle favorite"),
                                    ),
                                )
                                .clicked()
                            {
                                self.send(Action::Favorite(
                                    document.id.clone(),
                                    !document.favorite,
                                ));
                                self.refresh();
                                ui.close();
                            }
                        });
                    }
                });
                ui.add_space(14.0);
            }
        }
    }
}
