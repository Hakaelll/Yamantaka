use super::*;

impl YamantakaApp {
    /// Large text in a small desktop window uses a jacket-and-title row, keeping
    /// the physical book and the readable document name together.
    pub(super) fn compact_covers(
        &mut self,
        ui: &mut egui::Ui,
        documents: Vec<Document>,
        p: Palette,
    ) {
        self.library_selection.columns = 1;
        let cramped = ui.ctx().content_rect().height() < 290.0;
        for document in documents {
            ui.push_id(("compact-jacket", &document.id), |ui| {
                ui.horizontal_top(|ui| {
                    let response = book::miniature(
                        ui,
                        &document,
                        if cramped { 44.0 } else { 58.0 },
                        self.library_selection.model.contains(&document.id),
                        self.reduced_motion,
                        p,
                    );
                    self.library_book_response(ui, &response, &document);
                    response.on_hover_text(document.display_title());
                    let width = ui.available_width();
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, if cramped { 62.0 } else { 80.0 }),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(width);
                            ui.spacing_mut().item_spacing.y = 3.0;
                            ui.add(
                                egui::Label::new(
                                    RichText::new(document.display_title()).size(16.0),
                                )
                                .wrap(),
                            );
                            if !cramped {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(document.author.as_deref().unwrap_or(""))
                                            .size(12.0)
                                            .color(p.secondary),
                                    )
                                    .truncate(),
                                );
                            }
                            ui.horizontal(|ui| {
                                if theme::primary_icon(
                                    ui,
                                    self.tr("Leer documento", "Read document"),
                                    theme::Icon::Right,
                                    p,
                                )
                                .clicked()
                                {
                                    self.open(document.clone());
                                }
                                let details = theme::icon_button(
                                    ui,
                                    self.tr("Ver ficha", "Book details"),
                                    theme::Icon::Panel,
                                    p,
                                );
                                if details.clicked() {
                                    self.selected = Some(document.id.clone());
                                    self.inspector = true;
                                }
                            });
                        },
                    );
                });
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
            });
        }
    }
}
