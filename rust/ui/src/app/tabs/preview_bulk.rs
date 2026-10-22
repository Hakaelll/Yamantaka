use super::*;

#[derive(Default, Debug, PartialEq, Eq)]
pub(super) enum BulkPreview {
    #[default]
    Off,
    Start,
    Third,
    Fourth,
    Closed,
    Complete,
}
impl BulkPreview {
    pub(super) fn ready(&self) -> bool {
        matches!(self, Self::Off | Self::Complete)
    }
}
impl YamantakaApp {
    pub fn preview_bulk_tab_close(&mut self, to_right: bool) {
        self.preview_reader_tabs();
        if self.demo && self.capture.is_some() {
            self.tabs.bulk_preview = BulkPreview::Start;
            self.tabs.preview_close = if to_right { 6 } else { 5 };
        }
    }
    pub(in crate::app) fn advance_bulk_tab_preview(&mut self) {
        if !self.demo
            || self.capture.is_none()
            || self.tabs.busy()
            || self.opening.busy()
            || self.reader_preferences_loading
        {
            return;
        }
        match self.tabs.bulk_preview {
            BulkPreview::Start if self.tabs.preview == preview::TabPreview::Complete => {
                let document = self.documents[2].clone();
                self.open(document);
                self.tabs.bulk_preview = BulkPreview::Third;
            }
            BulkPreview::Third if self.active.as_ref().is_some_and(|(d, _)| d.id == "demo-2") => {
                let document = self.documents[3].clone();
                self.open(document);
                self.tabs.bulk_preview = BulkPreview::Fourth;
            }
            BulkPreview::Fourth if self.active.as_ref().is_some_and(|(d, _)| d.id == "demo-3") => {
                // Initial traversal reordered the pinned duplicate before the first reading.
                let target = if self.tabs.preview_close == 6 {
                    self.tabs
                        .model
                        .tabs()
                        .iter()
                        .find(|t| t.document() == "demo-1" && !t.pinned())
                        .unwrap()
                        .id()
                        .to_owned()
                } else {
                    self.tabs.preview_first.clone().unwrap()
                };
                self.close_other_reader_tabs(&target, self.tabs.preview_close == 6);
                self.tabs.bulk_preview = BulkPreview::Closed;
            }
            BulkPreview::Closed
                if self.tabs.pending.is_none()
                    && self.tabs.restoration.is_none()
                    && self.restoration.ready()
                    && self.viewport.pages.values().any(|p| p.image.is_some()) =>
            {
                self.tabs.bulk_preview = BulkPreview::Complete;
            }
            _ => {}
        }
    }
}
