use super::*;

pub(in crate::app) fn kind_label(kind: AnnotationKind, english: bool) -> &'static str {
    match (kind, english) {
        (AnnotationKind::Highlight, true) => "Highlight",
        (AnnotationKind::Highlight, false) => "Resaltado",
        (AnnotationKind::Underline, true) => "Underline",
        (AnnotationKind::Underline, false) => "Subrayado",
        (AnnotationKind::Strikeout, true) => "Strikeout",
        (AnnotationKind::Strikeout, false) => "Tachado",
        (AnnotationKind::TextNote, true) => "Text note",
        (AnnotationKind::TextNote, false) => "Nota de texto",
        (AnnotationKind::PageMarker, true) => "Page marker",
        (AnnotationKind::PageMarker, false) => "Marca de página",
        (AnnotationKind::PageComment, true) => "Page note",
        (AnnotationKind::PageComment, false) => "Nota de página",
        (AnnotationKind::Ink, true) => "Pen drawing",
        (AnnotationKind::Ink, false) => "Dibujo a lápiz",
        (AnnotationKind::Arrow, true) => "Arrow",
        (AnnotationKind::Arrow, false) => "Flecha",
        (AnnotationKind::Rectangle, true) => "Rectangle",
        (AnnotationKind::Rectangle, false) => "Recuadro",
    }
}
pub(in crate::app) fn review_label(state: AnnotationReviewState, english: bool) -> &'static str {
    match (state, english) {
        (AnnotationReviewState::Pending, true) => "Pending",
        (AnnotationReviewState::Pending, false) => "Pendiente",
        (AnnotationReviewState::Accepted, true) => "Accepted",
        (AnnotationReviewState::Accepted, false) => "Aceptada",
        (AnnotationReviewState::Resolved, true) => "Resolved",
        (AnnotationReviewState::Resolved, false) => "Resuelta",
    }
}
pub(in crate::app) fn color_label(color: AnnotationColor, english: bool) -> &'static str {
    match (color, english) {
        (AnnotationColor::Yellow, true) => "Yellow",
        (AnnotationColor::Yellow, false) => "Amarillo",
        (AnnotationColor::Green, true) => "Green",
        (AnnotationColor::Green, false) => "Verde",
        (AnnotationColor::Blue, true) => "Blue",
        (AnnotationColor::Blue, false) => "Azul",
        (AnnotationColor::Rose, true) => "Rose",
        (AnnotationColor::Rose, false) => "Rosa",
        (AnnotationColor::Violet, true) => "Violet",
        (AnnotationColor::Violet, false) => "Violeta",
    }
}
pub(in crate::app) fn mark_color(color: AnnotationColor) -> Color32 {
    match color {
        AnnotationColor::Yellow => Color32::from_rgb(190, 139, 20),
        AnnotationColor::Green => Color32::from_rgb(35, 141, 87),
        AnnotationColor::Blue => Color32::from_rgb(41, 120, 188),
        AnnotationColor::Rose => Color32::from_rgb(190, 69, 100),
        AnnotationColor::Violet => Color32::from_rgb(132, 84, 190),
    }
}
pub(in crate::app) fn color_choice(
    ui: &mut egui::Ui,
    choice: AnnotationColor,
    current: AnnotationColor,
    english: bool,
    p: Palette,
) -> egui::Response {
    theme::control(
        ui,
        color_label(choice, english),
        Some(theme::Icon::Swatch(mark_color(choice))),
        current == choice,
        false,
        p,
    )
}

impl YamantakaApp {
    pub(in crate::app) fn prepare_annotations(&mut self, document: &str) {
        self.reader
            .annotations
            .prepare(&self.runtime, &self.governor, document);
    }
    pub(in crate::app) fn annotation_panel(
        &mut self,
        ui: &mut egui::Ui,
        info: &PdfInfo,
        p: Palette,
    ) {
        let busy = self.reader.annotations.pending.is_some()
            || self.global_notes.writing()
            || self.library_tags.writing()
            || self
                .reference
                .as_ref()
                .is_some_and(|reference| reference.pane.annotations.writing());
        if !self.reader.annotations.reviewing() && !self.reader.annotations.editing() {
            self.annotation_filters_panel(ui, p);
            if self.reader.annotations.reviewing() {
                return;
            }
            ui.add_space(6.0);
        }
        let context = NotesContext {
            english: self.english,
            page: self.reader.page,
            page_count: info.pages.len(),
            busy,
            selectable: true,
            reveal_actions: false,
            governor: &self.governor,
            marks_error: self.reader.annotation_paint.error().map(str::to_owned),
        };
        if self.reader.annotations.reviewing() && !self.reader.annotations.editing() {
            if let Some(action) = self.reader.annotations.show_review(ui, &context, p) {
                self.apply_notes_action(false, action);
            }
            return;
        }
        let action = self.reader.annotations.show_editor(ui, &context, p, false);
        if let Some(action) = action {
            self.apply_notes_action(false, action);
        }
        if self.reader.annotations.reviewing() {
            return;
        }
        if self.reader.annotations.editor.is_none() || self.annotation_export.busy() {
            ui.add_space(16.0);
            self.annotation_export_panel(ui, p);
        }
        ui.add_space(12.0);
        self.annotation_bulk_panel(ui, p);
        let context = NotesContext {
            english: self.english,
            page: self.reader.page,
            page_count: info.pages.len(),
            busy,
            selectable: true,
            reveal_actions: false,
            governor: &self.governor,
            marks_error: self.reader.annotation_paint.error().map(str::to_owned),
        };
        let action = self.reader.annotations.show_list(ui, &context, p);
        if let Some(action) = action {
            self.apply_notes_action(false, action);
        }
    }
}
