use super::*;

impl LinkState {
    pub(in crate::app) fn request_visible(
        &mut self,
        runtime: &Runtime,
        document: &str,
        pages: &[u32],
    ) {
        self.retain(document, pages);
        for page in self.next_pages(pages) {
            if self.pages.contains_key(&page) {
                continue;
            }
            let cancel = Cancellation::default();
            match runtime.submit(Action::PageLinks(document.into(), page, cancel.clone())) {
                Ok(()) => {
                    self.pages.insert(page, LinkPhase::Pending(cancel));
                }
                Err(error) => {
                    self.pages
                        .insert(page, LinkPhase::Failed(error.to_string()));
                }
            }
        }
    }
    pub(in crate::app) fn regions(
        &mut self,
        ui: &mut egui::Ui,
        page: u32,
        paper: egui::Rect,
        rotation: PageRotation,
        palette: Palette,
        english: bool,
    ) -> Result<Option<yamantaka_domain::pdf_destination::NavigationDestination>, String> {
        let tr = |es, en| if english { en } else { es };
        let document = self.document.as_deref().unwrap_or("");
        let Some(phase) = self.pages.get(&page) else {
            return Ok(None);
        };
        let mut chosen: Option<(Arc<PdfPageLinks>, usize)> = None;
        let mut retry = false;
        match phase {
            LinkPhase::Ready(payload) => {
                for (link_index, link) in payload.links().iter().enumerate() {
                    let label = match &link.target {
                        PdfLinkTarget::Internal(destination) => format!(
                            "{} {}",
                            tr("Ir a la página", "Go to page"),
                            destination.page + 1
                        ),
                        PdfLinkTarget::External(_) => {
                            tr("Abrir enlace externo…", "Open external link…").to_owned()
                        }
                        PdfLinkTarget::Rejected(_) => {
                            tr("Enlace no permitido", "Link not permitted").to_owned()
                        }
                    };
                    for (index, bounds) in link.regions.iter().enumerate() {
                        let rect = reader_viewport::page_region(paper, rotation, *bounds);
                        let response = link_control(
                            ui,
                            rect,
                            ui.id().with((
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
                        if link.annotation_index == payload.links()[0].annotation_index
                            && index == 0
                        {
                            if matches!(self.preview, LinkPreview::Focus | LinkPreview::Activate) {
                                response.request_focus();
                            }
                            if self.preview == LinkPreview::Activate && response.has_focus() {
                                self.preview = LinkPreview::KeyPending;
                                ui.ctx().request_repaint();
                            }
                        }
                        if response.clicked() {
                            chosen = Some((payload.clone(), link_index));
                            if self.preview != LinkPreview::Off {
                                self.preview = LinkPreview::Complete;
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
                let label = tr("Reintentar enlaces", "Retry links");
                let response = ui.put(rect, egui::Button::new(label));
                retry = response.clicked();
                response.on_hover_text(error);
            }
            LinkPhase::Pending(_) => {}
        }
        if retry {
            self.pages.remove(&page);
            ui.ctx().request_repaint();
        }
        if let Some((payload, link_index)) = chosen {
            ui.ctx().request_repaint();
            return self.activate(payload, link_index, english);
        }
        Ok(None)
    }
    pub(super) fn activate(
        &mut self,
        payload: Arc<PdfPageLinks>,
        link_index: usize,
        english: bool,
    ) -> Result<Option<yamantaka_domain::pdf_destination::NavigationDestination>, String> {
        let tr = |es, en| if english { en } else { es };
        match &payload.links()[link_index].target {
            PdfLinkTarget::Internal(destination) => Ok(Some(*destination)),
            PdfLinkTarget::Rejected(reason) => Err(match reason {
                ExternalLinkRejection::InvalidTarget => tr("Este enlace no es válido o usa un esquema no permitido. Solo se admiten http, https y mailto.", "This link is invalid or uses a scheme that is not permitted. Only http, https and mailto are allowed."),
                ExternalLinkRejection::TooLong => tr("Este enlace supera el límite de tamaño y no se puede abrir.", "This link exceeds the size limit and cannot be opened."),
            }.to_owned()),
            PdfLinkTarget::External(_) => {
                self.confirmation = Some(ExternalConfirmation { payload, link_index, focus_cancel: true });
                Ok(None)
            }
        }
    }
}
