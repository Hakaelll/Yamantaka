use super::*;

pub(in crate::app) struct SelectionPage<'a> {
    pub document: &'a str,
    pub page: u32,
    pub paper: egui::Rect,
    pub rotation: PageRotation,
    pub english: bool,
}

impl SelectionState {
    pub(in crate::app) fn toolbar(
        &mut self,
        ui: &mut egui::Ui,
        english: bool,
        compact: bool,
        annotate: bool,
        p: Palette,
    ) -> Option<SelectionAction> {
        if !self.enabled {
            return None;
        }
        let tr = |es, en| if english { en } else { es };
        let mut action = None;
        ui.horizontal_wrapped(|ui| {
            if !compact && annotate {
                ui.label(
                    RichText::new(tr(
                        "Arrastra sobre el texto para seleccionar un pasaje.",
                        "Drag across text to select a passage.",
                    ))
                    .color(p.secondary),
                );
            }
            if ui
                .add_enabled_ui(self.range().is_some_and(|range| !range.is_empty()), |ui| {
                    theme::navigation(
                        ui,
                        if compact {
                            tr("Copiar", "Copy")
                        } else {
                            tr("Copiar pasaje", "Copy passage")
                        },
                        false,
                        p,
                    )
                })
                .inner
                .clicked()
            {
                action = Some(SelectionAction::Copy);
            }
            if annotate
                && theme::navigation(
                    ui,
                    if compact {
                        tr("Anotar", "Annotate")
                    } else {
                        tr("Anotar pasaje", "Annotate passage")
                    },
                    false,
                    p,
                )
                .clicked()
            {
                action = Some(SelectionAction::Annotate);
            }
            if theme::navigation(
                ui,
                if compact {
                    tr("Borrar", "Clear")
                } else {
                    tr("Borrar selección", "Clear selection")
                },
                false,
                p,
            )
            .clicked()
            {
                self.gesture = SelectionGesture::default();
            }
            if theme::navigation(ui, tr("Terminar", "Done"), false, p).clicked() {
                self.clear();
            }
        });
        if let Some(error) = self.pages.values().find_map(text::TextState::error) {
            ui.label(RichText::new(error).color(p.secondary));
            if theme::navigation(ui, tr("Reintentar selección", "Retry selection"), false, p)
                .clicked()
            {
                self.pages.retain(|_, state| state.error().is_none());
            }
        }
        ui.add_space(8.0);
        action
    }
    pub(in crate::app) fn show_page(
        &mut self,
        ui: &mut egui::Ui,
        page: SelectionPage<'_>,
        p: Palette,
        preview: Option<&mut SelectionPreview>,
    ) {
        if !self.enabled || self.document.as_deref() != Some(page.document) {
            return;
        }
        let Some(payload) = self
            .pages
            .get(&page.page)
            .and_then(text::TextState::payload)
        else {
            return;
        };
        if let Some(preview) = preview {
            preview.prepare(payload, page.paper, page.rotation, ui.ctx());
        }
        let id = ui.id().with(("select-pdf-text", page.document, page.page));
        self.focus_ids.insert(page.page, id);
        let label = if page.english {
            "Select PDF text. Double-click selects a word. Shift+arrows extends the selection; Ctrl+A selects the page; Ctrl+C copies the passage."
        } else {
            "Seleccionar texto del PDF. Doble clic selecciona una palabra. Mayús+flechas amplía la selección; Ctrl+A selecciona la página; Ctrl+C copia el pasaje."
        };
        selection_input(
            ui,
            payload,
            page.paper,
            page.rotation,
            id,
            label,
            &mut self.gesture,
        );
        if ui.memory(|memory| memory.has_focus(id)) {
            ui.painter().rect_stroke(
                page.paper,
                0,
                egui::Stroke::new(1.0, p.accent),
                egui::StrokeKind::Inside,
            );
        }
        if let Some(range) = self.range()
            && let Ok(Some(selected)) = range.page(payload)
        {
            for bounds in selected.regions() {
                ui.painter().rect_filled(
                    reader_viewport::page_region(page.paper, page.rotation, bounds),
                    1,
                    p.accent.gamma_multiply(0.25),
                );
            }
        }
    }
}

impl SelectionPreview {
    fn prepare(
        &mut self,
        payload: &PdfPageText,
        paper: egui::Rect,
        rotation: PageRotation,
        ctx: &egui::Context,
    ) {
        if *self != Self::Waiting {
            return;
        }
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
            *self = Self::Move(point(first, 0.15), point(last, 0.85));
        } else {
            *self = Self::Complete;
        }
        ctx.request_repaint();
    }
}
