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

    pub(super) fn close_book_details(&mut self) {
        self.relocation.cancel();
        self.file_check.cancel();
        self.file_comparison.cancel();
        self.inspector = false;
        self.library_selection.focus = self.selected.clone();
    }

    pub(super) fn details(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let document = self
            .documents
            .iter()
            .find(|doc| Some(doc.id.as_str()) == self.selected.as_deref())
            .cloned();
        let width = ui.available_width();
        let compact = width < 472.0;
        let inset = if width >= 760.0 { 42 } else { 16 };
        egui::ScrollArea::vertical()
            .id_salt(("inspection-content", self.selected.as_deref()))
            .animated(!self.reduced_motion)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin {
                        top: if compact { 4 } else { 12 },
                        ..egui::Margin::symmetric(inset, 12)
                    })
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width().min(1100.0));
                        if compact {
                            ui.spacing_mut().item_spacing.y = 4.0;
                        }
                        if theme::control(
                            ui,
                            self.tr("Volver a la biblioteca", "Back to library"),
                            Some(theme::Icon::Left),
                            false,
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.close_book_details();
                            return;
                        }
                        let Some(document) = document else {
                            ui.label(self.tr(
                                "Selecciona un libro para ver sus detalles.",
                                "Select a book to inspect its details.",
                            ));
                            return;
                        };
                        ui.add_space(if compact { 2.0 } else { 12.0 });
                        self.book_details_header(ui, &document, compact, p);
                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(16.0);
                        if ui.available_width() >= 760.0 {
                            ui.columns(2, |columns| {
                                self.book_metadata(&mut columns[0], &document, p);
                                self.book_file_details(&mut columns[1], &document, p);
                            });
                        } else {
                            self.book_metadata(ui, &document, p);
                            ui.add_space(16.0);
                            self.book_file_details(ui, &document, p);
                        }
                        ui.add_space(20.0);
                    });
            });
    }

    fn book_details_header(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        compact: bool,
        p: Palette,
    ) {
        let width = ui.available_width();
        let cover_width = if compact {
            (width * 0.28).clamp(68.0, 110.0)
        } else {
            (width * 0.31).min(184.0)
        };
        let read = if document.last_page_index > 0 || document.reading_progress > 0.0 {
            self.tr("Continuar", "Continue")
        } else {
            self.tr("Leer", "Read")
        };
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = if compact { 10.0 } else { 24.0 };
            if book::miniature(ui, document, cover_width, false, self.reduced_motion, p)
                .on_hover_text(read)
                .clicked()
            {
                self.open(document.clone());
            }
            ui.vertical(|ui| {
                if !compact {
                    ui.add_space(16.0);
                }
                ui.add(
                    egui::Label::new(RichText::new(document.display_title()).font(
                        egui::FontId::new(
                            if compact { 18.0 } else { 38.0 },
                            egui::FontFamily::Name("Editorial".into()),
                        ),
                    ))
                    .wrap(),
                );
                let author = document
                    .author
                    .as_deref()
                    .filter(|author| !author.trim().is_empty())
                    .unwrap_or(self.tr("Autor sin especificar", "Author not specified"));
                ui.add(
                    egui::Label::new(
                        RichText::new(author)
                            .size(if compact { 13.0 } else { 15.0 })
                            .color(p.secondary),
                    )
                    .wrap(),
                );
                if !compact {
                    ui.add_space(16.0);
                }
                ui.horizontal_wrapped(|ui| {
                    if theme::control(ui, read, Some(theme::Icon::Right), false, true, p).clicked()
                    {
                        self.open(document.clone());
                    }
                    if ui
                        .add_enabled_ui(!self.library_selection.writing(), |ui| {
                            theme::icon_toggle(
                                ui,
                                if document.favorite {
                                    self.tr("Quitar favorito", "Unfavorite")
                                } else {
                                    self.tr("Marcar como favorito", "Mark as favorite")
                                },
                                theme::Icon::Star,
                                document.favorite,
                                p,
                            )
                        })
                        .inner
                        .clicked()
                    {
                        self.send(Action::Favorite(document.id.clone(), !document.favorite));
                        self.refresh();
                    }
                });
            });
        });
    }

    fn book_metadata(&mut self, ui: &mut egui::Ui, document: &Document, p: Palette) {
        details_heading(ui, self.tr("Datos del libro", "Book details"));
        let page = document
            .last_page_index
            .saturating_add(1)
            .min(document.page_count);
        ui.label(format!(
            "{} {} · {} {page} · {:.0}% {}",
            document.page_count,
            self.tr("páginas", "pages"),
            self.tr("Página", "Page"),
            document.reading_progress.clamp(0.0, 1.0) * 100.0,
            self.tr("leído", "read"),
        ));
        if theme::navigation(
            ui,
            self.tr("Editar título y autor", "Edit title and author"),
            false,
            p,
        )
        .clicked()
        {
            self.begin_metadata_editor(document.clone());
        }
        self.reading_goal(ui, document);

        ui.add_space(18.0);
        details_heading(ui, self.tr("Organización", "Organization"));
        ui.horizontal_wrapped(|ui| {
            if theme::navigation(ui, self.tr("Colecciones", "Collections"), false, p).clicked() {
                self.show_collections(Some(document.clone()));
            }
            if theme::navigation(ui, self.tr("Etiquetas", "Tags"), false, p).clicked() {
                self.show_tags(Some(document.clone()));
            }
        });
    }

    fn book_file_details(&mut self, ui: &mut egui::Ui, document: &Document, p: Palette) {
        details_heading(ui, self.tr("Archivo y portada", "File and cover"));
        let path = display_path(&document.path);
        let filename = document
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&document.file_name);
        ui.add(egui::Label::new(RichText::new(filename).strong()).wrap());
        ui.add(egui::Label::new(RichText::new(&path).color(p.secondary).size(13.0)).truncate())
            .on_hover_text(&path);
        if document.missing {
            ui.label(self.tr("PDF no disponible", "PDF unavailable"));
        }
        let response = theme::navigation(ui, self.tr("Más acciones", "More actions"), false, p);
        let mut popup = egui::Popup::menu(&response)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        if self.inspector_actions_preview {
            response.scroll_to_me(Some(egui::Align::Center));
            popup = popup.open(true);
        }
        popup.show(|ui| {
            ui.set_width((ui.ctx().content_rect().width() - 32.0).clamp(120.0, 260.0));
            egui::ScrollArea::vertical()
                .max_height((ui.ctx().content_rect().height() - 48.0).max(60.0))
                .show(ui, |ui| self.inspector_actions(ui, document, p));
        });
        self.file_check_details(ui, document, p);
        self.relocation_details(ui, document, p);
        self.file_comparison_details(ui, document, p);

        ui.add_space(18.0);
        details_heading(
            ui,
            self.tr("Relaciones y revisiones", "Connections and revisions"),
        );
        if theme::navigation(ui, self.tr("Ver conexiones", "View connections"), false, p).clicked()
        {
            self.show_relations(document.clone());
        }
    }

    fn inspector_actions(&mut self, ui: &mut egui::Ui, document: &Document, p: Palette) {
        if ui
            .add_enabled_ui(!document.missing, |ui| {
                theme::navigation(ui, self.tr("Elegir portada", "Choose cover"), false, p)
            })
            .inner
            .clicked()
        {
            self.begin_cover_picker(document.clone());
            ui.close();
        }
        self.cover_rebuild_control(ui, document, p);
        if theme::navigation(ui, "Explorer", false, p).clicked()
            && let Err(error) = yamantaka_windows::reveal(&document.path)
        {
            self.error = Some(error.to_string());
        }
    }
}

fn details_heading(ui: &mut egui::Ui, label: &str) {
    ui.label(RichText::new(label).size(18.0).strong());
    ui.add_space(4.0);
}

fn display_path(path: &std::path::Path) -> String {
    let text = path.to_string_lossy();
    if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        text.strip_prefix(r"\\?\").unwrap_or(&text).to_owned()
    }
}
