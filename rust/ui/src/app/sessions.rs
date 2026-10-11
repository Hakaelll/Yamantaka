use super::*;
use yamantaka_application::reading_sessions::{
    ReadingSessionReply, lifecycle::ReadingSessionLifecycle,
};
use yamantaka_domain::{
    Failure, Result,
    reading_sessions::{ReaderSessionContext, ReadingSessionStart},
};

pub(super) struct SessionState {
    pub lifecycle: ReadingSessionLifecycle,
    origin: Instant,
    wall_origin: chrono::DateTime<chrono::Utc>,
    snapshot: Option<(u32, ReaderSessionContext)>,
    pub failed: Vec<String>,
    queue_retry: Option<String>,
    capture_recording: bool,
}
impl Default for SessionState {
    fn default() -> Self {
        Self {
            lifecycle: Default::default(),
            origin: Instant::now(),
            wall_origin: chrono::Utc::now(),
            snapshot: None,
            failed: Vec::new(),
            queue_retry: None,
            capture_recording: false,
        }
    }
}
impl SessionState {
    fn timestamp(&self) -> String {
        let elapsed =
            chrono::Duration::from_std(self.origin.elapsed()).unwrap_or(chrono::Duration::MAX);
        self.wall_origin
            .checked_add_signed(elapsed)
            .unwrap_or(chrono::DateTime::<chrono::Utc>::MAX_UTC)
            .to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
    }
    pub fn remember_failure(&mut self, id: String) {
        if !self.failed.contains(&id) {
            self.failed.push(id);
        }
    }
    pub fn retry_failed(&mut self) {
        for id in self.failed.drain(..) {
            self.lifecycle.retry(&id);
        }
    }
}

fn reading_context(
    preferences: &ReaderPreferences,
    zoom: f32,
    panel: Option<outline::ReaderPanelTab>,
) -> ReaderSessionContext {
    let page_layout = match preferences.effective_layout() {
        yamantaka_domain::PageLayout::Single => "single",
        yamantaka_domain::PageLayout::Spread => "facing",
        yamantaka_domain::PageLayout::Continuous => "continuous",
    };
    ReaderSessionContext {
        zoom_level: f64::from(zoom),
        page_layout: page_layout.into(),
        reading_mode: preferences.view_mode.persisted_name().into(),
        reading_theme: preferences.reading_theme.persisted_name().into(),
        scroll_offset: preferences.scroll_offset,
        panel_section: panel.map(|panel| {
            match panel {
                outline::ReaderPanelTab::Outline => "outline",
                outline::ReaderPanelTab::Thumbnails => "thumbnails",
                outline::ReaderPanelTab::Find => "search",
                outline::ReaderPanelTab::Bookmarks => "bookmarks",
                outline::ReaderPanelTab::Annotations => "annotations",
                outline::ReaderPanelTab::Text => "text",
            }
            .into()
        }),
    }
}

impl YamantakaApp {
    pub fn preview_sessions(&mut self) {
        self.sessions.capture_recording = self.demo && self.capture.is_some();
    }
    pub(super) fn session_capture_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.preferences.record_reading_sessions(),
            "document": self.sessions.lifecycle.current_document(),
            "snapshot": self.sessions.snapshot,
            "syntheticStore": if self.demo && self.sessions.capture_recording { self.store.file_name().map(|p| p.to_string_lossy()) } else { None },
        })
    }
    pub(super) fn cache_reading_context(&mut self) {
        if let (Some((document, _)), Some(preferences)) = (&self.active, &self.reader_preferences)
            && self.sessions.lifecycle.current_document() == Some(document.id.as_str())
            && preferences.document_id == document.id
        {
            self.sessions.snapshot = Some((
                self.page,
                reading_context(
                    preferences,
                    self.reader_scale,
                    self.panel.then_some(self.reader_panel_tab),
                ),
            ));
        }
    }
    pub(super) fn validate_reading_session_context(&self) -> Result<()> {
        if self.sessions.lifecycle.current_document().is_none() {
            return Ok(());
        }
        self.sessions
            .snapshot
            .as_ref()
            .ok_or_else(|| Failure::Invalid("Reading session context is unavailable".into()))
            .and_then(|(_, context)| context.validate())
    }
    pub(super) fn finish_reading_session(&mut self) -> Result<()> {
        if self.sessions.lifecycle.current_document().is_none() {
            return Ok(());
        }
        let (page, context) = self
            .sessions
            .snapshot
            .as_ref()
            .ok_or_else(|| Failure::Invalid("Reading session context is unavailable".into()))?;
        self.sessions.lifecycle.finish(
            self.sessions.timestamp(),
            *page,
            Some(context.clone()),
            self.sessions.origin.elapsed(),
        )?;
        self.sessions.snapshot = None;
        Ok(())
    }
    fn reader_is_visible(&self, ctx: &egui::Context) -> bool {
        matches!(self.surface, Surface::Reader)
            && ctx.input(|i| i.focused && !i.viewport().minimized.unwrap_or(false))
    }
    pub(super) fn observe_reading_session(&mut self, ctx: &egui::Context) {
        if self.demo && !self.sessions.capture_recording {
            return;
        }
        let interaction = ctx.input(|i| {
            i.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::PointerMoved(_)
                        | egui::Event::PointerButton { pressed: true, .. }
                        | egui::Event::MouseWheel { .. }
                        | egui::Event::Key { pressed: true, .. }
                        | egui::Event::Text(_)
                        | egui::Event::Touch { .. }
                )
            })
        });
        self.sessions.lifecycle.observe(
            self.sessions.origin.elapsed(),
            self.reader_is_visible(ctx),
            interaction,
        );
    }
    pub(super) fn sync_reading_session(&mut self, ctx: &egui::Context) {
        if self.demo && !self.sessions.capture_recording {
            return;
        }
        if ctx.input(|i| i.viewport().close_requested()) {
            if let Err(error) = self.persist_reading_sessions_on_close() {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                self.error = Some(error.to_string());
            }
            return;
        }
        if !self.preferences.record_reading_sessions() {
            if let Err(error) = self.finish_reading_session() {
                self.error = Some(error.to_string());
            }
        } else if self.sessions.lifecycle.current_document().is_none()
            && !self.reader_preferences_loading
            && let (Some((document, _)), Some(preferences)) =
                (&self.active, &self.reader_preferences)
            && preferences.document_id == document.id
        {
            let id = uuid::Uuid::new_v4().to_string();
            match self.sessions.lifecycle.begin(
                &ReadingSessionStart {
                    id: &id,
                    document: &document.id,
                    started_at: &self.sessions.timestamp(),
                    page: self.page,
                },
                self.sessions.origin.elapsed(),
                self.reader_is_visible(ctx),
            ) {
                Ok(()) => self.cache_reading_context(),
                Err(error) => self.error = Some(error.to_string()),
            }
        }
        if let Some(id) = self.sessions.queue_retry.take() {
            self.sessions.lifecycle.retry(&id);
        }
        while let Some(request) = self.sessions.lifecycle.next_request() {
            let mut reply = ReadingSessionReply {
                id: request.id.clone(),
                document: request.document.clone(),
                operation: request.operation(),
                cancel: request.cancel.clone(),
                result: Ok(()),
            };
            if let Err(error) = self
                .runtime
                .submit(Action::ReadingSession(Box::new(request)))
            {
                let queue_full = matches!(error, Failure::QueueFull);
                reply.result = Err(error.clone());
                let id = reply.id.clone();
                self.sessions.lifecycle.accept(reply);
                if queue_full {
                    self.sessions.queue_retry = Some(id);
                    ctx.request_repaint_after(Duration::from_millis(100));
                    break;
                }
                self.sessions.remember_failure(id);
                self.error = Some(error.to_string());
            }
        }
    }
    pub(super) fn persist_reading_sessions_on_close(&mut self) -> Result<()> {
        if self.demo && !self.sessions.capture_recording {
            return Ok(());
        }
        self.cache_reading_context();
        self.finish_reading_session()?;
        self.runtime.persist_reading_sessions(
            self.sessions.lifecycle.recovery_commands()?,
            self.preferences.reading_history_limit(),
        )?;
        self.sessions.lifecycle = Default::default();
        self.sessions.failed.clear();
        self.sessions.queue_retry = None;
        Ok(())
    }
}

pub(super) fn change_preference(
    preferences: &mut Preferences,
    key: &str,
    value: serde_json::Value,
    persist: impl FnOnce(&Preferences) -> Result<()>,
) -> Result<()> {
    let previous = preferences.rust.insert(key.into(), value);
    if let Err(error) = persist(preferences) {
        if let Some(value) = previous {
            preferences.rust.insert(key.into(), value);
        } else {
            preferences.rust.remove(key);
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recording_preserves_imported_privacy_and_rolls_back_failed_preference_saves() {
        let mut preferences = Preferences::default();
        assert!(preferences.record_reading_sessions());
        preferences
            .legacy
            .insert("recordReadingSessions".into(), false.into());
        preferences
            .legacy
            .insert("futureSetting".into(), "preserved".into());
        assert!(!preferences.record_reading_sessions());
        assert!(
            change_preference(
                &mut preferences,
                "recordReadingSessions",
                true.into(),
                |_| Err(Failure::Storage("synthetic failure".into()))
            )
            .is_err()
        );
        assert!(!preferences.record_reading_sessions());
        assert!(!preferences.rust.contains_key("recordReadingSessions"));
        change_preference(
            &mut preferences,
            "recordReadingSessions",
            true.into(),
            |_| Ok(()),
        )
        .unwrap();
        assert!(preferences.record_reading_sessions());
        assert_eq!(preferences.legacy["recordReadingSessions"], false);
        assert_eq!(preferences.legacy["futureSetting"], "preserved");
        preferences
            .rust
            .insert("recordReadingSessions".into(), "unknown".into());
        assert!(!preferences.record_reading_sessions());
    }
    #[test]
    fn context_records_effective_book_layout_and_live_reader_values() {
        let mut preferences = ReaderPreferences::new("a");
        preferences.view_mode = yamantaka_domain::reader_preferences::ReaderMode::Book;
        preferences.reading_theme = ReaderTheme::Sepia;
        preferences.scroll_offset = 123.5;
        let saved = reading_context(
            &preferences,
            1.75,
            Some(outline::ReaderPanelTab::Annotations),
        );
        assert_eq!(saved.page_layout, "facing");
        assert_eq!(saved.reading_mode, "book");
        assert_eq!(saved.reading_theme, "sepia");
        assert_eq!(saved.zoom_level, 1.75);
        assert_eq!(saved.scroll_offset, 123.5);
        assert_eq!(saved.panel_section.as_deref(), Some("annotations"));
        saved.validate().unwrap();
    }
}
