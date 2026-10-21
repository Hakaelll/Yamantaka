use std::{fs, path::Path};
use yamantaka_domain::{Cancellation, Failure};
use yamantaka_storage::{
    Database,
    migration::{LegacySource, MigrationProgress, capture_reviewed, review, verify_capture},
};

pub fn verify(root: &Path) {
    let database = root.join("review-source.sqlite3");
    let source = Database::open(&database).unwrap();
    source.execute_batch(super::DOCUMENT).unwrap();
    source.execute_batch("CREATE TABLE extra_data(id INTEGER PRIMARY KEY,body BLOB NOT NULL); INSERT INTO extra_data VALUES(1,zeroblob(2000000));").unwrap();
    let preferences = root.join("review-preferences.json");
    fs::write(&preferences, r#"{"flutter.languageCode":"es","custom":42}"#).unwrap();
    let legacy = LegacySource {
        database,
        preferences: preferences.clone(),
    };
    let plan = review(&legacy, || Ok(true), &Cancellation::default()).unwrap();
    assert_eq!(plan.source_schema(), 11);
    assert_eq!(plan.imported_rows()["documents"], 1);
    assert_eq!(plan.imported_rows()["extra_data"], 1);
    assert_eq!(plan.preferences_count(), 2);
    assert!(review(&legacy, || Ok(false), &Cancellation::default()).is_err());
    for phase in 0..3 {
        let target = root.join(format!("review-cancel-{phase}"));
        let cancel = Cancellation::default();
        let mut saw_partial_copy = false;
        let result = capture_reviewed(
            &plan,
            &target,
            || Ok(true),
            &cancel,
            |progress| match progress {
                MigrationProgress::Copying {
                    copied_pages,
                    total_pages,
                } if phase == 0 => {
                    assert!(copied_pages > 0 && copied_pages < total_pages);
                    saw_partial_copy = true;
                    cancel.cancel();
                }
                MigrationProgress::Validating if phase == 1 => cancel.cancel(),
                MigrationProgress::Installing if phase == 2 => cancel.cancel(),
                _ => {}
            },
        );
        assert!(matches!(result, Err(Failure::Cancelled)));
        assert!(!target.exists());
        assert!(phase != 0 || saw_partial_copy);
        assert!(!fs::read_dir(root).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".yamantaka-import-")
        }));
    }
    let final_gate = root.join("review-final-gate");
    let closed = std::cell::Cell::new(true);
    let result = capture_reviewed(
        &plan,
        &final_gate,
        || Ok(closed.get()),
        &Cancellation::default(),
        |progress| {
            if matches!(progress, MigrationProgress::Installing) {
                closed.set(false);
            }
        },
    );
    assert!(matches!(result, Err(Failure::LegacyRunning)));
    assert!(!final_gate.exists());
    let occupied = root.join("review-occupied-at-install");
    let result = capture_reviewed(
        &plan,
        &occupied,
        || Ok(true),
        &Cancellation::default(),
        |progress| {
            if matches!(progress, MigrationProgress::Installing) {
                fs::create_dir(&occupied).unwrap();
                fs::write(occupied.join("keep.txt"), "Existing destination").unwrap();
            }
        },
    );
    assert!(matches!(result, Err(Failure::DestinationOccupied)));
    assert_eq!(
        fs::read_to_string(occupied.join("keep.txt")).unwrap(),
        "Existing destination"
    );
    let target = root.join("review-copied");
    let receipt = capture_reviewed(
        &plan,
        &target,
        || Ok(true),
        &Cancellation::default(),
        |_| {},
    )
    .unwrap();
    assert_eq!(receipt.version, 3);
    verify_capture(&target, &Cancellation::default()).unwrap();
    let repeated = capture_reviewed(
        &plan,
        &target,
        || Ok(true),
        &Cancellation::default(),
        |_| {},
    )
    .unwrap();
    assert_eq!(receipt.completed_at, repeated.completed_at);
    source.execute_batch("PRAGMA user_version=10;").unwrap();
    assert!(matches!(
        capture_reviewed(
            &plan,
            &root.join("review-stale-schema"),
            || Ok(true),
            &Cancellation::default(),
            |_| {}
        ),
        Err(Failure::MigrationSourceChanged)
    ));
    source
        .execute_batch(
            "PRAGMA user_version=11; ALTER TABLE extra_data RENAME COLUMN body TO changed_body;",
        )
        .unwrap();
    assert!(matches!(
        capture_reviewed(
            &plan,
            &root.join("review-stale-columns"),
            || Ok(true),
            &Cancellation::default(),
            |_| {}
        ),
        Err(Failure::MigrationSourceChanged)
    ));
    source
        .execute_batch("ALTER TABLE extra_data RENAME COLUMN changed_body TO body;")
        .unwrap();
    let plan = review(&legacy, || Ok(true), &Cancellation::default()).unwrap();
    source
        .execute_batch("UPDATE documents SET title='Changed after review'")
        .unwrap();
    let refused = root.join("review-stale-database");
    assert!(
        capture_reviewed(
            &plan,
            &refused,
            || Ok(true),
            &Cancellation::default(),
            |_| {}
        )
        .is_err()
    );
    assert!(!refused.exists());
    let plan = review(&legacy, || Ok(true), &Cancellation::default()).unwrap();
    fs::write(&preferences, r#"{"flutter.languageCode":"en","custom":42}"#).unwrap();
    let refused = root.join("review-stale-preferences");
    assert!(
        capture_reviewed(
            &plan,
            &refused,
            || Ok(true),
            &Cancellation::default(),
            |_| {}
        )
        .is_err()
    );
    assert!(!refused.exists());
    assert!(fs::read_to_string(&preferences).unwrap().contains("en"));
    // A main-file hash alone cannot establish the contents of an active WAL.
    let copy = rusqlite::Connection::open(target.join("yamantaka.sqlite3")).unwrap();
    copy.execute_batch("PRAGMA journal_mode=WAL; UPDATE documents SET title='WAL-only change';")
        .unwrap();
    let receipt_path = target.join("migration-receipt.json");
    let mut record: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    record["captured_database_sha256"] = yamantaka_storage::identity::full_sha256(
        &target.join("yamantaka.sqlite3"),
        &Cancellation::default(),
    )
    .unwrap()
    .into();
    fs::write(&receipt_path, serde_json::to_vec(&record).unwrap()).unwrap();
    assert!(verify_capture(&target, &Cancellation::default()).is_err());
    drop(copy);
    for version in 1..=11 {
        let historical = Database::in_memory_at(version).unwrap();
        historical.execute_batch(super::DOCUMENT).unwrap();
        let path = root.join(format!("review-v{version}.sqlite3"));
        historical.snapshot(&path).unwrap();
        let legacy = LegacySource {
            database: path,
            preferences: preferences.clone(),
        };
        let plan = review(&legacy, || Ok(true), &Cancellation::default()).unwrap();
        assert_eq!(plan.source_schema(), version);
        let target = root.join(format!("review-copy-v{version}"));
        let receipt = capture_reviewed(
            &plan,
            &target,
            || Ok(true),
            &Cancellation::default(),
            |_| {},
        )
        .unwrap();
        assert_eq!(receipt.imported_rows["documents"], 1);
        verify_capture(&target, &Cancellation::default()).unwrap();
    }
    verify_schema_and_cleanup(root, &legacy);
}

fn verify_schema_and_cleanup(root: &Path, legacy: &LegacySource) {
    let plan = review(legacy, || Ok(true), &Cancellation::default()).unwrap();
    let target = root.join("review-schema-capture");
    capture_reviewed(
        &plan,
        &target,
        || Ok(true),
        &Cancellation::default(),
        |_| {},
    )
    .unwrap();
    let copy = rusqlite::Connection::open(target.join("yamantaka.sqlite3")).unwrap();
    copy.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_checkpoint(TRUNCATE);")
        .unwrap();
    let receipt_path = target.join("migration-receipt.json");
    let mut record: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    let before = yamantaka_storage::identity::full_sha256(
        &target.join("yamantaka.sqlite3"),
        &Cancellation::default(),
    )
    .unwrap();
    record["captured_database_sha256"] = before.clone().into();
    fs::write(&receipt_path, serde_json::to_vec(&record).unwrap()).unwrap();
    verify_capture(&target, &Cancellation::default()).unwrap();
    copy.execute_batch("CREATE INDEX wal_only_schema ON documents(title);")
        .unwrap();
    assert_eq!(
        yamantaka_storage::identity::full_sha256(
            &target.join("yamantaka.sqlite3"),
            &Cancellation::default()
        )
        .unwrap(),
        before
    );
    let accepted_wal_schema = verify_capture(&target, &Cancellation::default()).is_ok();
    drop(copy);
    let source = rusqlite::Connection::open(&legacy.database).unwrap();
    source.execute_batch("CREATE TRIGGER unsafe_cleanup AFTER UPDATE OF thumbnail_path ON documents BEGIN UPDATE documents SET title='Unexpected cleanup change' WHERE id=NEW.id; END;").unwrap();
    let plan = review(legacy, || Ok(true), &Cancellation::default()).unwrap();
    let unsafe_target = root.join("review-unsafe-cleanup");
    let accepted_cleanup = capture_reviewed(
        &plan,
        &unsafe_target,
        || Ok(true),
        &Cancellation::default(),
        |_| {},
    )
    .is_ok();
    assert_eq!(
        (accepted_wal_schema, accepted_cleanup),
        (false, false),
        "Schema changes and cleanup side effects must not be accepted"
    );
    assert!(!unsafe_target.exists());
}
