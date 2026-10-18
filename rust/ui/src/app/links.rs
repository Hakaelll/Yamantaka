use super::*;
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
        serde_json::json!({"residentPages":self.pages.len(), "readyPages":self.pages.values().filter(|phase| matches!(phase, LinkPhase::Ready(_))).count(), "internalLinks":count(|target| matches!(target, PdfLinkTarget::Internal(_))), "externalLinks":count(|target| matches!(target, PdfLinkTarget::External(_))), "rejectedLinks":count(|target| matches!(target, PdfLinkTarget::Rejected(_))), "confirmationOpen":self.confirmation.is_some()})
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
            self.link_preview = if activate {
                LinkPreview::Activate
            } else {
                LinkPreview::Focus
            };
        }
    }
    pub(super) fn external_link_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut confirmation) = self.links.confirmation.take() else {
            return;
        };
        let PdfLinkTarget::External(uri) =
            &confirmation.payload.links()[confirmation.link_index].target
        else {
            return;
        };
        let palette = Palette::new(self.dark);
        let compact = ctx.content_rect().height() < 420.0;
        let mut approved = false;
        let mut cancelled = false;
        let modal = egui::Modal::new(egui::Id::new("confirm-external-pdf-link"))
            .frame(
                egui::Frame::new()
                    .fill(palette.surface)
                    .stroke(egui::Stroke::new(1.0, palette.line))
                    .corner_radius(12)
                    .inner_margin(if compact { 12 } else { 24 }),
            )
            .show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width() - 48.0).clamp(160.0, 440.0));
                ui.label(
                    RichText::new(self.tr("Abrir enlace externo", "Open external link"))
                        .size(if compact { 20.0 } else { 24.0 })
                        .color(palette.ink),
                );
                ui.add_space(if compact { 4.0 } else { 12.0 });
                egui::ScrollArea::vertical()
                    .max_height((ctx.content_rect().height() - 144.0).max(48.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(self.tr(
                                "Windows abrirá este destino en otra aplicación.",
                                "Windows will open this destination in another application.",
                            ))
                            .color(palette.secondary),
                        );
                        ui.add_space(if compact { 4.0 } else { 12.0 });
                        ui.add(
                            egui::Label::new(RichText::new(uri.as_str()).color(palette.ink)).wrap(),
                        );
                    });
                ui.add_space(if compact { 4.0 } else { 16.0 });
                ui.horizontal_wrapped(|ui| {
                    let cancel =
                        theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, palette);
                    if confirmation.focus_cancel {
                        cancel.request_focus();
                        confirmation.focus_cancel = false;
                    }
                    cancelled = cancel.clicked();
                    approved = theme::control(
                        ui,
                        self.tr("Abrir", "Open"),
                        Some(theme::Icon::Right),
                        true,
                        false,
                        palette,
                    )
                    .clicked();
                });
            });
        cancelled |= modal.should_close();
        if cancelled {
            return;
        }
        if approved {
            // Native captures never dispatch to an external application.
            if self.demo && self.capture.is_some() {
                return;
            }
            if let Err(error) = yamantaka_windows::open_external_uri(uri) {
                self.error = Some(error.to_string());
            } else {
                return;
            }
        }
        self.links.confirmation = Some(confirmation);
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
        let mut chosen: Option<(Arc<PdfPageLinks>, usize)> = None;
        let mut retry = false;
        match phase {
            LinkPhase::Ready(payload) => {
                for (link_index, link) in payload.links().iter().enumerate() {
                    let label = match &link.target {
                        PdfLinkTarget::Internal(destination) => format!(
                            "{} {}",
                            self.tr("Ir a la página", "Go to page"),
                            destination.page + 1
                        ),
                        PdfLinkTarget::External(_) => self
                            .tr("Abrir enlace externo…", "Open external link…")
                            .to_owned(),
                        PdfLinkTarget::Rejected(_) => self
                            .tr("Enlace no permitido", "Link not permitted")
                            .to_owned(),
                    };
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
                            chosen = Some((payload.clone(), link_index));
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
        if let Some((payload, link_index)) = chosen {
            match &payload.links()[link_index].target {
                PdfLinkTarget::Internal(destination) => {
                    self.reader_destination = Some((document.into(), *destination))
                }
                PdfLinkTarget::Rejected(reason) => {
                    self.error = Some(match reason {
                        ExternalLinkRejection::InvalidTarget => self.tr("Este enlace no es válido o usa un esquema no permitido. Solo se admiten http, https y mailto.", "This link is invalid or uses a scheme that is not permitted. Only http, https and mailto are allowed."),
                        ExternalLinkRejection::TooLong => self.tr("Este enlace supera el límite de tamaño y no se puede abrir.", "This link exceeds the size limit and cannot be opened."),
                    }.to_owned());
                }
                PdfLinkTarget::External(_) => {
                    self.links.confirmation = Some(ExternalConfirmation {
                        payload,
                        link_index,
                        focus_cancel: true,
                    })
                }
            }
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
