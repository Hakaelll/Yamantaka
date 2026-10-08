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
        for document in documents {
            ui.push_id(("compact-jacket", &document.id), |ui| {
                ui.horizontal_top(|ui| {
                    let response = book::miniature(ui, &document, 76.0, self.reduced_motion, p);
                    if response.clicked() {
                        self.selected = Some(document.id.clone());
                    }
                    if response.double_clicked()
                        || response.has_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        self.open(document.clone());
                    }
                    response.on_hover_text(document.display_title());
                    let width = ui.available_width();
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, 108.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(width);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(document.display_title()).size(16.0),
                                )
                                .wrap(),
                            );
                            ui.label(
                                RichText::new(document.author.as_deref().unwrap_or(""))
                                    .size(12.0)
                                    .color(p.secondary),
                            );
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "{:.0}%",
                                        document.reading_progress.clamp(0.0, 1.0) * 100.0
                                    ))
                                    .size(12.0)
                                    .color(p.secondary),
                                );
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
