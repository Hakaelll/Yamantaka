use crate::{Failure, Result, reader::ReaderPosition, reader_preferences::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceComparison {
    pub primary: String,
    pub document: String,
    pub page: u32,
    pub point: [f64; 2],
    pub zoom_mode: String,
    pub zoom: f64,
    pub theme: String,
    pub rotation: i64,
    pub horizontal_margin: f64,
    pub vertical_margin: f64,
    pub ratio: f32,
    pub follow_page: bool,
    pub follow_zoom: bool,
    pub follow_position: bool,
}

impl WorkspaceComparison {
    pub fn preferences(&self) -> Result<ReaderPreferences> {
        let mut prefs = ReaderPreferences::new(&self.document);
        prefs.zoom_mode = ZoomMode::from_persisted(&self.zoom_mode)?;
        prefs.zoom_level = self.zoom;
        prefs.reading_theme = ReaderTheme::from_persisted(&self.theme)?;
        prefs.page_rotation = PageRotation::from_degrees(self.rotation)?;
        prefs.horizontal_margin = self.horizontal_margin;
        prefs.vertical_margin = self.vertical_margin;
        prefs.validate()?;
        Ok(prefs)
    }

    pub fn validate(&self) -> Result<()> {
        if self.primary.is_empty()
            || self.primary.len() > 4096
            || self.document.is_empty()
            || self.document.len() > 4096
            || !(self.zoom as f32).is_finite()
            || self.zoom as f32 <= 0.0
            || !self.ratio.is_finite()
            || !(0.2..=0.8).contains(&self.ratio)
        {
            return Err(Failure::Invalid(
                "Invalid saved comparison; stored data is preserved".into(),
            ));
        }
        ReaderPosition::at(self.page, self.point)?;
        self.preferences()?;
        Ok(())
    }
}
