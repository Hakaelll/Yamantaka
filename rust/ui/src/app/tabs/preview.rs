use super::*;

#[derive(Default, Debug, PartialEq, Eq)]
pub(super) enum TabPreview {
    #[default]
    Off,
    ConfigureFirst,
    WaitFirst,
    WaitSecond,
    ReturnFirst,
    Complete,
}
impl TabPreview {
    pub(super) fn ready(&self) -> bool {
        matches!(self, Self::Off | Self::Complete)
    }
}
impl YamantakaApp {
    pub fn preview_reader_tabs(&mut self) {
        if self.demo && self.capture.is_some() {
            self.preview_reader = true;
            self.tabs.preview = TabPreview::ConfigureFirst;
        }
    }
    pub(in crate::app) fn advance_tab_preview(&mut self) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        match self.tabs.preview {
            TabPreview::ConfigureFirst
                if self.reader_preferences.is_some() && self.active.is_some() =>
            {
                self.sync_tab_context();
                self.tabs.preview_first = self.tabs.model.active().map(|t| t.id().to_owned());
                let mut preferences = self.reader_preferences.clone().unwrap();
                preferences.zoom_mode = ZoomMode::Custom;
                preferences.zoom_level = 1.75;
                preferences.page_layout = yamantaka_domain::PageLayout::Continuous;
                preferences.reading_theme = ReaderTheme::Sepia;
                preferences.scroll_offset = 1500.0;
                self.navigate(1);
                self.apply_reader_preferences(preferences);
                self.restoration.scroll = Some(1500.0);
                self.tabs.preview = TabPreview::WaitFirst;
            }
            TabPreview::WaitFirst
                if self.restoration.ready()
                    && self.reader_pages.values().any(|p| p.image.is_some()) =>
            {
                if let Some(document) = self.documents.get(1).cloned() {
                    self.open(document);
                    self.tabs.preview = TabPreview::WaitSecond;
                }
            }
            TabPreview::WaitSecond
                if !self.opening.busy()
                    && self.active.as_ref().is_some_and(|(d, _)| d.id == "demo-1")
                    && self.reader_preferences.is_some() =>
            {
                let mut preferences = self.reader_preferences.clone().unwrap();
                preferences.zoom_mode = ZoomMode::FitWidth;
                preferences.page_rotation =
                    yamantaka_domain::reader_preferences::PageRotation::Deg90;
                preferences.horizontal_margin = 31.0;
                self.apply_reader_preferences(preferences);
                self.duplicate_reader_tab();
                if let Some(id) = self.tabs.model.active().map(|t| t.id().to_owned()) {
                    self.tabs.model.toggle_pinned(&id);
                }
                self.tabs.model.reorder(2, 0);
                let first = self.tabs.preview_first.clone().unwrap();
                self.select_reader_tab(&first);
                self.tabs.preview = TabPreview::ReturnFirst;
            }
            TabPreview::ReturnFirst
                if self.tabs.pending.is_none()
                    && self.tabs.restoration.is_none()
                    && self.restoration.ready()
                    && self.active.as_ref().is_some_and(|(d, _)| d.id == "demo-0")
                    && self.reader_preferences.is_some()
                    && self.reader_pages.values().any(|p| p.image.is_some()) =>
            {
                self.tabs.preview = TabPreview::Complete;
            }
            _ => {}
        }
    }
}
