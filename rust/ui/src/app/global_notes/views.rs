use super::*;

impl YamantakaApp {
    pub(super) fn global_notes_list(
        &mut self,
        ui: &mut egui::Ui,
        p: Palette,
        page: &AnnotationPage,
    ) {
        egui::ScrollArea::vertical()
            .id_salt("global-notes-list")
            .max_height(480.0)
            .show(ui, |ui| {
                for (i, row) in page.entries().iter().enumerate() {
                    let data = row.data();
                    ui.push_id(&data.id, |ui| {
                        let current = self.global_notes.cursor == i;
                        let reduced = ui.ctx().data(|d| {
                            d.get_temp::<bool>(egui::Id::new("yamantaka-reduced-motion"))
                                .unwrap_or(false)
                        });
                        let emphasis = if reduced {
                            f32::from(current)
                        } else {
                            ui.ctx().animate_bool_with_time(
                                ui.id().with("current-note"),
                                current,
                                0.14,
                            )
                        };
                        let fill = egui::lerp(
                            egui::Rgba::from(p.canvas)..=egui::Rgba::from(p.raised),
                            emphasis,
                        );
                        egui::Frame::new()
                            .fill(egui::Color32::from(fill))
                            .inner_margin(12)
                            .corner_radius(12)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                let title = data.document_title.as_deref().unwrap_or(
                                    self.tr("Documento no disponible", "Unavailable document"),
                                );
                                let short: String = title.chars().take(65).collect();
                                ui.horizontal_wrapped(|ui| {
                                    if theme::navigation(ui, &short, current, p)
                                        .on_hover_text(title)
                                        .clicked()
                                    {
                                        self.global_notes.cursor = i;
                                    }
                                    ui.add_enabled_ui(self.global_notes.pending.is_none(), |ui| {
                                        let label = if self.english {
                                            "Select"
                                        } else {
                                            "Seleccionar"
                                        };
                                        ui.checkbox(&mut self.global_notes.selected[i], label)
                                            .on_hover_text(title);
                                    });
                                });
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(
                                            data.values
                                                .selected_text
                                                .as_deref()
                                                .or(data.values.note.as_deref())
                                                .unwrap_or(""),
                                        )
                                        .color(p.ink),
                                    )
                                    .truncate(),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} {} · {} · {}",
                                        self.tr("Página", "Page"),
                                        u64::from(data.values.page_index) + 1,
                                        kind_label(data.values.kind, self.english),
                                        review_label(data.values.review_state, self.english)
                                    ))
                                    .size(12.0)
                                    .color(p.secondary),
                                );
                            });
                        ui.add_space(6.0);
                    });
                }
            });
    }
    pub(super) fn global_notes_detail(
        &mut self,
        ui: &mut egui::Ui,
        p: Palette,
        page: &AnnotationPage,
    ) {
        let Some(row) = page.entries().get(self.global_notes.cursor) else {
            return;
        };
        let data = row.data();
        egui::Frame::new()
            .fill(p.surface)
            .inner_margin(24)
            .corner_radius(12)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(format!(
                        "{} {} · {}",
                        self.tr("Página", "Page"),
                        u64::from(data.values.page_index) + 1,
                        kind_label(data.values.kind, self.english)
                    ))
                    .size(13.0)
                    .color(p.secondary),
                );
                ui.add_space(12.0);
                theme::display(
                    ui,
                    data.document_title
                        .as_deref()
                        .unwrap_or(self.tr("Documento no disponible", "Unavailable document")),
                    24.0,
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(review_label(data.values.review_state, self.english))
                        .color(p.accent),
                );
                ui.add_space(24.0);
                if let Some(text) = &data.values.selected_text {
                    ui.label(
                        RichText::new(text)
                            .font(egui::FontId::new(
                                24.0,
                                egui::FontFamily::Name("Editorial".into()),
                            ))
                            .color(p.ink),
                    );
                    ui.add_space(20.0);
                }
                if let Some(note) = &data.values.note {
                    ui.label(RichText::new(note).color(p.ink));
                }
                if !data.tags.is_empty() {
                    ui.add_space(16.0);
                    ui.label(RichText::new(data.tags.join(" · ")).color(p.secondary));
                }
                ui.add_space(24.0);
                if ui
                    .add_enabled_ui(!self.annotations.editing(), |ui| {
                        theme::control(
                            ui,
                            self.tr("Abrir en el documento", "Open in document"),
                            Some(theme::Icon::Right),
                            false,
                            true,
                            p,
                        )
                    })
                    .inner
                    .clicked()
                {
                    self.activate_global_note(
                        data.values.document_id.clone(),
                        data.values.page_index,
                    );
                }
            });
    }
}
