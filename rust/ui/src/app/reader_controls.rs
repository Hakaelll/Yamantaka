use super::*;

impl YamantakaApp {
    pub(super) fn compact_reader_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        page_count: u32,
        p: Palette,
    ) {
        egui::Frame::new()
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(18, 8))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.horizontal(|ui| {
                    if theme::icon_button(
                        ui,
                        if self.metadata_return_available() {
                            self.tr("Volver a los metadatos", "Back to metadata")
                        } else {
                            self.tr("Biblioteca", "Library")
                        },
                        theme::Icon::Book,
                        p,
                    )
                    .clicked()
                    {
                        self.show_surface(Surface::Library);
                        self.refresh();
                    }
                    if ui
                        .add_enabled_ui(self.page > 0, |ui| {
                            theme::icon_button(
                                ui,
                                self.tr("Página anterior", "Previous page"),
                                theme::Icon::Left,
                                p,
                            )
                        })
                        .inner
                        .clicked()
                    {
                        self.step_reader(false);
                    }
                    let mut shown = self.page + 1;
                    let response = ui.add_sized(
                        [64.0, 40.0],
                        egui::DragValue::new(&mut shown)
                            .range(1..=page_count)
                            .suffix(format!(" / {page_count}")),
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::DragValue,
                            ui.is_enabled(),
                            self.tr("Página", "Page"),
                        )
                    });
                    if response.changed() {
                        self.navigate(shown.saturating_sub(1));
                    }
                    if ui
                        .add_enabled_ui(self.page + 1 < page_count, |ui| {
                            theme::icon_button(
                                ui,
                                self.tr("Página siguiente", "Next page"),
                                theme::Icon::Right,
                                p,
                            )
                        })
                        .inner
                        .clicked()
                    {
                        self.step_reader(true);
                    }
                    let options = theme::icon_button(
                        ui,
                        self.tr("Opciones de lectura", "Reading options"),
                        theme::Icon::Settings,
                        p,
                    );
                    self.reader_options_popup(&options)
                        .show(|ui| self.reader_options_menu(ui, p));

                    if theme::icon_toggle(
                        ui,
                        self.tr("Navegador de lectura", "Reading navigator"),
                        theme::Icon::Panel,
                        self.panel,
                        p,
                    )
                    .clicked()
                    {
                        self.toggle_reader_panel();
                    }
                });
            });
    }
    pub(super) fn set_reader_zoom(&mut self, zoom: f32) {
        if let Some(mut preferences) = self.reader_preferences.clone() {
            preferences.zoom_mode = ZoomMode::Custom;
            preferences.zoom_level = zoom.clamp(0.2, 8.0) as f64;
            self.apply_reader_preferences(preferences);
            return;
        }
        self.zoom = zoom.clamp(0.2, 8.0);
        self.generation += 1;
        self.reset_render_request();
    }
}
