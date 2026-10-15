use super::*;

impl YamantakaApp {
    pub(super) fn reading_feature(&mut self, ui: &mut egui::Ui, document: &Document, p: Palette) {
        let wide = ui.available_width() >= 760.0;
        let cover_width = 42.0;
        let mut open = false;
        egui::Frame::new()
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(16, 8))
            .corner_radius(10.0)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    open |= ui
                        .push_id("continuity-book", |ui| {
                            book::miniature(ui, document, cover_width, self.reduced_motion, p)
                        })
                        .inner
                        .clicked();
                    ui.add_space(10.0);
                    let action_width = if wide { 200.0 } else { 0.0 };
                    let text_width = (ui.available_width() - action_width - 12.0).max(100.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(text_width, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(text_width);
                            ui.spacing_mut().item_spacing.y = 3.0;
                            ui.add(
                                egui::Label::new(RichText::new(document.display_title()).font(
                                    egui::FontId::new(
                                        20.0,
                                        egui::FontFamily::Name("Editorial".into()),
                                    ),
                                ))
                                .truncate(),
                            );
                            let author = document
                                .author
                                .as_deref()
                                .filter(|author| !author.trim().is_empty());
                            let progress = document.reading_progress.clamp(0.0, 1.0);
                            let position = format!(
                                "{} {} {} {} · {:.0}%",
                                self.tr("Página", "Page"),
                                document
                                    .last_page_index
                                    .saturating_add(1)
                                    .min(document.page_count),
                                self.tr("de", "of"),
                                document.page_count,
                                progress * 100.0
                            );
                            let summary = author.map_or_else(
                                || position.clone(),
                                |author| format!("{author} · {position}"),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&summary).size(13.0).color(p.secondary),
                                )
                                .truncate(),
                            )
                            .on_hover_text(summary);
                            let (track, _) = ui.allocate_exact_size(
                                egui::vec2(text_width.min(120.0), 2.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(track, 2.0, p.line);
                            let mut filled = track;
                            filled.max.x = filled.min.x + filled.width() * progress as f32;
                            ui.painter().rect_filled(filled, 2.0, p.accent);
                            if !wide {
                                ui.add_space(4.0);
                                open |= theme::control(
                                    ui,
                                    self.tr("Continuar leyendo", "Continue reading"),
                                    Some(theme::Icon::Right),
                                    false,
                                    true,
                                    p,
                                )
                                .clicked();
                            }
                        },
                    );
                    if wide {
                        ui.vertical(|ui| {
                            ui.add_space(10.0);
                            open |= theme::control(
                                ui,
                                self.tr("Continuar leyendo", "Continue reading"),
                                Some(theme::Icon::Right),
                                false,
                                true,
                                p,
                            )
                            .clicked();
                        });
                    }
                });
            });
        if open {
            self.open(document.clone());
        }
        ui.add_space(12.0);
    }
}
