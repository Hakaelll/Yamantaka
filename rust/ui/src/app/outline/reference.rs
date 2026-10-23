use super::*;
use crate::app::reference::ReferenceView;

impl ReferenceView {
    pub(in crate::app) fn reference_outline(
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
        let button = theme::icon_button(ui, label, theme::Icon::List, p);
        let mut popup = egui::Popup::menu(&button)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        if self.outline_preview {
            popup = popup.open(true);
        }
        let shown = popup.show(|ui| {
            ui.set_width((ui.ctx().content_rect().width() - 40.0).clamp(120.0, 420.0));
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
        });
        if self.outline_visible && shown.is_none() {
            self.pane.outline.cancel();
        }
        self.outline_visible = shown.is_some();
    }
}
