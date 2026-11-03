use super::*;
use yamantaka_domain::workspaces::WorkspaceComparison;

impl ReferenceView {
    pub(in crate::app) fn swap_positions(&mut self) {
        self.swapped = !self.swapped;
        self.size = egui::Vec2::ZERO;
    }
    pub(in crate::app) fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(0.2, 0.8);
        self.followed_position = None;
    }
    pub(in crate::app) fn snapshot(&self, primary: &str) -> Option<WorkspaceComparison> {
        if self.deferred {
            return None;
        }
        let (document, _) = self.pane.active.as_ref()?;
        let position = self.pane.navigation.as_ref()?.position();
        let prefs = self.pane.reader_preferences.as_ref()?;
        Some(WorkspaceComparison {
            bidirectional: self.bidirectional,
            swapped: self.swapped,
            differences: self.differences.clone(),
            layout: self.layout,
            scroll_offset: None,
            primary: primary.into(),
            document: document.id.clone(),
            page: position.page(),
            point: position.point(),
            zoom_mode: prefs.zoom_mode.persisted_name().into(),
            zoom: f64::from(self.pane.zoom),
            theme: prefs.reading_theme.persisted_name().into(),
            rotation: prefs.page_rotation.degrees().into(),
            horizontal_margin: prefs.horizontal_margin,
            vertical_margin: prefs.vertical_margin,
            ratio: self.ratio,
            follow_page: self.follow_page,
            follow_zoom: self.follow_zoom,
            follow_position: self.follow_position,
        })
    }

    pub(in crate::app) fn restore(
        &mut self,
        saved: &WorkspaceComparison,
        direction: ReadingDirection,
    ) -> yamantaka_domain::Result<()> {
        saved.validate()?;
        let preferences = saved.preferences()?;
        if self
            .pane
            .active
            .as_ref()
            .is_none_or(|(doc, _)| doc.id != saved.document)
        {
            return Err(yamantaka_domain::Failure::Invalid(
                "The saved reference is not open".into(),
            ));
        }
        self.pane.reader_preferences = Some(preferences);
        self.pane.zoom = saved.zoom as f32;
        if let Some(nav) = &mut self.pane.navigation {
            nav.update_anchor(ReaderPosition::at(saved.page, saved.point)?);
        }
        self.pane.navigation_changed();
        self.pane.rebuild_layout(direction)?;
        self.pane.restoration.scroll = saved.scroll_offset;
        self.layout = saved.layout;
        self.set_ratio(saved.ratio);
        self.follow_page = saved.follow_page;
        self.follow_zoom = saved.follow_zoom;
        self.follow_position = saved.follow_position;
        self.bidirectional = saved.bidirectional;
        self.swapped = saved.swapped;
        self.size = egui::Vec2::ZERO;
        self.differences = saved.differences.clone();
        self.difference_target = None;
        self.pending_primary_page = None;
        self.pending_primary_zoom = false;
        self.followed_position = None;
        Ok(())
    }
}

impl YamantakaApp {
    pub(in crate::app) fn capture_comparison(&self) -> Option<WorkspaceComparison> {
        let (primary, _) = self.reader.active.as_ref()?;
        self.reference.as_ref()?.snapshot(&primary.id)
    }
}
