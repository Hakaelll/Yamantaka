use super::*;

pub(super) struct BulkSelection(pub(super) [bool; 80]);
impl Default for BulkSelection {
    fn default() -> Self {
        Self([false; 80])
    }
}
impl BulkSelection {
    pub(super) fn count(&self) -> usize {
        self.0.iter().filter(|selected| **selected).count()
    }
}
impl AnnotationState {
    pub(super) fn review_mutation(
        &self,
        state: AnnotationReviewState,
    ) -> Option<AnnotationMutation> {
        if self.pending.is_some() || self.editor.is_some() {
            return None;
        }
        let page = self.page.as_ref()?;
        let ids: Vec<_> = page
            .entries()
            .iter()
            .zip(self.selected.0)
            .filter(|(_, selected)| *selected)
            .map(|(record, _)| record.data().id.clone())
            .collect();
        (!ids.is_empty()).then_some(AnnotationMutation::Review { ids, state })
    }
}
impl YamantakaApp {
    pub fn preview_annotation_bulk(&mut self, save: bool) {
        if self.demo && self.capture.is_some() {
            self.preview_annotation_list();
            self.annotations.bulk_preview = Some(save);
        }
    }
    pub(in crate::app) fn apply_annotation_bulk_preview(&mut self) {
        if !self.annotations.ready() {
            return;
        }
        let Some(save) = self.annotations.bulk_preview.take() else {
            return;
        };
        for selected in self.annotations.selected.0.iter_mut().take(3) {
            *selected = true;
        }
        if save
            && let Some(mutation) = self
                .annotations
                .review_mutation(AnnotationReviewState::Accepted)
        {
            self.annotation_command(Some(mutation), 0);
        }
    }
    pub(in crate::app) fn annotation_bulk_panel(&mut self, ui: &mut egui::Ui, p: Palette) {
        let Some(page) = self.annotations.page.clone() else {
            return;
        };
        if page.entries().is_empty() {
            return;
        }
        let enabled = self.annotations.pending.is_none() && !self.annotations.editing();
        ui.add_enabled_ui(enabled, |ui| {
            ui.horizontal_wrapped(|ui| {
                if theme::navigation(ui, self.tr("Seleccionar página", "Select page"), false, p)
                    .clicked()
                {
                    self.annotations.selected = Default::default();
                    for selected in self
                        .annotations
                        .selected
                        .0
                        .iter_mut()
                        .take(page.entries().len())
                    {
                        *selected = true;
                    }
                }
                if self.annotations.selected.count() > 0
                    && theme::navigation(
                        ui,
                        self.tr("Quitar selección", "Clear selection"),
                        false,
                        p,
                    )
                    .clicked()
                {
                    self.annotations.selected = Default::default();
                }
            });
            let count = self.annotations.selected.count();
            if count == 0 {
                return;
            }
            ui.label(
                RichText::new(if self.english {
                    format!("{count} selected on this list page")
                } else {
                    format!("{count} seleccionadas en esta página de la lista")
                })
                .color(p.secondary),
            );
            ui.horizontal_wrapped(|ui| {
                for (state, en, es) in [
                    (
                        AnnotationReviewState::Pending,
                        "Mark pending",
                        "Marcar pendientes",
                    ),
                    (AnnotationReviewState::Accepted, "Accept", "Aceptar"),
                    (AnnotationReviewState::Resolved, "Resolve", "Resolver"),
                ] {
                    if theme::navigation(ui, if self.english { en } else { es }, false, p).clicked()
                        && let Some(mutation) = self.annotations.review_mutation(state)
                    {
                        self.annotation_command(Some(mutation), page.offset);
                    }
                }
            });
        });
    }
}
