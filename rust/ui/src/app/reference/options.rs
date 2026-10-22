use super::*;

impl ReferenceView {
    pub(super) fn options_menu(
        &mut self,
        ui: &mut egui::Ui,
        primary: Option<&ReaderNavigation>,
        english: bool,
        p: Palette,
    ) {
        let tr = |es, en| if english { en } else { es };
        let height = (ui.ctx().content_rect().height() - 96.0).max(80.0);
        egui::ScrollArea::vertical()
            .max_height(height)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(tr("Página y zoom", "Page and zoom"))
                        .color(p.secondary)
                        .size(12.0),
                );
                ui.checkbox(&mut self.follow_page, tr("Seguir página", "Follow page"));
                ui.checkbox(&mut self.follow_zoom, tr("Seguir zoom", "Follow zoom"));
                ui.checkbox(
                    &mut self.bidirectional,
                    tr("En ambas direcciones", "In both directions"),
                );
                ui.checkbox(
                    &mut self.follow_position,
                    tr("Seguir posición", "Follow position"),
                )
                .on_hover_text(tr(
                    "La posición sigue siempre a la lectura principal.",
                    "Position always follows the main reading.",
                ));
                ui.separator();
                ui.label(
                    RichText::new(tr("Disposición", "Arrangement"))
                        .color(p.secondary)
                        .size(12.0),
                );
                for (layout, es, en) in [
                    (ComparisonLayout::Automatic, "Automática", "Automatic"),
                    (ComparisonLayout::SideBySide, "Lado a lado", "Side by side"),
                    (ComparisonLayout::Stacked, "Una encima de otra", "Stacked"),
                ] {
                    if theme::navigation(ui, tr(es, en), self.layout == layout, p).clicked() {
                        self.layout = layout;
                    }
                }
                if let Some(primary) = primary {
                    self.difference_options(ui, primary, english, p);
                }
            });
    }
}
