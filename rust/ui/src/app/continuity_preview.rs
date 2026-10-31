use super::*;
use yamantaka_domain::reader_preferences::PageRotation;

impl YamantakaApp {
    pub(super) fn advance_graphics_loss_preview(&mut self, ctx: &egui::Context) -> bool {
        let metadata_draft = std::env::args_os().any(|arg| arg == "--graphics-metadata-draft");
        let stored_edits = std::env::args_os().any(|arg| arg == "--graphics-stored-note-drafts");
        if stored_edits
            && (!self.reference_notes_preview.ready()
                || !self.reader.annotations.ready()
                || self
                    .reference
                    .as_ref()
                    .is_none_or(|view| !view.pane.annotations.ready()))
        {
            return false;
        }
        if metadata_draft
            && !self
                .metadata
                .as_ref()
                .is_some_and(metadata::MetadataEditor::capture_ready)
        {
            return false;
        }
        if self.preview_device_loss.is_none()
            || self.reader.loading
            || self
                .reader
                .viewport
                .pages
                .values()
                .all(|page| page.image.is_none())
        {
            return false;
        }
        if self.preview_reference || !matches!(self.reference_demo, 0 | 3) {
            return false;
        }
        if self.reader.page == 0 {
            self.navigate(1);
            return false;
        }
        if self.reference.as_ref().is_some_and(|reference| {
            reference.busy()
                || reference.pane.loading
                || reference
                    .pane
                    .viewport
                    .pages
                    .values()
                    .all(|page| page.image.is_none())
        }) {
            return false;
        }
        if !self.device_loss_configured {
            self.device_loss_configured = true;
            if stored_edits {
                self.reader
                    .annotations
                    .seed_stored_note_recovery("Unsaved primary annotation edit");
                self.reader.panel = true;
                self.reader.reader_panel_tab = outline::ReaderPanelTab::Annotations;
                let view = self.reference.as_mut().unwrap();
                view.pane
                    .annotations
                    .seed_stored_note_recovery("Unsaved reference annotation edit");
                view.pane.reader_panel_tab = outline::ReaderPanelTab::Annotations;
                view.navigator_requested = true;
            }
            if metadata_draft {
                self.metadata.as_mut().unwrap().preview_unsaved_title();
            }
            if self.reference.is_some() {
                self.set_reader_zoom(1.25);
                let direction = self.reading_direction();
                let reference = self.reference.as_mut().unwrap();
                reference.set_page(0);
                reference.set_zoom(ZoomMode::Custom, 1.25);
                let prefs = reference.pane.reader_preferences.as_mut().unwrap();
                prefs.page_rotation = PageRotation::from_degrees(90).unwrap();
                prefs.reading_theme = ReaderTheme::Sepia;
                prefs.horizontal_margin = 48.0;
                prefs.vertical_margin = 16.0;
                reference.pane.rebuild_layout(direction).unwrap();
                reference.layout = yamantaka_domain::workspaces::ComparisonLayout::SideBySide;
                reference.set_ratio(0.4);
                reference.swapped = true;
                reference.bidirectional = true;
                reference.follow_page = false;
                reference.follow_zoom = true;
                reference.follow_position = false;
                reference.differences.insert(0);
            }
            if std::env::args_os().any(|arg| arg == "--graphics-page-note-drafts") {
                self.reader.begin_page_note();
                self.reader
                    .annotations
                    .seed_page_note_recovery("Unsaved primary study note");
                if let Some(view) = &mut self.reference {
                    view.pane.begin_page_note();
                    view.pane
                        .annotations
                        .seed_page_note_recovery("Unsaved reference study note");
                }
            }
            return false;
        }
        if self
            .reader
            .viewport
            .pages
            .values()
            .any(|page| !page.plan.ready(page.image_key.as_ref()))
            || self.reference.as_ref().is_some_and(|reference| {
                reference
                    .pane
                    .viewport
                    .pages
                    .values()
                    .any(|page| !page.plan.ready(page.image_key.as_ref()))
            })
        {
            return false;
        }
        let save_primary = std::env::args_os()
            .any(|arg| arg == "--graphics-save-page-note" || arg == "--graphics-save-stored-note");
        if save_primary && !self.submit_annotation_recovery_preview() {
            return false;
        }
        let expected = serde_json::json!({"synthetic":true,"comparison":self.capture_comparison(),"primaryPage":self.reader.page,"primaryZoom":self.reader.zoom,"pageNotes":self.page_note_recovery_summary(),"savedPrimaryNote":save_primary,"metadataDraft":self.metadata_draft_summary(),"storedNotes":self.stored_note_recovery_summary()});
        let path = self
            .capture
            .as_ref()
            .unwrap()
            .with_extension("expected-comparison.json");
        std::fs::write(path, serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
        self.send(Action::Favorite("demo-0".into(), true));
        self.graphics_recovery.signal(ctx);
        self.preview_device_loss.take().unwrap().destroy();
        ctx.send_viewport_cmd(ViewportCommand::Close);
        true
    }
}
