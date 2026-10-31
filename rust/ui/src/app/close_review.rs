use super::*;

#[derive(Default)]
pub(super) struct CloseReview {
    reviewing: bool,
    waiting: bool,
    focus_return: bool,
    preview: u8,
    preview_return: bool,
}
impl CloseReview {
    pub(super) fn open(&self) -> bool {
        self.reviewing || self.waiting
    }
}

impl YamantakaApp {
    pub fn preview_editor_close(&mut self, return_to_editing: bool) {
        if self.demo && self.capture.is_some() {
            self.close_review.preview = 1;
            self.close_review.preview_return = return_to_editing;
        }
    }
    pub(super) fn advance_editor_close_preview(&mut self, ctx: &egui::Context) {
        if self.close_review.preview == 1
            && self.annotation_preview == 0
            && self.reference_notes_preview.ready()
            && self
                .metadata
                .as_ref()
                .is_none_or(metadata::MetadataEditor::capture_ready)
            && !self.editor_writes_pending()
        {
            if let Some(editor) = &mut self.metadata {
                editor.preview_unsaved_title();
            }
            if self.editor_drafts_open() {
                self.close_review.preview = 2;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        } else if self.close_review.preview == 2 && self.close_review.reviewing {
            assert!(self.editor_drafts_open());
            self.close_review.preview = 3;
            if self.close_review.preview_return {
                self.close_review.reviewing = false;
                self.close_review.waiting = false;
                assert!(self.editor_drafts_open());
            }
        }
    }
    pub(super) fn editor_close_capture_ready(&self) -> bool {
        matches!(self.close_review.preview, 0 | 3)
    }
    pub(super) fn editor_close_summary(&self) -> serde_json::Value {
        serde_json::json!({"reviewing":self.close_review.reviewing,"waiting":self.close_review.waiting,"preview":self.close_review.preview,"drafts":self.editor_drafts_open()})
    }
    fn editor_writes_pending(&self) -> bool {
        self.reader.annotations.writing()
            || self
                .reference
                .as_ref()
                .is_some_and(|view| view.pane.annotations.writing())
            || !self.retired_annotation_writes.is_empty()
            || self.global_notes.writing()
            || self
                .metadata
                .as_ref()
                .is_some_and(metadata::MetadataEditor::writing)
    }
    fn editor_drafts_open(&self) -> bool {
        self.reader.annotations.unsaved_editor()
            || self
                .reference
                .as_ref()
                .is_some_and(|view| view.pane.annotations.unsaved_editor())
            || self
                .metadata
                .as_ref()
                .is_some_and(metadata::MetadataEditor::unsaved)
    }
    pub(super) fn guard_editor_close(&mut self, ctx: &egui::Context) {
        // Capture completion closes its owned fixture, without discarding real user drafts.
        if self.capture_requested {
            return;
        }
        let requested = ctx.input(|input| input.viewport().close_requested());
        if !requested && !self.close_review.waiting {
            return;
        }
        if self.editor_writes_pending() {
            if !self.close_review.waiting {
                self.close_review.focus_return = true;
            }
            self.close_review.waiting = true;
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        } else if self.editor_drafts_open() {
            self.close_review.waiting = false;
            if !self.close_review.reviewing {
                self.close_review.reviewing = true;
                self.close_review.focus_return = true;
            }
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        } else if std::mem::take(&mut self.close_review.waiting) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
    pub(super) fn editor_close_dialog(&mut self, ctx: &egui::Context) {
        if self.close_review.waiting {
            self.guard_editor_close(ctx);
        }
        if !self.close_review.open() {
            return;
        }
        let p = Palette::new(self.dark);
        let compact = ctx.content_rect().height() < 360.0;
        let waiting = self.close_review.waiting;
        let mut keep = false;
        let mut discard = false;
        let modal = egui::Modal::new(egui::Id::new("reader-close-review"))
            .frame(egui::Frame::new().fill(p.surface).stroke(egui::Stroke::new(1.0, p.line))
                .corner_radius(12).inner_margin(if compact { 12 } else { 24 }))
            .show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width() - 72.0).clamp(160.0, 400.0));
                ui.label(RichText::new(self.tr("Antes de cerrar", "Before closing")).size(if compact {20.0} else {24.0}).color(p.ink));
                ui.add_space(if compact {6.0} else {12.0});
                egui::ScrollArea::vertical().max_height((ctx.content_rect().height() - if compact {156.0} else {220.0}).max(32.0)).show(ui, |ui| {
                    ui.label(self.tr(
                        if waiting { "Terminando de guardar los cambios. Puedes seguir en la aplicación." } else { "Hay notas o correcciones de metadatos sin guardar. Vuelve a la edición para conservarlas." },
                        if waiting { "Finishing saved changes. You can stay in the app." } else { "Notes or metadata corrections have not been saved. Return to editing to keep them." },
                    ));
                });
                ui.add_space(if compact {10.0} else {20.0});
                ui.horizontal_wrapped(|ui| {
                    let response = theme::control(ui, self.tr("Volver a editar", "Return to editing"), None, true, false, p);
                    if self.close_review.focus_return {
                        response.request_focus();
                        self.close_review.focus_return = false;
                    }
                    keep = response.clicked();
                    if !waiting {
                        discard = theme::control(ui, self.tr("Descartar y cerrar", "Discard and close"), None, false, false, p).clicked();
                    }
                });
            });
        if keep || modal.should_close() {
            self.close_review = Default::default();
            self.workspaces.close_after = false;
            self.quarantine.close_after = false;
            self.library_tags.close_after_write = false;
            self.library_selection.close_after_removal = false;
            self.legacy_import.close_after = false;
            self.backup_restore.close_after = false;
            self.backups.close_after = false;
        } else if discard && !self.editor_writes_pending() {
            self.reader.annotations.discard_editor();
            if let Some(view) = &mut self.reference {
                view.pane.annotations.discard_editor();
            }
            self.metadata = None;
            self.close_review = Default::default();
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}
