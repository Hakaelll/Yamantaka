use super::*;
mod admission_preview;
mod comparison_preview;
mod keyboard;
mod view;
pub(super) use comparison_preview::ComparisonSelectionPreview;
use std::collections::BTreeMap;
pub(super) use view::SelectionPage;
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
    focus_ids: BTreeMap<u32, egui::Id>,
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
            self.focus_ids.clear();
            self.document = Some(document.into());
        }
        self.focus_ids.retain(|page, _| visible.contains(page));
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
        let selected = self.range().filter(|range| {
            !self
                .pages
                .range(range.start().page()..=range.end().page())
                .any(|(_, state)| state.error().is_some())
        });
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
    pub(super) fn copy(&self, governor: &MemoryGovernor) -> Result<SelectedText> {
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
        self.reader.toggle_selection();
    }
    pub(super) fn request_selection_pages(&mut self, document: &str, pages: &[u32]) {
        self.reader
            .selection
            .request(&self.runtime, document, pages);
    }
    pub(super) fn selection_toolbar(&mut self, ui: &mut egui::Ui, palette: Palette) {
        if self.reference.is_none() {
            self.selection_actions(ui, palette);
        }
    }
    pub(super) fn selection_actions(&mut self, ui: &mut egui::Ui, palette: Palette) {
        if self.reader.annotations.editing() {
            return;
        }
        let compact = ui.available_width() < 620.0;
        match self
            .reader
            .selection
            .toolbar(ui, self.english, compact, true, palette)
        {
            Some(SelectionAction::Copy) => {
                self.copy_selection(ui.ctx(), false);
                if self.reference.is_some() {
                    ui.close();
                }
            }
            Some(SelectionAction::Annotate) => {
                self.annotate_selection();
                if self.reference.is_some() {
                    ui.close();
                }
            }
            None => {}
        }
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
        if self.password.is_some() || self.link_confirmation_open() {
            return;
        }
        let preview = (self.demo && self.capture.is_some()).then_some(&mut self.selection_preview);
        self.reader.selection.show_page(
            ui,
            SelectionPage {
                document,
                page,
                paper,
                rotation,
                english: self.english,
            },
            palette,
            preview,
        );
    }
    pub(super) fn copy_selection(&mut self, ctx: &egui::Context, reference: bool) {
        let state = if reference {
            let Some(reference) = &self.reference else {
                return;
            };
            &reference.pane.selection
        } else {
            &self.reader.selection
        };
        match state.copy(&self.governor) {
            Ok(text) => {
                if self.demo && self.capture.is_some() {
                    self.selection_copy_preview =
                        Some((state.document.clone().unwrap(), text.text().into()));
                } else {
                    ctx.copy_text(text.text().into());
                }
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum SelectionAction {
    Copy,
    Annotate,
}

impl SelectionState {
    pub(super) fn toggle(&mut self, document: &str, page: u32, text: &mut text::TextState) {
        if self.enabled {
            self.clear();
            return;
        }
        self.enabled = true;
        self.document = Some(document.into());
        if text.is_for(document, page) {
            self.pages.insert(page, std::mem::take(text));
        }
    }
    pub(super) fn request(&mut self, runtime: &Runtime, document: &str, visible: &[u32]) {
        if !self.enabled {
            return;
        }
        self.retain(document, visible);
        for page in self.next_pages(visible) {
            self.pages
                .entry(page)
                .or_default()
                .request(runtime, document, page);
        }
    }
    pub(super) fn owns(&self, document: &str, page: u32, cancel: &Cancellation) -> bool {
        self.document.as_deref() == Some(document)
            && self
                .pages
                .get(&page)
                .is_some_and(|state| state.owns(document, page, cancel))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
    let visible = paper.intersect(ui.clip_rect());
    if !visible.is_positive() {
        return;
    }
    let response = ui.interact(visible, id, egui::Sense::click_and_drag());
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, ui.is_enabled(), label));
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }
    ui.memory_mut(|memory| {
        memory.set_focus_lock_filter(
            id,
            egui::EventFilter {
                horizontal_arrows: true,
                escape: true,
                ..Default::default()
            },
        )
    });
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
