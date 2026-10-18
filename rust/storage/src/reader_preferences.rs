//! Per-document preferences stay in the unmodified Flutter schema v11.
//! Unknown persisted enum names produce a recoverable error, never defaults.
use crate::{Database, database::storage};
use rusqlite::{OptionalExtension, params};
use yamantaka_domain::{
    Failure, Result,
    reader_preferences::{
        CoverPolicy, InitialSide, PageRotation, ReaderMode, ReaderPreferences, ReaderTheme,
        ZoomMode,
    },
};

struct StoredPreferences {
    view_mode: String,
    reading_theme: String,
    zoom_mode: String,
    zoom_level: f64,
    page_layout: String,
    cover_policy: String,
    initial_side: String,
    page_rotation: i64,
    horizontal_margin: f64,
    vertical_margin: f64,
    controls_auto_hide: i64,
    scroll_offset: f64,
    updated_at: String,
}

pub fn load(database: &Database, document_id: &str) -> Result<Option<ReaderPreferences>> {
    let stored = database
        .connection
        .query_row(
            "SELECT view_mode,reading_theme,zoom_mode,zoom_level,page_layout,
        cover_policy,initial_side,page_rotation,horizontal_margin,vertical_margin,
        controls_auto_hide,scroll_offset,updated_at FROM document_preferences WHERE document_id=?",
            [document_id],
            |row| {
                Ok(StoredPreferences {
                    view_mode: row.get(0)?,
                    reading_theme: row.get(1)?,
                    zoom_mode: row.get(2)?,
                    zoom_level: row.get(3)?,
                    page_layout: row.get(4)?,
                    cover_policy: row.get(5)?,
                    initial_side: row.get(6)?,
                    page_rotation: row.get(7)?,
                    horizontal_margin: row.get(8)?,
                    vertical_margin: row.get(9)?,
                    controls_auto_hide: row.get(10)?,
                    scroll_offset: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        )
        .optional()
        .map_err(storage)?;
    let Some(row) = stored else {
        return Ok(None);
    };
    let mut preferences = ReaderPreferences::new(document_id);
    preferences.view_mode = ReaderMode::from_persisted(&row.view_mode)?;
    preferences.reading_theme = ReaderTheme::from_persisted(&row.reading_theme)?;
    preferences.zoom_mode = ZoomMode::from_persisted(&row.zoom_mode)?;
    preferences.zoom_level = row.zoom_level;
    preferences.load_layout_name(&row.page_layout)?;
    preferences.cover_policy = CoverPolicy::from_persisted(&row.cover_policy)?;
    preferences.initial_side = InitialSide::from_persisted(&row.initial_side)?;
    preferences.page_rotation = PageRotation::from_degrees(row.page_rotation)?;
    preferences.horizontal_margin = row.horizontal_margin;
    preferences.vertical_margin = row.vertical_margin;
    preferences.controls_auto_hide = match row.controls_auto_hide {
        0 => false,
        1 => true,
        _ => {
            return Err(Failure::Invalid(
                "Invalid saved reader controls visibility".into(),
            ));
        }
    };
    preferences.scroll_offset = row.scroll_offset;
    validate_timestamp(&row.updated_at)?;
    preferences.updated_at = Some(row.updated_at);
    preferences.validate()?;
    Ok(Some(preferences))
}

fn validate_timestamp(value: &str) -> Result<()> {
    // Flutter writes UTC but historical ISO DateTime text may omit a timezone.
    if chrono::DateTime::parse_from_rfc3339(value).is_err()
        && chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f").is_err()
    {
        return Err(Failure::Invalid(
            "Invalid saved reader preference ISO timestamp".into(),
        ));
    }
    Ok(())
}

/// A single atomic upsert updates only recognized preference columns. Extension
/// columns, document metadata/progress and all dependent rows remain intact.
pub fn save(
    database: &mut Database,
    document_id: &str,
    preferences: &ReaderPreferences,
) -> Result<()> {
    preferences.validate()?;
    if document_id != preferences.document_id {
        return Err(Failure::Invalid(
            "Reader preference document ID does not match its owner".into(),
        ));
    }
    let timestamp = preferences
        .updated_at
        .clone()
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    validate_timestamp(&timestamp)?;
    database
        .connection
        .execute(
            "INSERT INTO document_preferences (
            document_id,view_mode,reading_theme,zoom_mode,zoom_level,page_layout,
            cover_policy,initial_side,page_rotation,horizontal_margin,vertical_margin,
            controls_auto_hide,scroll_offset,updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
        ON CONFLICT(document_id) DO UPDATE SET
            view_mode=excluded.view_mode,reading_theme=excluded.reading_theme,
            zoom_mode=excluded.zoom_mode,zoom_level=excluded.zoom_level,
            page_layout=excluded.page_layout,cover_policy=excluded.cover_policy,
            initial_side=excluded.initial_side,page_rotation=excluded.page_rotation,
            horizontal_margin=excluded.horizontal_margin,vertical_margin=excluded.vertical_margin,
            controls_auto_hide=excluded.controls_auto_hide,scroll_offset=excluded.scroll_offset,
            updated_at=excluded.updated_at",
            params![
                document_id,
                preferences.view_mode.persisted_name(),
                preferences.reading_theme.persisted_name(),
                preferences.zoom_mode.persisted_name(),
                preferences.zoom_level,
                preferences.persisted_layout_name(),
                preferences.cover_policy.persisted_name(),
                preferences.initial_side.persisted_name(),
                preferences.page_rotation.degrees(),
                preferences.horizontal_margin,
                preferences.vertical_margin,
                preferences.controls_auto_hide,
                preferences.scroll_offset,
                timestamp
            ],
        )
        .map_err(storage)?;
    Ok(())
}
