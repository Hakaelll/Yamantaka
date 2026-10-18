use super::*;

#[derive(Default)]
pub(super) struct Actions {
    pub(super) close: bool,
    pub(super) save: Option<MetadataRequest>,
    pub(super) reanalyze: Option<(Box<MetadataReview>, Cancellation)>,
    pub(super) source: Option<MetadataField>,
    pub(super) retry_preview: bool,
}

impl YamantakaApp {
    pub(in crate::app) fn metadata_view(&mut self, ui: &mut egui::Ui) {
        self.prepare_metadata_reference();
        let Some(mut editor) = self.metadata.take() else {
            return;
        };
        let p = Palette::new(self.dark);
        let compact = ui.ctx().content_rect().height() < 600.0;
        let writing = editor.pending.as_ref().is_some_and(|(_, writing)| *writing);
        let mut actions = Actions::default();
        egui::ScrollArea::vertical()
            .id_salt("metadata-review")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(
                        if compact { 14 } else { 42 },
                        if compact { 10 } else { 26 },
                    ))
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width().min(1280.0));
                        ui.horizontal_top(|ui| {
                            actions.close = ui
                                .add_enabled_ui(!writing, |ui| {
                                    theme::icon_button(
                                        ui,
                                        self.tr("Volver a la biblioteca", "Back to library"),
                                        theme::Icon::Left,
                                        p,
                                    )
                                })
                                .inner
                                .clicked();
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(self.tr("Título y autor", "Title and author"))
                                        .font(egui::FontId::new(
                                            if compact { 24.0 } else { 32.0 },
                                            egui::FontFamily::Name("Editorial".into()),
                                        )),
                                );
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(&editor.document.file_name)
                                            .size(13.0)
                                            .color(p.secondary),
                                    )
                                    .truncate(),
                                );
                            });
                        });
                        ui.add_space(if compact { 14.0 } else { 24.0 });
                        if ui.available_width() >= 1040.0 {
                            let left = (ui.available_width() - 344.0).max(340.0);
                            ui.horizontal_top(|ui| {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(left, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        self.metadata_form(
                                            ui,
                                            &mut editor,
                                            &mut actions,
                                            p,
                                            compact,
                                        );
                                    },
                                );
                                ui.add_space(24.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(312.0, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        self.metadata_reference_panel(
                                            ui,
                                            &mut editor,
                                            &mut actions,
                                            p,
                                        );
                                    },
                                );
                            });
                        } else {
                            self.metadata_form(ui, &mut editor, &mut actions, p, compact);
                            ui.add_space(22.0);
                            self.metadata_reference_panel(ui, &mut editor, &mut actions, p);
                        }
                    });
            });
        actions.close |= !writing && ui.input(|input| input.key_pressed(egui::Key::Escape));
        if actions.close {
            drop(editor);
            self.runtime.wake();
            return;
        }
        if actions.retry_preview {
            editor.reference.retry();
        }
        self.metadata = Some(editor);
        if let Some(request) = actions.save {
            self.send_metadata(request);
        }
        if let Some((expected, cancel)) = actions.reanalyze {
            self.send_metadata_reanalysis(expected, cancel);
        }
        if let Some(field) = actions.source {
            self.request_metadata_source(field);
        }
        self.prepare_metadata_reference();
    }
}
