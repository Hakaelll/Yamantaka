use super::*;

impl LinkState {
    pub(in crate::app) fn dialog(
        &mut self,
        ctx: &egui::Context,
        palette: Palette,
        english: bool,
        is_capture: bool,
    ) -> Option<String> {
        let tr = |es, en| if english { en } else { es };
        let mut error = None;
        let mut confirmation = self.confirmation.take()?;
        let PdfLinkTarget::External(uri) =
            &confirmation.payload.links()[confirmation.link_index].target
        else {
            return None;
        };
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
                    RichText::new(tr("Abrir enlace externo", "Open external link"))
                        .size(if compact { 20.0 } else { 24.0 })
                        .color(palette.ink),
                );
                ui.add_space(if compact { 4.0 } else { 12.0 });
                egui::ScrollArea::vertical()
                    .max_height((ctx.content_rect().height() - 144.0).max(48.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(tr(
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
                    let cancel = theme::navigation(ui, tr("Cancelar", "Cancel"), false, palette);
                    if confirmation.focus_cancel {
                        cancel.request_focus();
                        confirmation.focus_cancel = false;
                    }
                    cancelled = cancel.clicked();
                    approved = theme::control(
                        ui,
                        tr("Abrir", "Open"),
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
            return None;
        }
        if approved {
            // Native captures never dispatch to an external application.
            if is_capture {
                return None;
            }
            if let Err(failure) = yamantaka_windows::open_external_uri(uri) {
                error = Some(failure.to_string());
            } else {
                return None;
            }
        }
        self.confirmation = Some(confirmation);
        error
    }
}
