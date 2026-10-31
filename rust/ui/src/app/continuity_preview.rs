use super::*;
use yamantaka_domain::reader_preferences::PageRotation;

impl YamantakaApp {
    pub(super) fn advance_graphics_loss_preview(&mut self, ctx: &egui::Context) -> bool {
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
        let expected = serde_json::json!({"synthetic":true,"comparison":self.capture_comparison(),"primaryPage":self.reader.page,"primaryZoom":self.reader.zoom});
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
