use super::*;
use serde::Deserialize;
use yamantaka_domain::reader_tabs::DocumentTabs;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct LegacySplit {
    enabled: bool,
    axis: String,
    ratio: Option<f32>,
    secondary_tab_id: Option<String>,
    sync_page: bool,
    sync_zoom: bool,
    comparison_mode: bool,
}

pub(super) fn decode(
    split: &serde_json::Value,
    tabs: &DocumentTabs,
) -> Result<Option<WorkspaceComparison>> {
    let split: LegacySplit = serde_json::from_value(split.clone()).map_err(storage)?;
    if !split.enabled {
        return Ok(None);
    }
    if split.comparison_mode {
        return Err(invalid(
            "Saved overlay settings are preserved but cannot yet be displayed in the native reader",
        ));
    }
    let layout = match split.axis.as_str() {
        "" | "vertical" => ComparisonLayout::SideBySide,
        "horizontal" => ComparisonLayout::Stacked,
        _ => {
            return Err(invalid(
                "Unknown saved comparison axis; original settings are preserved",
            ));
        }
    };
    let primary = tabs
        .active()
        .ok_or_else(|| invalid("The saved comparison has no main reading"))?;
    let secondary = tabs
        .tabs()
        .iter()
        .find(|tab| Some(tab.id()) == split.secondary_tab_id.as_deref())
        .ok_or_else(|| {
            invalid("The saved reference reading is missing; original settings are preserved")
        })?;
    let prefs = secondary.preferences();
    let comparison = WorkspaceComparison {
        layout,
        scroll_offset: Some(prefs.scroll_offset),
        primary: primary.document().into(),
        document: secondary.document().into(),
        page: secondary.page(),
        point: prefs.page_rotation.unrotate_point([0.5, 0.0]),
        zoom_mode: prefs.zoom_mode.persisted_name().into(),
        zoom: prefs.zoom_level,
        theme: prefs.reading_theme.persisted_name().into(),
        rotation: prefs.page_rotation.degrees().into(),
        horizontal_margin: prefs.horizontal_margin,
        vertical_margin: prefs.vertical_margin,
        ratio: split.ratio.unwrap_or(0.5),
        follow_page: split.sync_page,
        follow_zoom: split.sync_zoom,
        follow_position: false,
    };
    comparison.validate()?;
    Ok(Some(comparison))
}
