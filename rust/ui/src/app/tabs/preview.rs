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
    WaitClosed,
    WaitReopened,
    WaitLastClosed,
    WaitInactiveClosed,
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
    pub fn preview_tab_closure(&mut self, mode: usize) {
        self.preview_reader_tabs();
        if self.demo && self.capture.is_some() {
            self.tabs.preview_close = mode;
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
                    && self.viewport.pages.values().any(|p| p.image.is_some()) =>
            {
                if matches!(self.tabs.preview_close, 2 | 3) {
                    let id = self.tabs.model.active().unwrap().id().to_owned();
                    self.close_reader_tab(&id);
                    self.tabs.preview = TabPreview::WaitLastClosed;
                } else if let Some(document) = self.documents.get(1).cloned() {
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
                    && self.viewport.pages.values().any(|p| p.image.is_some()) =>
            {
                if self.tabs.preview_close == 4 {
                    let ids: Vec<_> = self
                        .tabs
                        .model
                        .tabs()
                        .iter()
                        .filter(|t| t.document() == "demo-1")
                        .map(|t| t.id().to_owned())
                        .collect();
                    self.close_reader_tab(&ids[0]);
                    self.close_reader_tab(&ids[1]);
                    self.tabs.preview = TabPreview::WaitInactiveClosed;
                } else if self.tabs.preview_close == 1 {
                    let id = self.tabs.model.active().unwrap().id().to_owned();
                    self.close_reader_tab(&id);
                    self.tabs.preview = TabPreview::WaitClosed;
                } else {
                    self.tabs.preview = TabPreview::Complete;
                }
            }
            TabPreview::WaitInactiveClosed
                if !self.tabs.busy()
                    && self.tabs.model.tabs().len() == 1
                    && self.active.as_ref().is_some_and(|(d, _)| d.id == "demo-0") =>
            {
                self.tabs.preview = TabPreview::Complete;
            }
            TabPreview::WaitClosed
                if !self.tabs.busy()
                    && !self.opening.busy()
                    && self.tabs.restoration.is_none()
                    && self.active.as_ref().is_some_and(|(d, _)| d.id == "demo-1") =>
            {
                self.reopen_reader_tab();
                self.tabs.preview = TabPreview::WaitReopened;
            }
            TabPreview::WaitLastClosed
                if !self.tabs.busy()
                    && self.active.is_none()
                    && matches!(self.surface, Surface::Library) =>
            {
                if self.tabs.preview_close == 3 {
                    self.tabs.preview = TabPreview::Complete;
                } else {
                    self.reopen_reader_tab();
                    self.tabs.preview = TabPreview::WaitReopened;
                }
            }
            TabPreview::WaitReopened
                if !self.tabs.busy()
                    && !self.opening.busy()
                    && self.tabs.restoration.is_none()
                    && self.restoration.ready()
                    && self.active.as_ref().is_some_and(|(d, _)| d.id == "demo-0")
                    && self.reader_preferences.is_some()
                    && self.viewport.pages.values().any(|p| p.image.is_some()) =>
            {
                self.tabs.preview = TabPreview::Complete;
            }
            _ => {}
        }
    }
}
