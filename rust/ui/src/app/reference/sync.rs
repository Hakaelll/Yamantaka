use super::*;

impl ReferenceView {
    pub(in crate::app) fn set_page(&mut self, page: u32) {
        if let Some(nav) = &mut self.pane.navigation {
            nav.go_to_page(page);
            let page = nav.position().page();
            self.pane.navigation_changed();
            if self.bidirectional && self.follow_page {
                self.pending_primary_page = Some(page);
            } else {
                self.follow_page = false;
            }
            self.follow_position = false;
        }
    }

    pub(in crate::app) fn set_zoom(&mut self, mode: ZoomMode, zoom: f32) {
        self.pending_primary_zoom = self.bidirectional && self.follow_zoom;
        if !self.pending_primary_zoom {
            self.follow_zoom = false;
        }
        self.followed_position = None;
        if let Some(prefs) = &mut self.pane.reader_preferences {
            self.pane.zoom = zoom;
            prefs.zoom_mode = mode;
            prefs.zoom_level = f64::from(zoom);
            self.pane.viewport.reset_requests();
        }
    }
}

impl YamantakaApp {
    pub(in crate::app) fn apply_reference_sync(&mut self) -> bool {
        let Some(reference) = &mut self.reference else {
            return false;
        };
        let page = reference.pending_primary_page.take();
        let zoom = std::mem::take(&mut reference.pending_primary_zoom)
            .then_some(reference.pane.viewport.scale);
        if !reference.bidirectional || reference.busy() || reference.close_requested {
            return false;
        }
        let mut changed = false;
        if let Some(page) = page
            && let Some(nav) = &self.reader.navigation
            && nav.position().page() != page.min(nav.page_count() - 1)
        {
            self.navigate(page);
            changed = true;
        }
        if let Some(zoom) = zoom
            && zoom.is_finite()
            && zoom > 0.0
            && (self.reader.viewport.scale - zoom).abs() > 0.0001
            && let Some(mut preferences) = self.reader.reader_preferences.clone()
        {
            preferences.zoom_mode = ZoomMode::Custom;
            preferences.zoom_level = f64::from(zoom);
            self.apply_reader_preferences(preferences);
            changed = true;
        }
        changed
    }
}
