use super::*;
use yamantaka_domain::bookmarks::{BOOKMARK_PAGE_SIZE, Bookmark, BookmarkChange, BookmarkPage};

#[derive(Default)]
pub(super) struct BookmarkState {
    document: Option<String>,
    page: Option<BookmarkPage>,
    pending: Option<(Cancellation, bool)>,
    clear_edit: bool,
    saved: bool,
    error: Option<String>,
    editing: Option<Bookmark>,
    label: String,
}

impl BookmarkState {
    pub(super) fn ready(&self) -> bool {
        self.page.is_some() && self.pending.is_none()
    }
    fn bind(&mut self, document: &str) -> bool {
        if self.document.as_deref() == Some(document) {
            return false;
        }
        // Accepted writes belong to their document even when the reader changes.
        // Query results can be cancelled; stale writes are ignored by identity.
        if let Some((cancel, false)) = &self.pending {
            cancel.cancel();
        }
        *self = Self {
            document: Some(document.into()),
            ..Default::default()
        };
        true
    }

    pub(super) fn accept(
        &mut self,
        document: &str,
        cancel: &Cancellation,
        result: Result<BookmarkPage, String>,
    ) {
        if self.document.as_deref() != Some(document)
            || !self
                .pending
                .as_ref()
                .is_some_and(|(pending, _)| pending.same_request(cancel))
        {
            return;
        }
        let writing = self.pending.take().unwrap().1;
        match result {
            Ok(page) => {
                self.page = Some(page);
                self.error = None;
                self.saved = writing;
                if writing && self.clear_edit {
                    self.editing = None;
                    self.label.clear();
                }
            }
            Err(error) => self.error = Some(error),
        }
    }
}

impl YamantakaApp {
    fn bookmark_command(&mut self, offset: usize, change: Option<BookmarkChange>) {
        if self.bookmarks.pending.is_some() {
            return;
        }
        let Some(document) = self.bookmarks.document.clone() else {
            return;
        };
        let cancel = Cancellation::default();
        let writing = change.is_some();
        self.bookmarks.clear_edit = self.bookmarks.editing.as_ref().is_some_and(|editing| {
            matches!(&change, Some(BookmarkChange::Rename { id, .. } | BookmarkChange::Remove { id }) if id == &editing.id)
        });
        let action = match change {
            Some(change) => Action::ChangeBookmark(document, change, offset, cancel.clone()),
            None => Action::Bookmarks(document, offset, cancel.clone()),
        };
        self.bookmarks.error = None;
        self.bookmarks.saved = false;
        match self.runtime.submit(action) {
            Ok(()) => self.bookmarks.pending = Some((cancel, writing)),
            Err(error) => self.bookmarks.error = Some(error.to_string()),
        }
    }

    pub(super) fn reader_panel(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        info: &PdfInfo,
        p: Palette,
    ) {
        if !self.panel {
            return;
        }
        if self.bookmarks.bind(&document.id)
            && (self.reader_panel_tab == outline::ReaderPanelTab::Bookmarks)
        {
            self.bookmark_command(0, None);
        }
        if (self.reader_panel_tab == outline::ReaderPanelTab::Bookmarks)
            && self.bookmarks.page.is_none()
            && self.bookmarks.pending.is_none()
            && self.bookmarks.error.is_none()
        {
            self.bookmark_command(0, None);
        }
        if self.reader_panel_tab == outline::ReaderPanelTab::Text {
            let identity = (document.id.clone(), self.page, self.generation);
            if self.panel_requested.as_ref() != Some(&identity) {
                self.panel_text.clear();
                self.panel_requested = Some(identity);
                self.send(Action::Text(
                    document.id.clone(),
                    self.page,
                    self.cancel.clone(),
                ));
            }
        }
        if self.reader_panel_tab == outline::ReaderPanelTab::Outline {
            self.request_outline(&document.id);
        }
        // Narrow windows use the same scrollable content as a full-width drawer,
        // rather than leaving a sliver of PDF alongside a fixed-width sidebar.
        if ui.ctx().content_rect().width() < 620.0 {
            egui::Frame::new()
                .fill(p.surface)
                .inner_margin(16)
                .show(ui, |ui| {
                    let height = ui.available_height().max(80.0);
                    egui::ScrollArea::vertical()
                        .id_salt("reader-drawer")
                        .max_height(height)
                        .show(ui, |ui| self.reader_panel_content(ui, info, p));
                });
        } else {
            egui::Panel::left("reader-panel")
                .default_size(300.0)
                .size_range(220.0..=500.0)
                .show(ui, |ui| {
                    ui.add_space(14.0);
                    egui::ScrollArea::vertical()
                        .id_salt("reader-navigator")
                        .show(ui, |ui| {
                            self.reader_panel_content(ui, info, p);
                        });
                });
        }
    }

    fn reader_panel_content(&mut self, ui: &mut egui::Ui, info: &PdfInfo, p: Palette) {
        let compact = ui.ctx().content_rect().width() < 620.0;
        ui.horizontal_wrapped(|ui| {
            for (tab, es, en) in [
                (outline::ReaderPanelTab::Outline, "Índice", "Contents"),
                (
                    outline::ReaderPanelTab::Bookmarks,
                    "Marcadores",
                    "Bookmarks",
                ),
                (outline::ReaderPanelTab::Text, "Texto", "Text"),
            ] {
                if theme::navigation(ui, self.tr(es, en), self.reader_panel_tab == tab, p).clicked()
                {
                    self.reader_panel_tab = tab;
                }
            }
            if !compact
                && theme::icon_button(
                    ui,
                    self.tr("Cerrar panel", "Close panel"),
                    theme::Icon::Minus,
                    p,
                )
                .clicked()
            {
                self.panel = false;
            }
        });
        let compact = ui.ctx().content_rect().width() < 620.0;
        ui.add_space(if compact { 8.0 } else { 18.0 });
        if self.reader_panel_tab == outline::ReaderPanelTab::Text {
            ui.heading(self.tr("Texto de la página", "Page text"));
            if theme::navigation(ui, self.tr("Copiar texto", "Copy text"), false, p).clicked() {
                ui.ctx().copy_text(self.panel_text.clone());
            }
            ui.label(&self.panel_text);
            return;
        }
        if self.reader_panel_tab == outline::ReaderPanelTab::Outline {
            self.outline_panel_content(ui, p);
            return;
        }
        if !compact {
            ui.label(
                RichText::new(self.tr("Tu recorrido de lectura", "Your reading trail")).size(22.0),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(self.tr(
                    "Guarda pasajes para volver a ellos.",
                    "Keep passages you want to revisit.",
                ))
                .color(p.secondary),
            );
            ui.add_space(14.0);
        }
        let busy = self.bookmarks.pending.is_some();
        let offset = self.bookmarks.page.as_ref().map_or(0, |page| page.offset);
        let add = format!(
            "{} {}",
            self.tr("Guardar página", "Bookmark page"),
            u64::from(self.page) + 1
        );
        if ui
            .add_enabled_ui(!busy && (self.page as usize) < info.pages.len(), |ui| {
                theme::control(ui, &add, Some(theme::Icon::Add), false, true, p)
            })
            .inner
            .clicked()
        {
            self.bookmark_command(offset, Some(BookmarkChange::Ensure { page: self.page }));
        }
        if busy {
            ui.label(
                RichText::new(self.tr("Actualizando marcadores…", "Updating bookmarks…"))
                    .color(p.secondary),
            );
        } else if self.bookmarks.saved {
            ui.label(
                RichText::new(self.tr("Cambios guardados.", "Changes saved.")).color(p.secondary),
            );
        }
        if let Some(error) = self.bookmarks.error.clone() {
            ui.label(self.tr(
                "No se pudieron actualizar los marcadores.",
                "Bookmarks could not be updated.",
            ));
            ui.collapsing(self.tr("Detalles", "Details"), |ui| {
                ui.label(error);
            });
            if !busy
                && theme::navigation(ui, self.tr("Volver a cargar", "Reload"), false, p).clicked()
            {
                self.bookmark_command(offset, None);
            }
        }
        if let Some(editing) = self.bookmarks.editing.clone() {
            ui.add_space(16.0);
            let label = ui.label(format!(
                "{} {}",
                self.tr("Etiqueta · página", "Label · page"),
                u64::from(editing.page_index) + 1
            ));
            ui.add_enabled(
                !busy,
                egui::TextEdit::singleline(&mut self.bookmarks.label)
                    .desired_width(f32::INFINITY)
                    .char_limit(4096),
            )
            .labelled_by(label.id);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled_ui(!busy, |ui| {
                        theme::navigation(ui, self.tr("Guardar etiqueta", "Save label"), false, p)
                    })
                    .inner
                    .clicked()
                {
                    self.bookmark_command(
                        offset,
                        Some(BookmarkChange::Rename {
                            id: editing.id,
                            label: Some(self.bookmarks.label.clone()),
                        }),
                    );
                }
                if !busy && theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked()
                {
                    self.bookmarks.editing = None;
                    self.bookmarks.label.clear();
                }
            });
        }
        ui.add_space(20.0);
        let Some(page) = self.bookmarks.page.clone() else {
            return;
        };
        if page.total == 0 {
            ui.label(
                RichText::new(self.tr(
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
                let valid = (bookmark.page_index as usize) < info.pages.len();
                let title = bookmark
                    .label
                    .as_deref()
                    .unwrap_or_else(|| self.tr("Pasaje guardado", "Saved passage"));
                ui.label(RichText::new(title).size(15.0));
                ui.horizontal_wrapped(|ui| {
                    let caption = format!(
                        "{} {}",
                        self.tr("Página", "Page"),
                        u64::from(bookmark.page_index) + 1
                    );
                    if ui
                        .add_enabled_ui(valid, |ui| {
                            theme::navigation(ui, &caption, self.page == bookmark.page_index, p)
                        })
                        .inner
                        .clicked()
                    {
                        self.navigate(bookmark.page_index);
                        if ui.ctx().content_rect().width() < 620.0 {
                            self.panel = false;
                        }
                    }
                    if ui
                        .add_enabled_ui(!busy, |ui| {
                            theme::navigation(ui, self.tr("Editar", "Edit"), false, p)
                        })
                        .inner
                        .clicked()
                    {
                        self.bookmarks.label = bookmark.label.clone().unwrap_or_default();
                        self.bookmarks.editing = Some(bookmark.clone());
                    }
                    if ui
                        .add_enabled_ui(!busy, |ui| {
                            theme::navigation(ui, self.tr("Eliminar", "Remove"), false, p)
                        })
                        .inner
                        .clicked()
                    {
                        self.bookmark_command(
                            offset,
                            Some(BookmarkChange::Remove {
                                id: bookmark.id.clone(),
                            }),
                        );
                    }
                });
                if !valid {
                    ui.label(
                        RichText::new(self.tr(
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
                            self.tr("Marcadores anteriores", "Previous bookmarks"),
                            theme::Icon::Left,
                            p,
                        )
                    })
                    .inner
                    .clicked()
                {
                    self.bookmark_command(page.offset.saturating_sub(BOOKMARK_PAGE_SIZE), None);
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
                                self.tr("Marcadores siguientes", "Next bookmarks"),
                                theme::Icon::Right,
                                p,
                            )
                        },
                    )
                    .inner
                    .clicked()
                {
                    self.bookmark_command(page.offset + BOOKMARK_PAGE_SIZE, None);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn page() -> BookmarkPage {
        BookmarkPage {
            entries: vec![],
            total: 0,
            offset: 0,
        }
    }
    #[test]
    fn stale_document_and_request_results_cannot_replace_current_bookmarks() {
        let mut state = BookmarkState::default();
        state.bind("a");
        let old = Cancellation::default();
        state.pending = Some((old.clone(), false));
        state.bind("b");
        assert!(old.is_cancelled());
        let current = Cancellation::default();
        state.pending = Some((current.clone(), false));
        state.accept("a", &old, Ok(page()));
        state.accept("b", &old, Err("stale".into()));
        assert!(state.page.is_none());
        assert!(state.error.is_none());
        assert!(state.pending.is_some());
        state.accept("b", &current, Ok(page()));
        assert!(state.page.is_some());
        assert!(state.pending.is_none());
    }
    #[test]
    fn changing_documents_does_not_cancel_an_accepted_write() {
        let mut state = BookmarkState::default();
        state.bind("a");
        let write = Cancellation::default();
        state.pending = Some((write.clone(), true));
        state.bind("b");
        assert!(!write.is_cancelled());
        state.accept("a", &write, Ok(page()));
        assert!(state.page.is_none());
    }
    #[test]
    fn failed_edit_keeps_its_draft_for_retry() {
        let mut state = BookmarkState::default();
        state.bind("a");
        let write = Cancellation::default();
        state.pending = Some((write.clone(), true));
        state.label = "Keep this draft".into();
        state.accept("a", &write, Err("failed".into()));
        assert_eq!(state.label, "Keep this draft");
        assert_eq!(state.error.as_deref(), Some("failed"));
        assert!(state.pending.is_none());
    }
    #[test]
    fn an_unrelated_successful_write_preserves_an_open_label_draft() {
        let mut state = BookmarkState::default();
        state.bind("a");
        let write = Cancellation::default();
        state.pending = Some((write.clone(), true));
        state.label = "Unsubmitted label".into();
        state.accept("a", &write, Ok(page()));
        assert_eq!(state.label, "Unsubmitted label");
        assert!(state.saved);
        assert!(state.ready());
    }
}
