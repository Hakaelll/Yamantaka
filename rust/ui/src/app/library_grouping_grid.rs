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
            let gap = if compact { 20.0 } else { 26.0 };
            let preferred = if compact { 172.0 } else { 212.0 };
            let columns = ((ui.available_width() + gap) / (preferred + gap))
                .floor()
                .max(1.0) as usize;
            self.library_selection.columns = columns;
            let slot =
                ((ui.available_width() - gap * (columns - 1) as f32) / columns as f32).max(100.0);
            let width = slot.min(preferred);
            for row in documents.chunks(columns) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for document in row {
                        let response = ui
                            .allocate_ui_with_layout(
                                egui::vec2(slot, book::cover_height(width) + book::CAPTION_HEIGHT),
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    ui.push_id(&document.id, |ui| {
                                        let response = book::cover(
                                            ui,
                                            document,
                                            width,
                                            self.library_selection.model.contains(&document.id),
                                            self.reduced_motion,
                                            p,
                                            self.preview_tilt.filter(|_| document.id == "demo-0"),
                                        );
                                        let menu_rect = egui::Rect::from_min_size(
                                            egui::pos2(
                                                response.rect.right() - 32.0,
                                                response.rect.bottom() - 35.0,
                                            ),
                                            egui::vec2(32.0, 26.0),
                                        );
                                        let label = format!(
                                            "{} · {}",
                                            self.tr("Acciones del libro", "Book actions"),
                                            document.display_title()
                                        );
                                        let actions = ui
                                            .put(menu_rect, egui::Button::new("⋯").frame(false))
                                            .on_hover_text(&label);
                                        actions.widget_info(|| {
                                            egui::WidgetInfo::labeled(
                                                egui::WidgetType::Button,
                                                actions.enabled(),
                                                &label,
                                            )
                                        });
                                        if actions.has_focus() {
                                            ui.painter().rect_stroke(
                                                actions.rect,
                                                4.0,
                                                egui::Stroke::new(2.0, p.accent),
                                                egui::StrokeKind::Inside,
                                            );
                                        }
                                        egui::Popup::menu(&actions).show(|ui| {
                                            ui.set_max_width(
                                                (ui.ctx().content_rect().width() - 24.0)
                                                    .clamp(160.0, 300.0),
                                            );
                                            self.library_book_menu(ui, document);
                                        });
                                        response
                                    })
                                    .inner
                                },
                            )
                            .inner;
                        self.library_book_response(ui, &response, document);
                    }
                });
                ui.add_space(8.0);
            }
        }
    }
}

impl YamantakaApp {
    pub(super) fn library_catalogue(&mut self, ui: &mut egui::Ui, compact: bool, p: Palette) {
        self.grouped_library(ui, compact, p);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.offset > 0,
                    egui::Button::new(self.tr("Anterior", "Previous")),
                )
                .clicked()
            {
                self.offset = self.offset.saturating_sub(80);
                self.refresh();
            }
            ui.label(format!(
                "{}–{} / {}",
                self.offset + 1,
                (self.offset + self.documents.len()).min(self.count as usize),
                self.count
            ));
            if ui
                .add_enabled(
                    self.offset + self.documents.len() < self.count as usize,
                    egui::Button::new(self.tr("Siguiente", "Next")),
                )
                .clicked()
            {
                self.offset += 80;
                self.refresh();
            }
        });
    }
}
