use crate::{Failure, Result, reader::ReaderPosition, reader_preferences::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComparisonLayout {
    #[default]
    Automatic,
    SideBySide,
    Stacked,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceComparison {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reading: Option<ComparisonReading>,
    #[serde(default)]
    pub layout: ComparisonLayout,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll_offset: Option<f64>,
    #[serde(default)]
    pub bidirectional: bool,
    #[serde(default)]
    pub swapped: bool,
    #[serde(default)]
    pub differences: std::collections::BTreeSet<u32>,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComparisonReading {
    pub mode: String,
    pub layout: String,
    pub cover: String,
    pub side: String,
    pub auto_hide: bool,
}
impl From<&ReaderPreferences> for ComparisonReading {
    fn from(preferences: &ReaderPreferences) -> Self {
        Self {
            mode: preferences.view_mode.persisted_name().into(),
            layout: preferences.persisted_layout_name().into(),
            cover: preferences.cover_policy.persisted_name().into(),
            side: preferences.initial_side.persisted_name().into(),
            auto_hide: preferences.controls_auto_hide,
        }
    }
}

impl WorkspaceComparison {
    pub fn preferences(&self) -> Result<ReaderPreferences> {
        let mut prefs = ReaderPreferences::new(&self.document);
        if let Some(reading) = &self.reading {
            prefs.view_mode = ReaderMode::from_persisted(&reading.mode)?;
            prefs.load_layout_name(&reading.layout)?;
            prefs.cover_policy = CoverPolicy::from_persisted(&reading.cover)?;
            prefs.initial_side = InitialSide::from_persisted(&reading.side)?;
            prefs.controls_auto_hide = reading.auto_hide;
        }
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
            || self.scroll_offset.is_some_and(|offset| !offset.is_finite())
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
