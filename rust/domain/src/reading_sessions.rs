use crate::{Cancellation, Failure, Result};
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderSessionContext {
    pub zoom_level: f64,
    pub page_layout: String,
    pub reading_mode: String,
    pub reading_theme: String,
    pub scroll_offset: f64,
    pub panel_section: Option<String>,
}
impl ReaderSessionContext {
    pub fn validate(&self) -> Result<()> {
        if !self.zoom_level.is_finite()
            || self.zoom_level <= 0.0
            || !self.scroll_offset.is_finite()
            || self.scroll_offset < 0.0
            || [&self.page_layout, &self.reading_mode, &self.reading_theme]
                .into_iter()
                .any(|s| s.is_empty() || s.len() > 128)
            || self.panel_section.as_ref().is_some_and(|s| s.len() > 512)
        {
            return Err(Failure::Invalid(
                "Reading session context is invalid".into(),
            ));
        }
        Ok(())
    }
}
pub struct ReadingSessionStart<'a> {
    pub id: &'a str,
    pub document: &'a str,
    pub started_at: &'a str,
    pub page: u32,
}
pub struct ReadingSessionFinish<'a> {
    pub id: &'a str,
    pub document: &'a str,
    pub ended_at: &'a str,
    pub page: u32,
    pub active_seconds: u64,
    pub context: Option<&'a ReaderSessionContext>,
}
pub trait ReadingSessionRepository {
    fn start_reading_session(
        &mut self,
        start: &ReadingSessionStart<'_>,
        cancel: &Cancellation,
    ) -> Result<()>;
    fn finish_reading_session(
        &mut self,
        finish: &ReadingSessionFinish<'_>,
        cancel: &Cancellation,
    ) -> Result<()>;
}

pub fn validate_session_identity(id: &str, document: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        || document.is_empty()
    {
        return Err(Failure::Invalid(
            "Reading session identity is invalid".into(),
        ));
    }
    Ok(())
}
