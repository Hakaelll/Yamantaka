use super::*;
use yamantaka_domain::search::{ExclusionPage, ExclusionQuery};

#[derive(Default)]
pub(super) struct State {
    pub(super) open: bool,
    query: ExclusionQuery,
    page: Option<ExclusionPage>,
    pending: Option<Cancellation>,
    error: Option<String>,
    included: bool,
    preview: u8,
}
impl Drop for State {
    fn drop(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
    }
}
impl YamantakaApp {
    pub(in crate::app) fn search_exclusions_ready(&self) -> bool {
        self.search.exclusions.pending.is_none() && self.search.exclusions.preview == 0
    }
    pub(in crate::app) fn search_exclusions_summary(&self) -> serde_json::Value {
        let state = &self.search.exclusions;
        serde_json::json!({"open":state.open,"preview":state.preview,"pending":state.pending.is_some(),"total":state.page.as_ref().map(|p|p.total),"rows":state.page.as_ref().map(|p|p.entries.len()),"offset":state.page.as_ref().map(|p|p.offset),"error":state.error.is_some()})
    }
    pub(crate) fn preview_search_exclusions(&mut self) {
        if self.demo && self.capture.is_some() {
            self.show_search();
            self.search.exclusions.preview = 1;
            self.open_search_exclusions();
        }
    }
    pub(in crate::app) fn refresh_search_exclusions(&mut self) {
        if self.search.exclusions.open && self.search.exclusions.pending.is_none() {
            self.load_search_exclusions(None);
        }
    }
    pub(super) fn advance_search_exclusions_preview(&mut self) {
        let state = &self.search.exclusions;
        if state.preview == 0 || state.pending.is_some() {
            return;
        }
        assert!(
            state.error.is_none(),
            "Exclusions operation failed: {:?}",
            state.error
        );
        let Some(page) = state.page.clone() else {
            return;
        };
        match state.preview {
            1 => {
                assert_eq!((page.total, page.offset, page.entries.len()), (165, 0, 80));
                self.search.exclusions.query.offset = 80;
                self.search.exclusions.preview = 2;
                self.load_search_exclusions(None);
            }
            2 => {
                assert_eq!((page.total, page.offset, page.entries.len()), (165, 80, 80));
                self.search.exclusions.query.offset = 160;
                self.search.exclusions.preview = 3;
                self.load_search_exclusions(None);
            }
            3 => {
                assert_eq!((page.total, page.offset, page.entries.len()), (165, 160, 5));
                self.search.exclusions.preview = 4;
                self.load_search_exclusions(Some(page.entries[0].id.clone()));
            }
            4 => {
                assert_eq!((page.total, page.offset, page.entries.len()), (164, 160, 4));
                self.search.exclusions.query.text = "catalogue 000".into();
                self.search.exclusions.query.offset = 0;
                self.search.exclusions.preview = 5;
                self.load_search_exclusions(None);
            }
            5 => {
                assert_eq!(page.total, 1);
                assert_eq!(page.entries[0].id, "excluded-000");
                self.search.exclusions.query.text = "no matching book".into();
                self.search.exclusions.preview = 6;
                self.load_search_exclusions(None);
            }
            6 => {
                assert_eq!(page.total, 0);
                assert!(page.entries.is_empty());
                self.search.exclusions.query.text = "Lectura".into();
                self.search.exclusions.preview = 7;
                self.load_search_exclusions(None);
            }
            7 => {
                assert_eq!((page.total, page.entries.len()), (164, 80));
                assert!(
                    page.entries
                        .iter()
                        .all(|entry| entry.author == "Lectura personal")
                );
                self.search.exclusions.query.text.clear();
                self.search.exclusions.preview = 8;
                self.load_search_exclusions(None);
            }
            8 => {
                assert_eq!((page.total, page.offset, page.entries.len()), (164, 0, 80));
                assert!(!matches!(
                    self.runtime.index_progress().phase,
                    IndexPhase::Running
                ));
                self.search.exclusions.preview = 0;
            }
            _ => unreachable!(),
        }
    }
    pub(in crate::app) fn open_search_exclusions(&mut self) {
        self.search.exclusions.open = true;
        self.load_search_exclusions(None);
    }
    fn load_search_exclusions(&mut self, include: Option<String>) {
        let state = &mut self.search.exclusions;
        if let Some(cancel) = state.pending.take() {
            cancel.cancel();
        }
        state.error = None;
        state.included = include.is_some();
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::SearchExclusions(
            state.query.clone(),
            include,
            cancel.clone(),
        )) {
            Ok(()) => state.pending = Some(cancel),
            Err(error) => state.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_search_exclusions(
        &mut self,
        result: yamantaka_domain::Result<ExclusionPage>,
        cancel: &Cancellation,
    ) {
        let state = &mut self.search.exclusions;
        if cancel.is_cancelled()
            || !state
                .pending
                .as_ref()
                .is_some_and(|pending| pending.same_request(cancel))
        {
            return;
        }
        state.pending = None;
        match result {
            Ok(page) => {
                state.query.offset = page.offset;
                state.page = Some(page);
                if state.included {
                    self.send(Action::IndexStatistics);
                    self.refresh_selected_index_state();
                    self.run_search();
                }
            }
            Err(error) => state.error = Some(error.to_string()),
        }
    }
    pub(super) fn search_exclusions_view(&mut self, ui: &mut egui::Ui, p: Palette) {
        if !self.search.exclusions.open {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            theme::display(
                ui,
                self.tr("Documentos excluidos", "Excluded documents"),
                26.0,
            );
            if theme::navigation(ui, self.tr("Volver a búsqueda", "Back to search"), false, p)
                .clicked()
            {
                self.search.exclusions.open = false;
                if let Some(cancel) = self.search.exclusions.pending.take() {
                    cancel.cancel();
                }
            }
        });
        ui.label(RichText::new(self.tr(
            "Estos libros no aparecen en la búsqueda global. Incluirlos permite indexarlos cuando reanudes el índice.",
            "These books are hidden from global search. Including them makes them eligible when you resume indexing.",
        )).color(p.secondary));
        let busy = self.search.exclusions.pending.is_some();
        ui.horizontal_wrapped(|ui| {
            let label =
                ui.label(self.tr("Filtrar por título o autor", "Filter by title or author"));
            let response = ui
                .add_enabled(
                    !(busy && self.search.exclusions.included),
                    egui::TextEdit::singleline(&mut self.search.exclusions.query.text)
                        .char_limit(256)
                        .desired_width(240.0),
                )
                .labelled_by(label.id);
            if response.changed() {
                self.search.exclusions.query.offset = 0;
                self.load_search_exclusions(None);
            }
            if ui
                .add_enabled_ui(!busy, |ui| {
                    theme::navigation(ui, self.tr("Actualizar", "Refresh"), false, p)
                })
                .inner
                .clicked()
            {
                self.load_search_exclusions(None);
            }
        });
        if let Some(error) = &self.search.exclusions.error {
            Self::file_status_label(ui, error, p);
        }
        if let Some(page) = self.search.exclusions.page.clone() {
            let blocked = busy || self.search.exclusions.error.is_some();
            egui::ScrollArea::vertical()
                .id_salt("search-exclusions")
                .max_height(280.0)
                .show(ui, |ui| {
                    for entry in &page.entries {
                        ui.push_id(&entry.id, |ui| {
                            ui.horizontal(|ui| {
                                let width = (ui.available_width() - 130.0).max(40.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(width, 44.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.set_min_width(width);
                                        ui.add(
                                            egui::Label::new(RichText::new(&entry.title).strong())
                                                .truncate(),
                                        )
                                        .on_hover_text(&entry.title);
                                        let detail = if entry.missing {
                                            self.tr("Archivo no disponible", "File unavailable")
                                        } else {
                                            &entry.author
                                        };
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(detail).color(p.secondary),
                                            )
                                            .truncate(),
                                        );
                                    },
                                );
                                if ui
                                    .add_enabled_ui(!blocked, |ui| {
                                        theme::navigation(
                                            ui,
                                            self.tr("Incluir", "Include"),
                                            false,
                                            p,
                                        )
                                    })
                                    .inner
                                    .clicked()
                                {
                                    self.load_search_exclusions(Some(entry.id.clone()));
                                }
                            });
                        });
                    }
                });
            if page.entries.is_empty() {
                ui.label(self.tr(
                    "No hay documentos excluidos que coincidan.",
                    "No matching excluded documents.",
                ));
            }
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled_ui(!blocked && page.offset > 0, |ui| {
                        theme::navigation(ui, self.tr("Anterior", "Previous"), false, p)
                    })
                    .inner
                    .clicked()
                {
                    self.search.exclusions.query.offset = page.offset.saturating_sub(80);
                    self.load_search_exclusions(None);
                }
                ui.label(format!(
                    "{}–{} / {}",
                    if page.total == 0 { 0 } else { page.offset + 1 },
                    page.offset + page.entries.len(),
                    page.total
                ));
                if ui
                    .add_enabled_ui(
                        !blocked && ((page.offset + page.entries.len()) as u64) < page.total,
                        |ui| theme::navigation(ui, self.tr("Siguiente", "Next"), false, p),
                    )
                    .inner
                    .clicked()
                {
                    self.search.exclusions.query.offset = page.offset + 80;
                    self.load_search_exclusions(None);
                }
            });
        }
        ui.separator();
    }
}
