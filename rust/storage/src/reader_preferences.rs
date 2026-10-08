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

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::{LibraryRepository, PageLayout};

    fn insert_document(db: &Database) {
        db.execute_batch("INSERT INTO documents(
            id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,
            fingerprint,last_page_index,reading_progress,title_override,is_favorite)
            VALUES('d','C:/原本.pdf','c:/原本.pdf','原本.pdf',20,100,'2026-10-08T12:00:00Z',
            '2026-10-08T12:00:00Z','2026-10-08T12:00:00Z','untouched',7,0.4,'Manual title',1);
            INSERT INTO bookmarks(id,document_id,page_index,label,created_at,updated_at)
            VALUES('bookmark','d',7,'Keep me','2026-10-08T12:00:00Z','2026-10-08T12:00:00Z');").unwrap();
    }
    fn populated(db: &Database) {
        db.execute_batch("INSERT INTO document_preferences(
            document_id,view_mode,reading_theme,zoom_mode,zoom_level,page_layout,cover_policy,initial_side,
            page_rotation,horizontal_margin,vertical_margin,controls_auto_hide,scroll_offset,updated_at)
            VALUES('d','book','black','custom',2.75,'facing','firstPageIsCover','right',
            270,17.25,48.5,0,1839.125,'2026-10-08T12:34:56.123456+02:00');").unwrap();
    }
    fn serialized_row(db: &Database, table: &str) -> Vec<Vec<rusqlite::types::Value>> {
        let mut stmt = db
            .connection
            .prepare(&format!("SELECT * FROM {table}"))
            .unwrap();
        let count = stmt.column_count();
        stmt.query_map([], |row| (0..count).map(|i| row.get(i)).collect())
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    }
    #[test]
    fn absent_preferences_are_not_materialized_or_inferred_from_document() {
        let db = Database::in_memory_at(11).unwrap();
        insert_document(&db);
        assert!(db.get_reader_preferences("d").unwrap().is_none());
        assert!(db.get_reader_preferences("missing").unwrap().is_none());
        assert!(serialized_row(&db, "document_preferences").is_empty());
        assert_eq!(db.schema_version().unwrap(), 11);
    }
    #[test]
    fn real_v1_preferences_survive_migration_without_replacing_manual_values() {
        let mut db = Database::in_memory_at(1).unwrap();
        db.execute_batch("INSERT INTO documents(
            id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint)
            VALUES('d','C:/original.pdf','c:/original.pdf','original.pdf',20,100,
            '2026-10-08T12:00:00Z','2026-10-08T12:00:00Z','2026-10-08T12:00:00Z','untouched');
            INSERT INTO document_preferences(document_id,view_mode,reading_theme,zoom_mode,zoom_level,
            page_layout,page_rotation,horizontal_margin,vertical_margin,controls_auto_hide,scroll_offset,updated_at)
            VALUES('d','book','sepia','fitWidth',1.75,'facing',180,12.5,36.25,0,940.5,
            '2020-02-03T04:05:06.123456Z');").unwrap();
        let original_documents = serialized_row(&db, "documents");
        db.migrate_current().unwrap();
        let p = db.get_reader_preferences("d").unwrap().unwrap();
        assert_eq!(p.view_mode, ReaderMode::Book);
        assert_eq!(p.reading_theme, ReaderTheme::Sepia);
        assert_eq!(p.zoom_mode, ZoomMode::FitWidth);
        assert_eq!(p.zoom_level, 1.75);
        assert_eq!(p.page_layout, PageLayout::Spread);
        assert_eq!(p.page_rotation, PageRotation::Deg180);
        assert_eq!((p.horizontal_margin, p.vertical_margin), (12.5, 36.25));
        assert_eq!(p.scroll_offset, 940.5);
        assert!(!p.controls_auto_hide);
        assert_eq!(p.cover_policy, CoverPolicy::NoCover);
        assert_eq!(p.initial_side, InitialSide::Left);
        assert_eq!(p.updated_at.as_deref(), Some("2020-02-03T04:05:06.123456Z"));
        // Migration appends columns; each original document value is preserved.
        let migrated = serialized_row(&db, "documents");
        assert_eq!(
            &migrated[0][..original_documents[0].len()],
            original_documents[0]
        );
        let original_preferences = serialized_row(&db, "document_preferences");
        db.save_reader_preferences("d", &p).unwrap();
        assert_eq!(
            serialized_row(&db, "document_preferences"),
            original_preferences
        );
    }
    #[test]
    fn populated_flutter_row_roundtrips_every_field_and_extension_column() {
        let mut db = Database::in_memory_at(11).unwrap();
        insert_document(&db);
        populated(&db);
        db.execute_batch(
            "ALTER TABLE document_preferences ADD COLUMN future_manual_setting TEXT;
            UPDATE document_preferences SET future_manual_setting='user decision';",
        )
        .unwrap();
        let original = serialized_row(&db, "document_preferences");
        let p = db.get_reader_preferences("d").unwrap().unwrap();
        assert_eq!(p.view_mode, ReaderMode::Book);
        assert_eq!(p.reading_theme, ReaderTheme::Black);
        assert_eq!(p.zoom_mode, ZoomMode::Custom);
        assert_eq!(p.zoom_level, 2.75);
        assert_eq!(p.page_layout, PageLayout::Spread);
        assert_eq!(p.cover_policy, CoverPolicy::FirstPageIsCover);
        assert_eq!(p.initial_side, InitialSide::Right);
        assert_eq!(p.page_rotation, PageRotation::Deg270);
        assert_eq!((p.horizontal_margin, p.vertical_margin), (17.25, 48.5));
        assert!(!p.controls_auto_hide);
        assert_eq!(p.scroll_offset, 1839.125);
        db.save_reader_preferences("d", &p).unwrap();
        assert_eq!(serialized_row(&db, "document_preferences"), original);
    }
    #[test]
    fn changes_preserve_document_progress_related_rows_and_reopen_exactly() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("independent.sqlite3");
        let mut db = Database::open(&path).unwrap();
        insert_document(&db);
        populated(&db);
        let documents = serialized_row(&db, "documents");
        let bookmarks = serialized_row(&db, "bookmarks");
        let mut p = db.get_reader_preferences("d").unwrap().unwrap();
        p.reading_theme = ReaderTheme::Gray;
        p.page_layout = PageLayout::Continuous;
        p.zoom_mode = ZoomMode::FitWidth;
        p.page_rotation = PageRotation::Deg90;
        p.horizontal_margin = 0.0;
        p.controls_auto_hide = true;
        db.save_reader_preferences("d", &p).unwrap();
        assert_eq!(serialized_row(&db, "documents"), documents);
        assert_eq!(serialized_row(&db, "bookmarks"), bookmarks);
        drop(db);
        let db = Database::open(&path).unwrap();
        let loaded = db.get_reader_preferences("d").unwrap().unwrap();
        assert_eq!(loaded.reading_theme, p.reading_theme);
        assert_eq!(loaded.page_layout, p.page_layout);
        assert_eq!(loaded.zoom_mode, p.zoom_mode);
        assert_eq!(loaded.page_rotation, p.page_rotation);
        assert_eq!(loaded.horizontal_margin, p.horizontal_margin);
        assert_eq!(loaded.updated_at, p.updated_at);
        assert!(loaded.controls_auto_hide);
        db.integrity().unwrap();
    }
    #[test]
    fn invalid_preference_save_never_mutates_original_row() {
        let mut db = Database::in_memory_at(11).unwrap();
        insert_document(&db);
        populated(&db);
        let original = serialized_row(&db, "document_preferences");
        let p = db.get_reader_preferences("d").unwrap().unwrap();
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 16385.0] {
            let mut invalid = p.clone();
            invalid.vertical_margin = value;
            assert!(db.save_reader_preferences("d", &invalid).is_err());
        }
        for value in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
            let mut invalid = p.clone();
            invalid.zoom_level = value;
            assert!(db.save_reader_preferences("d", &invalid).is_err());
        }
        let mut invalid = p.clone();
        invalid.scroll_offset = f64::INFINITY;
        assert!(db.save_reader_preferences("d", &invalid).is_err());
        invalid = p.clone();
        invalid.updated_at = Some("not ISO".into());
        assert!(db.save_reader_preferences("d", &invalid).is_err());
        assert!(db.save_reader_preferences("another", &p).is_err());
        assert_eq!(serialized_row(&db, "document_preferences"), original);
        let missing = ReaderPreferences::new("absent");
        assert!(db.save_reader_preferences("absent", &missing).is_err());
        assert_eq!(serialized_row(&db, "document_preferences"), original);
    }
    #[test]
    fn malformed_historical_rows_fail_recoverably_without_silent_reset() {
        let db = Database::in_memory_at(11).unwrap();
        insert_document(&db);
        populated(&db);
        for update in [
            "reading_theme='futureCustomTheme'",
            "page_layout='unknownLayout'",
            "cover_policy='hasCover'",
            "page_rotation=45",
            "horizontal_margin=-1",
            "scroll_offset=1e999",
            "updated_at='invalid'",
        ] {
            db.execute_batch(&format!("UPDATE document_preferences SET {update}"))
                .unwrap();
            let before = serialized_row(&db, "document_preferences");
            assert!(
                matches!(db.get_reader_preferences("d"), Err(Failure::Invalid(_))),
                "{update}"
            );
            assert_eq!(serialized_row(&db, "document_preferences"), before);
            db.execute_batch("DELETE FROM document_preferences")
                .unwrap();
            populated(&db);
        }
    }
    #[test]
    fn accepted_legacy_alias_is_preserved_until_explicit_layout_change() {
        let mut db = Database::in_memory_at(11).unwrap();
        insert_document(&db);
        populated(&db);
        db.execute_batch("UPDATE document_preferences SET page_layout='spread'")
            .unwrap();
        let mut p = db.get_reader_preferences("d").unwrap().unwrap();
        assert_eq!(p.page_layout, PageLayout::Spread);
        p.reading_theme = ReaderTheme::Sepia;
        db.save_reader_preferences("d", &p).unwrap();
        assert_eq!(
            db.get_reader_preferences("d")
                .unwrap()
                .unwrap()
                .persisted_layout_name(),
            "spread"
        );
        p.page_layout = PageLayout::Single;
        db.save_reader_preferences("d", &p).unwrap();
        assert_eq!(
            db.get_reader_preferences("d")
                .unwrap()
                .unwrap()
                .persisted_layout_name(),
            "single"
        );
    }
    #[test]
    fn new_preferences_cover_all_themes_zoom_modes_pairing_and_rotation() {
        let mut db = Database::in_memory_at(11).unwrap();
        insert_document(&db);
        let mut p = ReaderPreferences::new("d");
        for theme in [
            ReaderTheme::Light,
            ReaderTheme::Ivory,
            ReaderTheme::Sepia,
            ReaderTheme::Gray,
            ReaderTheme::Dark,
            ReaderTheme::Black,
        ] {
            for zoom in [ZoomMode::Custom, ZoomMode::FitWidth, ZoomMode::FitPage] {
                for rotation in [
                    PageRotation::Deg0,
                    PageRotation::Deg90,
                    PageRotation::Deg180,
                    PageRotation::Deg270,
                ] {
                    p.reading_theme = theme;
                    p.zoom_mode = zoom;
                    p.page_rotation = rotation;
                    p.cover_policy = CoverPolicy::NoCover;
                    p.initial_side = InitialSide::Left;
                    db.save_reader_preferences("d", &p).unwrap();
                    let loaded = db.get_reader_preferences("d").unwrap().unwrap();
                    assert_eq!(loaded.reading_theme, theme);
                    assert_eq!(loaded.zoom_mode, zoom);
                    assert_eq!(loaded.page_rotation, rotation);
                    assert!(loaded.updated_at.is_some());
                }
            }
        }
    }
}
