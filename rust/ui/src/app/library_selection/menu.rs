use super::*;
impl YamantakaApp {
    pub(in crate::app) fn library_selection_control(&mut self, ui: &mut egui::Ui, p: Palette) {
        let count = self.library_selection.model.ids().len();
        let label = if count > 0 {
            format!("{} ({count})", self.tr("Selección", "Selection"))
        } else if self.library_selection.notice.is_some() {
            self.tr("Resultado", "Result").to_owned()
        } else {
            self.tr("Seleccionar", "Select").to_owned()
        };
        let compact = ui.ctx().content_rect().width() < 420.0;
        let active = self.library_selection.model.mode || count > 0;
        let response=if compact {theme::icon_toggle(ui,&label,theme::Icon::Check,active,p)} else {theme::navigation(ui,&label,active,p)}
            .on_hover_text(self.tr("Ctrl + clic para alternar; Mayús + clic para un intervalo. La selección pertenece a esta página.","Ctrl-click to toggle; Shift-click for a range. Selection belongs to this page."));
        let mut popup = egui::Popup::menu(&response);
        if self.library_selection.preview_menu {
            popup = popup.open(true);
        }
        popup.show(|ui| {
            ui.set_max_width((ui.ctx().content_rect().width() - 24.0).max(160.0));
            egui::ScrollArea::vertical()
                .id_salt("library-selection-menu")
                .max_height((ui.ctx().content_rect().height() - 24.0).max(40.0))
                .show(ui, |ui| {
                    if compact {
                        ui.label(&label);
                    }
                    let busy = self.library_selection.pending.is_some();
                    if busy {
                        if let Some(progress) = self.cover_batch_progress() {
                            ui.label(progress);
                        } else {
                            ui.label(self.tr("Guardando selección…", "Saving selection…"));
                        }
                        if theme::navigation(
                            ui,
                            self.tr("Cancelar pendientes", "Cancel remaining"),
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.library_selection.cancel();
                            self.runtime.wake();
                        }
                    } else if count > 0 {
                        for (value, es, en) in [
                            (true, "Añadir a favoritos", "Add to favorites"),
                            (false, "Quitar de favoritos", "Remove from favorites"),
                        ] {
                            if theme::navigation(ui, self.tr(es, en), false, p).clicked() {
                                self.submit_library_bulk(
                                    BulkAction::Favorite(value),
                                    self.library_selection.model.ids().to_vec(),
                                );
                                ui.close();
                            }
                        }
                    }
                    if count > 0 && !busy {
                        for (kind, es, en) in [
                            (
                                bulk_picker::TargetKind::Tag,
                                "Asignar etiqueta",
                                "Assign tag",
                            ),
                            (
                                bulk_picker::TargetKind::Collection(true),
                                "Añadir a colección",
                                "Add to collection",
                            ),
                            (
                                bulk_picker::TargetKind::Collection(false),
                                "Quitar de colección",
                                "Remove from collection",
                            ),
                        ] {
                            if theme::navigation(ui, self.tr(es, en), false, p).clicked() {
                                self.open_bulk_picker(kind);
                                ui.close();
                            }
                        }
                        if theme::navigation(
                            ui,
                            self.tr("Regenerar portadas…", "Regenerate covers…"),
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.open_cover_batch_review();
                            ui.close();
                        }
                        if theme::navigation(ui, self.tr("Archivar", "Archive"), false, p)
                            .on_hover_text(self.tr(
                                "Añade los libros a la colección Archivado.",
                                "Adds the books to the Archivado collection.",
                            ))
                            .clicked()
                        {
                            self.submit_library_bulk(
                                BulkAction::Archive,
                                self.library_selection.model.ids().to_vec(),
                            );
                            ui.close();
                        }
                    }
                    if let Some(notice) = &self.library_selection.notice {
                        ui.label(notice);
                    }
                    if count > 0 || busy || self.library_selection.notice.is_some() {
                        ui.separator();
                    }
                    ui.add_enabled_ui(!busy, |ui| {
                        let label = self.tr("Modo de selección", "Selection mode");
                        let mut mode = self.library_selection.model.mode;
                        if ui.checkbox(&mut mode, label).changed() {
                            if !mode {
                                self.library_selection.model.clear();
                            } else {
                                self.library_selection.model.mode = true;
                            }
                        }
                        if theme::navigation(
                            ui,
                            self.tr("Seleccionar esta página", "Select this page"),
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.library_selection.model.all(&self.library_page_ids());
                            ui.close();
                        }
                        if theme::navigation(
                            ui,
                            self.tr("Limpiar selección", "Clear selection"),
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.library_selection.model.clear();
                            self.library_selection.notice = None;
                            ui.close();
                        }
                    });
                });
        });
    }
}
