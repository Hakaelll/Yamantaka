use super::*;

impl YamantakaApp {
    pub(super) fn file_status_label(
        ui: &mut egui::Ui,
        message: &str,
        p: Palette,
    ) -> egui::Response {
        let width = ui
            .available_width()
            .min((ui.ctx().content_rect().width() - 64.0).max(1.0));
        ui.add_sized(
            [width, 0.0],
            egui::Label::new(RichText::new(message).color(p.secondary)).wrap(),
        )
    }

    pub(super) fn details(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let document = self
            .documents
            .iter()
            .find(|doc| Some(doc.id.as_str()) == self.selected.as_deref())
            .cloned();
        egui::ScrollArea::vertical()
            .id_salt("inspection-content")
            .animated(!self.reduced_motion)
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(24, 12))
                    .show(ui, |ui| {
                        let Some(document) = document else {
                            ui.label(self.tr(
                                "Selecciona un libro para ver sus detalles.",
                                "Select a book to inspect its details.",
                            ));
                            return;
                        };
                        let wide = ui.available_width() > 900.0;
                        let title_width = if wide {
                            (ui.available_width() * 0.43).min(560.0)
                        } else {
                            ui.available_width()
                        };
                        ui.horizontal_wrapped(|ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(title_width, 60.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(title_width);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(document.display_title()).font(
                                                egui::FontId::new(
                                                    22.0,
                                                    egui::FontFamily::Name("Editorial".into()),
                                                ),
                                            ),
                                        )
                                        .truncate(),
                                    )
                                    .on_hover_text(document.display_title());
                                    ui.label(
                                        RichText::new(document.author.as_deref().unwrap_or(
                                            self.tr(
                                                "Autor sin especificar",
                                                "Author not specified",
                                            ),
                                        ))
                                        .color(p.secondary)
                                        .size(13.0),
                                    );
                                },
                            );
                            ui.allocate_ui_with_layout(
                                egui::vec2(180.0, 60.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.label(format!(
                                        "{:.0}% {}",
                                        document.reading_progress.clamp(0.0, 1.0) * 100.0,
                                        self.tr("leído", "read")
                                    ));
                                    ui.label(
                                        RichText::new(format!(
                                            "{} {} · {} {}",
                                            document.page_count,
                                            self.tr("páginas", "pages"),
                                            self.tr("Página", "Page"),
                                            document
                                                .last_page_index
                                                .saturating_add(1)
                                                .min(document.page_count)
                                        ))
                                        .color(p.secondary)
                                        .size(13.0),
                                    );
                                },
                            );
                            if wide {
                                ui.add_space((ui.available_size_before_wrap().x - 350.0).max(0.0));
                            }
                            ui.horizontal_wrapped(|ui| {
                                if theme::control(
                                    ui,
                                    self.tr("Leer", "Read"),
                                    Some(theme::Icon::Right),
                                    false,
                                    true,
                                    p,
                                )
                                .clicked()
                                {
                                    self.open(document.clone());
                                }
                                if theme::navigation(
                                    ui,
                                    if document.favorite {
                                        self.tr("Quitar favorito", "Unfavorite")
                                    } else {
                                        self.tr("Favorito", "Favorite")
                                    },
                                    document.favorite,
                                    p,
                                )
                                .clicked()
                                {
                                    self.send(Action::Favorite(
                                        document.id.clone(),
                                        !document.favorite,
                                    ));
                                    self.refresh();
                                }
                                if ui
                                    .add_enabled_ui(!document.missing, |ui| {
                                        theme::control(
                                            ui,
                                            self.tr("Elegir portada", "Choose cover"),
                                            None,
                                            false,
                                            false,
                                            p,
                                        )
                                    })
                                    .inner
                                    .clicked()
                                {
                                    self.begin_cover_picker(document.clone());
                                }
                                if theme::navigation(
                                    ui,
                                    self.tr("Título y autor", "Title and author"),
                                    false,
                                    p,
                                )
                                .clicked()
                                {
                                    self.begin_metadata_editor(document.clone());
                                }
                                if theme::navigation(ui, "Explorer", false, p).clicked()
                                    && let Err(error) = yamantaka_windows::reveal(&document.path)
                                {
                                    self.error = Some(error.to_string());
                                }
                                if theme::icon_button(
                                    ui,
                                    self.tr("Cerrar detalles", "Close details"),
                                    theme::Icon::Minus,
                                    p,
                                )
                                .clicked()
                                {
                                    self.relocation.cancel();
                                    self.file_check.cancel();
                                    self.file_comparison.cancel();
                                    self.inspector = false;
                                }
                            });
                        });
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);
                        self.file_check_details(ui, &document, p);
                        self.relocation_details(ui, &document, p);
                        self.file_comparison_details(ui, &document, p);
                        if theme::navigation(
                            ui,
                            self.tr("Organizar en colecciones", "Organize in collections"),
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.show_collections(Some(document.clone()));
                        }
                        self.reading_goal(ui, &document);
                        ui.add_space(8.0);
                        let path = display_path(&document.path);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(self.tr("Archivo original", "Original file"))
                                    .color(p.secondary)
                                    .size(12.0),
                            );
                            let filename = document
                                .path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or(&document.file_name);
                            ui.label(RichText::new(filename).size(13.0));
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&path).color(p.secondary).size(12.0),
                                )
                                .truncate(),
                            )
                            .on_hover_text(&path);
                        });
                    });
            });
    }
}

fn display_path(path: &std::path::Path) -> String {
    let text = path.to_string_lossy();
    if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        text.strip_prefix(r"\\?\").unwrap_or(&text).to_owned()
    }
}
