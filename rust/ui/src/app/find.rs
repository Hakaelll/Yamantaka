use super::*;
use yamantaka_application::document_find::{FindBatch, FindCursor, FindProgress, FindRequest};
use yamantaka_domain::pdf_find::PdfFindQuery;

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
        }
    }
}
impl Drop for FindState {
    fn drop(&mut self) {
        self.stop();
    }
}
impl FindState {
    pub(super) fn take_preview(&mut self) -> Option<String> {
        self.preview.take()
    }
    pub(super) fn restore_preview(&mut self, preview: Option<String>) {
        self.preview = preview;
    }
    pub(super) fn stop(&mut self) {
        if let FindPhase::Running { token, .. } = &self.phase {
            token.cancel();
            self.phase = FindPhase::Stopped;
        }
    }
    pub(super) fn capture_summary(&self) -> serde_json::Value {
        match &self.phase {
            FindPhase::Ready { batch, selected } => serde_json::json!({
                "matches": batch.matches().count(), "selected": selected,
                "page": selected.and_then(|index| batch.matches().nth(index).map(|(page, _)| page)),
                "continuation": batch.next.map(|cursor| [cursor.page, cursor.ordinal]),
            }),
            _ => serde_json::Value::Null,
        }
    }
    pub(super) fn ready(&self) -> bool {
        !matches!(self.phase, FindPhase::Running { .. } | FindPhase::Idle)
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
        self.panel = !self.panel;
        if !self.panel {
            self.find.stop();
        }
    }
    pub fn preview_find(&mut self, step: Option<bool>) {
        if self.demo && self.capture.is_some() {
            self.panel = true;
            self.reader_panel_tab = outline::ReaderPanelTab::Find;
            self.find.preview = Some("reading".into());
            self.find_preview_step = step;
        }
    }
    pub(super) fn show_find(&mut self) {
        self.panel = true;
        self.reader_panel_tab = outline::ReaderPanelTab::Find;
        self.find.focus = true;
    }
    fn submit_find(&mut self, request: FindRequest, edge: SelectionEdge, wrap: bool) {
        self.find.stop();
        let token = request.cancellation.clone();
        let progress = request.progress.clone();
        self.find.document = Some(request.document_id.clone());
        self.find.phase = match self.runtime.submit(Action::FindDocument(request)) {
            Ok(()) => FindPhase::Running {
                token,
                progress,
                edge,
                wrap,
            },
            Err(error) => FindPhase::Failed(error.to_string()),
        };
    }
    fn new_find(&mut self) {
        let Some((document, info)) = &self.active else {
            return;
        };
        let request = PdfFindQuery::new(
            &self.find.editor,
            self.find.match_case,
            self.find.whole_words,
        )
        .and_then(|query| {
            self.find.query = Some(query.clone());
            FindRequest::new(
                document.id.clone(),
                query,
                info.pages.len() as u32,
                FindCursor::default(),
                Cancellation::default(),
            )
        });
        match request {
            Ok(request) => self.submit_find(request, SelectionEdge::First, false),
            Err(error) => {
                self.find.stop();
                self.find.phase = FindPhase::Failed(error.to_string());
            }
        }
    }
    pub(super) fn choose_find_match(&mut self, index: usize) {
        let FindPhase::Ready { batch, selected } = &mut self.find.phase else {
            return;
        };
        let Some((page, item)) = batch.matches().nth(index) else {
            return;
        };
        *selected = Some(index);
        let position = item.position(page);
        let changed = self
            .navigation
            .as_mut()
            .is_some_and(|navigation| match position {
                Some(position) => navigation.navigate_to(position),
                None => navigation.go_to_page(page),
            });
        if changed {
            self.reader_scroll_anchor = true;
            self.apply_navigation();
        }
    }
    pub(super) fn accept_find(
        &mut self,
        document: &str,
        cancel: &Cancellation,
        result: Result<FindBatch, String>,
    ) {
        let wrap_edge = match &self.find.phase {
            FindPhase::Running {
                edge, wrap: true, ..
            } => Some(*edge),
            _ => None,
        };
        if self.find.accept(document, cancel, result) {
            if let Some(edge) = wrap_edge
                && matches!(&self.find.phase, FindPhase::Ready { batch, .. } if batch.matches().next().is_none())
            {
                self.wrap_find(edge);
                return;
            }
            let index = match &self.find.phase {
                FindPhase::Ready { selected, .. } => *selected,
                _ => None,
            };
            if let Some(index) = index {
                self.choose_find_match(index);
                if let Some(forward) = self.find_preview_step.take() {
                    if self.find_preview_closed {
                        self.panel = false;
                    }
                    self.step_find(forward);
                }
            }
        }
    }
    fn wrap_find(&mut self, edge: SelectionEdge) {
        let Some((document, info)) = &self.active else {
            return;
        };
        let Some(query) = self.find.query.clone() else {
            return;
        };
        let request = match edge {
            SelectionEdge::First => FindRequest::new(
                document.id.clone(),
                query,
                info.pages.len() as u32,
                FindCursor::default(),
                Cancellation::default(),
            ),
            SelectionEdge::Last => FindRequest::previous(
                document.id.clone(),
                query,
                info.pages.len() as u32,
                None,
                Cancellation::default(),
            ),
        };
        match request {
            Ok(request) => self.submit_find(request, edge, false),
            Err(error) => {
                self.find.stop();
                self.find.phase = FindPhase::Failed(error.to_string());
            }
        }
    }
    pub(super) fn step_find(&mut self, forward: bool) {
        let FindPhase::Ready {
            batch,
            selected: Some(selected),
        } = &self.find.phase
        else {
            return;
        };
        let count = batch.matches().count();
        let inside = if forward {
            (*selected + 1 < count).then_some(*selected + 1)
        } else {
            selected.checked_sub(1)
        };
        if let Some(index) = inside {
            self.choose_find_match(index);
            return;
        }
        let Some((document, info)) = &self.active else {
            return;
        };
        let Some(query) = self.find.query.clone() else {
            return;
        };
        let wrap = if forward {
            batch.next.is_some()
        } else {
            batch
                .start
                .is_some_and(|start| start != FindCursor::default())
        };
        let cancel = Cancellation::default();
        let request = if forward {
            FindRequest::new(
                document.id.clone(),
                query,
                info.pages.len() as u32,
                batch.next.unwrap_or_default(),
                cancel,
            )
        } else {
            let boundary = batch.start.filter(|start| *start != FindCursor::default());
            FindRequest::previous(
                document.id.clone(),
                query,
                info.pages.len() as u32,
                boundary,
                cancel,
            )
        };
        match request {
            Ok(request) => self.submit_find(
                request,
                if forward {
                    SelectionEdge::First
                } else {
                    SelectionEdge::Last
                },
                wrap,
            ),
            Err(error) => {
                self.find.stop();
                self.find.phase = FindPhase::Failed(error.to_string());
            }
        }
    }
    pub(super) fn find_panel_content(&mut self, ui: &mut egui::Ui, info: &PdfInfo, p: Palette) {
        if let Some(query) = self.find.preview.take() {
            self.find.editor = query;
            self.new_find();
        }
        let compact = ui.ctx().content_rect().width() < 620.0;
        if !compact {
            ui.label(RichText::new(self.tr("Encuentra un pasaje", "Find a passage")).size(22.0));
            ui.add_space(6.0);
        }
        let label = self.tr("Buscar en este PDF", "Search this PDF");
        let find_label = self.tr("Buscar", "Find");
        let (response, pressed) = ui
            .horizontal(|ui| {
                let width = (ui.available_width() - 48.0).max(40.0);
                let response = ui.add_sized(
                    [width, 40.0],
                    egui::TextEdit::singleline(&mut self.find.editor)
                        .id_salt("reader-find-query")
                        .hint_text(label)
                        .char_limit(1024),
                );
                let pressed = theme::primary_icon(ui, find_label, theme::Icon::Search, p).clicked();
                (response, pressed)
            })
            .inner;
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, label));
        if self.find.focus {
            response.request_focus();
            self.find.focus = false;
        }
        let enter = submit_on_enter(ui, &response);
        let mut changed = response.changed();
        let case_label = self.tr("Mayúsculas", "Match case");
        let words_label = self.tr("Palabras completas", "Whole words");
        ui.horizontal_wrapped(|ui| {
            changed |= ui.checkbox(&mut self.find.match_case, case_label).changed();
            changed |= ui
                .checkbox(&mut self.find.whole_words, words_label)
                .changed();
        });
        if changed {
            self.find.stop();
            self.find.phase = FindPhase::Idle;
            self.find.query = None;
        }
        ui.add_space(6.0);
        if pressed || enter {
            self.new_find();
        }
        ui.add_space(12.0);
        match &self.find.phase {
            FindPhase::Idle => {
                ui.label(
                    RichText::new(self.tr(
                        "Busca una palabra o frase para volver al texto.",
                        "Find a word or phrase and return to the text.",
                    ))
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
                let error = error.clone();
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
                    self.find.stop();
                }
            }
            FindPhase::Ready { batch, selected } => {
                let count = batch.matches().count();
                if count == 0 {
                    ui.label(self.tr(
                        "No hay coincidencias en este recorrido.",
                        "No matches in this search range.",
                    ));
                    ui.label(
                        RichText::new(self.tr(
                            "Los PDF escaneados pueden no contener texto buscable.",
                            "Scanned PDFs may contain no searchable text.",
                        ))
                        .color(p.secondary),
                    );
                    return;
                }
                let selected = *selected;
                ui.label(format!(
                    "{} / {} {}",
                    selected.map_or(0, |index| index + 1),
                    count,
                    self.tr("en este grupo", "in this batch")
                ));
                let mut step = None;
                ui.horizontal(|ui| {
                    if theme::icon_button(
                        ui,
                        self.tr(
                            "Coincidencia anterior · Shift+F3",
                            "Previous match · Shift+F3",
                        ),
                        theme::Icon::Left,
                        p,
                    )
                    .clicked()
                    {
                        step = Some(false);
                    }
                    if theme::icon_button(
                        ui,
                        self.tr("Coincidencia siguiente · F3", "Next match · F3"),
                        theme::Icon::Right,
                        p,
                    )
                    .clicked()
                    {
                        step = Some(true);
                    }
                });
                let mut choose = None;
                if let FindPhase::Ready { batch, .. } = &self.find.phase {
                    egui::ScrollArea::vertical()
                        .id_salt("reader-find-results")
                        .max_height(ui.available_height().max(40.0))
                        .show(ui, |ui| {
                            for (index, (page, item)) in batch.matches().enumerate() {
                                let label = format!(
                                    "{} {}  ·  {}",
                                    self.tr("Página", "Page"),
                                    page + 1,
                                    item.snippet
                                );
                                let response = ui.add_sized(
                                    [ui.available_width(), 56.0],
                                    egui::Button::new(RichText::new(&label).size(14.0))
                                        .wrap()
                                        .selected(selected == Some(index)),
                                );
                                response.widget_info(|| {
                                    egui::WidgetInfo::selected(
                                        egui::WidgetType::Button,
                                        ui.is_enabled(),
                                        selected == Some(index),
                                        &label,
                                    )
                                });
                                theme::record_control_bounds(ui, &label, response.rect);
                                if response.clicked() {
                                    choose = Some(index);
                                }
                            }
                        });
                }
                if let Some(index) = choose {
                    self.choose_find_match(index);
                    if compact {
                        self.panel = false;
                    }
                }
                if let Some(forward) = step {
                    self.step_find(forward);
                }
            }
        }
    }
    pub(super) fn paint_find_matches(
        &self,
        ui: &egui::Ui,
        page: u32,
        paper: egui::Rect,
        rotation: yamantaka_domain::reader_preferences::PageRotation,
        p: Palette,
    ) {
        let FindPhase::Ready { batch, selected } = &self.find.phase else {
            return;
        };
        for (index, (match_page, item)) in batch.matches().enumerate() {
            if match_page != page {
                continue;
            }
            for bounds in &item.rectangles {
                let rect = match_rect(paper, rotation, bounds.min(), bounds.max());
                let active = *selected == Some(index);
                ui.painter().rect_filled(
                    rect,
                    1,
                    p.accent.gamma_multiply(if active { 0.30 } else { 0.14 }),
                );
                if active {
                    ui.painter().rect_stroke(
                        rect,
                        1,
                        egui::Stroke::new(1.0, p.accent),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
    }
}
fn submit_on_enter(ui: &egui::Ui, response: &egui::Response) -> bool {
    (response.has_focus() || response.lost_focus())
        && ui.input(|input| input.key_pressed(egui::Key::Enter))
}
fn match_rect(
    paper: egui::Rect,
    rotation: yamantaka_domain::reader_preferences::PageRotation,
    min: [f64; 2],
    max: [f64; 2],
) -> egui::Rect {
    let points = [
        [min[0], min[1]],
        [max[0], min[1]],
        [min[0], max[1]],
        [max[0], max[1]],
    ];
    let mut rect = egui::Rect::NOTHING;
    for point in points {
        let point = rotation.rotate_point(point);
        rect.extend_with(
            paper.min
                + egui::vec2(
                    point[0] as f32 * paper.width(),
                    point[1] as f32 * paper.height(),
                ),
        );
    }
    rect
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::reader_preferences::PageRotation;
    #[test]
    fn enter_submits_when_the_native_single_line_editor_releases_focus() {
        let context = egui::Context::default();
        let mut text = "reading".to_owned();
        let id = egui::Id::new("reader-find-query-test");
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            let response = ui.add(egui::TextEdit::singleline(&mut text).id(id));
            response.request_focus();
        });
        output.textures_delta.clear();
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut submitted = false;
        let mut lost_focus = false;
        let mut output = context.run_ui(input, |ui| {
            let response = ui.add(egui::TextEdit::singleline(&mut text).id(id));
            submitted |= submit_on_enter(ui, &response);
            lost_focus |= response.lost_focus();
        });
        output.textures_delta.clear();
        assert!(lost_focus);
        assert!(submitted);
    }
    #[test]
    fn pending_find_rejects_other_documents_consumers_and_cancelled_replies() {
        let current = Cancellation::default();
        let mut find = FindState {
            document: Some("reader".into()),
            phase: FindPhase::Running {
                token: current.clone(),
                progress: FindProgress::default(),
                edge: SelectionEdge::First,
                wrap: false,
            },
            editor: String::new(),
            match_case: false,
            whole_words: false,
            query: None,
            focus: false,
            preview: None,
        };
        assert!(!find.accept("other", &current, Err("wrong document".into())));
        assert!(!find.accept(
            "reader",
            &Cancellation::default(),
            Err("wrong token".into())
        ));
        assert!(!find.ready());
        current.cancel();
        assert!(!find.accept("reader", &current, Err("late".into())));
        find.stop();
        assert!(find.ready());
    }
    #[test]
    fn pending_find_drop_cancels_native_consumer_and_matching_failure_is_visible() {
        let current = Cancellation::default();
        let find = FindState {
            document: None,
            phase: FindPhase::Running {
                token: current.clone(),
                progress: FindProgress::default(),
                edge: SelectionEdge::Last,
                wrap: false,
            },
            editor: String::new(),
            match_case: false,
            whole_words: false,
            query: None,
            focus: false,
            preview: None,
        };
        drop(find);
        assert!(current.is_cancelled());
        let current = Cancellation::default();
        let mut find = FindState {
            document: Some("reader".into()),
            phase: FindPhase::Running {
                token: current.clone(),
                progress: FindProgress::default(),
                edge: SelectionEdge::First,
                wrap: false,
            },
            editor: String::new(),
            match_case: false,
            whole_words: false,
            query: None,
            focus: false,
            preview: None,
        };
        assert!(find.accept("reader", &current, Err("recoverable".into())));
        assert!(matches!(find.phase, FindPhase::Failed(ref error) if error == "recoverable"));
    }
    #[test]
    fn search_rectangles_follow_literal_quarter_turns_inside_scaled_paper() {
        let paper = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(200.0, 400.0));
        for (rotation, expected) in [
            (PageRotation::Deg0, [30.0, 100.0, 70.0, 180.0]),
            (PageRotation::Deg90, [130.0, 60.0, 170.0, 140.0]),
            (PageRotation::Deg180, [150.0, 260.0, 190.0, 340.0]),
            (PageRotation::Deg270, [50.0, 300.0, 90.0, 380.0]),
        ] {
            let rect = match_rect(paper, rotation, [0.1, 0.2], [0.3, 0.4]);
            for (value, expected) in [rect.left(), rect.top(), rect.right(), rect.bottom()]
                .into_iter()
                .zip(expected)
            {
                assert!((value - expected).abs() < 0.001);
            }
        }
    }
}
