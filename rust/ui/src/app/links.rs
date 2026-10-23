use super::*;
mod dialog;
mod regions;
use yamantaka_domain::{
    pdf_links::{ExternalLinkRejection, PdfLinkTarget, PdfPageLinks},
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
    pub(super) preview: LinkPreview,
    document: Option<String>,
    pages: HashMap<u32, LinkPhase>,
    confirmation: Option<ExternalConfirmation>,
}
struct ExternalConfirmation {
    payload: Arc<PdfPageLinks>,
    link_index: usize,
    focus_cancel: bool,
}

impl LinkState {
    pub(super) fn owns(&self, document: &str, page: u32, cancel: &Cancellation) -> bool {
        self.document.as_deref() == Some(document)
            && !cancel.is_cancelled()
            && matches!(self.pages.get(&page), Some(LinkPhase::Pending(current)) if current.same_request(cancel))
    }

    pub(super) fn confirmation_open(&self) -> bool {
        self.confirmation.is_some()
    }
    pub(super) fn ready(&self) -> bool {
        !self.pages.is_empty()
            && self
                .pages
                .values()
                .all(|phase| matches!(phase, LinkPhase::Ready(_)))
    }
    pub(super) fn capture_summary(&self) -> serde_json::Value {
        let count = |accept: fn(&PdfLinkTarget) -> bool| {
            self.pages
                .values()
                .filter_map(|phase| match phase {
                    LinkPhase::Ready(payload) => Some(
                        payload
                            .links()
                            .iter()
                            .filter(|link| accept(&link.target))
                            .count(),
                    ),
                    _ => None,
                })
                .sum::<usize>()
        };
        serde_json::json!({"keyboardActivated":self.preview==LinkPreview::Complete,"residentPages":self.pages.len(), "readyPages":self.pages.values().filter(|phase| matches!(phase, LinkPhase::Ready(_))).count(), "internalLinks":count(|target| matches!(target, PdfLinkTarget::Internal(_))), "externalLinks":count(|target| matches!(target, PdfLinkTarget::External(_))), "rejectedLinks":count(|target| matches!(target, PdfLinkTarget::Rejected(_))), "confirmationOpen":self.confirmation.is_some()})
    }

    pub(super) fn clear(&mut self) {
        self.confirmation = None;
        self.pages.clear();
        self.document = None;
    }
    fn retain(&mut self, document: &str, pages: &[u32]) {
        if self.document.as_deref() != Some(document) {
            self.clear();
            self.document = Some(document.into());
        }
        self.pages.retain(|page, _| pages.contains(page));
        if self
            .confirmation
            .as_ref()
            .is_some_and(|confirmation| !pages.contains(&confirmation.payload.page()))
        {
            self.confirmation = None;
        }
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
            self.reader.links.preview = if activate {
                LinkPreview::Activate
            } else {
                LinkPreview::Focus
            };
        }
    }
    pub(super) fn link_confirmation_open(&self) -> bool {
        self.reader.links.confirmation_open()
            || self
                .reference
                .as_ref()
                .is_some_and(|reference| reference.pane.links.confirmation_open())
    }
    pub(super) fn external_link_dialog(&mut self, ctx: &egui::Context) {
        let p = Palette::new(self.dark);
        let is_capture = self.demo && self.capture.is_some();
        let error = if self.reader.links.confirmation_open() {
            self.reader.links.dialog(ctx, p, self.english, is_capture)
        } else if let Some(reference) = &mut self.reference {
            reference
                .pane
                .links
                .dialog(ctx, p, self.english, is_capture)
        } else {
            None
        };
        if let Some(error) = error {
            self.error = Some(error);
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
