use super::*;

impl OutlineState {
    pub(in crate::app) fn request(&mut self, runtime: &Runtime, document: &str) {
        if self.document.as_deref() != Some(document) {
            *self = OutlineState::default();
            self.document = Some(document.into());
        }
        if self.pending.is_some() || self.outline.is_some() || self.error.is_some() {
            return;
        }
        let cancel = Cancellation::default();
        match runtime.submit(Action::Outline(document.into(), cancel.clone())) {
            Ok(()) => self.pending = Some(cancel),
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn show(
        &mut self,
        ui: &mut egui::Ui,
        page: u32,
        p: Palette,
        english: bool,
        compact: bool,
        height: Option<f32>,
    ) -> Option<yamantaka_domain::pdf_destination::NavigationDestination> {
        let tr = |es, en| if english { en } else { es };
        if !compact {
            ui.label(RichText::new(tr("Explora el documento", "Explore this document")).size(22.0));
            ui.add_space(6.0);
        }
        if let Some(error) = self.error.clone() {
            ui.label(tr(
                "No se pudo cargar el índice.",
                "Contents could not be loaded.",
            ));
            ui.collapsing(tr("Detalles", "Details"), |ui| {
                ui.label(error);
            });
            if theme::navigation(ui, tr("Volver a intentar", "Try again"), false, p).clicked() {
                self.error = None;
                ui.ctx().request_repaint();
            }
            return None;
        }
        let Some(outline) = self.outline.clone() else {
            ui.label(
                RichText::new(tr("Cargando capítulos…", "Loading chapters…")).color(p.secondary),
            );
            return None;
        };
        if outline.entries().is_empty() {
            ui.label(
                RichText::new(tr(
                    "Este documento no tiene índice.",
                    "This document has no contents.",
                ))
                .color(p.secondary),
            );
            return None;
        }
        if !compact {
            ui.label(
                RichText::new(tr(
                    "Elige un capítulo para continuar leyendo.",
                    "Choose a chapter to continue reading.",
                ))
                .color(p.secondary),
            );
            ui.add_space(14.0);
        }
        let row_height = 40.0;
        let mut toggle = None;
        let mut destination = None;
        let mut scroll = egui::ScrollArea::vertical().id_salt("outline-chapters");
        if let Some(height) = height {
            scroll = scroll
                .max_height(height)
                .min_scrolled_height(height.min(320.0));
        }
        scroll.show_rows(ui, row_height, self.rows.len(), |ui, range| {
            for index in range {
                let row = self.rows[index];
                let entry = &outline.entries()[row.entry];
                ui.push_id(row.entry, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space((row.depth.min(6) * 12) as f32);
                        if row.children {
                            let collapsed = self.collapsed.contains(&row.entry);
                            let label = format!(
                                "{} {}",
                                tr(
                                    if collapsed { "Desplegar" } else { "Plegar" },
                                    if collapsed { "Expand" } else { "Collapse" }
                                ),
                                entry.title
                            );
                            if theme::icon_button(
                                ui,
                                &label,
                                if collapsed {
                                    theme::Icon::Plus
                                } else {
                                    theme::Icon::Minus
                                },
                                p,
                            )
                            .clicked()
                            {
                                toggle = Some(row.entry);
                            }
                        }
                        if !row.children {
                            ui.add_space(40.0 + ui.spacing().item_spacing.x);
                        }
                        let mut label = entry.title.clone();
                        if let Some(target) = entry.destination {
                            label.push_str(&format!(" · {}", u64::from(target.page) + 1));
                            let selected = page == target.page;
                            let response = ui
                                .add_sized(
                                    [ui.available_width().max(40.0), row_height],
                                    egui::Button::new(RichText::new(&label).color(if selected {
                                        p.ink
                                    } else {
                                        p.secondary
                                    }))
                                    .selected(selected)
                                    .stroke(egui::Stroke::NONE)
                                    .fill(if selected {
                                        p.raised
                                    } else {
                                        Color32::TRANSPARENT
                                    })
                                    .corner_radius(8)
                                    .wrap_mode(egui::TextWrapMode::Truncate),
                                )
                                .on_hover_text(&entry.title);
                            theme::record_control_bounds(ui, &label, response.rect);
                            if response.clicked() {
                                destination = Some(target);
                            }
                        } else {
                            ui.add_sized(
                                [ui.available_width().max(40.0), row_height],
                                egui::Label::new(RichText::new(&label).color(p.secondary))
                                    .truncate(),
                            )
                            .on_hover_text(&entry.title);
                        }
                    });
                });
            }
        });
        if let Some(index) = toggle {
            self.toggle(index);
            ui.ctx().request_repaint();
        }
        destination
    }
}
