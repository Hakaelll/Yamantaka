use super::*;

impl YamantakaApp {
    pub(super) fn reading_shortcut(&mut self, ui: &mut egui::Ui, p: Palette) {
        let active = self.tabs.model.active().map(|tab| tab.document());
        let Some(document) = self
            .documents
            .iter()
            .find(|document| Some(document.id.as_str()) == active)
            .or_else(|| {
                self.documents
                    .iter()
                    .find(|document| document.reading_progress > 0.0)
            })
            .cloned()
        else {
            return;
        };
        let label = self.tr("Continuar leyendo", "Continue reading");
        let response = if ui.ctx().content_rect().width() >= 1100.0 {
            theme::control(ui, label, Some(theme::Icon::Right), false, false, p)
        } else {
            theme::icon_button(ui, label, theme::Icon::Right, p)
        };
        if response
            .on_hover_text(format!(
                "{}\n{} {} / {} · {:.0}%",
                document.display_title(),
                self.tr("Página", "Page"),
                document
                    .last_page_index
                    .saturating_add(1)
                    .min(document.page_count),
                document.page_count,
                document.reading_progress.clamp(0.0, 1.0) * 100.0,
            ))
            .clicked()
        {
            self.open(document);
        }
    }
}
