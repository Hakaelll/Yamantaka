use super::ReaderSessionContext;
use crate::{
    Failure, Result,
    reader_preferences::{ReaderMode, ReaderPreferences, ReaderTheme, ZoomMode, parse_layout_name},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPanel {
    Outline,
    Thumbnails,
    Search,
    Bookmarks,
    Annotations,
    Text,
}
#[derive(Debug, Clone)]
pub struct ReaderSessionRestoration {
    zoom: f64,
    layout: String,
    mode: ReaderMode,
    theme: ReaderTheme,
    scroll: f64,
    panel: Option<SessionPanel>,
}
impl TryFrom<ReaderSessionContext> for ReaderSessionRestoration {
    type Error = Failure;
    fn try_from(context: ReaderSessionContext) -> Result<Self> {
        context.validate()?;
        let zoom = context.zoom_level as f32;
        if !zoom.is_finite() || zoom <= 0.0 || !(context.scroll_offset as f32).is_finite() {
            return Err(Failure::Invalid("Stored reading context cannot be represented by the reader; original data is preserved".into()));
        }
        parse_layout_name(&context.page_layout)?;
        let mode = ReaderMode::from_persisted(&context.reading_mode)?;
        let theme = ReaderTheme::from_persisted(&context.reading_theme)?;
        let panel = match context.panel_section.as_deref() {
            None => None,
            Some("outline") => Some(SessionPanel::Outline),
            Some("thumbnails") => Some(SessionPanel::Thumbnails),
            Some("search") => Some(SessionPanel::Search),
            Some("bookmarks") => Some(SessionPanel::Bookmarks),
            Some("annotations") => Some(SessionPanel::Annotations),
            Some("text") => Some(SessionPanel::Text),
            Some(_) => {
                return Err(Failure::Invalid(
                    "Unknown stored reading panel; original data is preserved".into(),
                ));
            }
        };
        Ok(Self {
            zoom: context.zoom_level,
            layout: context.page_layout,
            mode,
            theme,
            scroll: context.scroll_offset,
            panel,
        })
    }
}
impl ReaderSessionRestoration {
    pub fn panel(&self) -> Option<SessionPanel> {
        self.panel
    }
    pub fn apply(&self, current: &ReaderPreferences) -> Result<ReaderPreferences> {
        let mut restored = current.clone();
        restored.zoom_level = self.zoom;
        restored.zoom_mode = ZoomMode::Custom;
        restored.load_layout_name(&self.layout)?;
        restored.view_mode = self.mode;
        restored.reading_theme = self.theme;
        restored.scroll_offset = self.scroll;
        restored.updated_at = None;
        restored.validate()?;
        Ok(restored)
    }
}
