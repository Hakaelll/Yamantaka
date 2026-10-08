use serde_json::{Value, json};
use std::fs;
use yamantaka_domain::{Cancellation, LibraryQuery, LibraryRepository};
use yamantaka_storage::{
    Database,
    backup::{TABLE_ORDER, install_preview, preview, preview_payload},
    preferences::Preferences,
};

fn payload(version: u64) -> Value {
    let tables = TABLE_ORDER
        .into_iter()
        .map(|name| (name.to_string(), json!([])))
        .collect::<serde_json::Map<_, _>>();
    json!({"format":"yamantaka-local-backup","version":version,"containsPdfFiles":false,"exportedAt":"2026-10-08T10:00:00Z","settings":{"darkMode":true},"tables":tables})
}
#[test]
fn all_nine_formats_restore_and_v9_exports_with_hashes() {
    let root = tempfile::tempdir().unwrap();
    for version in 1..=9 {
        let restored = preview_payload(payload(version), &Cancellation::default()).unwrap();
        let export = root.path().join(format!("{version}.json"));
        restored
            .database
            .export_backup(&export, &restored.preferences, &Cancellation::default())
            .unwrap();
        let exported = preview(&export, &Cancellation::default()).unwrap();
        assert_eq!(exported.backup_version, 9);
        assert_eq!(exported.preferences.get("darkMode").unwrap(), true);
    }
}

#[test]
fn every_backup_contract_restores_populated_relations_and_round_trips_all_columns() {
    let root = tempfile::tempdir().unwrap();
    let current: Value =
        serde_json::from_str(include_str!("fixtures/populated-backup-v9.json")).unwrap();
    let contracts: Value =
        serde_json::from_str(include_str!("../migrations/backup-contracts.json")).unwrap();
    for version in 1..=9 {
        let mut historic = current.clone();
        historic["version"] = json!(version);
        let version_key = version.to_string();
        let contract = &contracts[version_key.as_str()];
        for table in TABLE_ORDER {
            let columns = contract[table].as_array().unwrap();
            for row in historic["tables"][table].as_array_mut().unwrap() {
                row.as_object_mut().unwrap().retain(|name, _| {
                    columns
                        .iter()
                        .any(|column| column.as_str() == Some(name.as_str()))
                });
            }
        }
        let restored = preview_payload(historic.clone(), &Cancellation::default()).unwrap();
        restored.database.integrity().unwrap();
        assert_eq!(restored.counts["documents"], 2);
        for table in TABLE_ORDER {
            assert!(restored.counts[table] > 0, "unpopulated fixture: {table}");
        }
        let export = root.path().join(format!("populated-{version}.json"));
        restored
            .database
            .export_backup(&export, &restored.preferences, &Cancellation::default())
            .unwrap();
        // Revalidate the exported hashes and constraints, then compare the full
        // historical projection, not just counts or a single document ID.
        preview(&export, &Cancellation::default()).unwrap();
        let exported: Value = serde_json::from_slice(&fs::read(export).unwrap()).unwrap();
        assert_eq!(exported["settings"], historic["settings"]);
        for table in TABLE_ORDER {
            let mut actual = exported["tables"][table].clone();
            for row in actual.as_array_mut().unwrap() {
                row.as_object_mut().unwrap().retain(|name, _| {
                    contract[table]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|column| column.as_str() == Some(name.as_str()))
                });
            }
            assert_eq!(
                actual, historic["tables"][table],
                "backup version {version}, table {table}"
            );
        }
    }
}
#[test]
fn tampering_and_invalid_foreign_keys_do_not_touch_library() {
    let root = tempfile::tempdir().unwrap();
    let backup = root.path().join("backup.json");
    let db = Database::in_memory_at(11).unwrap();
    db.export_backup(&backup, &Preferences::default(), &Cancellation::default())
        .unwrap();
    let mut exported: Value = serde_json::from_slice(&fs::read(&backup).unwrap()).unwrap();
    exported["settings"]["darkMode"] = json!(false);
    assert!(preview_payload(exported, &Cancellation::default()).is_err());
    let mut invalid = payload(9);
    invalid["tables"]["bookmarks"] = json!([{"id":"b","document_id":"missing","page_index":0,"label":null,"created_at":"2026-10-08","updated_at":"2026-10-08"}]);
    assert!(preview_payload(invalid, &Cancellation::default()).is_err());
    assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 0);
}
#[test]
fn replacement_keeps_complete_safety_store_and_cancel_does_not_write() {
    let root = tempfile::tempdir().unwrap();
    let active = root.path().join("active");
    fs::create_dir(&active).unwrap();
    fs::write(active.join("sentinel"), "old state").unwrap();
    let restored = preview_payload(payload(1), &Cancellation::default()).unwrap();
    let safety = install_preview(restored, &active, &Cancellation::default()).unwrap();
    assert_eq!(
        fs::read_to_string(safety.join("sentinel")).unwrap(),
        "old state"
    );
    assert!(active.join("yamantaka.sqlite3").exists());
    let cancel = Cancellation::default();
    let restored = preview_payload(payload(9), &Cancellation::default()).unwrap();
    cancel.cancel();
    assert!(install_preview(restored, &active, &cancel).is_err());
    assert!(active.join("yamantaka.sqlite3").exists());
}

#[test]
fn interrupted_rename_recovers_complete_original_store() {
    let root = tempfile::tempdir().unwrap();
    let active = root.path().join("active");
    let safety = root.path().join("pre-restore-fixture");
    fs::create_dir(&safety).unwrap();
    fs::write(safety.join("sentinel"), "coherent old store").unwrap();
    let journal = root.path().join(".active.restore-journal.json");
    fs::write(
        &journal,
        serde_json::to_vec(
            &json!({"version":1,"active":active,"safety":safety,"replacement_sha256":"unused"}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(yamantaka_storage::backup::recover_interrupted_restore(&active).unwrap());
    assert_eq!(
        fs::read_to_string(active.join("sentinel")).unwrap(),
        "coherent old store"
    );
    assert!(!journal.exists());
    assert!(!yamantaka_storage::backup::recover_interrupted_restore(&active).unwrap());
}
