use super::*;

impl YamantakaApp {
    pub(super) fn tag_editor(&mut self, ui: &mut egui::Ui, tag: &Tag, busy: bool, p: Palette) {
        if !self
            .library_tags
            .editing
            .as_ref()
            .is_some_and(|editor| editor.tag().id == tag.id)
        {
            return;
        }
        let mut editor = self.library_tags.editing.take().unwrap();
        let mut close = false;
        let response = ui.add_enabled_ui(!busy, |ui| match &mut editor {
            TagEditor::Rename { tag, value } => {
                let label = ui.label(self.tr("Nuevo nombre", "New name"));
                let response = ui
                    .add_sized(
                        [ui.available_width().min(400.0), 36.0],
                        egui::TextEdit::singleline(value)
                            .char_limit(60)
                            .background_color(p.surface)
                            .margin(egui::vec2(10.0, 8.0)),
                    )
                    .labelled_by(label.id);
                theme::record_control_bounds(
                    ui,
                    self.tr("Nuevo nombre", "New name"),
                    response.rect,
                );
                let valid = !value.trim().is_empty() && value.trim().encode_utf16().count() <= 60;
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled_ui(valid, |ui| {
                            theme::control(
                                ui,
                                self.tr("Guardar nombre", "Save name"),
                                None,
                                true,
                                false,
                                p,
                            )
                        })
                        .inner
                        .clicked()
                    {
                        self.change_tag(TagChange::Rename {
                            expected: (&*tag).into(),
                            name: value.clone(),
                        });
                    }
                    close =
                        theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked();
                });
            }
            TagEditor::Color { tag, value } => {
                ui.label(self.tr("Color de la etiqueta", "Tag color"));
                ui.horizontal_wrapped(|ui| {
                    for (key, es, en) in [
                        ("blue", "Azul", "Blue"),
                        ("violet", "Violeta", "Violet"),
                        ("green", "Verde", "Green"),
                        ("orange", "Naranja", "Orange"),
                        ("red", "Rojo", "Red"),
                        ("gray", "Gris", "Gray"),
                    ] {
                        if theme::navigation(ui, self.tr(es, en), *value == key, p).clicked() {
                            *value = key.into();
                        }
                    }
                });
                let label =
                    ui.label(self.tr("Nombre del color o #RRGGBB", "Color name or #RRGGBB"));
                let response = ui
                    .add_sized(
                        [ui.available_width().min(240.0), 36.0],
                        egui::TextEdit::singleline(value)
                            .char_limit(7)
                            .background_color(p.surface)
                            .margin(egui::vec2(10.0, 8.0)),
                    )
                    .labelled_by(label.id);
                theme::record_control_bounds(
                    ui,
                    self.tr("Color personalizado", "Custom color"),
                    response.rect,
                );
                let valid = ["blue", "violet", "purple", "green", "red", "orange", "gray"]
                    .contains(&value.as_str())
                    || value.strip_prefix('#').is_some_and(|hex| {
                        hex.len() == 6 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
                    });
                ui.horizontal_wrapped(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(16.0, 24.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(rect.center(), 5.0, super::view::color(value, p));
                    if ui
                        .add_enabled_ui(valid, |ui| {
                            theme::control(
                                ui,
                                self.tr("Guardar color", "Save color"),
                                None,
                                true,
                                false,
                                p,
                            )
                        })
                        .inner
                        .clicked()
                    {
                        self.change_tag(TagChange::Color {
                            expected: (&*tag).into(),
                            color: value.clone(),
                        });
                    }
                    close =
                        theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked();
                });
            }
        });
        if self.demo
            && self.capture.is_some()
            && self.library_tags.preview.is_none()
            && ui.available_width() < 700.0
        {
            ui.scroll_to_rect(response.response.rect, Some(egui::Align::Max));
        }
        if !close {
            self.library_tags.editing = Some(editor);
        }
    }
}
