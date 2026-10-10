use super::*;
use yamantaka_application::search_index::{IndexCommand, IndexPhase};
use yamantaka_domain::search::{IndexStats, SearchKind, SearchQuery, SearchResult};

pub(super) struct SearchState {
    pub enabled: bool,
    text: String,
    kinds: Vec<SearchKind>,
    missing: bool,
    stale: bool,
    offset: usize,
    pub results: Vec<SearchResult>,
    pub selected: Option<SearchResult>,
    pub excluded: Option<bool>,
    pub stats: Option<IndexStats>,
    pub has_more: bool,
    pub searching: bool,
    pub completed: bool,
    pub error: Option<String>,
    pub preview_results: bool,
    pub preview_action: Option<&'static str>,
    pub preview_open_page: bool,
    pub preview_excluded: bool,
    last_phase: Option<(u64, IndexPhase)>,
    deadline: Option<Instant>,
    cancel: Cancellation,
    selection_cancel: Cancellation,
    focus: bool,
    manual_paused: bool,
    confirm_clear: bool,
}
impl SearchState {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            text: String::new(),
            kinds: Vec::new(),
            missing: false,
            stale: false,
            offset: 0,
            results: Vec::new(),
            selected: None,
            excluded: None,
            stats: None,
            has_more: false,
            searching: false,
            completed: false,
            error: None,
            preview_results: false,
            preview_action: None,
            preview_open_page: false,
            preview_excluded: false,
            last_phase: None,
            deadline: None,
            cancel: Cancellation::default(),
            selection_cancel: Cancellation::default(),
            focus: false,
            manual_paused: false,
            confirm_clear: false,
        }
    }
}
impl YamantakaApp {
    pub(super) fn select_search_result(&mut self, result: SearchResult) {
        self.search.selection_cancel.cancel();
        self.search.selection_cancel = Cancellation::default();
        self.search.excluded = None;
        self.send(Action::IndexState(
            result.document_id.clone(),
            self.search.selection_cancel.clone(),
        ));
        self.search.selected = Some(result);
    }
    pub(super) fn refresh_selected_index_state(&mut self) {
        if let Some(result) = self.search.selected.clone() {
            self.select_search_result(result);
        }
    }
    pub(crate) fn preview_search(&mut self, enabled: bool) {
        self.show_search();
        if enabled {
            self.search.preview_results = true;
            self.search.text = "Yamantaka".into();
            self.index_command(IndexCommand::Start { rebuild: false });
        }
    }
    pub(crate) fn preview_search_action(&mut self, action: &'static str) {
        self.search.preview_action = Some(action);
        self.search.preview_open_page = action == "open";
        self.search.preview_excluded = action == "exclude";
        if action == "open" {
            self.search.text = "Mixed".into();
        }
    }
    pub(super) fn activate_search_result(&mut self, result: &SearchResult) {
        let cancel = self
            .open_intent
            .lookup(result.document_id.clone(), result.page);
        self.send(Action::GetDocument(result.document_id.clone(), cancel));
    }
    pub(super) fn exclude_search_result(&mut self, result: &SearchResult, excluded: bool) {
        self.index_command(IndexCommand::Exclude {
            document_id: result.document_id.clone(),
            excluded,
        });
        self.search.excluded = Some(excluded);
        self.run_search();
    }
    pub(super) fn tools_menu(&mut self, ui: &mut egui::Ui, compact: bool, p: Palette) {
        let label = self.tr("Herramientas", "Tools");
        let response = if compact {
            theme::icon_button(ui, label, theme::Icon::Menu, p)
        } else {
            theme::control(ui, label, Some(theme::Icon::Menu), false, false, p)
        };
        egui::Popup::menu(&response).show(|ui| {
            if theme::navigation(
                ui,
                self.tr("Notas de la biblioteca", "Library notes"),
                matches!(self.surface, Surface::Notes),
                p,
            )
            .clicked()
            {
                self.show_global_notes();
                ui.close();
            }
            if theme::control(
                ui,
                self.tr("Búsqueda global", "Global search"),
                Some(theme::Icon::Search),
                matches!(self.surface, Surface::Search),
                false,
                p,
            )
            .on_hover_text("Ctrl+Shift+F")
            .clicked()
            {
                self.show_search();
                ui.close();
            }
            if theme::control(
                ui,
                self.tr("Ajustes", "Settings"),
                Some(theme::Icon::Settings),
                matches!(self.surface, Surface::Settings),
                false,
                p,
            )
            .clicked()
            {
                self.show_surface(Surface::Settings);
                ui.close();
            }
        });
    }
    pub(super) fn show_search(&mut self) {
        self.show_surface(Surface::Search);
        self.search.focus = true;
        self.send(Action::IndexStatistics);
    }
    pub(super) fn poll_search(&mut self, ctx: &egui::Context) {
        let progress = self.runtime.index_progress();
        let phase = (progress.generation, progress.phase);
        if self.search.last_phase != Some(phase) {
            self.search.last_phase = Some(phase);
            if matches!(
                progress.phase,
                IndexPhase::Completed | IndexPhase::Cancelled
            ) {
                self.send(Action::IndexStatistics);
                self.run_search();
            }
        }
        if let Some(deadline) = self.search.deadline {
            if Instant::now() >= deadline {
                self.search.deadline = None;
                self.run_search();
            } else {
                ctx.request_repaint_after(deadline.saturating_duration_since(Instant::now()));
            }
        }
    }
    fn run_search(&mut self) {
        self.search.error = None;
        self.search.cancel.cancel();
        self.search.cancel = Cancellation::default();
        self.search.deadline = None;
        self.search.completed = false;
        if !self.search.enabled || self.search.text.trim().is_empty() {
            self.search.results.clear();
            self.search.searching = false;
            return;
        }
        self.search.searching = true;
        let query = SearchQuery {
            text: self.search.text.clone(),
            kinds: self.search.kinds.clone(),
            missing: self.search.missing.then_some(true),
            stale: self.search.stale.then_some(true),
            offset: self.search.offset,
            ..Default::default()
        };
        if let Err(error) = self
            .runtime
            .submit(Action::Search(query, self.search.cancel.clone()))
        {
            self.search.searching = false;
            self.error = Some(error.to_string());
        }
    }
    fn set_index_enabled(&mut self, enabled: bool) {
        let previous = self
            .preferences
            .rust
            .insert("globalIndexEnabled".into(), enabled.into());
        if !self.demo
            && let Err(error) = self.preferences.save(&self.store.join("preferences.json"))
        {
            if let Some(value) = previous {
                self.preferences
                    .rust
                    .insert("globalIndexEnabled".into(), value);
            } else {
                self.preferences.rust.remove("globalIndexEnabled");
            }
            self.error = Some(error.to_string());
            return;
        }
        self.search.enabled = enabled;
        self.send(Action::Index(IndexCommand::Enable(enabled)));
        if !enabled {
            self.search.cancel.cancel();
            self.search.results.clear();
            self.search.searching = false;
        } else {
            self.run_search();
        }
    }
    fn index_command(&mut self, command: IndexCommand) {
        if matches!(&command, IndexCommand::Start { .. }) {
            self.search.manual_paused = false;
            self.send(Action::Index(IndexCommand::Pause(false)));
        }
        self.send(Action::Index(command));
        self.send(Action::IndexStatistics);
    }
    pub(super) fn global_search(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let cramped = ui.available_width() < 420.0;
        egui::ScrollArea::vertical()
            .id_salt("global-search-page")
            .animated(!self.reduced_motion)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(if cramped { 16 } else { 36 }, 16))
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width().min(1280.0));
                        theme::display(
                            ui,
                            self.tr("Búsqueda global", "Global search"),
                            if cramped { 22.0 } else { 36.0 },
                        );
                        if !cramped {
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(self.tr(
                                    "Encuentra una idea y vuelve a su contexto.",
                                    "Find an idea and return to its context.",
                                ))
                                .color(p.secondary),
                            );
                        }
                        ui.add_space(if cramped { 8.0 } else { 20.0 });
                        let width = ui.available_width();
                        let response = ui.add_sized(
                            [width, 40.0],
                            egui::TextEdit::singleline(&mut self.search.text)
                                .id_salt("global-search-input")
                                .hint_text(if self.english {
                                    "Search text, notes and bookmarks"
                                } else {
                                    "Buscar texto, notas y marcadores"
                                })
                                .background_color(p.surface)
                                .margin(egui::vec2(36.0, 10.0)),
                        );
                        theme::icon(
                            ui.painter(),
                            egui::pos2(response.rect.left() + 18.0, response.rect.center().y),
                            theme::Icon::Search,
                            p.secondary,
                        );
                        if self.search.focus {
                            response.request_focus();
                            self.search.focus = false;
                        }
                        if response.changed() {
                            self.search.cancel.cancel();
                            self.search.offset = 0;
                            self.search.completed = false;
                            self.search.deadline =
                                Some(Instant::now() + Duration::from_millis(180));
                            ui.ctx().request_repaint_after(Duration::from_millis(180));
                        }
                        if response.has_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter))
                        {
                            self.run_search();
                        }
                        ui.add_space(12.0);
                        self.search_index_controls(ui, p);
                        if self.search.enabled {
                            ui.add_space(16.0);
                            self.search_filters(ui, p);
                            ui.add_space(16.0);
                            ui.separator();
                            ui.add_space(16.0);
                            self.search_results(ui, p);
                        }
                    });
            });
    }
    fn search_index_controls(&mut self, ui: &mut egui::Ui, p: Palette) {
        if !self.search.enabled {
            ui.label(
                RichText::new(self.tr(
                    "Busca dentro de tu biblioteca, solo en este dispositivo.",
                    "Search inside your library, only on this device.",
                ))
                .color(p.secondary),
            );
            ui.add_space(8.0);
            if theme::control(
                ui,
                self.tr("Activar búsqueda local", "Enable local search"),
                Some(theme::Icon::Search),
                false,
                true,
                p,
            )
            .clicked()
            {
                self.set_index_enabled(true);
            }
            ui.add_space(8.0);
            ui.label(RichText::new(self.tr("El índice guarda texto extraído. Puedes desactivarlo o borrarlo cuando quieras.", "The index stores extracted text. You can disable or erase it at any time.")).color(p.secondary));
            return;
        }
        let progress = self.runtime.index_progress();
        let running = matches!(
            progress.phase,
            IndexPhase::Running | IndexPhase::Paused | IndexPhase::Cancelling
        );
        let phase = match progress.phase {
            IndexPhase::Running => self.tr("Indexando", "Indexing"),
            IndexPhase::Paused => self.tr("En pausa", "Paused"),
            IndexPhase::Cancelling => self.tr("Cerrando índice", "Closing index"),
            IndexPhase::Cancelled => self.tr("Indexación detenida", "Indexing stopped"),
            IndexPhase::Completed => self.tr("Indexación completada", "Indexing complete"),
            _ => self.tr("Búsqueda local activada", "Local search enabled"),
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(phase).color(p.ink).strong());
            if progress.total_pages > 0 {
                ui.label(
                    RichText::new(format!(
                        "{} / {} {}",
                        progress.current_page,
                        progress.total_pages,
                        self.tr("páginas", "pages")
                    ))
                    .color(p.secondary),
                );
            }
            if let Some(stats) = &self.search.stats {
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        stats.indexed,
                        self.tr("documentos indexados", "indexed documents")
                    ))
                    .color(p.secondary),
                );
            }
        });
        ui.horizontal_wrapped(|ui| {
            if !running {
                if theme::control(
                    ui,
                    self.tr("Indexar biblioteca", "Index library"),
                    Some(theme::Icon::Search),
                    false,
                    true,
                    p,
                )
                .clicked()
                {
                    self.index_command(IndexCommand::Start { rebuild: false });
                }
            } else if progress.phase != IndexPhase::Cancelling {
                let label = if self.search.manual_paused {
                    self.tr("Reanudar", "Resume")
                } else {
                    self.tr("Pausar", "Pause")
                };
                if theme::navigation(ui, label, false, p).clicked() {
                    self.search.manual_paused = !self.search.manual_paused;
                    self.index_command(IndexCommand::Pause(self.search.manual_paused));
                }
                if theme::navigation(ui, self.tr("Detener", "Stop"), false, p).clicked() {
                    self.index_command(IndexCommand::Cancel);
                }
            }
            let menu = theme::icon_button(
                ui,
                self.tr("Gestionar índice", "Manage index"),
                theme::Icon::Settings,
                p,
            );
            egui::Popup::menu(&menu).show(|ui| {
                ui.add_enabled_ui(!running, |ui| {
                    if theme::navigation(
                        ui,
                        self.tr("Recrear índice completo", "Rebuild complete index"),
                        false,
                        p,
                    )
                    .clicked()
                    {
                        self.index_command(IndexCommand::Start { rebuild: true });
                        ui.close();
                    }
                });
                if theme::navigation(
                    ui,
                    self.tr("Borrar índice local", "Erase local index"),
                    false,
                    p,
                )
                .clicked()
                {
                    self.search.confirm_clear = true;
                    ui.close();
                }
                if theme::navigation(
                    ui,
                    self.tr("Desactivar búsqueda local", "Disable local search"),
                    false,
                    p,
                )
                .clicked()
                {
                    self.set_index_enabled(false);
                    ui.close();
                }
            });
        });
        if progress.phase == IndexPhase::Paused && !self.search.manual_paused {
            ui.label(
                RichText::new(self.tr(
                    "Continuará cuando el lector o la importación liberen su espacio.",
                    "Will continue when the reader or import releases its space.",
                ))
                .color(p.secondary),
            );
        }
        if progress.failed_documents > 0 {
            ui.label(
                RichText::new(format!(
                    "{} {}",
                    progress.failed_documents,
                    self.tr(
                        "documentos necesitan revisión. Puedes volver a intentar indexarlos.",
                        "documents need attention. You can retry indexing them."
                    )
                ))
                .color(p.secondary),
            );
        }
        if self.search.confirm_clear {
            ui.add_space(8.0);
            ui.label(self.tr(
                "Se borrará el texto del índice. Tus PDF y anotaciones se conservan.",
                "Indexed text will be erased. Your PDFs and annotations are preserved.",
            ));
            ui.horizontal_wrapped(|ui| {
                if theme::navigation(ui, self.tr("Borrar índice", "Erase index"), false, p)
                    .clicked()
                {
                    self.index_command(IndexCommand::Clear);
                    self.search.confirm_clear = false;
                    self.run_search();
                }
                if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked() {
                    self.search.confirm_clear = false;
                }
            });
        }
    }
    fn kind_label(&self, kind: SearchKind) -> &str {
        match kind {
            SearchKind::Metadata => self.tr("Documentos", "Documents"),
            SearchKind::PageText => self.tr("Texto", "Text"),
            SearchKind::Annotation => self.tr("Anotaciones", "Annotations"),
            SearchKind::Bookmark => self.tr("Marcadores", "Bookmarks"),
            SearchKind::Collection => self.tr("Colecciones", "Collections"),
            SearchKind::Tag => self.tr("Etiquetas", "Tags"),
            SearchKind::Relation => self.tr("Relaciones", "Relations"),
        }
    }
    fn search_filters(&mut self, ui: &mut egui::Ui, p: Palette) {
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            if theme::navigation(ui, self.tr("Todo", "All"), self.search.kinds.is_empty(), p)
                .clicked()
            {
                self.search.kinds.clear();
                changed = true;
            }
            for kind in [
                SearchKind::Metadata,
                SearchKind::PageText,
                SearchKind::Annotation,
                SearchKind::Bookmark,
                SearchKind::Collection,
                SearchKind::Tag,
                SearchKind::Relation,
            ] {
                let selected = self.search.kinds.contains(&kind);
                if theme::navigation(ui, self.kind_label(kind), selected, p).clicked() {
                    if selected {
                        self.search.kinds.retain(|value| *value != kind);
                    } else {
                        self.search.kinds.push(kind);
                    }
                    changed = true;
                }
            }
            let response = theme::icon_button(
                ui,
                self.tr("Filtros de archivo", "File filters"),
                theme::Icon::Sort,
                p,
            );
            egui::Popup::menu(&response).show(|ui| {
                changed |= ui
                    .checkbox(
                        &mut self.search.missing,
                        if self.english {
                            "Missing files only"
                        } else {
                            "Solo archivos ausentes"
                        },
                    )
                    .changed();
                changed |= ui
                    .checkbox(
                        &mut self.search.stale,
                        if self.english {
                            "Changed files only"
                        } else {
                            "Solo archivos modificados"
                        },
                    )
                    .changed();
            });
            if (!self.search.kinds.is_empty() || self.search.missing || self.search.stale)
                && theme::navigation(
                    ui,
                    self.tr("Restablecer filtros", "Reset filters"),
                    false,
                    p,
                )
                .clicked()
            {
                self.search.kinds.clear();
                self.search.missing = false;
                self.search.stale = false;
                changed = true;
            }
        });
        if changed {
            self.search.offset = 0;
            self.run_search();
        }
    }
    fn search_results(&mut self, ui: &mut egui::Ui, p: Palette) {
        if let Some(error) = &self.search.error {
            ui.label(error);
            if theme::navigation(ui, self.tr("Reintentar búsqueda", "Retry search"), false, p)
                .clicked()
            {
                self.run_search();
            }
        }
        if self.search.searching {
            ui.horizontal_wrapped(|ui| {
                ui.label(self.tr("Buscando…", "Searching…"));
                if theme::navigation(ui, self.tr("Cancelar búsqueda", "Cancel search"), false, p)
                    .clicked()
                {
                    self.search.cancel.cancel();
                    self.search.searching = false;
                }
            });
        }
        if self.search.results.is_empty() {
            let label = if self.search.text.trim().is_empty() {
                self.tr(
                    "¿Qué idea quieres encontrar?",
                    "What idea would you like to find?",
                )
            } else if self.search.completed {
                self.tr(
                    "No hay resultados con estos filtros.",
                    "No results with these filters.",
                )
            } else {
                self.tr(
                    "Escribe una palabra o una frase para buscar.",
                    "Enter a word or phrase to search.",
                )
            };
            ui.label(RichText::new(label).size(18.0));
            if self.search.completed {
                ui.label(
                    RichText::new(self.tr(
                        "Prueba otras palabras o indexa los documentos pendientes.",
                        "Try different words or index pending documents.",
                    ))
                    .color(p.secondary),
                );
            }
        }
        let width = ui.available_width();
        if width >= 900.0 {
            let height =
                (ui.ctx().content_rect().bottom() - ui.cursor().top() - 24.0).clamp(180.0, 640.0);
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(width * 0.55 - 16.0, height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| self.search_list(ui, p),
                );
                ui.add_space(24.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(width * 0.45 - 16.0, height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| self.search_preview(ui, p),
                );
            });
        } else {
            self.search_list(ui, p);
            ui.add_space(20.0);
            self.search_preview(ui, p);
        }
    }
    fn search_list(&mut self, ui: &mut egui::Ui, p: Palette) {
        let results = self.search.results.clone();
        let height = ui
            .available_height()
            .min(ui.ctx().content_rect().height() - 300.0)
            .clamp(160.0, 640.0);
        egui::ScrollArea::vertical()
            .id_salt("global-search-results")
            .max_height(height)
            .show(ui, |ui| {
                for result in results {
                    let selected = self.search.selected.as_ref().is_some_and(|value| {
                        value.document_id == result.document_id
                            && value.page == result.page
                            && value.kind == result.kind
                    });
                    let response = egui::Frame::new()
                        .fill(if selected { p.raised } else { p.canvas })
                        .inner_margin(egui::Margin::same(14))
                        .corner_radius(8)
                        .show(ui, |ui| {
                            ui.set_min_width((ui.available_width() - 28.0).max(0.0));
                            ui.label(RichText::new(&result.title).size(17.0).strong());
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    RichText::new(self.kind_label(result.kind))
                                        .size(12.0)
                                        .color(p.secondary),
                                );
                                if let Some(page) = result.page {
                                    ui.label(
                                        RichText::new(format!(
                                            "{} {}",
                                            self.tr("Página", "Page"),
                                            page + 1
                                        ))
                                        .size(12.0)
                                        .color(p.secondary),
                                    );
                                }
                                if result.missing {
                                    ui.label(self.tr("Ausente", "Missing"));
                                }
                                if result.stale {
                                    ui.label(self.tr("Modificado", "Changed"));
                                }
                            });
                            ui.label(RichText::new(&result.snippet).color(p.secondary));
                        })
                        .response;
                    let response = ui.interact(
                        response.rect,
                        response.id.with((
                            "result",
                            &result.document_id,
                            result.page,
                            result.kind.as_str(),
                        )),
                        egui::Sense::click(),
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::SelectableLabel,
                            ui.is_enabled(),
                            selected,
                            &result.title,
                        )
                    });
                    if response.hovered() && !selected {
                        ui.painter().rect_stroke(
                            response.rect,
                            8.0,
                            egui::Stroke::new(1.0, p.line),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if response.has_focus() {
                        ui.painter().rect_stroke(
                            response.rect.shrink(1.0),
                            8.0,
                            egui::Stroke::new(2.0, p.accent),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if response.clicked() {
                        self.select_search_result(result);
                    }
                    ui.add_space(6.0);
                }
            });
        if self.search.offset > 0 || self.search.has_more {
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled_ui(self.search.offset > 0, |ui| {
                    if theme::navigation(ui, self.tr("Anterior", "Previous"), false, p).clicked() {
                        self.search.offset = self.search.offset.saturating_sub(80);
                        self.run_search();
                    }
                });
                ui.add_enabled_ui(self.search.has_more, |ui| {
                    if theme::navigation(ui, self.tr("Siguiente", "Next"), false, p).clicked() {
                        self.search.offset += 80;
                        self.run_search();
                    }
                });
            });
        }
    }
    fn search_preview(&mut self, ui: &mut egui::Ui, p: Palette) {
        let Some(result) = self.search.selected.clone() else {
            if !self.search.results.is_empty() {
                ui.label(
                    RichText::new(self.tr(
                        "Selecciona un resultado para ver su contexto.",
                        "Select a result to see its context.",
                    ))
                    .color(p.secondary),
                );
            }
            return;
        };
        egui::Frame::new()
            .fill(p.surface)
            .inner_margin(egui::Margin::same(24))
            .corner_radius(12)
            .show(ui, |ui| {
                theme::display(ui, &result.title, 24.0);
                ui.add_space(8.0);
                ui.label(RichText::new(self.kind_label(result.kind)).color(p.secondary));
                if let Some(page) = result.page {
                    ui.label(format!("{} {}", self.tr("Página", "Page"), page + 1));
                }
                ui.add_space(20.0);
                ui.label(&result.snippet);
                ui.add_space(20.0);
                ui.add_enabled_ui(!result.missing, |ui| {
                    let label = if result.page.is_some() {
                        self.tr("Abrir página", "Open page")
                    } else {
                        self.tr("Abrir documento", "Open document")
                    };
                    if theme::control(ui, label, Some(theme::Icon::Right), false, true, p).clicked()
                    {
                        self.activate_search_result(&result);
                    }
                });
                if result.missing {
                    ui.label(self.tr(
                        "Localiza el archivo desde la biblioteca antes de abrirlo.",
                        "Locate the file from the library before opening it.",
                    ));
                }
                if let Some(excluded) = self.search.excluded {
                    ui.add_space(12.0);
                    let label = if excluded {
                        self.tr("Incluir en el índice", "Include in index")
                    } else {
                        self.tr("Excluir del índice", "Exclude from index")
                    };
                    if theme::navigation(ui, label, false, p).clicked() {
                        self.exclude_search_result(&result, !excluded);
                    }
                }
            });
    }
}
impl Drop for SearchState {
    fn drop(&mut self) {
        self.cancel.cancel();
        self.selection_cancel.cancel();
    }
}
