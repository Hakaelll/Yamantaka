use super::*;
use yamantaka_domain::{
    pdf_destination::NavigationDestination, pdf_links::PdfPageLinks,
    reader_preferences::PageRotation,
};

const MAX_VISIBLE_LINK_PAGES: usize = 16;

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
    pub(super) fn clear(&mut self) {
        self.pages.clear();
        self.document = None;
    }
    fn retain(&mut self, document: &str, pages: &[u32]) {
        if self.document.as_deref() != Some(document) {
            self.clear();
            self.document = Some(document.into());
        }
        self.pages.retain(|page, _| {
            pages
                .iter()
                .take(MAX_VISIBLE_LINK_PAGES)
                .any(|visible| visible == page)
        });
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
    pub(super) fn request_visible_links(&mut self, document: &str, pages: &[u32]) {
        self.links.retain(document, pages);
        for &page in pages.iter().take(MAX_VISIBLE_LINK_PAGES) {
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
                        let response = ui.interact(
                            rect,
                            egui::Id::new((
                                "pdf-link",
                                document,
                                page,
                                link.annotation_index,
                                index,
                            )),
                            egui::Sense::click(),
                        );
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Link,
                                ui.is_enabled(),
                                &label,
                            )
                        });
                        if response.hovered() || response.has_focus() {
                            ui.painter()
                                .rect_filled(rect, 1, palette.accent.gamma_multiply(0.12));
                            ui.painter().rect_stroke(
                                rect,
                                1,
                                egui::Stroke::new(
                                    if response.has_focus() { 2.0 } else { 1.0 },
                                    palette.accent,
                                ),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if response.clicked() {
                            chosen = Some(link.destination);
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
    fn document_switch_and_drop_cancel_pending_pages_and_visible_limit_is_enforced() {
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
        assert_eq!(state.pages.len(), MAX_VISIBLE_LINK_PAGES);
        assert!(tokens[16..].iter().all(Cancellation::is_cancelled));
        state.retain("other", &[0]);
        assert!(tokens.iter().all(Cancellation::is_cancelled));
        let cancel = Cancellation::default();
        state.pages.insert(0, LinkPhase::Pending(cancel.clone()));
        drop(state);
        assert!(cancel.is_cancelled());
    }
}
