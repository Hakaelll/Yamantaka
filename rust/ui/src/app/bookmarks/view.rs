use super::*;

impl BookmarkState {
    pub(in crate::app) fn show(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        current_page: u32,
        page_count: usize,
        english: bool,
        p: Palette,
    ) -> Option<u32> {
        let tr = |es, en| if english { en } else { es };
        let mut jump = None;
        let busy = self.pending.is_some();
        let offset = self.page.as_ref().map_or(0, |page| page.offset);
        let add = format!(
            "{} {}",
            tr("Guardar página", "Bookmark page"),
            u64::from(current_page) + 1
        );
        if ui
            .add_enabled_ui(!busy && (current_page as usize) < page_count, |ui| {
                theme::control(ui, &add, Some(theme::Icon::Add), false, true, p)
            })
            .inner
            .clicked()
        {
            self.command(
                runtime,
                offset,
                Some(BookmarkChange::Ensure { page: current_page }),
            );
        }
        if busy {
            ui.label(
                RichText::new(tr("Actualizando marcadores…", "Updating bookmarks…"))
                    .color(p.secondary),
            );
        } else if self.saved {
            ui.label(RichText::new(tr("Cambios guardados.", "Changes saved.")).color(p.secondary));
        }
        if let Some(error) = self.error.clone() {
            ui.label(tr(
                "No se pudieron actualizar los marcadores.",
                "Bookmarks could not be updated.",
            ));
            ui.collapsing(tr("Detalles", "Details"), |ui| {
                ui.label(error);
            });
            if !busy && theme::navigation(ui, tr("Volver a cargar", "Reload"), false, p).clicked() {
                self.command(runtime, offset, None);
            }
        }
        if let Some(editing) = self.editing.clone() {
            ui.add_space(16.0);
            let label = ui.label(format!(
                "{} {}",
                tr("Etiqueta · página", "Label · page"),
                u64::from(editing.page_index) + 1
            ));
            ui.add_enabled(
                !busy,
                egui::TextEdit::singleline(&mut self.label)
                    .desired_width(f32::INFINITY)
                    .char_limit(4096),
            )
            .labelled_by(label.id);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled_ui(!busy, |ui| {
                        theme::navigation(ui, tr("Guardar etiqueta", "Save label"), false, p)
                    })
                    .inner
                    .clicked()
                {
                    self.command(
                        runtime,
                        offset,
                        Some(BookmarkChange::Rename {
                            id: editing.id,
                            label: Some(self.label.clone()),
                        }),
                    );
                }
                if !busy && theme::navigation(ui, tr("Cancelar", "Cancel"), false, p).clicked() {
                    self.editing = None;
                    self.label.clear();
                }
            });
        }
        ui.add_space(20.0);
        let page = self.current().cloned()?;
        if page.total == 0 {
            ui.label(
                RichText::new(tr(
                    "Aún no tienes marcadores en este documento.",
                    "No bookmarks in this document yet.",
                ))
                .color(p.secondary),
            );
        }
        for bookmark in page.entries {
            ui.push_id(&bookmark.id, |ui| {
                ui.separator();
                ui.add_space(8.0);
                let valid = (bookmark.page_index as usize) < page_count;
                let title = bookmark
                    .label
                    .as_deref()
                    .unwrap_or_else(|| tr("Pasaje guardado", "Saved passage"));
                ui.label(RichText::new(title).size(15.0));
                ui.horizontal_wrapped(|ui| {
                    let caption = format!(
                        "{} {}",
                        tr("Página", "Page"),
                        u64::from(bookmark.page_index) + 1
                    );
                    if ui
                        .add_enabled_ui(valid, |ui| {
                            theme::navigation(ui, &caption, current_page == bookmark.page_index, p)
                        })
                        .inner
                        .clicked()
                    {
                        jump = Some(bookmark.page_index);
                    }
                    if ui
                        .add_enabled_ui(!busy, |ui| {
                            theme::navigation(ui, tr("Editar", "Edit"), false, p)
                        })
                        .inner
                        .clicked()
                    {
                        self.label = bookmark.label.clone().unwrap_or_default();
                        self.editing = Some(bookmark.clone());
                    }
                    if ui
                        .add_enabled_ui(!busy, |ui| {
                            theme::navigation(ui, tr("Eliminar", "Remove"), false, p)
                        })
                        .inner
                        .clicked()
                    {
                        self.command(
                            runtime,
                            offset,
                            Some(BookmarkChange::Remove {
                                id: bookmark.id.clone(),
                            }),
                        );
                    }
                });
                if !valid {
                    ui.label(
                        RichText::new(tr(
                            "Esta página ya no está en el PDF actual.",
                            "This page is absent from the current PDF.",
                        ))
                        .color(p.secondary),
                    );
                }
                ui.add_space(10.0);
            });
        }
        if page.total > BOOKMARK_PAGE_SIZE as u64 {
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled_ui(!busy && page.offset > 0, |ui| {
                        theme::icon_button(
                            ui,
                            tr("Marcadores anteriores", "Previous bookmarks"),
                            theme::Icon::Left,
                            p,
                        )
                    })
                    .inner
                    .clicked()
                {
                    self.command(
                        runtime,
                        page.offset.saturating_sub(BOOKMARK_PAGE_SIZE),
                        None,
                    );
                }
                ui.label(format!(
                    "{} / {}",
                    page.offset / BOOKMARK_PAGE_SIZE + 1,
                    page.total.div_ceil(BOOKMARK_PAGE_SIZE as u64)
                ));
                if ui
                    .add_enabled_ui(
                        !busy && (page.offset as u64 + BOOKMARK_PAGE_SIZE as u64) < page.total,
                        |ui| {
                            theme::icon_button(
                                ui,
                                tr("Marcadores siguientes", "Next bookmarks"),
                                theme::Icon::Right,
                                p,
                            )
                        },
                    )
                    .inner
                    .clicked()
                {
                    self.command(runtime, page.offset + BOOKMARK_PAGE_SIZE, None);
                }
            });
        }
        jump
    }
}
