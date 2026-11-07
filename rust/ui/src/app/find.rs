use super::*;
mod controller;
mod highlights;
mod reference;
pub(super) struct FindTarget {
    pub page: u32,
    pub position: Option<yamantaka_domain::reader::ReaderPosition>,
}
use yamantaka_application::document_find::{FindBatch, FindCursor, FindProgress, FindRequest};
use yamantaka_domain::pdf_find::{DocumentTextPresence, PdfFindQuery};

#[derive(Clone, Copy)]
enum SelectionEdge {
    First,
    Last,
}
enum FindPhase {
    Idle,
    Running {
        token: Cancellation,
        progress: FindProgress,
        edge: SelectionEdge,
        wrap: bool,
    },
    Ready {
        batch: FindBatch,
        selected: Option<usize>,
    },
    Stopped,
    Failed(String),
}
pub(super) struct FindState {
    document: Option<String>,
    editor: String,
    match_case: bool,
    whole_words: bool,
    query: Option<PdfFindQuery>,
    phase: FindPhase,
    focus: bool,
    preview: Option<String>,
    highlights: highlights::Highlights,
}
impl Default for FindState {
    fn default() -> Self {
        Self {
            document: None,
            editor: String::new(),
            match_case: false,
            whole_words: false,
            query: None,
            phase: FindPhase::Idle,
            focus: false,
            preview: None,
            highlights: Default::default(),
        }
    }
}
impl Drop for FindState {
    fn drop(&mut self) {
        self.stop();
    }
}
impl FindState {
    pub(super) fn owns(&self, document: &str, cancel: &Cancellation) -> bool {
        matches!(&self.phase, FindPhase::Running { token, .. } if self.document.as_deref() == Some(document) && !cancel.is_cancelled() && token.same_request(cancel))
    }
    pub(super) fn take_preview(&mut self) -> Option<String> {
        self.preview.take()
    }
    pub(super) fn restore_preview(&mut self, preview: Option<String>) {
        self.preview = preview;
    }
    pub(super) fn stop(&mut self) {
        self.highlights.clear();
        if let FindPhase::Running { token, .. } = &self.phase {
            token.cancel();
            self.phase = FindPhase::Stopped;
        }
    }
    pub(super) fn capture_summary(&self) -> serde_json::Value {
        match &self.phase {
            FindPhase::Ready { batch, selected } => serde_json::json!({
                "matches": batch.matches().count(), "selected": selected,
                "textPresence": format!("{:?}", batch.text_presence),
                "page": selected.and_then(|index| batch.matches().nth(index).map(|(page, _)| page)),
                "continuation": batch.next.map(|cursor| [cursor.page, cursor.ordinal]),
            }),
            _ => serde_json::Value::Null,
        }
    }
    pub(super) fn ready(&self) -> bool {
        !matches!(self.phase, FindPhase::Running { .. } | FindPhase::Idle)
    }
    pub(super) fn counter(&self) -> Option<String> {
        let FindPhase::Ready { batch, selected } = &self.phase else {
            return None;
        };
        let index = selected.map_or(0, |index| index + 1);
        Some(format!("{index} / {}", batch.matches().count()))
    }
    pub(super) fn accept(
        &mut self,
        document: &str,
        cancel: &Cancellation,
        result: Result<FindBatch, String>,
    ) -> bool {
        let FindPhase::Running { token, edge, .. } = &self.phase else {
            return false;
        };
        if self.document.as_deref() != Some(document)
            || cancel.is_cancelled()
            || !token.same_request(cancel)
        {
            return false;
        }
        self.phase = match result {
            Ok(batch) => {
                let count = batch.matches().count();
                let selected = if count == 0 {
                    None
                } else {
                    Some(match edge {
                        SelectionEdge::First => 0,
                        SelectionEdge::Last => count - 1,
                    })
                };
                FindPhase::Ready { batch, selected }
            }
            Err(error) => FindPhase::Failed(error),
        };
        true
    }
}

impl YamantakaApp {
    pub(super) fn toggle_reader_panel(&mut self) {
        self.reader.panel = !self.reader.panel;
    }
    pub fn preview_find(&mut self, query: &str, step: Option<bool>) {
        if self.demo && self.capture.is_some() {
            self.reader.panel = true;
            self.reader.reader_panel_tab = outline::ReaderPanelTab::Find;
            self.reader.find.preview = Some(query.into());
            self.find_preview_step = step;
        }
    }
    pub(super) fn show_find(&mut self) {
        self.reader.panel = true;
        self.select_study_tab(outline::ReaderPanelTab::Find);
        self.reader.find.focus = true;
    }

    fn new_find(&mut self) {
        if let Some((document, info)) = &self.reader.active {
            self.reader.find.start(&self.runtime, document, info);
        }
    }
    pub(super) fn choose_find_match(&mut self, index: usize) {
        if let Some(target) = self.reader.find.choose(index) {
            self.apply_find_target(target);
        }
    }
    fn apply_find_target(&mut self, target: FindTarget) {
        if self.reader.navigate_find(target) {
            self.apply_navigation();
        }
    }
    pub(super) fn accept_find(
        &mut self,
        document: &str,
        cancel: &Cancellation,
        result: Result<FindBatch, String>,
    ) {
        let Some((active, info)) = &self.reader.active else {
            return;
        };
        if active.id != document {
            return;
        }
        if let Some(target) =
            self.reader
                .find
                .accept_result(&self.runtime, active, info, cancel, result)
        {
            self.apply_find_target(target);
            if let Some(forward) = self.find_preview_step.take() {
                if self.find_preview_closed {
                    self.reader.panel = false;
                }
                self.step_find(forward);
            }
        }
    }

    pub(super) fn step_find(&mut self, forward: bool) {
        if let Some((document, info)) = &self.reader.active
            && let Some(target) = self
                .reader
                .find
                .step(&self.runtime, document, info, forward)
        {
            self.apply_find_target(target);
        }
    }
    pub(super) fn find_panel_content(
        &mut self,
        ui: &mut egui::Ui,
        info: &PdfInfo,
        p: Palette,
        compact: bool,
    ) {
        if let Some(query) = self.reader.find.preview.take() {
            self.reader.find.editor = query;
            self.new_find();
        }
        let can_step = matches!(
            self.reader.find.phase,
            FindPhase::Ready {
                selected: Some(_),
                ..
            }
        );
        let label = self.tr("Buscar en este PDF", "Search this PDF");
        let find_label = self.tr("Buscar", "Find");
        let mut step = None;
        let mut filters_changed = false;
        let (response, pressed) = ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let reserved = if compact { 184.0 } else { 276.0 };
                let width = (ui.available_width() - reserved).clamp(72.0, 380.0);
                let response = ui.add_sized(
                    [width, 40.0],
                    egui::TextEdit::singleline(&mut self.reader.find.editor)
                        .id_salt("reader-find-query")
                        .hint_text(label)
                        .char_limit(1024),
                );
                let pressed = theme::primary_icon(ui, find_label, theme::Icon::Search, p).clicked();
                ui.add_enabled_ui(can_step, |ui| {
                    for (forward, label, shortcut, icon) in [
                        (
                            false,
                            self.tr("Coincidencia anterior", "Previous match"),
                            shortcuts::Shortcut::PreviousMatch,
                            theme::Icon::Left,
                        ),
                        (
                            true,
                            self.tr("Coincidencia siguiente", "Next match"),
                            shortcuts::Shortcut::NextMatch,
                            theme::Icon::Right,
                        ),
                    ] {
                        let binding =
                            shortcuts::bindings::caption(self.shortcut_bindings.token(shortcut));
                        if theme::icon_button(ui, &format!("{label} · {binding}"), icon, p)
                            .clicked()
                        {
                            step = Some(forward);
                        }
                    }
                });
                let options = theme::icon_button(
                    ui,
                    self.tr("Opciones de búsqueda", "Search options"),
                    theme::Icon::Settings,
                    p,
                );
                let case_label = self.tr("Mayúsculas", "Match case");
                let words_label = self.tr("Palabras completas", "Whole words");
                egui::Popup::menu(&options)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        filters_changed |= ui
                            .checkbox(&mut self.reader.find.match_case, case_label)
                            .changed();
                        filters_changed |= ui
                            .checkbox(&mut self.reader.find.whole_words, words_label)
                            .changed();
                    });
                if !compact && let Some(counter) = self.reader.find.counter() {
                    ui.label(counter).on_hover_text(
                        self.tr("Coincidencias en este grupo", "Matches in this batch"),
                    );
                }
                (response, pressed)
            })
            .inner;
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, label));
        theme::record_control_bounds(ui, label, response.rect);
        if self.reader.find.focus {
            response.request_focus();
            self.reader.find.focus = false;
        }
        if response.changed() || filters_changed {
            self.reader.find.stop();
            self.reader.find.phase = FindPhase::Idle;
            self.reader.find.query = None;
        }
        if pressed || submit_on_enter(ui, &response) {
            self.new_find();
        } else if let Some(forward) = step {
            self.step_find(forward);
        }
        egui::ScrollArea::vertical()
            .id_salt("reader-find-results")
            .min_scrolled_height(0.0)
            .auto_shrink([false, false])
            .show(ui, |ui| self.find_results(ui, info, p));
    }

    fn find_results(&mut self, ui: &mut egui::Ui, info: &PdfInfo, p: Palette) {
        self.reader.find.highlight_notice(ui, self.english, p);
        match &self.reader.find.phase {
            FindPhase::Idle => {
                ui.label(
                    RichText::new(
                        self.tr("Escribe una palabra o frase.", "Enter a word or phrase."),
                    )
                    .color(p.secondary),
                );
            }
            FindPhase::Stopped => {
                ui.label(self.tr("Búsqueda cancelada.", "Search cancelled."));
            }
            FindPhase::Failed(error) => {
                ui.label(self.tr(
                    "No se pudo completar la búsqueda.",
                    "Search could not be completed.",
                ));
                ui.collapsing(self.tr("Detalles", "Details"), |ui| {
                    ui.label(error);
                });
                if theme::navigation(ui, self.tr("Reintentar", "Try again"), false, p).clicked() {
                    self.new_find();
                }
            }
            FindPhase::Running { progress, .. } => {
                ui.label(format!(
                    "{} {} / {}",
                    self.tr("Páginas revisadas", "Pages searched"),
                    progress.scanned_pages(),
                    info.pages.len()
                ));
                if theme::navigation(ui, self.tr("Cancelar búsqueda", "Cancel search"), false, p)
                    .clicked()
                {
                    self.reader.find.stop();
                }
            }
            FindPhase::Ready { batch, selected } => {
                if batch.matches().next().is_none() {
                    ui.label(
                        if batch.text_presence == DocumentTextPresence::Unavailable {
                            self.tr(
                                "Este PDF no contiene texto extraíble.",
                                "This PDF contains no extractable text.",
                            )
                        } else {
                            self.tr("No se encontraron coincidencias.", "No matches found.")
                        },
                    );
                    return;
                }
                ui.set_max_width(ui.available_width().min(760.0));
                let mut choose = None;
                for (index, (page, item)) in batch.matches().enumerate() {
                    let label = format!(
                        "{} {}  ·  {}",
                        self.tr("Página", "Page"),
                        page + 1,
                        item.snippet
                    );
                    if theme::bounded_choice(ui, &label, *selected == Some(index), p)
                        .on_hover_text(&label)
                        .clicked()
                    {
                        choose = Some(index);
                    }
                }
                if let Some(index) = choose {
                    self.choose_find_match(index);
                    if bookmarks::compact_study(ui.ctx()) {
                        self.close_study_tray();
                    }
                }
            }
        }
    }
}

fn submit_on_enter(ui: &egui::Ui, response: &egui::Response) -> bool {
    (response.has_focus() || response.lost_focus())
        && ui.input(|input| input.key_pressed(egui::Key::Enter))
}
