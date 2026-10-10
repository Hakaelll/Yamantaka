use crate::{Cancellation, Failure, Result};
use serde::{Deserialize, Serialize};
mod context_decode;
pub mod history;
pub mod restoration;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", from = "StoredContext")]
pub struct ReaderSessionContext {
    pub zoom_level: f64,
    pub page_layout: String,
    pub reading_mode: String,
    pub reading_theme: String,
    pub scroll_offset: f64,
    pub panel_section: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredContext {
    zoom_level: Option<f64>,
    #[serde(default, deserialize_with = "context_decode::name")]
    page_layout: Option<String>,
    #[serde(default, deserialize_with = "context_decode::name")]
    reading_mode: Option<String>,
    #[serde(default, deserialize_with = "context_decode::name")]
    reading_theme: Option<String>,
    scroll_offset: Option<f64>,
    #[serde(default, deserialize_with = "context_decode::panel")]
    panel_section: Option<String>,
}
impl From<StoredContext> for ReaderSessionContext {
    fn from(value: StoredContext) -> Self {
        Self {
            zoom_level: value.zoom_level.unwrap_or(1.0),
            page_layout: value.page_layout.unwrap_or_else(|| "single".into()),
            reading_mode: value.reading_mode.unwrap_or_else(|| "document".into()),
            reading_theme: value.reading_theme.unwrap_or_else(|| "ivory".into()),
            scroll_offset: value.scroll_offset.unwrap_or(0.0),
            panel_section: value.panel_section,
        }
    }
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
    fn reading_history(
        &self,
        _query: &history::ReadingHistoryQuery,
        _governor: &crate::memory::MemoryGovernor,
        _cancel: &Cancellation,
    ) -> Result<history::ReadingHistoryPage> {
        Err(Failure::Unsupported("Reading session history".into()))
    }
    fn trim_reading_history(&mut self, _limit: usize, _cancel: &Cancellation) -> Result<u64> {
        Err(Failure::Unsupported("Trimming reading history".into()))
    }
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
