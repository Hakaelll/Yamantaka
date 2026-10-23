use super::*;
mod keyboard;
use std::collections::BTreeMap;
use yamantaka_domain::{
    Failure, Result,
    pdf_text::PdfPageText,
    reader_preferences::PageRotation,
    text_selection::{SelectedText, SelectionPosition, TextSelection},
};

#[derive(Default)]
pub(super) struct SelectionState {
    pub(super) enabled: bool,
    document: Option<String>,
    pages: BTreeMap<u32, text::TextState>,
    gesture: SelectionGesture,
}
impl SelectionState {
    pub(super) fn extend_preview(&mut self, end_page: u32) -> bool {
        let Some(range) = self.range() else {
            return false;
        };
        let payload = self.pages.get(&end_page).and_then(text::TextState::payload);
        let ready = payload.is_some();
        let extent =
            SelectionPosition::new(end_page, payload.map_or(0, |page| page.characters().len()))
                .unwrap();
        self.gesture = SelectionGesture::Selected(range.start(), extent);
        ready
    }

    pub(super) fn capture_summary(&self) -> serde_json::Value {
        let range = self.range();
        serde_json::json!({"enabled": self.enabled, "residentPages":self.pages.len(), "readyPages":self.pages.values().filter(|state| state.payload().is_some()).count(), "start":range.map(|range| [range.start().page(), range.start().character() as u32]), "end":range.map(|range| [range.end().page(), range.end().character() as u32])})
    }
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
    pub(super) fn preview_ready(&self) -> bool {
        self.range().is_some_and(|range| !range.is_empty())
            || self
                .pages
                .values()
                .filter_map(text::TextState::payload)
                .any(|page| {
                    page.characters()
                        .iter()
                        .all(|character| character.region.is_none())
                })
    }
    pub(super) fn retain_visible(&mut self, document: &str, visible: &[u32]) {
        if self.enabled {
            self.retain(document, visible);
        }
    }
    pub(super) fn retry_page(&mut self, page: u32) {
        self.pages.remove(&page);
    }
    pub(super) fn text_state(&self, page: u32) -> Option<&text::TextState> {
        self.pages.get(&page)
    }
    fn range(&self) -> Option<TextSelection> {
        self.gesture.range()
    }
    fn retain(&mut self, document: &str, visible: &[u32]) {
        if self.document.as_deref() != Some(document) {
            self.pages.clear();
            self.gesture = SelectionGesture::default();
            self.document = Some(document.into());
        }
        let selection = self.range();
        self.pages.retain(|page, _| {
            visible.contains(page)
                || selection.is_some_and(|range| {
                    *page >= range.start().page() && *page <= range.end().page()
                })
        });
    }
    pub(super) fn accept(
        &mut self,
        document: &str,
        page: u32,
        cancel: &Cancellation,
        result: std::result::Result<Arc<PdfPageText>, String>,
    ) {
        if self.document.as_deref() == Some(document)
            && let Some(state) = self.pages.get_mut(&page)
        {
            state.accept(document, page, cancel, result);
        }
    }
    fn next_pages(&self, visible: &[u32]) -> Vec<u32> {
        let available =
            2usize.saturating_sub(self.pages.values().filter(|state| state.pending()).count());
        let mut pages = Vec::with_capacity(available);
        let selected = self.range();
        let needed = visible.iter().copied().chain(
            selected
                .into_iter()
                .flat_map(|range| range.start().page()..=range.end().page()),
        );
        for page in needed {
            if pages.len() == available {
                break;
            }
            if !self.pages.contains_key(&page) && !pages.contains(&page) {
                pages.push(page);
            }
        }
        pages
    }
    pub(super) fn annotation_source(
        &self,
        governor: &MemoryGovernor,
    ) -> Result<yamantaka_domain::text_selection::snapshot::TextSelectionSnapshot> {
        let range = self
            .range()
            .filter(|range| !range.is_empty())
            .ok_or_else(|| Failure::Invalid("Select a passage before adding a note".into()))?;
        let pages = (range.start().page()..=range.end().page()).map(|page| {
            self.pages
                .get(&page)
                .and_then(text::TextState::payload)
                .cloned()
                .ok_or_else(|| Failure::Invalid("Selected page text is not ready".into()))
        });
        yamantaka_domain::text_selection::snapshot::TextSelectionSnapshot::new(
            range, pages, governor,
        )
    }
    fn copy(&self, governor: &MemoryGovernor) -> Result<SelectedText> {
        Ok(self.annotation_source(governor)?.into_text())
    }
}

impl YamantakaApp {
    pub fn preview_selection(&mut self, keyboard: bool) {
        if self.demo && self.capture.is_some() {
            self.selection_preview = SelectionPreview::Waiting;
            self.selection_keyboard_preview = keyboard;
        }
    }
    pub(super) fn toggle_text_selection(&mut self) {
        if self.reader.selection.enabled {
            self.reader.selection.clear();
        } else if let Some((document, _)) = &self.reader.active {
            self.reader.selection.enabled = true;
            self.reader.selection.document = Some(document.id.clone());
            if self.reader.text.payload().is_some()
                || self.reader.text.pending()
                || self.reader.text.error().is_some()
            {
                self.reader
                    .selection
                    .pages
                    .insert(self.reader.page, std::mem::take(&mut self.reader.text));
            }
        }
    }
    pub(super) fn request_selection_pages(&mut self, document: &str, pages: &[u32]) {
        if !self.reader.selection.enabled {
            return;
        }
        for page in self.reader.selection.next_pages(pages) {
            let state = self.reader.selection.pages.entry(page).or_default();
            if let Some(cancel) = state.begin(document, page)
                && let Err(error) =
                    self.runtime
                        .submit(Action::TextGeometry(document.into(), page, cancel.clone()))
            {
                state.accept(document, page, &cancel, Err(error.to_string()));
            }
        }
    }
    pub(super) fn selection_toolbar(&mut self, ui: &mut egui::Ui, palette: Palette) {
        if !self.reader.selection.enabled || self.reader.annotations.editing() {
            return;
        }
        let compact = ui.ctx().content_rect().width() < 620.0;
        let mut copy = false;
        ui.horizontal_wrapped(|ui| {
            if !compact {
                ui.label(
                    RichText::new(self.tr(
                        "Arrastra sobre el texto para seleccionar un pasaje.",
                        "Drag across text to select a passage.",
                    ))
                    .color(palette.secondary),
                );
            }
            copy = ui
                .add_enabled_ui(
                    self.reader
                        .selection
                        .range()
                        .is_some_and(|range| !range.is_empty()),
                    |ui| {
                        theme::navigation(
                            ui,
                            if compact {
                                self.tr("Copiar", "Copy")
                            } else {
                                self.tr("Copiar pasaje", "Copy passage")
                            },
                            false,
                            palette,
                        )
                    },
                )
                .inner
                .clicked();
            if theme::navigation(
                ui,
                if compact {
                    self.tr("Anotar", "Annotate")
                } else {
                    self.tr("Anotar pasaje", "Annotate passage")
                },
                false,
                palette,
            )
            .clicked()
            {
                self.annotate_selection();
            }
            if theme::navigation(
                ui,
                if compact {
                    self.tr("Borrar", "Clear")
                } else {
                    self.tr("Borrar selección", "Clear selection")
                },
                false,
                palette,
            )
            .clicked()
            {
                self.reader.selection.gesture = SelectionGesture::default();
            }
            if theme::navigation(ui, self.tr("Terminar", "Done"), false, palette).clicked() {
                self.reader.selection.clear();
            }
        });
        let modal = self.password.is_some() || self.link_confirmation_open();
        copy |= self.reader.selection.edit_commands(ui.ctx(), modal);
        if copy {
            match self.reader.selection.copy(&self.governor) {
                Ok(text) => ui.ctx().copy_text(text.text().into()),
                Err(error) => self.error = Some(error.to_string()),
            }
        }
        if let Some(error) = self
            .reader
            .selection
            .pages
            .values()
            .find_map(text::TextState::error)
        {
            ui.label(RichText::new(error).color(palette.secondary));
            if theme::navigation(
                ui,
                self.tr("Reintentar selección", "Retry selection"),
                false,
                palette,
            )
            .clicked()
            {
                self.reader
                    .selection
                    .pages
                    .retain(|_, state| state.error().is_none());
            }
        }
        ui.add_space(8.0);
    }
    pub(super) fn selection_page(
        &mut self,
        ui: &mut egui::Ui,
        document: &str,
        page: u32,
        paper: egui::Rect,
        rotation: PageRotation,
        palette: Palette,
    ) {
        if !self.reader.selection.enabled
            || self.reader.selection.document.as_deref() != Some(document)
            || self.password.is_some()
            || self.link_confirmation_open()
        {
            return;
        }
        let Some(payload) = self
            .reader
            .selection
            .pages
            .get(&page)
            .and_then(text::TextState::payload)
        else {
            return;
        };
        if self.demo
            && self.capture.is_some()
            && self.selection_preview == SelectionPreview::Waiting
        {
            let mut bounds = payload
                .characters()
                .iter()
                .filter_map(|character| character.region);
            if let Some(first) = bounds.next() {
                let last = bounds.next_back().unwrap_or(first);
                let point = |bounds: yamantaka_domain::pdf_geometry::NormalizedPageRect,
                             fraction: f64| {
                    let min = bounds.min();
                    let max = bounds.max();
                    let point = rotation.rotate_point([
                        min[0] + (max[0] - min[0]) * fraction,
                        (min[1] + max[1]) * 0.5,
                    ]);
                    paper.min
                        + egui::vec2(
                            point[0] as f32 * paper.width(),
                            point[1] as f32 * paper.height(),
                        )
                };
                let start = point(first, 0.15);
                let end = point(last, 0.85);
                self.selection_preview = SelectionPreview::Move(start, end);
            } else {
                self.selection_preview = SelectionPreview::Complete;
            }
            ui.ctx().request_repaint();
        }
        selection_input(
            ui,
            payload,
            paper,
            rotation,
            egui::Id::new(("select-pdf-text", document, page)),
            self.tr("Seleccionar texto del PDF. Mayús+flechas amplía la selección; Ctrl+A selecciona la página; Ctrl+C copia el pasaje.", "Select PDF text. Shift+arrows extends the selection; Ctrl+A selects the page; Ctrl+C copies the passage."),
            &mut self.reader.selection.gesture,
        );
        if ui.memory(|memory| memory.has_focus(egui::Id::new(("select-pdf-text", document, page))))
        {
            ui.painter().rect_stroke(
                paper,
                0,
                egui::Stroke::new(1.0, palette.accent),
                egui::StrokeKind::Inside,
            );
        }
        if let Some(range) = self.reader.selection.range()
            && let Ok(Some(selected)) = range.page(payload)
        {
            for bounds in selected.regions() {
                ui.painter().rect_filled(
                    reader_viewport::page_region(paper, rotation, bounds),
                    1,
                    palette.accent.gamma_multiply(0.25),
                );
            }
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub(super) enum SelectionPreview {
    #[default]
    Off,
    Waiting,
    Move(egui::Pos2, egui::Pos2),
    Press(egui::Pos2, egui::Pos2),
    Drag(egui::Pos2),
    Release(egui::Pos2),
    KeyboardHome,
    KeyboardExtend,
    Complete,
}
impl SelectionPreview {
    pub(super) fn input(&mut self, input: &mut egui::RawInput, keyboard: bool) {
        let key = |key, modifiers| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        let button = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        *self = match *self {
            Self::Move(start, end) => {
                input.events.push(egui::Event::PointerMoved(start));
                Self::Press(start, end)
            }
            Self::Press(start, end) => {
                input.events.push(button(start, true));
                Self::Drag(end)
            }
            Self::Drag(end) => {
                input.events.push(egui::Event::PointerMoved(end));
                Self::Release(end)
            }
            Self::Release(end) => {
                input.events.push(button(end, false));
                if keyboard {
                    Self::KeyboardHome
                } else {
                    Self::Complete
                }
            }
            Self::KeyboardHome => {
                input
                    .events
                    .push(key(egui::Key::Home, egui::Modifiers::CTRL));
                Self::KeyboardExtend
            }
            Self::KeyboardExtend => {
                input
                    .events
                    .push(key(egui::Key::ArrowRight, egui::Modifiers::SHIFT));
                Self::Complete
            }
            other => other,
        };
    }
}

#[derive(Default, Clone, Copy)]
enum SelectionGesture {
    #[default]
    Empty,
    Dragging(SelectionPosition, SelectionPosition),
    Selected(SelectionPosition, SelectionPosition),
}
impl SelectionGesture {
    fn range(self) -> Option<TextSelection> {
        match self {
            Self::Empty => None,
            Self::Dragging(anchor, extent) | Self::Selected(anchor, extent) => {
                Some(TextSelection::new(anchor, extent))
            }
        }
    }
}

fn selection_input(
    ui: &mut egui::Ui,
    payload: &PdfPageText,
    paper: egui::Rect,
    rotation: PageRotation,
    id: egui::Id,
    label: &str,
    gesture: &mut SelectionGesture,
) {
    let response = ui.interact(paper, id, egui::Sense::click_and_drag());
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, ui.is_enabled(), label));
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }
    if response.clicked()
        && let Some(point) = response.interact_pointer_pos()
        && let Some(index) = caret_at(payload, paper, rotation, point)
        && let Ok(position) = SelectionPosition::new(payload.page(), index)
    {
        *gesture = SelectionGesture::Selected(position, position);
    }
    let down = ui.input(|input| input.pointer.primary_down());
    if !down && let SelectionGesture::Dragging(anchor, extent) = *gesture {
        *gesture = SelectionGesture::Selected(anchor, extent);
    }
    if response.drag_started()
        && let Some(origin) = ui.input(|input| input.pointer.press_origin())
        && let Some(index) = caret_at(payload, paper, rotation, origin)
        && let Ok(position) = SelectionPosition::new(payload.page(), index)
    {
        *gesture = SelectionGesture::Dragging(position, position);
    }
    if down
        && let Some(point) = ui.input(|input| input.pointer.interact_pos())
        && paper.contains(point)
        && let SelectionGesture::Dragging(anchor, _) = *gesture
        && let Some(index) = caret_at(payload, paper, rotation, point)
        && let Ok(extent) = SelectionPosition::new(payload.page(), index)
    {
        *gesture = SelectionGesture::Dragging(anchor, extent);
    }
    response
        .on_hover_cursor(egui::CursorIcon::Text)
        .on_hover_text(label);
}

fn caret_at(
    payload: &PdfPageText,
    paper: egui::Rect,
    rotation: PageRotation,
    point: egui::Pos2,
) -> Option<usize> {
    let rectangle = |index: usize| {
        payload
            .characters()
            .get(index)?
            .region
            .map(|region| reader_viewport::page_region(paper, rotation, region))
    };
    let (index, bounds) = payload
        .characters()
        .iter()
        .enumerate()
        .filter_map(|(index, _)| rectangle(index).map(|bounds| (index, bounds)))
        .min_by(|(_, a), (_, b)| {
            a.distance_to_pos(point)
                .total_cmp(&b.distance_to_pos(point))
        })?;
    let center = bounds.center();
    let direction = index
        .checked_sub(1)
        .and_then(rectangle)
        .map(|previous| center - previous.center())
        .into_iter()
        .chain(rectangle(index + 1).map(|next| next.center() - center))
        .filter(|direction| {
            direction.length_sq() > 0.0
                && direction.length() < bounds.height().max(bounds.width()) * 3.0
        })
        .min_by(|a, b| a.length_sq().total_cmp(&b.length_sq()))
        .unwrap_or_else(|| match rotation {
            PageRotation::Deg0 => egui::vec2(1.0, 0.0),
            PageRotation::Deg90 => egui::vec2(0.0, 1.0),
            PageRotation::Deg180 => egui::vec2(-1.0, 0.0),
            PageRotation::Deg270 => egui::vec2(0.0, -1.0),
        });
    Some(index + usize::from((point - center).dot(direction) >= 0.0))
}
