use super::*;

impl TextState {
    pub(in crate::app) fn show(&self, ui: &mut egui::Ui, english: bool, p: Palette) -> bool {
        let tr = |es, en| if english { en } else { es };
        let compact = ui.ctx().content_rect().width() < 620.0;
        if compact {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(tr("Texto de la página", "Page text"))
                        .size(20.0)
                        .color(p.ink),
                );
                if let Some(page) = &self.page
                    && !page.text().trim().is_empty()
                    && theme::navigation(ui, tr("Copiar texto", "Copy text"), false, p).clicked()
                {
                    ui.ctx().copy_text(page.text().into());
                }
            });
        } else {
            ui.heading(tr("Texto de la página", "Page text"));
        }
        if self.pending.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new(tr("Extrayendo texto…", "Extracting text…")).color(p.secondary),
                );
            });
            return false;
        }
        if let Some(error) = &self.error {
            ui.label(RichText::new(error).color(p.secondary));
            if theme::navigation(ui, tr("Reintentar", "Retry"), false, p).clicked() {
                return true;
            }
            return false;
        }
        let Some(page) = &self.page else { return false };
        if page.text().trim().is_empty() {
            ui.label(
                RichText::new(tr(
                    "Esta página no contiene texto extraíble.",
                    "This page contains no extractable text.",
                ))
                .color(p.secondary),
            );
            return false;
        }
        if !compact && theme::navigation(ui, tr("Copiar texto", "Copy text"), false, p).clicked() {
            ui.ctx().copy_text(page.text().into());
        }
        ui.add_space(8.0);
        ui.add(egui::Label::new(RichText::new(page.text()).color(p.ink)).wrap());
        false
    }
}
