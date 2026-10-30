use super::*;
use yamantaka_domain::reading_sessions::restoration::{ReaderSessionRestoration, SessionPanel};

#[derive(Default)]
pub(super) struct RestorationState {
    pending: Option<(String, Cancellation, ReaderSessionRestoration)>,
    pub(super) scroll: Option<f64>,
}
impl RestorationState {
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && self.scroll.is_none()
    }
    pub(super) fn clear(&mut self) {
        self.pending = None;
        self.scroll = None;
    }
    pub(super) fn arm(
        &mut self,
        id: String,
        cancel: Cancellation,
        context: ReaderSessionRestoration,
    ) {
        self.clear();
        self.pending = Some((id, cancel, context));
    }
    fn take(&mut self, id: &str, cancel: &Cancellation) -> Option<ReaderSessionRestoration> {
        if cancel.is_cancelled()
            || !self
                .pending
                .as_ref()
                .is_some_and(|(target, token, _)| target == id && token.same_request(cancel))
        {
            return None;
        }
        self.pending.take().map(|(_, _, context)| context)
    }
}
impl YamantakaApp {
    pub(super) fn restoration_summary(&self) -> serde_json::Value {
        serde_json::json!({"pending":self.reader.restoration.pending.is_some(),"scrollPending":self.reader.restoration.scroll,"panelOpen":self.reader.panel,"preferences":self.reader.reader_preferences.as_ref().map(|preferences|serde_json::json!({"document":preferences.document_id,"zoom":preferences.zoom_level,"layout":preferences.persisted_layout_name(),"mode":preferences.view_mode.persisted_name(),"theme":preferences.reading_theme.persisted_name(),"scroll":preferences.scroll_offset}))})
    }
    pub(super) fn restore_loaded_preferences(
        &mut self,
        preferences: ReaderPreferences,
    ) -> ReaderPreferences {
        let Some(context) = self.reader.restoration.take(
            &preferences.document_id,
            &self.reader.reader_preferences_cancel,
        ) else {
            return preferences;
        };
        match context.apply(&preferences) {
            Ok(restored) => {
                self.reader.restoration.scroll = Some(restored.scroll_offset);
                self.reader.panel = context.panel().is_some();
                if let Some(panel) = context.panel() {
                    self.reader.reader_panel_tab = match panel {
                        SessionPanel::Outline => outline::ReaderPanelTab::Outline,
                        SessionPanel::Thumbnails => outline::ReaderPanelTab::Thumbnails,
                        SessionPanel::Search => outline::ReaderPanelTab::Find,
                        SessionPanel::Bookmarks => outline::ReaderPanelTab::Bookmarks,
                        SessionPanel::Annotations => outline::ReaderPanelTab::Annotations,
                        SessionPanel::Text => outline::ReaderPanelTab::Text,
                        SessionPanel::Positions => outline::ReaderPanelTab::Positions,
                    };
                }
                restored
            }
            Err(error) => {
                self.error = Some(error.to_string());
                preferences
            }
        }
    }
}
