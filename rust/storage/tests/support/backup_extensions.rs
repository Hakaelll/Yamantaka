use serde_json::{Value, json};
use std::{fs, path::Path};
use yamantaka_domain::{Cancellation, LibraryRepository, cover_snapshot::CoverPlan};
use yamantaka_storage::backup::{self, hash_json, preview_payload};

fn rehash(payload: &mut Value, name: &str) {
    let rows = if name == "rustPreferences" {
        json!([payload[name]])
    } else if name == "settings" {
        json!([payload["settings"]])
    } else {
        payload["tables"][name].clone()
    };
    let category = payload["manifest"]["categories"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == name)
        .unwrap();
    category["rows"] = json!(rows.as_array().unwrap().len());
    category["sha256"] = hash_json(&rows).unwrap().into();
}
pub fn verify(root: &Path) {
    let input: Value =
        serde_json::from_str(include_str!("../fixtures/populated-backup-v9.json")).unwrap();
    let mut source = preview_payload(input, &Cancellation::default()).unwrap();
    source.database.execute_batch("INSERT INTO document_covers SELECT id,1,'discarded-small-cache','discarded-large-cache','2026-11-02','2026-11-01',1,CASE WHEN id=(SELECT min(id) FROM documents) THEN 'manual' ELSE 'automatic' END,4,'contain',0.75 FROM documents;").unwrap();
    source
        .preferences
        .legacy
        .insert("sharedSetting".into(), json!("legacy value"));
    source
        .preferences
        .rust
        .insert("sharedSetting".into(), json!("rust value"));
    source.preferences.rust.insert(
        "futureSetting".into(),
        json!({"items":[1,"kept"],"enabled":true}),
    );
    let path = root.join("optional-covers-and-preferences.json");
    source
        .database
        .export_backup(&path, &source.preferences, &Cancellation::default())
        .unwrap();
    let payload: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(payload["version"], 9);
    assert_eq!(payload["settings"]["sharedSetting"], "rust value");
    let rows = payload["tables"]["document_covers"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        1,
        "Automatic detection results must remain recreatable"
    );
    let id = rows[0]["document_id"].as_str().unwrap();
    assert_eq!(
        rows[0],
        json!({"document_id":id,"source_page_index":1,"small_thumbnail_path":"","large_thumbnail_path":"","generated_at":"2026-11-02","source_modified_at":"2026-11-01","confidence":1.0,"selection_mode":"manual","algorithm_version":4,"crop_mode":"contain","page_aspect_ratio":0.75})
    );
    assert!(!fs::read_to_string(&path).unwrap().contains("discarded-"));
    let plan = backup::review(&path, &Cancellation::default()).unwrap();
    assert_eq!(plan.counts()["document_covers"], 1);
    let restored = backup::prepare_reviewed(&plan, &Cancellation::default()).unwrap();
    assert_eq!(
        restored.database.cover_snapshot(id).unwrap().plan,
        CoverPlan::Render(1)
    );
    assert_eq!(
        serde_json::to_value(&restored.preferences).unwrap(),
        serde_json::to_value(&source.preferences).unwrap()
    );
    assert_eq!(
        restored.preferences.get("sharedSetting"),
        Some(&json!("rust value"))
    );
    let roundtrip = root.join("optional-roundtrip.json");
    restored
        .database
        .export_backup(&roundtrip, &restored.preferences, &Cancellation::default())
        .unwrap();
    let actual: Value = serde_json::from_slice(&fs::read(roundtrip).unwrap()).unwrap();
    assert_eq!(
        actual["tables"]["document_covers"],
        payload["tables"]["document_covers"]
    );
    assert_eq!(actual["rustPreferences"], payload["rustPreferences"]);
    for mode in [
        "missing-manifest",
        "tamper-page",
        "missing-cover-rows",
        "missing-cover-hash",
        "duplicate-cover-hash",
        "dangling-cover",
        "invalid-confidence",
        "tamper-preferences",
        "future-preferences",
        "missing-preference-hash",
        "mismatched-settings",
    ] {
        let mut changed = payload.clone();
        match mode {
            "missing-manifest" => {
                changed.as_object_mut().unwrap().remove("manifest");
            }
            "tamper-page" => {
                changed["tables"]["document_covers"][0]["source_page_index"] = json!(0)
            }
            "missing-cover-rows" => {
                changed["tables"]
                    .as_object_mut()
                    .unwrap()
                    .remove("document_covers");
            }
            "missing-cover-hash" => changed["manifest"]["categories"]
                .as_array_mut()
                .unwrap()
                .retain(|c| c["name"] != "document_covers"),
            "duplicate-cover-hash" => {
                let cover = changed["manifest"]["categories"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|c| c["name"] == "document_covers")
                    .unwrap()
                    .clone();
                changed["manifest"]["categories"]
                    .as_array_mut()
                    .unwrap()
                    .push(cover);
            }
            "dangling-cover" => {
                changed["tables"]["document_covers"][0]["document_id"] = json!("missing document");
                rehash(&mut changed, "document_covers");
            }
            "invalid-confidence" => {
                changed["tables"]["document_covers"][0]["confidence"] = json!(2);
                rehash(&mut changed, "document_covers");
            }
            "tamper-preferences" => {
                changed["rustPreferences"]["rust"]["sharedSetting"] = json!("tampered")
            }
            "future-preferences" => {
                changed["rustPreferences"]["version"] = json!(2);
                rehash(&mut changed, "rustPreferences");
            }
            "missing-preference-hash" => changed["manifest"]["categories"]
                .as_array_mut()
                .unwrap()
                .retain(|c| c["name"] != "rustPreferences"),
            "mismatched-settings" => {
                changed["settings"]["sharedSetting"] = json!("disagrees");
                rehash(&mut changed, "settings");
            }
            _ => unreachable!(),
        }
        assert!(
            preview_payload(changed, &Cancellation::default()).is_err(),
            "Unsafe optional backup accepted: {mode}"
        );
    }
    let mut with_cache = payload.clone();
    with_cache["tables"]["document_covers"][0]["small_thumbnail_path"] =
        json!("untrusted-cache.png");
    rehash(&mut with_cache, "document_covers");
    let clean = preview_payload(with_cache, &Cancellation::default()).unwrap();
    assert_eq!(
        clean.database.cover_snapshot(id).unwrap().plan,
        CoverPlan::Render(1)
    );
    let clean_file = root.join("clean-cache-paths.json");
    clean
        .database
        .export_backup(&clean_file, &clean.preferences, &Cancellation::default())
        .unwrap();
    assert!(
        !fs::read_to_string(clean_file)
            .unwrap()
            .contains("untrusted-cache.png")
    );
    let mut legacy = payload.clone();
    legacy.as_object_mut().unwrap().remove("rustPreferences");
    legacy["tables"]
        .as_object_mut()
        .unwrap()
        .remove("document_covers");
    legacy["manifest"]["categories"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["name"] != "document_covers" && c["name"] != "rustPreferences");
    let legacy = preview_payload(legacy, &Cancellation::default()).unwrap();
    assert_eq!(legacy.counts["document_covers"], 0);
    assert!(legacy.preferences.rust.is_empty());
    assert_eq!(
        legacy.preferences.get("sharedSetting"),
        Some(&json!("rust value"))
    );
}
