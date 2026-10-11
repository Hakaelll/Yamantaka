use serde_json::{Value, json};
use std::fs;
use yamantaka_domain::{
    reader_preferences::{PageRotation, ReaderPreferences, ReaderTheme, ZoomMode},
    reader_tabs::{DocumentTabs, ReaderTab},
    reading_sessions::restoration::SessionPanel,
};
use yamantaka_storage::reader_continuity::{load, save};

fn model() -> DocumentTabs {
    let mut model = DocumentTabs::default();
    let mut preferences = ReaderPreferences::new("document-東京");
    preferences.zoom_mode = ZoomMode::Custom;
    preferences.zoom_level = 1.75;
    preferences.scroll_offset = 1500.0;
    preferences.page_rotation = PageRotation::Deg90;
    preferences.reading_theme = ReaderTheme::Sepia;
    preferences.horizontal_margin = 31.0;
    preferences.vertical_margin = 27.0;
    preferences.controls_auto_hide = false;
    preferences.load_layout_name("spread").unwrap();
    model
        .open(
            ReaderTab::new(
                "first".into(),
                "document-東京".into(),
                8,
                preferences,
                Some(SessionPanel::Annotations),
            )
            .unwrap()
            .with_pinned(true),
            true,
        )
        .unwrap();
    model
        .open(
            ReaderTab::new(
                "second".into(),
                "document-東京".into(),
                2,
                ReaderPreferences::new("document-東京"),
                None,
            )
            .unwrap(),
            true,
        )
        .unwrap();
    model.close("first");
    model
}

#[test]
fn independent_readings_round_trip_and_reopen_with_full_context() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("reader-continuity.json");
    assert!(load(&path).unwrap().is_none());
    save(&path, &model()).unwrap();
    let mut restored = load(&path).unwrap().unwrap();
    assert_eq!(restored.active().unwrap().id(), "second");
    assert_eq!(restored.active().unwrap().page(), 2);
    assert_eq!(
        restored.reopen_last_closed().unwrap().as_deref(),
        Some("first")
    );
    let active = restored.active().unwrap();
    assert_eq!(active.page(), 8);
    assert!(active.pinned());
    assert_eq!(active.panel(), Some(SessionPanel::Annotations));
    let p = active.preferences();
    assert_eq!(p.persisted_layout_name(), "spread");
    assert_eq!(p.zoom_mode, ZoomMode::Custom);
    assert_eq!(p.zoom_level, 1.75);
    assert_eq!(p.scroll_offset, 1500.0);
    assert_eq!(p.page_rotation, PageRotation::Deg90);
    assert_eq!(p.reading_theme, ReaderTheme::Sepia);
    assert_eq!((p.horizontal_margin, p.vertical_margin), (31.0, 27.0));
    assert!(!p.controls_auto_hide);
    assert_eq!(restored.hot_documents(), ["document-東京"]);
}

#[test]
fn save_preserves_unknown_root_tab_and_preferences_fields_across_reopening() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("reader-continuity.json");
    save(&path, &model()).unwrap();
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["future"] = json!({"language":"日本語"});
    value["closed"][0]["future_tab"] = json!([1, 2, 3]);
    value["closed"][0]["preferences"]["future_reader"] = json!({"value": true});
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let mut restored = load(&path).unwrap().unwrap();
    restored.reopen_last_closed().unwrap();
    save(&path, &restored).unwrap();
    let value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(value["future"], json!({"language":"日本語"}));
    assert_eq!(value["tabs"][1]["future_tab"], json!([1, 2, 3]));
    assert_eq!(
        value["tabs"][1]["preferences"]["future_reader"],
        json!({"value": true})
    );
}

#[test]
fn invalid_or_future_snapshot_cannot_be_overwritten_by_a_fresh_session() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("reader-continuity.json");
    save(&path, &model()).unwrap();
    let original: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for mutate in 0..5 {
        let mut value = original.clone();
        match mutate {
            0 => value["version"] = json!(2),
            1 => value["active"] = json!("missing"),
            2 => value["closed"][0]["panel"] = json!("future-panel"),
            3 => value["closed"][0]["preferences"]["zoom"] = json!(-1),
            _ => value["closed"][0]["id"] = value["tabs"][0]["id"].clone(),
        }
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert!(load(&path).is_err());
        assert!(save(&path, &DocumentTabs::default()).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn oversized_input_and_output_fail_without_replacing_existing_data() {
    use yamantaka_storage::reader_continuity::MAX_CONTINUITY_BYTES;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("reader-continuity.json");
    let bytes = vec![b' '; MAX_CONTINUITY_BYTES + 1];
    fs::write(&path, &bytes).unwrap();
    assert!(load(&path).is_err());
    assert!(save(&path, &model()).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::remove_file(&path).unwrap();
    let mut model = DocumentTabs::default();
    model
        .open(
            ReaderTab::new(
                "x".repeat(MAX_CONTINUITY_BYTES),
                "document".into(),
                0,
                ReaderPreferences::new("document"),
                None,
            )
            .unwrap(),
            true,
        )
        .unwrap();
    assert!(save(&path, &model).is_err());
    assert!(!path.exists());
}
