use super::*;

impl YamantakaApp {
    pub(super) fn reader_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        info: &PdfInfo,
        p: Palette,
    ) {
        egui::Frame::new()
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(18, 8))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if theme::control(
                        ui,
                        self.tr("Biblioteca", "Library"),
                        Some(theme::Icon::Left),
                        false,
                        false,
                        p,
                    )
                    .clicked()
                    {
                        self.show_surface(Surface::Library);
                        self.refresh();
                    }
                    if ui.available_width() > 880.0 {
                        ui.add_space(12.0);
                        ui.add_sized(
                            [ui.available_width().min(320.0), 40.0],
                            egui::Label::new(RichText::new(document.display_title()).font(
                                egui::FontId::new(22.0, egui::FontFamily::Name("Editorial".into())),
                            ))
                            .truncate(),
                        )
                        .on_hover_text(document.display_title());
                    }
                    ui.add_space(8.0);
                    egui::Frame::new()
                        .fill(p.raised)
                        .corner_radius(8.0)
                        .inner_margin(egui::Margin::symmetric(3, 2))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
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
                                let page_label = ui.label(
                                    RichText::new(self.tr("Página", "Page"))
                                        .size(13.0)
                                        .color(p.secondary),
                                );
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut shown)
                                            .range(1..=info.pages.len().max(1) as u32),
                                    )
                                    .labelled_by(page_label.id)
                                    .changed()
                                {
                                    self.navigate(shown.saturating_sub(1));
                                }
                                ui.label(
                                    RichText::new(format!("/ {}", info.pages.len()))
                                        .size(13.0)
                                        .color(p.secondary),
                                );
                                if ui
                                    .add_enabled_ui(
                                        (self.page as usize) + 1 < info.pages.len(),
                                        |ui| {
                                            theme::icon_button(
                                                ui,
                                                self.tr("Página siguiente", "Next page"),
                                                theme::Icon::Right,
                                                p,
                                            )
                                        },
                                    )
                                    .inner
                                    .clicked()
                                {
                                    self.step_reader(true);
                                }
                            });
                        });
                    ui.add_space(8.0);
                    egui::Frame::new()
                        .fill(p.raised)
                        .corner_radius(8.0)
                        .inner_margin(egui::Margin::symmetric(3, 2))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if theme::icon_button(
                                    ui,
                                    self.tr("Alejar", "Zoom out"),
                                    theme::Icon::Minus,
                                    p,
                                )
                                .clicked()
                                {
                                    self.set_reader_zoom(self.reader_scale / 1.2);
                                }
                                ui.label(
                                    RichText::new(self.reader_zoom_label())
                                        .size(13.0)
                                        .color(p.secondary),
                                );
                                if theme::icon_button(
                                    ui,
                                    self.tr("Acercar", "Zoom in"),
                                    theme::Icon::Plus,
                                    p,
                                )
                                .clicked()
                                {
                                    self.set_reader_zoom(self.reader_scale * 1.2);
                                }
                                if theme::icon_button(
                                    ui,
                                    self.tr("Ajustar página", "Fit page"),
                                    theme::Icon::Fit,
                                    p,
                                )
                                .clicked()
                                {
                                    self.set_reader_fit(ZoomMode::FitPage);
                                }
                            });
                        });
                    ui.add_space(8.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::control(
                            ui,
                            self.tr("Navegador", "Navigator"),
                            Some(theme::Icon::Panel),
                            self.panel,
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.toggle_reader_panel();
                        }
                        let response = theme::icon_button(
                            ui,
                            self.tr("Opciones de lectura", "Reading options"),
                            theme::Icon::Settings,
                            p,
                        );
                        self.reader_options_popup(&response)
                            .show(|ui| self.reader_options_menu(ui, p));
                    });
                });
            });
    }
}
