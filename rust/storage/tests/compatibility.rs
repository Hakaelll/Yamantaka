use std::fs;
#[path = "support/automation.rs"]
mod automation_verification;
#[path = "support/collections.rs"]
mod collection_verification;
#[path = "support/library_grouping.rs"]
mod grouping_verification;
#[path = "support/managed_imports.rs"]
mod managed_import_verification;
#[path = "support/migration_review.rs"]
mod migration_review_verification;
#[path = "support/relations.rs"]
mod relation_verification;
#[path = "support/library_removal.rs"]
mod removal_verification;
#[path = "support/revisions.rs"]
mod revision_verification;
#[path = "support/smart_views.rs"]
mod smart_view_verification;
#[path = "support/tags.rs"]
mod tag_verification;
#[path = "support/workspaces.rs"]
mod workspace_verification;
use yamantaka_domain::{Cancellation, LibraryQuery, LibraryRepository};
use yamantaka_storage::{
    Database,
    identity::exact_duplicate,
    migration::{LegacySource, capture, verify_capture},
    preferences::Preferences,
};

const DOCUMENT: &str = "INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,title) VALUES('original','C:/文献/ensayo.pdf','c:/文献/ensayo.pdf','ensayo.pdf',7,900,'2026-10-08','2026-10-08','2026-10-08','legacy','Original')";

#[test]
fn all_eleven_historical_schemas_preserve_ids_relationships_and_preferences() {
    for version in 1..=11 {
        let mut db = Database::in_memory_at(version).unwrap();
        db.execute_batch(DOCUMENT).unwrap();
        db.execute_batch(
            "INSERT INTO bookmarks VALUES('bookmark','original',2,'A/B','2026-10-08','2026-10-08')",
        )
        .unwrap();
        db.migrate_current().unwrap();
        db.migrate_current().unwrap();
        assert_eq!(db.schema_version().unwrap(), 11);
        assert_eq!(db.get("original").unwrap().unwrap().id, "original");
        assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 1);
        db.integrity().unwrap();
    }
    workspace_verification::preserved_split_references(DOCUMENT);
}

#[test]
fn wal_capture_includes_committed_frames_and_never_overwrites() {
    let root = tempfile::tempdir().unwrap();
    let source_path = root.path().join("legacy.sqlite3");
    let source = Database::open(&source_path).unwrap();
    source.execute_batch(DOCUMENT).unwrap();
    source.execute_batch("INSERT INTO document_covers VALUES('original',4,'legacy-small','legacy-large','2026-10-08T00:00:00Z','2026-10-08T00:00:00Z',1,'manual',4,'contain',0.75);").unwrap();
    assert!(source_path.with_file_name("legacy.sqlite3-wal").exists());
    let prefs = root.path().join("shared_preferences.json");
    fs::write(
        &prefs,
        r#"{"darkMode":true,"flutter.languageCode":"es","custom":42}"#,
    )
    .unwrap();
    let legacy = LegacySource {
        database: source_path,
        preferences: prefs.clone(),
    };
    let target = root.path().join("rust-store");
    let receipt = capture(&legacy, &target, || Ok(true), &Cancellation::default()).unwrap();
    assert_eq!(receipt.imported_rows["documents"], 1);
    verify_capture(&target, &Cancellation::default()).unwrap();
    let repeated = capture(&legacy, &target, || Ok(true), &Cancellation::default()).unwrap();
    assert_eq!(repeated.completed_at, receipt.completed_at);
    assert_eq!(
        repeated.captured_database_sha256,
        receipt.captured_database_sha256
    );
    assert_eq!(
        repeated.preserved_data_sha256,
        receipt.preserved_data_sha256
    );
    let captured_preferences = target.join("preferences.json");
    let original_preferences = fs::read(&captured_preferences).unwrap();
    let mut edited = Preferences::load(&captured_preferences).unwrap();
    edited.rust.insert("unexpected-change".into(), true.into());
    edited.save(&captured_preferences).unwrap();
    let accepted_changed_preferences = verify_capture(&target, &Cancellation::default()).is_ok();
    fs::write(&captured_preferences, &original_preferences).unwrap();
    fs::remove_file(&captured_preferences).unwrap();
    let accepted_missing_preferences = verify_capture(&target, &Cancellation::default()).is_ok();
    fs::write(&captured_preferences, &original_preferences).unwrap();
    assert_eq!(
        (accepted_changed_preferences, accepted_missing_preferences),
        (false, false),
        "A verified migration must retain its exact preferences file"
    );
    let copied = rusqlite::Connection::open_with_flags(
        target.join("yamantaka.sqlite3"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let choice: (u32,String,String,String,u32,f64) = copied.query_row(
        "SELECT source_page_index,selection_mode,small_thumbnail_path,large_thumbnail_path,algorithm_version,page_aspect_ratio FROM document_covers WHERE document_id='original'", [],
        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
    ).unwrap();
    assert_eq!(
        choice,
        (4, "manual".into(), String::new(), String::new(), 4, 0.75)
    );
    assert_eq!(source.cover_source_page("original").unwrap(), Some(4));
    assert_eq!(
        Preferences::load(&target.join("preferences.json"))
            .unwrap()
            .get("languageCode")
            .unwrap(),
        "es"
    );
    assert!(prefs.exists());
    assert_eq!(
        source.get("original").unwrap().unwrap().display_title(),
        "Original"
    );
    source
        .execute_batch("UPDATE documents SET title='Changed source'")
        .unwrap();
    assert!(capture(&legacy, &target, || Ok(true), &Cancellation::default()).is_err());
    migration_review_verification::verify(root.path());
    managed_import_verification::verify(root.path());
}

#[test]
fn live_flutter_and_cancelled_capture_leave_no_destination() {
    let root = tempfile::tempdir().unwrap();
    let source = LegacySource {
        database: root.path().join("missing"),
        preferences: root.path().join("missing-prefs"),
    };
    let target = root.path().join("target");
    assert!(capture(&source, &target, || Ok(false), &Cancellation::default()).is_err());
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(capture(&source, &target, || Ok(true), &cancel).is_err());
    assert!(!target.exists());
}

#[test]
fn overrides_win_and_queries_are_bounded_and_escape_wildcards() {
    let mut db = Database::in_memory_at(11).unwrap();
    db.execute_batch(DOCUMENT).unwrap();
    db.execute_batch("UPDATE documents SET title_override='Manual 100%',source_title='Reanalysis'")
        .unwrap();
    assert_eq!(
        db.query(&LibraryQuery {
            text: "%".into(),
            page_size: 5000,
            ..Default::default()
        })
        .unwrap()
        .len(),
        1
    );
    assert_eq!(
        db.get("original").unwrap().unwrap().display_title(),
        "Manual 100%"
    );
    db.save_position("original", 9000).unwrap();
    let saved = db.get("original").unwrap().unwrap();
    assert_eq!(saved.last_page_index, 6);
    assert_eq!(saved.reading_progress, 1.0);
    collection_verification::verify(&mut db);
    smart_view_verification::verify(&mut db);
    tag_verification::verify(&mut db);
    removal_verification::verify();
    grouping_verification::verify();
    relation_verification::verify(DOCUMENT);
    revision_verification::verify(DOCUMENT);
    automation_verification::verify();
}

#[test]
fn partial_fingerprint_collision_is_not_an_exact_duplicate() {
    let root = tempfile::tempdir().unwrap();
    let left = root.path().join("left.pdf");
    let right = root.path().join("right.pdf");
    let mut content = vec![42; 65537];
    fs::write(&left, &content).unwrap();
    content[65536] = 43;
    fs::write(&right, &content).unwrap();
    assert!(!exact_duplicate(&left, &right, &Cancellation::default()).unwrap());
    fs::write(&right, fs::read(&left).unwrap()).unwrap();
    assert!(exact_duplicate(&left, &right, &Cancellation::default()).unwrap());
}

#[test]
fn atomic_preferences_keep_unknown_keys_and_reject_future_versions() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("preferences.json");
    let mut prefs = Preferences::default();
    prefs
        .rust
        .insert("future".into(), serde_json::json!({"rtl": "مرحبا"}));
    prefs.save(&path).unwrap();
    prefs.save(&path).unwrap();
    assert_eq!(
        Preferences::load(&path).unwrap().get("future"),
        prefs.get("future")
    );
    fs::write(&path, r#"{"version":2,"legacy":{}}"#).unwrap();
    assert!(Preferences::load(&path).is_err());
}
