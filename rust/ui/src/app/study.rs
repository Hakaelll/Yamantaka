use super::*;
use annotations::NotesAction;
use yamantaka_application::annotations::{AnnotationMutation, AnnotationReply};
use yamantaka_domain::{
    annotations::{
        AnnotationColor, AnnotationDraft, AnnotationKind, AnnotationSource,
        batch::AnnotationDraftBatch,
        study::{MAX_STUDY_POINTS, StudyGeometry},
    },
    reader_preferences::PageRotation,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Tool {
    #[default]
    Select,
    Highlight,
    Underline,
    Note,
    Pen,
    Arrow,
    Rectangle,
}

impl Tool {
    const ALL: [Self; 7] = [
        Self::Select,
        Self::Highlight,
        Self::Underline,
        Self::Note,
        Self::Pen,
        Self::Arrow,
        Self::Rectangle,
    ];

    fn label(self, en: bool) -> &'static str {
        match (self, en) {
            (Self::Select, false) => "Seleccionar",
            (Self::Select, true) => "Select",
            (Self::Highlight, false) => "Resaltar",
            (Self::Highlight, true) => "Highlight",
            (Self::Underline, false) => "Subrayar",
            (Self::Underline, true) => "Underline",
            (Self::Note, false) => "Nota",
            (Self::Note, true) => "Note",
            (Self::Pen, false) => "Lápiz",
            (Self::Pen, true) => "Pen",
            (Self::Arrow, false) => "Flecha",
            (Self::Arrow, true) => "Arrow",
            (Self::Rectangle, false) => "Recuadro",
            (Self::Rectangle, true) => "Rectangle",
        }
    }

    pub(super) fn drawing(self) -> Option<AnnotationKind> {
        match self {
            Self::Pen => Some(AnnotationKind::Ink),
            Self::Arrow => Some(AnnotationKind::Arrow),
            Self::Rectangle => Some(AnnotationKind::Rectangle),
            _ => None,
        }
    }

    fn icon(self) -> Option<theme::Icon> {
        match self {
            Self::Pen => Some(theme::Icon::Pencil),
            Self::Arrow => Some(theme::Icon::Arrow),
            Self::Rectangle => Some(theme::Icon::Rectangle),
            _ => None,
        }
    }
}

pub(super) struct State {
    pub active: bool,
    tool: Tool,
    color: AnnotationColor,
    bounds: Option<egui::Rect>,
    preview: u8,
}

impl Default for State {
    fn default() -> Self {
        Self {
            active: false,
            tool: Tool::Select,
            color: AnnotationColor::Yellow,
            bounds: None,
            preview: 0,
        }
    }
}

impl State {
    pub(super) fn owns_input(&self, ctx: &egui::Context) -> bool {
        let Some(bounds) = self.bounds else {
            return false;
        };
        if let Some(pressed) = ctx.input(|input| {
            input
                .pointer
                .any_pressed()
                .then(|| input.pointer.interact_pos())
                .flatten()
        }) {
            return bounds.contains(pressed)
                || [tool_menu(), color_menu()].into_iter().any(|id| {
                    egui::Popup::is_id_open(ctx, id)
                        && ctx.layer_id_at(pressed).is_some_and(|layer| layer.id == id)
                });
        }
        ctx.memory(|memory| memory.focused())
            .and_then(|id| ctx.read_response(id))
            .is_some_and(|response| {
                bounds.contains(response.rect.center())
                    || [tool_menu(), color_menu()].contains(&response.layer_id.id)
            })
    }
}

fn tool_menu() -> egui::Id {
    egui::Id::new("study-tool-menu")
}
fn color_menu() -> egui::Id {
    egui::Id::new("study-color-menu")
}

#[derive(Default)]
pub(super) struct Pane {
    pub hidden: bool,
    stroke: Option<Stroke>,
    intent: Option<Intent>,
    save: Option<Save>,
}

enum Intent {
    PageNote(u32),
    Drawing(Stroke),
}

struct Stroke {
    page: u32,
    kind: AnnotationKind,
    color: AnnotationColor,
    points: Vec<[f64; 2]>,
}

impl Stroke {
    fn new(page: u32, kind: AnnotationKind, color: AnnotationColor, point: [f64; 2]) -> Self {
        Self {
            page,
            kind,
            color,
            points: vec![point],
        }
    }

    fn extend(&mut self, point: [f64; 2]) {
        if self.points.last() == Some(&point) {
            return;
        }
        if self.kind != AnnotationKind::Ink && self.points.len() == 2 {
            self.points[1] = point;
            return;
        }
        if self.points.len() == MAX_STUDY_POINTS {
            let mut index = 0;
            self.points.retain(|_| {
                let keep = index % 2 == 0;
                index += 1;
                keep
            });
        }
        self.points.push(point);
    }
}

fn page_point(paper: egui::Rect, rotation: PageRotation, point: egui::Pos2) -> [f64; 2] {
    let point = paper.clamp(point) - paper.min;
    rotation.unrotate_point([
        f64::from(point.x / paper.width()),
        f64::from(point.y / paper.height()),
    ])
}

pub(super) struct PageInput {
    pub tool: Tool,
    pub color: AnnotationColor,
    pub page: u32,
    pub paper: egui::Rect,
    pub rotation: PageRotation,
}

impl Pane {
    pub fn cancel_gesture(&mut self) -> bool {
        self.intent = None;
        self.stroke.take().is_some()
    }

    pub fn has_unsaved(&self) -> bool {
        self.save.is_some()
    }

    pub fn discard(&mut self) {
        self.save = None;
    }

    pub fn keep_visible(&mut self, pages: &[u32]) {
        if self
            .stroke
            .as_ref()
            .is_some_and(|stroke| !pages.contains(&stroke.page))
        {
            self.cancel_gesture();
        }
    }

    pub fn page_input(&mut self, ui: &egui::Ui, input: PageInput) {
        let PageInput {
            tool,
            color,
            page,
            paper,
            rotation,
        } = input;
        let visible = paper.intersect(ui.clip_rect());
        if !visible.is_positive() || self.save.is_some() {
            return;
        }
        if tool == Tool::Note {
            if ui
                .interact(
                    visible,
                    ui.id().with(("study-page-note", page)),
                    egui::Sense::click(),
                )
                .clicked_by(egui::PointerButton::Primary)
            {
                self.intent = Some(Intent::PageNote(page));
            }
            return;
        }
        let Some(kind) = tool.drawing() else { return };
        let response = ui.interact(
            visible,
            ui.id().with(("study-draw", page)),
            egui::Sense::drag(),
        );
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(start) = ui.input(|input| input.pointer.press_origin())
        {
            self.stroke = Some(Stroke::new(
                page,
                kind,
                color,
                page_point(paper, rotation, start),
            ));
        }
        if let Some(stroke) = self.stroke.as_mut().filter(|stroke| stroke.page == page) {
            if let Some(pointer) = ui.input(|input| input.pointer.interact_pos()) {
                stroke.extend(page_point(paper, rotation, pointer));
            }
            if response.drag_stopped_by(egui::PointerButton::Primary) {
                let stroke = self.stroke.take().unwrap();
                if stroke.points.len() >= 2
                    && (stroke.kind != AnnotationKind::Rectangle
                        || (stroke.points[0][0] != stroke.points[1][0]
                            && stroke.points[0][1] != stroke.points[1][1]))
                {
                    self.intent = Some(Intent::Drawing(stroke));
                }
            } else {
                annotations::paint::paint_drawing(
                    ui,
                    paper,
                    rotation,
                    stroke.kind,
                    &stroke.points,
                    stroke.color,
                );
            }
        }
        response.on_hover_cursor(egui::CursorIcon::Crosshair);
    }

    fn accept(&mut self, reply: &AnnotationReply) {
        if !self
            .save
            .as_ref()
            .and_then(|save| save.pending.as_ref())
            .is_some_and(|token| token.same_request(&reply.cancel))
        {
            return;
        }
        match &reply.result {
            Ok(_) => self.save = None,
            Err(error) => {
                let save = self.save.as_mut().unwrap();
                save.pending = None;
                save.error = Some(error.to_string());
            }
        }
    }
}

enum Draft {
    Marks(Arc<AnnotationDraftBatch>),
    Drawing(Arc<AnnotationDraft>),
}

struct Save {
    document: String,
    draft: Draft,
    pending: Option<Cancellation>,
    error: Option<String>,
}

impl reader_pane::ReaderPane {
    fn submit_study_save(&mut self, runtime: &Runtime, governor: &MemoryGovernor) {
        let Some(save) = &mut self.study.save else {
            return;
        };
        if save.pending.is_some() || self.annotations.writing() || self.annotations.editing() {
            return;
        }
        self.annotations.bind(&save.document);
        self.annotations.cancel_query();
        let mutation = match &save.draft {
            Draft::Marks(batch) => AnnotationMutation::CreateBatch {
                batch: Arc::clone(batch),
                confirm_page_note: false,
                tags: Vec::new(),
            },
            Draft::Drawing(draft) => AnnotationMutation::Create {
                draft: Arc::clone(draft),
                confirm_page_note: false,
                tags: Vec::new(),
            },
        };
        self.annotations
            .command(runtime, governor, Some(mutation), 0);
        save.pending = self.annotations.study_write_token();
        if save.pending.is_some() {
            save.error = None;
        } else {
            save.error = self.annotations.study_write_error().map(str::to_owned);
        }
    }

    fn queue_study_save(&mut self, document: String, draft: Draft) {
        self.study.save = Some(Save {
            document,
            draft,
            pending: None,
            error: None,
        });
    }
}

impl YamantakaApp {
    pub fn preview_study(&mut self) {
        if self.demo && self.capture.is_some() {
            self.study.active = true;
            self.study.preview = 1;
            self.study.tool = Tool::Arrow;
            self.study.color = AnnotationColor::Violet;
        }
    }

    pub fn preview_study_selection(&mut self) {
        if self.demo && self.capture.is_some() {
            self.study.preview = 3;
            self.preview_selection(false);
        }
    }

    pub(super) fn study_passive_preview(&self) -> bool {
        self.study.preview == 3
    }

    pub(super) fn study_capture_ready(&self) -> bool {
        matches!(self.study.preview, 0 | 3)
            || (self.study.preview == 2
                && !self.reader.study.has_unsaved()
                && self.reader.annotation_paint.ready())
    }

    pub(super) fn study_pane(&self, reference: bool) -> &reader_pane::ReaderPane {
        if reference && let Some(view) = &self.reference {
            &view.pane
        } else {
            &self.reader
        }
    }

    fn study_pane_mut(&mut self, reference: bool) -> &mut reader_pane::ReaderPane {
        if reference && let Some(view) = &mut self.reference {
            &mut view.pane
        } else {
            &mut self.reader
        }
    }

    pub(super) fn toggle_study(&mut self) {
        self.study.active = !self.study.active;
        self.study.tool = Tool::Select;
        self.reader.viewport.scroll_anchor = true;
        self.reader.study.cancel_gesture();
        if let Some(reference) = &mut self.reference {
            reference.pane.viewport.scroll_anchor = true;
            reference.pane.study.cancel_gesture();
        }
    }

    pub(super) fn study_tool(&self, reference: bool) -> Tool {
        if self.study.active
            && !self.reader_focus
            && !self.reader_chrome_blocked
            && reference == self.reading_focus.reference()
            && !self.study_pane(reference).annotations.editing()
        {
            self.study.tool
        } else {
            Tool::Select
        }
    }

    pub(super) fn study_color(&self) -> AnnotationColor {
        self.study.color
    }

    pub(super) fn study_escape(&mut self, ctx: &egui::Context, modal: bool) {
        let reference = self.reading_focus.reference();
        if reference {
            self.reader.study.cancel_gesture();
        } else if let Some(view) = &mut self.reference {
            view.pane.study.cancel_gesture();
        }
        if !ctx.input(|input| input.focused) || modal || !matches!(self.surface, Surface::Reader) {
            self.reader.study.cancel_gesture();
            if let Some(reference) = &mut self.reference {
                reference.pane.study.cancel_gesture();
            }
            return;
        }
        if !self.study.active
            || self.study_pane(reference).annotations.editing()
            || !ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            return;
        }
        if egui::Popup::is_any_open(ctx) {
            egui::Popup::close_all(ctx);
        } else {
            let pane = self.study_pane_mut(reference);
            if pane.study.cancel_gesture() || pane.selection.cancel_pointer_gesture() {
                ctx.stop_dragging();
            } else if pane.selection.has_passage() {
                pane.selection.dismiss_passage();
            } else {
                self.toggle_study();
            }
        }
    }

    pub(super) fn study_toolbar(&mut self, ui: &mut egui::Ui, p: Palette) {
        self.study.bounds = None;
        let reference = self.reference.is_some() && self.reading_focus.reference();
        let visible = self.study.active && !self.reader_focus;
        let save = self.study_pane(reference).study.save.as_ref();
        let saving = save.is_some_and(|save| save.pending.is_some());
        let save_error = save
            .filter(|save| save.pending.is_none())
            .map(|save| save.error.clone().unwrap_or_default());
        if !visible && save_error.is_none() {
            return;
        }
        if visible && let Some((document, _)) = self.study_pane(reference).active.as_ref() {
            let document = document.id.clone();
            let pane = if reference {
                &mut self.reference.as_mut().unwrap().pane
            } else {
                &mut self.reader
            };
            pane.annotations
                .prepare(&self.runtime, &self.governor, &document);
        }
        let en = self.english;
        let tr = |es, en_label| if en { en_label } else { es };
        let mut next_tool = None;
        let mut history = None;
        let mut hide = false;
        let mut exit = false;
        let mut retry = false;
        let mut discard = false;
        let pane = self.study_pane(reference);
        let can_undo = pane.annotations.history_available(false);
        let can_redo = pane.annotations.history_available(true);
        let hidden = pane.study.hidden;
        let comparison = self.reference.is_some();
        let bar = egui::Frame::NONE
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                if visible {
                    ui.horizontal(|ui| {
                        let wide = ui.available_width() >= if comparison { 1190.0 } else { 1100.0 };
                        if comparison && ui.available_width() >= 530.0 {
                            ui.label(
                                RichText::new(if reference {
                                    tr("Referencia", "Reference")
                                } else {
                                    tr("Principal", "Main")
                                })
                                .color(p.secondary),
                            );
                        }
                        if wide {
                            for tool in Tool::ALL {
                                if tool == Tool::Pen {
                                    ui.separator();
                                }
                                if theme::control(
                                    ui,
                                    tool.label(en),
                                    tool.icon(),
                                    self.study.tool == tool,
                                    false,
                                    p,
                                )
                                .clicked()
                                {
                                    next_tool = Some(tool);
                                }
                            }
                        } else {
                            let label = format!(
                                "{} · {}",
                                tr("Herramientas", "Tools"),
                                self.study.tool.label(en)
                            );
                            let title = if ui.available_width() < 285.0 {
                                self.study.tool.label(en)
                            } else {
                                &label
                            };
                            let button = theme::control(
                                ui,
                                title,
                                self.study.tool.icon().or(Some(theme::Icon::Down)),
                                false,
                                false,
                                p,
                            );
                            button.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::Button,
                                    ui.is_enabled(),
                                    &label,
                                )
                            });
                            theme::record_control_bounds(ui, &label, button.rect);
                            egui::Popup::menu(&button).id(tool_menu()).show(|ui| {
                                egui::ScrollArea::vertical()
                                    .max_height((ui.ctx().content_rect().height() - 76.0).max(92.0))
                                    .show(ui, |ui| {
                                        for tool in Tool::ALL {
                                            if theme::control(
                                                ui,
                                                tool.label(en),
                                                tool.icon(),
                                                self.study.tool == tool,
                                                false,
                                                p,
                                            )
                                            .clicked()
                                            {
                                                next_tool = Some(tool);
                                                ui.close();
                                            }
                                        }
                                        ui.separator();
                                        history = history_controls(ui, tr, can_undo, can_redo, p);
                                        hide |= marks_control(ui, tr, hidden, p);
                                    });
                            });
                        }
                        color_control(ui, en, &mut self.study.color, p);
                        if wide {
                            history = history_controls(ui, tr, can_undo, can_redo, p);
                            hide = marks_control(ui, tr, hidden, p);
                        }
                        exit = theme::icon_button(
                            ui,
                            tr("Salir de Estudio", "Exit Study"),
                            theme::Icon::Close,
                            p,
                        )
                        .clicked();
                        if saving && ui.available_width() >= 145.0 {
                            ui.label(
                                RichText::new(tr("Guardando marca…", "Saving mark…"))
                                    .color(p.secondary),
                            );
                        }
                    });
                }
                if let Some(error) = save_error {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(tr(
                            "No se pudo guardar la marca.",
                            "The mark could not be saved.",
                        ))
                        .on_hover_text(error);
                        retry =
                            theme::navigation(ui, tr("Reintentar", "Retry"), false, p).clicked();
                        discard =
                            theme::navigation(ui, tr("Descartar", "Discard"), false, p).clicked();
                    });
                }
            });
        self.study.bounds = Some(bar.response.rect);
        if exit {
            self.toggle_study();
        }
        if hide {
            self.study_pane_mut(reference).study.hidden = !hidden;
        }
        if let Some(tool) = next_tool {
            self.study.tool = tool;
            let pane = self.study_pane_mut(reference);
            pane.study.cancel_gesture();
            if tool == Tool::Note && pane.selection.has_passage() {
                self.apply_study_action(ui.ctx(), reference, selection::SelectionAction::Annotate);
            } else {
                pane.selection.dismiss_passage();
            }
        }
        if let Some(redo) = history {
            self.apply_notes_action(
                reference,
                NotesAction::Change(
                    if redo {
                        AnnotationMutation::Redo
                    } else {
                        AnnotationMutation::Undo
                    },
                    0,
                ),
            );
        }
        if discard {
            self.study_pane_mut(reference).study.discard();
        }
        if retry {
            self.submit_study_save(reference);
        }
    }

    pub(super) fn study_after_viewport(&mut self, ui: &mut egui::Ui, p: Palette) {
        if self.study.preview == 1 && self.reader.annotation_paint.ready() {
            self.study.preview = 2;
            self.apply_study_intent(
                false,
                Intent::Drawing(Stroke {
                    page: self.reader.page,
                    kind: AnnotationKind::Arrow,
                    color: self.study.color,
                    points: vec![[0.68, 0.23], [0.56, 0.135]],
                }),
            );
        }
        let reference = self.reference.is_some() && self.reading_focus.reference();
        let tool = self.study_tool(reference);
        let color = self.study.color;
        let en = self.english;
        let allowed = !self.reader_chrome_blocked && !self.reader_focus;
        let pane = self.study_pane_mut(reference);
        let completed = pane.selection.take_completed_pointer();
        let intent = pane.study.intent.take();
        if !allowed || pane.annotations.editing() {
            return;
        }
        let action = if completed {
            match tool {
                Tool::Highlight => Some(selection::SelectionAction::Highlight),
                Tool::Underline => Some(selection::SelectionAction::Underline),
                Tool::Note => Some(selection::SelectionAction::Annotate),
                _ => None,
            }
        } else {
            None
        };
        let action = action.or_else(|| {
            tool.drawing()
                .is_none()
                .then(|| pane.selection.context_actions(ui, en, color, p))
                .flatten()
        });
        if reference && let Some(view) = &mut self.reference {
            view.input_layers
                .extend(view.pane.selection.context_layers());
        }
        if let Some(action) = action {
            self.apply_study_action(ui.ctx(), reference, action);
        } else if let Some(intent) = intent {
            self.apply_study_intent(reference, intent);
        }
    }

    fn apply_study_action(
        &mut self,
        ctx: &egui::Context,
        reference: bool,
        action: selection::SelectionAction,
    ) {
        use selection::SelectionAction;
        match action {
            SelectionAction::Copy => self.copy_selection(ctx, reference),
            SelectionAction::Color(color) => self.study.color = color,
            SelectionAction::Annotate => self.study_note(reference, None, AnnotationKind::TextNote),
            SelectionAction::Highlight | SelectionAction::Underline => {
                let kind = if matches!(action, SelectionAction::Highlight) {
                    AnnotationKind::Highlight
                } else {
                    AnnotationKind::Underline
                };
                let pane = self.study_pane(reference);
                if pane.study.has_unsaved() || pane.annotations.editing() {
                    return;
                }
                let Some((document, _)) = &pane.active else {
                    return;
                };
                let result =
                    pane.selection
                        .annotation_source(&self.governor)
                        .and_then(|snapshot| {
                            AnnotationDraftBatch::from_selection(
                                AnnotationSource {
                                    document_id: &document.id,
                                    fingerprint: &document.fingerprint,
                                    rotation: pane
                                        .reader_preferences
                                        .as_ref()
                                        .map_or(PageRotation::Deg0, |prefs| prefs.page_rotation),
                                },
                                snapshot.selection(),
                                snapshot.pages(),
                                kind,
                                self.study.color,
                                None,
                                &self.governor,
                            )
                        });
                match result {
                    Ok(batch) if batch.requires_page_note_confirmation() => {
                        self.study_note(reference, None, kind)
                    }
                    Ok(batch) => {
                        let id = document.id.clone();
                        let pane = self.study_pane_mut(reference);
                        pane.queue_study_save(id, Draft::Marks(Arc::new(batch)));
                        pane.selection.dismiss_passage();
                        self.submit_study_save(reference);
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
        }
    }

    fn study_note(&mut self, reference: bool, page: Option<u32>, kind: AnnotationKind) {
        let color = self.study.color;
        let governor = self.governor.clone();
        let pane = self.study_pane_mut(reference);
        if let Some(page) = page {
            pane.begin_page_note_at(page, color);
        } else {
            pane.annotate_selection_as(&governor, kind, color);
        }
        if pane.annotations.editing() {
            pane.selection.dismiss_passage();
            if reference && let Some(view) = &mut self.reference {
                view.navigator_requested = true;
            }
        }
    }

    fn apply_study_intent(&mut self, reference: bool, intent: Intent) {
        match intent {
            Intent::PageNote(page) => {
                self.study_note(reference, Some(page), AnnotationKind::PageComment)
            }
            Intent::Drawing(stroke) => {
                let pane = self.study_pane(reference);
                let Some((document, _)) = &pane.active else {
                    return;
                };
                let id = document.id.clone();
                let draft = StudyGeometry::new(stroke.kind, stroke.points).and_then(|geometry| {
                    AnnotationDraft::drawing(
                        AnnotationSource {
                            document_id: &id,
                            fingerprint: &document.fingerprint,
                            rotation: pane
                                .reader_preferences
                                .as_ref()
                                .map_or(PageRotation::Deg0, |prefs| prefs.page_rotation),
                        },
                        stroke.page,
                        stroke.color,
                        geometry,
                        &self.governor,
                    )
                });
                match draft {
                    Ok(draft) => {
                        let pane = self.study_pane_mut(reference);
                        pane.study.hidden = false;
                        pane.queue_study_save(id, Draft::Drawing(Arc::new(draft)));
                        self.submit_study_save(reference);
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
        }
    }

    fn submit_study_save(&mut self, reference: bool) {
        let blocked = self.global_notes.writing()
            || self.library_tags.writing()
            || self.study_pane(!reference).annotations.writing() && self.reference.is_some();
        let pane = if reference {
            &mut self.reference.as_mut().unwrap().pane
        } else {
            &mut self.reader
        };
        if !blocked {
            pane.submit_study_save(&self.runtime, &self.governor);
        } else if let Some(save) = &mut pane.study.save {
            save.error = Some(
                if self.english {
                    "Another annotation is being saved."
                } else {
                    "Hay otra anotación guardándose."
                }
                .into(),
            );
        }
    }

    pub(super) fn accept_study_reply(&mut self, reply: &AnnotationReply) {
        self.reader.study.accept(reply);
        if let Some(reference) = &mut self.reference {
            reference.pane.study.accept(reply);
        }
    }
}

fn history_controls(
    ui: &mut egui::Ui,
    tr: impl Fn(&'static str, &'static str) -> &'static str,
    undo: bool,
    redo: bool,
    p: Palette,
) -> Option<bool> {
    let undo = ui
        .add_enabled_ui(undo, |ui| {
            theme::navigation(ui, tr("Deshacer", "Undo"), false, p)
        })
        .inner;
    let redo = ui
        .add_enabled_ui(redo, |ui| {
            theme::navigation(ui, tr("Rehacer", "Redo"), false, p)
        })
        .inner;
    if undo.clicked() || redo.clicked() {
        Some(redo.clicked())
    } else {
        None
    }
}

fn marks_control(
    ui: &mut egui::Ui,
    tr: impl Fn(&'static str, &'static str) -> &'static str,
    hidden: bool,
    p: Palette,
) -> bool {
    let label = if hidden {
        tr("Mostrar marcas", "Show marks")
    } else {
        tr("Ocultar marcas", "Hide marks")
    };
    theme::navigation(ui, label, hidden, p).clicked()
}

pub(super) fn color_control(ui: &mut egui::Ui, en: bool, color: &mut AnnotationColor, p: Palette) {
    let label = annotations::panel::color_label(*color, en);
    let button = theme::icon_button(ui, &format!("Color · {label}"), theme::Icon::Down, p);
    ui.painter().circle_filled(
        button.rect.center() - egui::vec2(10.0, 0.0),
        4.0,
        annotations::panel::mark_color(*color),
    );
    egui::Popup::menu(&button).id(color_menu()).show(|ui| {
        for choice in [
            AnnotationColor::Yellow,
            AnnotationColor::Green,
            AnnotationColor::Blue,
            AnnotationColor::Rose,
            AnnotationColor::Violet,
        ] {
            let label = annotations::panel::color_label(choice, en);
            let response = theme::navigation(ui, label, *color == choice, p);
            if response.clicked() {
                *color = choice;
                ui.close();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stroke_stays_on_its_start_page_and_keeps_the_endpoint() {
        let paper = egui::Rect::from_min_max(egui::pos2(20.0, 40.0), egui::pos2(220.0, 440.0));
        assert_eq!(
            page_point(paper, PageRotation::Deg90, egui::pos2(250.0, 140.0)),
            [0.25, 0.0]
        );
        let mut stroke = Stroke::new(2, AnnotationKind::Ink, AnnotationColor::Violet, [0.0, 0.0]);
        for sample in 1..=4096 {
            stroke.extend([f64::from(sample) / 4096.0, 0.5]);
        }
        assert_eq!(stroke.page, 2);
        assert_eq!(stroke.points.first(), Some(&[0.0, 0.0]));
        assert_eq!(stroke.points.last(), Some(&[1.0, 0.5]));
        assert!(stroke.points.len() <= MAX_STUDY_POINTS);
    }

    #[test]
    fn a_pointer_drag_creates_one_drawing_when_released_over_another_page() {
        let ctx = egui::Context::default();
        let mut pane = Pane::default();
        let button = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        let mut frame = |events| {
            let mut frame = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    for page in 0..2 {
                        pane.page_input(
                            ui,
                            PageInput {
                                tool: Tool::Arrow,
                                color: AnnotationColor::Violet,
                                page,
                                paper: egui::Rect::from_min_size(
                                    egui::pos2(20.0, 20.0 + page as f32 * 120.0),
                                    egui::vec2(100.0, 100.0),
                                ),
                                rotation: PageRotation::Deg0,
                            },
                        );
                    }
                },
            );
            frame.textures_delta.clear();
        };
        let start = egui::pos2(45.0, 45.0);
        let end = egui::pos2(95.0, 160.0);
        frame(vec![egui::Event::PointerMoved(start)]);
        frame(vec![button(start, true)]);
        frame(vec![egui::Event::PointerMoved(end)]);
        frame(vec![button(end, false)]);
        let Some(Intent::Drawing(stroke)) = pane.intent.take() else {
            panic!("no drawing was created")
        };
        assert_eq!(stroke.page, 0);
        assert_eq!(stroke.points, [[0.25, 0.25], [0.75, 1.0]]);
    }
}
