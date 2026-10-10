use super::*;
use unicode_segmentation::UnicodeSegmentation;

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
    fn selected_ids(&self) -> Option<Vec<String>> {
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
        (!ids.is_empty()).then_some(ids)
    }
    pub(super) fn review_mutation(
        &self,
        state: AnnotationReviewState,
    ) -> Option<AnnotationMutation> {
        self.selected_ids()
            .map(|ids| AnnotationMutation::Review { ids, state })
    }
    pub(super) fn tag_mutation(&self) -> Option<AnnotationMutation> {
        let tag = self.bulk_tag.trim();
        if tag.is_empty() || tag.len() > 4096 || tag.graphemes(true).count() > 80 {
            return None;
        }
        self.selected_ids().map(|ids| AnnotationMutation::AddTag {
            ids,
            tag: tag.into(),
        })
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
            ui.add_space(8.0);
            let label = ui.label(self.tr("Añadir una etiqueta", "Add one tag"));
            ui.add(
                egui::TextEdit::singleline(&mut self.annotations.bulk_tag)
                    .desired_width(f32::INFINITY)
                    .char_limit(4096),
            )
            .labelled_by(label.id);
            let units = self.annotations.bulk_tag.trim().graphemes(true).count();
            ui.label(
                RichText::new(format!("{units} / 80"))
                    .size(12.0)
                    .color(p.secondary),
            );
            if ui
                .add_enabled_ui(
                    !self.annotations.bulk_tag.trim().is_empty()
                        && self.annotations.bulk_tag.trim().len() <= 4096
                        && units <= 80,
                    |ui| {
                        theme::navigation(
                            ui,
                            self.tr("Añadir a las seleccionadas", "Add to selected"),
                            false,
                            p,
                        )
                    },
                )
                .inner
                .clicked()
                && let Some(mutation) = self.annotations.tag_mutation()
            {
                self.annotation_command(Some(mutation), page.offset);
            }
        });
    }
}

impl YamantakaApp {
    pub fn preview_annotation_tag(&mut self, save: bool) {
        if self.demo && self.capture.is_some() {
            self.preview_annotation_list();
            self.annotations.bulk_tag_preview = Some(save);
        }
    }
    pub(in crate::app) fn apply_annotation_tag_preview(&mut self) {
        if !self.annotations.ready() {
            return;
        }
        let Some(save) = self.annotations.bulk_tag_preview.take() else {
            return;
        };
        for selected in self.annotations.selected.0.iter_mut().take(3) {
            *selected = true;
        }
        self.annotations.bulk_tag = "Study".into();
        if save && let Some(mutation) = self.annotations.tag_mutation() {
            self.annotation_command(Some(mutation), 0);
        }
    }
}
