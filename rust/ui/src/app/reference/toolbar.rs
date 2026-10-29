use super::*;

impl ReferenceView {
    pub(super) fn toolbar_tools(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        primary: Option<&ReaderNavigation>,
        can_choose: bool,
        english: bool,
        p: Palette,
    ) {
        let tr = |es, en| if english { en } else { es };
        ui.add_enabled_ui(can_choose, |ui| {
            let button = theme::icon_button(
                ui,
                tr("Elegir libro de referencia", "Choose reference book"),
                theme::Icon::Book,
                p,
            );
            if button.clicked() {
                self.chooser.reopen();
            }
            let mut popup = egui::Popup::menu(&button)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
            if self.chooser.preview {
                popup = popup.open(true);
            }
            popup.show(|ui| {
                self.selection = self.chooser.show(
                    ui,
                    runtime,
                    english,
                    p,
                    self.pane.active.as_ref().map(|(doc, _)| doc.id.as_str()),
                );
            });
        });
        let out = theme::icon_button(
            ui,
            tr("Alejar referencia", "Zoom reference out"),
            theme::Icon::Minus,
            p,
        )
        .clicked();
        let zoom_in = theme::icon_button(
            ui,
            tr("Acercar referencia", "Zoom reference in"),
            theme::Icon::Plus,
            p,
        )
        .clicked();
        let fit = theme::navigation(ui, tr("Ajustar", "Fit"), false, p).clicked();
        if out || zoom_in {
            let zoom =
                (self.pane.viewport.scale * if zoom_in { 1.2 } else { 1.0 / 1.2 }).clamp(0.2, 8.0);
            self.set_zoom(ZoomMode::Custom, zoom);
        }
        if fit {
            self.set_zoom(ZoomMode::FitPage, self.pane.zoom);
        }
        let linked = theme::navigation(
            ui,
            tr("Vincular", "Link views"),
            self.follow_page || self.follow_zoom || self.follow_position,
            p,
        );
        let mut popup = egui::Popup::menu(&linked)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        if self.sync_preview {
            popup = popup.open(true);
        }
        popup.show(|ui| self.options_menu(ui, primary, english, p));
    }
}
