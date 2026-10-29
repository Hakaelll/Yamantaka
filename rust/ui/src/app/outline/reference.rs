use super::*;
use crate::app::reference::ReferenceView;

impl ReferenceView {
    pub(in crate::app) fn reference_outline_contents(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        english: bool,
        p: Palette,
    ) {
        let label = if english {
            "Reference contents"
        } else {
            "Índice de referencia"
        };
        ui.label(RichText::new(label).size(18.0));
        let Some((document, _)) = &self.pane.active else {
            return;
        };
        self.pane.outline.request(runtime, &document.id);
        let height = (ui.ctx().content_rect().height() - 160.0).max(80.0);
        if let Some(destination) =
            self.pane
                .outline
                .show(ui, self.pane.page, p, english, true, Some(height))
        {
            self.pane.reader_destination = Some((document.id.clone(), destination));
            ui.ctx().request_repaint();
            ui.close();
        }
    }
}
