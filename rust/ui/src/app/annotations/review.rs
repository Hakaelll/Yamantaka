use super::*;

impl AnnotationState {
    pub(in crate::app) fn reviewing(&self) -> bool {
        self.reviewing
    }

    pub(super) fn set_reviewing(&mut self, active: bool) {
        self.invalidate_records();
        self.reviewing = active;
        self.review_revealed = false;
    }

    pub(in crate::app) fn show_review(
        &mut self,
        ui: &mut egui::Ui,
        context: &NotesContext<'_>,
        p: Palette,
    ) -> Option<NotesAction> {
        let tr = |es, en| if context.english { en } else { es };
        let page = self.page.clone();
        let offset = page.as_ref().map_or(0, |page| page.offset);
        let busy = context.busy || self.pending.is_some();
        let narrow = ui.available_width() < 460.0;
        if narrow {
            ui.spacing_mut().item_spacing.y = 2.0;
        }
        let mut action = None;
        let header = ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled_ui(!self.writing(), |ui| {
                    theme::control(
                        ui,
                        tr("Volver a notas", "Back to notes"),
                        Some(theme::Icon::Left),
                        false,
                        false,
                        p,
                    )
                })
                .inner
                .clicked()
            {
                self.set_reviewing(false);
                action = Some(NotesAction::Read(0));
            }
            ui.label(
                RichText::new(tr("Repaso", "Review"))
                    .size(18.0)
                    .strong()
                    .color(p.ink),
            );
            if let Some(page) = &page {
                let remaining = match page.total {
                    1 => tr("1 pendiente", "1 pending").to_owned(),
                    total => format!("{total} {}", tr("pendientes", "pending")),
                };
                ui.label(RichText::new(remaining).size(13.0).color(p.secondary));
            }
            if self.history_available(false)
                && !busy
                && theme::navigation(ui, tr("Deshacer", "Undo"), false, p).clicked()
            {
                action = Some(NotesAction::Change(AnnotationMutation::Undo, offset));
            }
        });
        if action.is_some() {
            return action;
        }
        if !narrow {
            ui.separator();
        }

        if let Some(error) = &self.error {
            ui.label(tr(
                "No se pudo actualizar el repaso.",
                "The review could not be updated.",
            ));
            if theme::navigation(ui, tr("Recargar", "Reload"), false, p).clicked() {
                return Some(NotesAction::Read(offset));
            }
            ui.collapsing(tr("Detalles", "Details"), |ui| {
                ui.label(error);
            });
        } else if self.pending.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(tr("Actualizando repaso…", "Updating review…"));
            });
        }
        let page = page.as_deref()?;
        let Some(record) = page.entries().first() else {
            if page.total > 0 && !busy && self.error.is_none() {
                return Some(NotesAction::Read((page.total - 1) as usize));
            }
            if page.total == 0 {
                ui.label(
                    RichText::new(tr(
                        "No quedan anotaciones pendientes en este libro.",
                        "There are no pending annotations in this book.",
                    ))
                    .color(p.secondary),
                );
            }
            return None;
        };
        let data = record.data();
        let values = &data.values;
        let page_label = format!(
            "{} {}",
            tr("Página", "Page"),
            u64::from(values.page_index) + 1
        );
        let position = format!("{} {} {}", offset + 1, tr("de", "of"), page.total);
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 18.0;
            ui.horizontal_wrapped(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(10.0, 14.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(rect.center(), 4.0, panel::mark_color(values.color));
                let label = format!(
                    "{page_label} · {} · {position}",
                    panel::kind_label(values.kind, context.english)
                );
                ui.label(RichText::new(label).size(13.0).color(p.secondary));
            });
        });
        let note = values
            .note
            .as_deref()
            .filter(|text| !text.trim().is_empty());
        let passage = values
            .selected_text
            .as_deref()
            .filter(|text| !text.trim().is_empty());
        if let Some(note) = note {
            ui.label(RichText::new(note).color(p.ink));
        }
        let can_reveal = note.is_some() && passage.is_some() && !self.review_revealed;
        if let Some(passage) = passage.filter(|_| !can_reveal) {
            ui.label(RichText::new(passage).italics().color(p.secondary));
        }
        if note.is_none() && passage.is_none() {
            ui.label(
                RichText::new(tr(
                    "Abre el PDF para repasar esta marca.",
                    "Open the PDF to review this mark.",
                ))
                .color(p.secondary),
            );
        }
        if values.status == AttachmentStatus::ReviewRequired {
            ui.label(
                RichText::new(tr(
                    "El PDF cambió. Revisa la página antes de resolver la anotación.",
                    "The PDF changed. Check the page before resolving the annotation.",
                ))
                .color(p.secondary),
            );
        }

        if !narrow {
            ui.add_space(4.0);
        }
        ui.horizontal_wrapped(|ui| {
            if can_reveal
                && theme::navigation(ui, tr("Mostrar cita", "Reveal passage"), false, p).clicked()
            {
                self.review_revealed = true;
            }
            if ui
                .add_enabled_ui(!busy && self.error.is_none(), |ui| {
                    theme::control(
                        ui,
                        tr("Resolver", "Resolve"),
                        Some(theme::Icon::Check),
                        false,
                        true,
                        p,
                    )
                })
                .inner
                .clicked()
            {
                let offset = offset.min(page.total.saturating_sub(2) as usize);
                action = Some(NotesAction::Change(
                    AnnotationMutation::Review {
                        ids: vec![data.id.clone()],
                        state: AnnotationReviewState::Resolved,
                    },
                    offset,
                ));
            }
            if narrow {
                ui.end_row();
            }
            if ui
                .add_enabled_ui((values.page_index as usize) < context.page_count, |ui| {
                    theme::navigation(ui, tr("Ver en PDF", "Show in PDF"), false, p)
                })
                .inner
                .clicked()
            {
                action = NotesAction::open_annotation(ui, data);
            }
            for (enabled, offset, label, icon) in [
                (
                    offset > 0,
                    offset.saturating_sub(1),
                    tr("Anterior", "Previous"),
                    theme::Icon::Left,
                ),
                (
                    (offset as u64 + 1) < page.total,
                    offset + 1,
                    tr("Siguiente", "Next"),
                    theme::Icon::Right,
                ),
            ] {
                if ui
                    .add_enabled_ui(!busy && enabled, |ui| {
                        if narrow {
                            theme::icon_button(ui, label, icon, p)
                        } else {
                            theme::control(ui, label, Some(icon), false, false, p)
                        }
                    })
                    .inner
                    .clicked()
                {
                    action = Some(NotesAction::Read(offset));
                }
            }
        });
        if matches!(
            &action,
            Some(NotesAction::Read(_) | NotesAction::Change(..))
        ) {
            header.response.scroll_to_me_animation(
                Some(egui::Align::TOP),
                egui::style::ScrollAnimation::none(),
            );
        }
        action
    }
}
