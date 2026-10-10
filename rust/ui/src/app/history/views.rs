use super::*;

fn reading_date(value: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|date| {
            date.with_timezone(&chrono::Local)
                .format("%Y-%m-%d · %H:%M")
                .to_string()
        })
        .unwrap_or_else(|_| value.to_owned())
}
fn active_time(seconds: u64) -> String {
    format!(
        "{}h {:02}m {:02}s",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}
impl YamantakaApp {
    pub(super) fn history_list(
        &mut self,
        ui: &mut egui::Ui,
        p: Palette,
        page: &ReadingHistoryPage,
    ) {
        egui::ScrollArea::vertical()
            .id_salt("journal-sessions")
            .max_height(480.0)
            .show(ui, |ui| {
                for (index, entry) in page.entries().iter().enumerate() {
                    ui.push_id(&entry.id, |ui| {
                        let selected = index == self.history.cursor;
                        let amount = if self.reduced_motion {
                            f32::from(selected)
                        } else {
                            ui.ctx().animate_bool_with_time(
                                ui.id().with("session-selection"),
                                selected,
                                0.14,
                            )
                        };
                        let fill = egui::Color32::from(egui::lerp(
                            egui::Rgba::from(p.canvas)..=egui::Rgba::from(p.raised),
                            amount,
                        ));
                        egui::Frame::new()
                            .fill(fill)
                            .inner_margin(14)
                            .corner_radius(12)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    RichText::new(reading_date(&entry.started_at))
                                        .color(p.secondary)
                                        .size(12.0),
                                );
                                let response = ui.add(
                                    egui::Button::new(RichText::new(&entry.title).size(17.0))
                                        .wrap()
                                        .frame(false)
                                        .selected(selected),
                                );
                                theme::record_control_bounds(ui, &entry.title, response.rect);
                                if response.clicked() {
                                    self.history.cursor = index;
                                }
                                ui.label(format!(
                                    "{} · {} {}–{}",
                                    active_time(entry.active_seconds),
                                    self.tr("Páginas", "Pages"),
                                    entry.start_page.saturating_add(1),
                                    entry.end_page.saturating_add(1)
                                ));
                            });
                        ui.add_space(6.0);
                    });
                }
            });
    }
    pub(super) fn history_detail(
        &mut self,
        ui: &mut egui::Ui,
        p: Palette,
        page: &ReadingHistoryPage,
    ) {
        let Some(entry) = page.entries().get(self.history.cursor) else {
            return;
        };
        let compact = ui.available_width() < 600.0;
        egui::Frame::new()
            .fill(p.surface)
            .inner_margin(if compact { 12 } else { 20 })
            .corner_radius(16)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.add(
                    egui::Label::new(RichText::new(&entry.title).font(egui::FontId::new(
                        if compact { 18.0 } else { 28.0 },
                        egui::FontFamily::Name("Editorial".into()),
                    )))
                    .wrap(),
                );
                if entry.missing {
                    ui.label(self.tr(
                        "El archivo necesita volver a localizarse en la biblioteca.",
                        "Locate the file again in the library.",
                    ));
                }
                if self.annotations.editing() {
                    ui.label(self.tr(
                        "Termina o cancela la nota del lector antes de abrir otra lectura.",
                        "Finish or cancel the reader note before opening another reading.",
                    ));
                }
                ui.add_enabled_ui(!entry.missing && !self.annotations.editing(), |ui| {
                    if theme::control(
                        ui,
                        &format!(
                            "{} {}",
                            self.tr("Abrir página", "Open page"),
                            entry.end_page.saturating_add(1)
                        ),
                        Some(theme::Icon::Right),
                        false,
                        true,
                        p,
                    )
                    .clicked()
                    {
                        let cancel = self
                            .open_intent
                            .lookup(entry.document.clone(), Some(entry.end_page));
                        self.send(Action::GetDocument(entry.document.clone(), cancel));
                    }
                });
                if compact {
                    ui.add_space(8.0);
                    ui.label(format!(
                        "{} · {}",
                        reading_date(&entry.started_at),
                        active_time(entry.active_seconds)
                    ));
                    egui::CollapsingHeader::new(
                        self.tr("Detalles de la sesión", "Session details"),
                    )
                    .show(ui, |ui| {
                        ui.add_space(16.0);
                        ui.label(self.tr("Tiempo de lectura activo", "Active reading time"));
                        ui.label(
                            RichText::new(active_time(entry.active_seconds))
                                .size(24.0)
                                .color(p.accent),
                        );
                        ui.add_space(16.0);
                        ui.label(format!(
                            "{} {}",
                            self.tr("Inicio", "Started"),
                            reading_date(&entry.started_at)
                        ));
                        ui.label(format!(
                            "{} {}",
                            self.tr("Final", "Finished"),
                            reading_date(&entry.ended_at)
                        ));
                        ui.add_space(16.0);
                    });
                } else {
                    ui.add_space(16.0);
                    ui.label(self.tr("Tiempo de lectura activo", "Active reading time"));
                    ui.label(
                        RichText::new(active_time(entry.active_seconds))
                            .size(24.0)
                            .color(p.accent),
                    );
                    ui.add_space(16.0);
                    ui.label(format!(
                        "{} {}",
                        self.tr("Inicio", "Started"),
                        reading_date(&entry.started_at)
                    ));
                    ui.label(format!(
                        "{} {}",
                        self.tr("Final", "Finished"),
                        reading_date(&entry.ended_at)
                    ));
                    ui.add_space(16.0);
                }
            });
    }
}
