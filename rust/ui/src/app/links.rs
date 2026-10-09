use super::*;
use yamantaka_domain::{
    pdf_destination::NavigationDestination, pdf_links::PdfPageLinks,
    reader_preferences::PageRotation,
};

const MAX_PENDING_LINK_PAGES: usize = 2;

#[derive(Default, PartialEq, Eq)]
pub(super) enum LinkPreview {
    #[default]
    Off,
    Focus,
    Activate,
    KeyPending,
    Complete,
}

enum LinkPhase {
    Pending(Cancellation),
    Ready(Arc<PdfPageLinks>),
    Failed(String),
}
impl Drop for LinkPhase {
    fn drop(&mut self) {
        if let Self::Pending(cancel) = self {
            cancel.cancel();
        }
    }
}

#[derive(Default)]
pub(super) struct LinkState {
    document: Option<String>,
    pages: HashMap<u32, LinkPhase>,
}
impl LinkState {
    pub(super) fn ready(&self) -> bool {
        !self.pages.is_empty()
            && self
                .pages
                .values()
                .all(|phase| matches!(phase, LinkPhase::Ready(_)))
    }
    pub(super) fn capture_summary(&self) -> serde_json::Value {
        serde_json::json!({"residentPages":self.pages.len(), "readyPages":self.pages.values().filter(|phase| matches!(phase, LinkPhase::Ready(_))).count(), "internalLinks":self.pages.values().filter_map(|phase| match phase { LinkPhase::Ready(payload) => Some(payload.links().len()), _ => None }).sum::<usize>()})
    }
    pub(super) fn clear(&mut self) {
        self.pages.clear();
        self.document = None;
    }
    fn retain(&mut self, document: &str, pages: &[u32]) {
        if self.document.as_deref() != Some(document) {
            self.clear();
            self.document = Some(document.into());
        }
        self.pages.retain(|page, _| pages.contains(page));
    }
    fn next_pages(&self, visible: &[u32]) -> Vec<u32> {
        let pending = self
            .pages
            .values()
            .filter(|phase| matches!(phase, LinkPhase::Pending(_)))
            .count();
        let mut next = Vec::with_capacity(MAX_PENDING_LINK_PAGES);
        for &page in visible {
            if next.len() >= MAX_PENDING_LINK_PAGES.saturating_sub(pending) {
                break;
            }
            if !self.pages.contains_key(&page) && !next.contains(&page) {
                next.push(page);
            }
        }
        next
    }
    pub(super) fn accept(
        &mut self,
        document: &str,
        page: u32,
        cancel: &Cancellation,
        result: std::result::Result<Arc<PdfPageLinks>, String>,
    ) {
        if self.document.as_deref() != Some(document) || cancel.is_cancelled() {
            return;
        }
        let Some(LinkPhase::Pending(current)) = self.pages.get(&page) else {
            return;
        };
        if !current.same_request(cancel) {
            return;
        }
        let phase = match result {
            Ok(payload) if payload.page() == page => LinkPhase::Ready(payload),
            Ok(_) => LinkPhase::Failed("PDF links belong to a different page".into()),
            Err(error) => LinkPhase::Failed(error),
        };
        self.pages.insert(page, phase);
    }
}

impl YamantakaApp {
    pub fn preview_links(&mut self, activate: bool) {
        if self.demo && self.capture.is_some() {
            self.link_preview = if activate {
                LinkPreview::Activate
            } else {
                LinkPreview::Focus
            };
        }
    }
    pub(super) fn request_visible_links(&mut self, document: &str, pages: &[u32]) {
        self.links.retain(document, pages);
        for page in self.links.next_pages(pages) {
            if self.links.pages.contains_key(&page) {
                continue;
            }
            let cancel = Cancellation::default();
            match self
                .runtime
                .submit(Action::PageLinks(document.into(), page, cancel.clone()))
            {
                Ok(()) => {
                    self.links.pages.insert(page, LinkPhase::Pending(cancel));
                }
                Err(error) => {
                    self.links
                        .pages
                        .insert(page, LinkPhase::Failed(error.to_string()));
                }
            }
        }
    }

    pub(super) fn reader_link_regions(
        &mut self,
        ui: &mut egui::Ui,
        document: &str,
        page: u32,
        paper: egui::Rect,
        rotation: PageRotation,
        palette: Palette,
    ) {
        let Some(phase) = self.links.pages.get(&page) else {
            return;
        };
        let mut chosen: Option<NavigationDestination> = None;
        let mut retry = false;
        match phase {
            LinkPhase::Ready(payload) => {
                for link in payload.links() {
                    let label = format!(
                        "{} {}",
                        self.tr("Ir a la página", "Go to page"),
                        link.destination.page + 1
                    );
                    for (index, bounds) in link.regions.iter().enumerate() {
                        let rect = reader_viewport::page_region(paper, rotation, *bounds);
                        let response = link_control(
                            ui,
                            rect,
                            egui::Id::new((
                                "pdf-link",
                                document,
                                page,
                                link.annotation_index,
                                index,
                            )),
                            &label,
                            palette,
                        );
                        theme::record_control_bounds(ui, &label, rect);
                        if page == self.page
                            && link.annotation_index == payload.links()[0].annotation_index
                            && index == 0
                        {
                            if matches!(
                                self.link_preview,
                                LinkPreview::Focus | LinkPreview::Activate
                            ) {
                                response.request_focus();
                            }
                            if self.link_preview == LinkPreview::Activate && response.has_focus() {
                                self.link_preview = LinkPreview::KeyPending;
                                ui.ctx().request_repaint();
                            }
                        }
                        if response.clicked() {
                            chosen = Some(link.destination);
                            if self.link_preview != LinkPreview::Off {
                                self.link_preview = LinkPreview::Complete;
                            }
                        }
                        response
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(&label);
                    }
                }
            }
            LinkPhase::Failed(error) => {
                let rect = egui::Rect::from_min_size(
                    paper.min + egui::vec2(8.0, 8.0),
                    egui::vec2(paper.width().clamp(1.0, 220.0), 32.0),
                );
                let label = self.tr("Reintentar enlaces", "Retry links");
                let response = ui.put(rect, egui::Button::new(label));
                retry = response.clicked();
                response.on_hover_text(error);
            }
            LinkPhase::Pending(_) => {}
        }
        if retry {
            self.links.pages.remove(&page);
        }
        if let Some(destination) = chosen {
            self.reader_destination = Some((document.into(), destination));
            ui.ctx().request_repaint();
        }
    }
}

fn link_control(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    id: egui::Id,
    label: &str,
    palette: Palette,
) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Link, ui.is_enabled(), label));
    if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, 1, palette.accent.gamma_multiply(0.12));
        ui.painter().rect_stroke(
            rect,
            1,
            egui::Stroke::new(if response.has_focus() { 2.0 } else { 1.0 }, palette.accent),
            egui::StrokeKind::Inside,
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation};
    fn payload(governor: &MemoryGovernor, page: u32) -> Arc<PdfPageLinks> {
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                4096,
            ))
            .unwrap();
        Arc::new(PdfPageLinks::from_reserved(page, Vec::new(), 0, lease).unwrap())
    }
    #[test]
    fn cancelled_replaced_and_wrong_document_replies_do_not_enter_visible_state() {
        let governor = MemoryGovernor::default();
        let mut state = LinkState::default();
        state.retain("book", &[0]);
        let current = Cancellation::default();
        state.pages.insert(0, LinkPhase::Pending(current.clone()));
        state.accept("old", 0, &current, Ok(payload(&governor, 0)));
        state.accept(
            "book",
            0,
            &Cancellation::default(),
            Ok(payload(&governor, 0)),
        );
        assert!(matches!(state.pages.get(&0), Some(LinkPhase::Pending(_))));
        assert_eq!(governor.usage().text, 0);
        current.cancel();
        state.accept("book", 0, &current, Ok(payload(&governor, 0)));
        assert!(matches!(state.pages.get(&0), Some(LinkPhase::Pending(_))));
        assert_eq!(governor.usage().text, 0);
    }
    #[test]
    fn leaving_a_page_cancels_work_and_releases_ready_payloads() {
        let governor = MemoryGovernor::default();
        let mut state = LinkState::default();
        state.retain("book", &[0, 1]);
        let cancel = Cancellation::default();
        state.pages.insert(0, LinkPhase::Pending(cancel.clone()));
        state
            .pages
            .insert(1, LinkPhase::Ready(payload(&governor, 1)));
        assert!(governor.usage().text > 0);
        state.retain("book", &[2]);
        assert!(cancel.is_cancelled());
        assert!(state.pages.is_empty());
        assert_eq!(governor.usage().text, 0);
        state.accept("book", 0, &cancel, Ok(payload(&governor, 0)));
        assert!(state.pages.is_empty());
    }
    #[test]
    fn matching_success_failure_and_wrong_page_finish_only_their_request() {
        let governor = MemoryGovernor::default();
        let mut state = LinkState::default();
        state.retain("book", &[0]);
        for outcome in 0..3 {
            let cancel = Cancellation::default();
            state.pages.insert(0, LinkPhase::Pending(cancel.clone()));
            let result = match outcome {
                0 => Ok(payload(&governor, 0)),
                1 => Err("fixture failure".into()),
                _ => Ok(payload(&governor, 1)),
            };
            state.accept("book", 0, &cancel, result);
            if outcome == 0 {
                assert!(matches!(state.pages.get(&0), Some(LinkPhase::Ready(_))));
            } else {
                assert!(matches!(state.pages.get(&0), Some(LinkPhase::Failed(_))));
            }
            state.clear();
            assert_eq!(governor.usage().text, 0);
            state.retain("book", &[0]);
        }
    }
    #[test]
    fn document_switch_and_drop_cancel_all_pending_pages() {
        let mut state = LinkState::default();
        state.retain("book", &[0]);
        let tokens: Vec<_> = (0..20)
            .map(|page| {
                let cancel = Cancellation::default();
                state.pages.insert(page, LinkPhase::Pending(cancel.clone()));
                cancel
            })
            .collect();
        state.retain("book", &(0..20).collect::<Vec<_>>());
        assert_eq!(state.pages.len(), 20);
        assert!(tokens.iter().all(|cancel| !cancel.is_cancelled()));
        state.retain("other", &[0]);
        assert!(tokens.iter().all(Cancellation::is_cancelled));
        let cancel = Cancellation::default();
        state.pages.insert(0, LinkPhase::Pending(cancel.clone()));
        drop(state);
        assert!(cancel.is_cancelled());
    }
    #[test]
    fn bounded_pending_admission_eventually_covers_every_visible_page() {
        let governor = MemoryGovernor::default();
        let visible: Vec<_> = (0..40).collect();
        let mut state = LinkState::default();
        state.retain("book", &visible);
        while state.pages.len() < visible.len() {
            let next = state.next_pages(&visible);
            assert!(!next.is_empty() && next.len() <= MAX_PENDING_LINK_PAGES);
            for &page in &next {
                state
                    .pages
                    .insert(page, LinkPhase::Pending(Cancellation::default()));
            }
            assert!(state.next_pages(&visible).is_empty());
            for page in next {
                let Some(LinkPhase::Pending(cancel)) = state.pages.get(&page) else {
                    unreachable!();
                };
                let cancel = cancel.clone();
                state.accept("book", page, &cancel, Ok(payload(&governor, page)));
            }
        }
        assert!(state.next_pages(&visible).is_empty());
        assert!(
            visible
                .iter()
                .all(|page| matches!(state.pages.get(page), Some(LinkPhase::Ready(_))))
        );
        state.clear();
        assert_eq!(governor.usage().text, 0);
    }

    #[test]
    fn link_regions_activate_with_pointer_enter_and_space_but_not_the_gap() {
        for activation in ["pointer", "enter", "space", "gap"] {
            let context = egui::Context::default();
            let ids = [
                egui::Id::new("link-region-first"),
                egui::Id::new("link-region-second"),
            ];
            let rectangles = [
                egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(80.0, 30.0)),
                egui::Rect::from_min_size(egui::pos2(140.0, 20.0), egui::vec2(80.0, 30.0)),
            ];
            let mut clicked = false;
            let mut focus = false;
            let mut draw = |input: egui::RawInput, request_focus: bool| {
                let mut output = context.run_ui(input, |ui| {
                    for (index, rect) in rectangles.iter().enumerate() {
                        let response =
                            link_control(ui, *rect, ids[index], "Go to page 2", Palette::new(true));
                        if request_focus && index == 0 {
                            response.request_focus();
                        }
                        focus |= response.has_focus();
                        clicked |= response.clicked();
                    }
                });
                output.textures_delta.clear();
            };
            let keyboard = activation == "enter" || activation == "space";
            draw(egui::RawInput::default(), keyboard);
            if keyboard {
                draw(
                    egui::RawInput {
                        events: vec![egui::Event::Key {
                            key: if activation == "enter" {
                                egui::Key::Enter
                            } else {
                                egui::Key::Space
                            },
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        }],
                        ..Default::default()
                    },
                    false,
                );
            } else {
                let point = if activation == "gap" {
                    egui::pos2(120.0, 35.0)
                } else {
                    rectangles[0].center()
                };
                draw(
                    egui::RawInput {
                        events: vec![egui::Event::PointerMoved(point)],
                        ..Default::default()
                    },
                    false,
                );
                for pressed in [true, false] {
                    draw(
                        egui::RawInput {
                            events: vec![egui::Event::PointerButton {
                                pos: point,
                                button: egui::PointerButton::Primary,
                                pressed,
                                modifiers: egui::Modifiers::NONE,
                            }],
                            ..Default::default()
                        },
                        false,
                    );
                }
            }
            assert_eq!(clicked, activation != "gap", "{activation}");
            if keyboard {
                assert!(focus);
            }
        }
    }
}
