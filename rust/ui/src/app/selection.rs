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
    fn copy(&self, governor: &MemoryGovernor) -> Result<SelectedText> {
        let range = self
            .range()
            .filter(|range| !range.is_empty())
            .ok_or_else(|| Failure::Invalid("No text passage is selected".into()))?;
        let count =
            usize::try_from(u64::from(range.end().page()) - u64::from(range.start().page()) + 1)
                .map_err(|_| Failure::MemoryBudget)?;
        let _lease = governor
            .try_reserve_one(yamantaka_domain::memory::Cost::new(
                yamantaka_domain::memory::MemoryCategory::Text,
                yamantaka_domain::memory::MemoryRepresentation::Cpu,
                count
                    .checked_mul(std::mem::size_of::<Arc<PdfPageText>>())
                    .ok_or(Failure::MemoryBudget)?,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let mut pages = Vec::new();
        pages
            .try_reserve_exact(count)
            .map_err(|_| Failure::MemoryBudget)?;
        for page in range.start().page()..=range.end().page() {
            pages.push(
                self.pages
                    .get(&page)
                    .and_then(text::TextState::payload)
                    .cloned()
                    .ok_or_else(|| Failure::Invalid("Selected page text is not ready".into()))?,
            );
        }
        range.copy_text(&pages, governor)
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
        if self.selection.enabled {
            self.selection.clear();
        } else if let Some((document, _)) = &self.active {
            self.selection.enabled = true;
            self.selection.document = Some(document.id.clone());
            if self.text.payload().is_some() || self.text.pending() || self.text.error().is_some() {
                self.selection
                    .pages
                    .insert(self.page, std::mem::take(&mut self.text));
            }
        }
    }
    pub(super) fn request_selection_pages(&mut self, document: &str, pages: &[u32]) {
        if !self.selection.enabled {
            return;
        }
        for page in self.selection.next_pages(pages) {
            let state = self.selection.pages.entry(page).or_default();
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
        if !self.selection.enabled {
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
                    self.selection
                        .range()
                        .is_some_and(|range| !range.is_empty()),
                    |ui| {
                        theme::navigation(
                            ui,
                            self.tr("Copiar pasaje", "Copy passage"),
                            false,
                            palette,
                        )
                    },
                )
                .inner
                .clicked();
            if theme::navigation(
                ui,
                self.tr("Borrar selección", "Clear selection"),
                false,
                palette,
            )
            .clicked()
            {
                self.selection.gesture = SelectionGesture::default();
            }
            if theme::navigation(ui, self.tr("Terminar", "Done"), false, palette).clicked() {
                self.selection.clear();
            }
        });
        let modal = self.password.is_some() || self.links.confirmation_open();
        copy |= self.selection.edit_commands(ui.ctx(), modal);
        if copy {
            match self.selection.copy(&self.governor) {
                Ok(text) => ui.ctx().copy_text(text.text().into()),
                Err(error) => self.error = Some(error.to_string()),
            }
        }
        if let Some(error) = self
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
                self.selection
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
        if !self.selection.enabled
            || self.selection.document.as_deref() != Some(document)
            || self.password.is_some()
            || self.links.confirmation_open()
        {
            return;
        }
        let Some(payload) = self
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
            &mut self.selection.gesture,
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
        if let Some(range) = self.selection.range()
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

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::{
        memory::{Cost, MemoryCategory, MemoryRepresentation},
        pdf_geometry::NormalizedPageRect,
        pdf_text::TextCharacter,
    };

    fn payload(governor: &MemoryGovernor, page: u32, rtl: bool) -> Arc<PdfPageText> {
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                4096,
            ))
            .unwrap();
        let characters = [0..1, 1..3, 3..4]
            .into_iter()
            .enumerate()
            .map(|(index, bytes)| {
                let left = if rtl {
                    0.7 - index as f64 * 0.1
                } else {
                    0.1 + index as f64 * 0.1
                };
                TextCharacter {
                    bytes,
                    region: Some(NormalizedPageRect::new([left, 0.2], [left + 0.05, 0.3]).unwrap()),
                }
            })
            .collect();
        Arc::new(PdfPageText::from_reserved(page, "aéb".into(), characters, lease).unwrap())
    }
    fn install(state: &mut SelectionState, governor: &MemoryGovernor, page: u32) -> Cancellation {
        let text = state.pages.entry(page).or_default();
        let cancel = text.begin("study", page).unwrap();
        text.accept("study", page, &cancel, Ok(payload(governor, page, false)));
        cancel
    }
    fn position(page: u32, character: usize) -> SelectionPosition {
        SelectionPosition::new(page, character).unwrap()
    }

    #[test]
    fn admission_is_bounded_and_document_change_cancels_pending_and_drops_text() {
        let governor = MemoryGovernor::default();
        let mut state = SelectionState {
            enabled: true,
            ..Default::default()
        };
        state.retain("study", &[0, 1, 2, 3]);
        assert_eq!(state.next_pages(&[0, 1, 2, 3]), vec![0, 1]);
        let cancel = state.pages.entry(0).or_default().begin("study", 0).unwrap();
        let other = state.pages.entry(1).or_default().begin("study", 1).unwrap();
        assert!(state.next_pages(&[0, 1, 2, 3]).is_empty());
        state.accept("other", 0, &cancel, Ok(payload(&governor, 0, false)));
        assert_eq!(governor.usage().text, 0);
        state.accept("study", 0, &cancel, Ok(payload(&governor, 0, false)));
        assert_eq!(state.next_pages(&[0, 1, 2, 3]), vec![2]);
        state.retain("second", &[0]);
        assert!(other.is_cancelled());
        assert_eq!(governor.usage().text, 0);
        state.accept("study", 0, &cancel, Ok(payload(&governor, 0, false)));
        assert!(state.pages.is_empty());
        assert_eq!(governor.usage().text, 0);
    }

    #[test]
    fn selected_pages_survive_scroll_and_copy_cannot_skip_pending_pages() {
        let governor = MemoryGovernor::default();
        let mut state = SelectionState {
            enabled: true,
            ..Default::default()
        };
        state.retain("study", &[0, 1]);
        install(&mut state, &governor, 0);
        state.gesture = SelectionGesture::Selected(position(0, 1), position(1, 1));
        assert!(state.copy(&governor).is_err());
        install(&mut state, &governor, 1);
        state.retain("study", &[]);
        assert_eq!(state.pages.len(), 2);
        assert_eq!(state.copy(&governor).unwrap().text(), "éb\na");
        state.gesture = SelectionGesture::Empty;
        state.retain("study", &[1]);
        assert_eq!(state.pages.len(), 1);
        state.clear();
        assert_eq!(governor.usage().text, 0);
    }

    #[test]
    fn caret_follows_native_progression_and_all_user_rotations() {
        let governor = MemoryGovernor::default();
        let paper = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(1000.0, 800.0));
        for rtl in [false, true] {
            let payload = payload(&governor, 0, rtl);
            for rotation in [
                PageRotation::Deg0,
                PageRotation::Deg90,
                PageRotation::Deg180,
                PageRotation::Deg270,
            ] {
                for (x, expected) in if rtl {
                    [(0.745, 0), (0.705, 1)]
                } else {
                    [(0.105, 0), (0.145, 1)]
                } {
                    let point = rotation.rotate_point([x, 0.25]);
                    let point = paper.min
                        + egui::vec2(
                            point[0] as f32 * paper.width(),
                            point[1] as f32 * paper.height(),
                        );
                    assert_eq!(caret_at(&payload, paper, rotation, point), Some(expected));
                }
            }
        }
    }

    #[test]
    fn production_pointer_drag_keeps_press_anchor_and_stops_extending_after_release() {
        let governor = MemoryGovernor::default();
        let payload = payload(&governor, 0, false);
        let ctx = egui::Context::default();
        let paper = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(600.0, 500.0));
        let start = egui::pos2(63.0, 125.0);
        let end = egui::pos2(205.0, 125.0);
        let mut gesture = SelectionGesture::Empty;
        let mut draw = |events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    selection_input(
                        ui,
                        &payload,
                        paper,
                        PageRotation::Deg0,
                        egui::Id::new("selection-test"),
                        "Select PDF text",
                        &mut gesture,
                    )
                },
            );
            output.textures_delta.clear();
        };
        draw(Vec::new());
        draw(vec![egui::Event::PointerMoved(start)]);
        draw(vec![egui::Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }]);
        draw(vec![egui::Event::PointerMoved(end)]);
        draw(vec![egui::Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        draw(vec![egui::Event::PointerMoved(start)]);
        let range = gesture.range().unwrap();
        assert_eq!(range.start(), position(0, 0));
        assert_eq!(range.end(), position(0, 3));
        assert!(matches!(gesture, SelectionGesture::Selected(..)));
        assert_eq!(range.page(&payload).unwrap().unwrap().text(), "aéb");
    }
}
