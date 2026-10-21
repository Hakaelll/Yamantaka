//! Versioned JSON compatibility; restoration is staged before installation.
use crate::{
    Database,
    database::{storage, validate},
    preferences::{Preferences, atomic_json},
};
use rusqlite::{params_from_iter, types::Value as SqlValue};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::Read, path::Path};
use yamantaka_domain::{Cancellation, Failure, Result};

type Contracts = BTreeMap<String, BTreeMap<String, Vec<String>>>;
pub const MAX_BACKUP_BYTES: u64 = 128 * 1024 * 1024;
pub const TABLE_ORDER: [&str; 23] = [
    "documents",
    "document_enrichment",
    "document_topics",
    "document_preferences",
    "document_notes",
    "bookmarks",
    "reading_sessions",
    "annotations",
    "tags",
    "annotation_tags",
    "document_tags",
    "collections",
    "collection_documents",
    "reading_profiles",
    "workspaces",
    "workspace_tabs",
    "document_relations",
    "document_revision_groups",
    "document_revision_members",
    "automation_rules",
    "operation_journal",
    "duplicate_merge_history",
    "app_metadata",
];

fn contracts() -> Contracts {
    serde_json::from_str(include_str!("../migrations/backup-contracts.json"))
        .expect("Committed backup contract must be valid")
}
pub fn hash_json(value: &Value) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).map_err(storage)?)
    ))
}
pub struct RestorePreview {
    pub database: Database,
    pub preferences: Preferences,
    pub counts: BTreeMap<String, usize>,
    pub backup_version: u64,
}

/// Parse, validate category hashes, and restore constraints into an isolated DB.
/// Neither preview nor failure touches the active library or any original PDF.
pub fn preview(path: &Path, cancel: &Cancellation) -> Result<RestorePreview> {
    cancel.check()?;
    let mut file = fs::File::open(path).map_err(storage)?;
    let metadata = file.metadata().map_err(storage)?;
    if !metadata.is_file() || metadata.len() > MAX_BACKUP_BYTES {
        return Err(Failure::Invalid(
            "Backup exceeds 128 MiB or is not a file".into(),
        ));
    }
    let mut bytes = Vec::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        cancel.check()?;
        let count = file.read(&mut buffer).map_err(storage)?;
        if count == 0 {
            break;
        }
        if bytes.len() as u64 + count as u64 > MAX_BACKUP_BYTES {
            return Err(Failure::Invalid("Backup exceeds 128 MiB".into()));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    cancel.check()?;
    let payload: Value = serde_json::from_slice(&bytes).map_err(storage)?;
    preview_payload(payload, cancel)
}
pub fn preview_payload(mut payload: Value, cancel: &Cancellation) -> Result<RestorePreview> {
    cancel.check()?;
    let version = payload["version"]
        .as_u64()
        .filter(|v| (1..=9).contains(v))
        .ok_or_else(|| Failure::Invalid("Unsupported backup version".into()))?;
    if payload["format"] != "yamantaka-local-backup"
        || payload["containsPdfFiles"] != false
        || payload["exportedAt"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .is_none()
        || !payload["settings"].is_object()
    {
        return Err(Failure::Invalid("Invalid backup envelope".into()));
    }
    if let Some(manifest) = payload.get("manifest") {
        if manifest["format"] != payload["format"]
            || manifest["version"] != payload["version"]
            || manifest["containsPdfFiles"] != false
            || manifest["schemaVersion"].as_u64().is_none_or(|v| v > 11)
        {
            return Err(Failure::Invalid("Incompatible backup manifest".into()));
        }
        let categories = manifest["categories"]
            .as_array()
            .ok_or_else(|| Failure::Invalid("Invalid manifest categories".into()))?;
        for category in categories {
            let name = category["name"]
                .as_str()
                .ok_or_else(|| Failure::Invalid("Invalid category name".into()))?;
            let rows = if name == "settings" {
                json!([payload["settings"]])
            } else {
                payload["tables"][name].clone()
            };
            if let Some(count) = rows.as_array().map(Vec::len)
                && (category["rows"].as_u64() != Some(count as u64)
                    || category["sha256"].as_str() != Some(hash_json(&rows)?.as_str()))
            {
                return Err(Failure::Invalid(format!("Backup hash mismatch: {name}")));
            }
        }
    }
    let tables = payload["tables"]
        .as_object_mut()
        .ok_or_else(|| Failure::Invalid("Invalid backup tables".into()))?;
    // These are exactly the absent-table adapters in Flutter v9.
    for table in [
        "document_notes",
        "document_enrichment",
        "document_topics",
        "document_relations",
        "document_revision_groups",
        "document_revision_members",
        "automation_rules",
        "operation_journal",
        "duplicate_merge_history",
    ] {
        tables.entry(table).or_insert_with(|| json!([]));
    }
    if version == 1 {
        for table in [
            "annotations",
            "tags",
            "annotation_tags",
            "document_tags",
            "collections",
            "collection_documents",
            "reading_profiles",
            "workspaces",
            "workspace_tabs",
        ] {
            tables.entry(table).or_insert_with(|| json!([]));
        }
    }
    let contracts = contracts();
    let required = &contracts[&version.to_string()];
    let current = &contracts["9"];
    let mut counts = BTreeMap::new();
    let mut database = Database::in_memory_at(11)?;
    let tx = database.connection.transaction().map_err(storage)?;
    for table in TABLE_ORDER {
        cancel.check()?;
        let rows = tables
            .get(table)
            .and_then(Value::as_array)
            .filter(|rows| rows.len() <= 100000)
            .ok_or_else(|| Failure::Invalid(format!("Invalid backup table: {table}")))?;
        counts.insert(table.into(), rows.len());
        let columns = &current[table];
        let sql = format!(
            "INSERT INTO {table} ({}) VALUES ({})",
            columns.join(","),
            vec!["?"; columns.len()].join(",")
        );
        let mut statement = tx.prepare(&sql).map_err(storage)?;
        for row in rows {
            cancel.check()?;
            let map = row
                .as_object()
                .ok_or_else(|| Failure::Invalid(format!("Invalid row: {table}")))?;
            if required[table].iter().any(|c| !map.contains_key(c))
                || map
                    .values()
                    .any(|v| !(v.is_null() || v.is_number() || v.is_string()))
            {
                return Err(Failure::Invalid(format!(
                    "Incomplete or invalid row: {table}"
                )));
            }
            let values: Vec<_> = columns
                .iter()
                .map(|column| {
                    let value = map
                        .get(column)
                        .filter(|v| !v.is_null())
                        .cloned()
                        .unwrap_or_else(|| default_value(table, column));
                    match value {
                        Value::String(s) => SqlValue::Text(s),
                        Value::Number(n) if n.is_i64() => SqlValue::Integer(n.as_i64().unwrap()),
                        Value::Number(n) => SqlValue::Real(n.as_f64().unwrap()),
                        _ => SqlValue::Null,
                    }
                })
                .collect();
            statement
                .execute(params_from_iter(values))
                .map_err(storage)?;
        }
    }
    validate(&tx)?;
    tx.commit().map_err(storage)?;
    let preferences = Preferences {
        legacy: serde_json::from_value(payload["settings"].clone()).map_err(storage)?,
        ..Default::default()
    };
    Ok(RestorePreview {
        database,
        preferences,
        counts,
        backup_version: version,
    })
}

fn default_value(table: &str, column: &str) -> Value {
    match (table, column) {
        ("documents", "is_favorite" | "restore_blocked") | ("workspaces", "focus_mode") => json!(0),
        ("annotations", "review_state") | ("workspace_tabs", "queue_status") => json!("pending"),
        ("workspaces", "query_json") => json!("{}"),
        ("workspaces", "grouping") | ("operation_journal", "resume_capability") => json!("none"),
        ("workspaces", "sort_mode") => json!("lastOpened"),
        ("workspaces", "density") => json!("comfortable"),
        ("workspaces", "visible_columns_json") => json!("[\"document\",\"status\",\"progress\"]"),
        ("operation_journal", "phase") => json!("planned"),
        ("document_preferences", "cover_policy") => json!("noCover"),
        ("document_preferences", "initial_side") => json!("left"),
        _ => Value::Null,
    }
}

mod export;
pub use export::BackupProgress;

/// Installation requires closing the active database first. The pre-restore copy
/// remains in its own directory and can be selected for rollback on next launch.
pub fn install_preview(
    preview: RestorePreview,
    active: &Path,
    cancel: &Cancellation,
) -> Result<std::path::PathBuf> {
    cancel.check()?;
    let parent = active
        .parent()
        .ok_or_else(|| Failure::Invalid("Missing restore parent".into()))?;
    let stage = tempfile::Builder::new()
        .prefix(".restore-")
        .tempdir_in(parent)
        .map_err(storage)?;
    preview
        .database
        .snapshot(&stage.path().join("yamantaka.sqlite3"))?;
    preview
        .preferences
        .save(&stage.path().join("preferences.json"))?;
    cancel.check()?;
    let safety = parent.join(format!("pre-restore-{}", uuid::Uuid::new_v4()));
    let journal = restore_journal_path(active)?;
    if journal.exists() {
        return Err(Failure::Invalid(
            "Recover the interrupted restore before starting another".into(),
        ));
    }
    atomic_json(
        &journal,
        &RestoreJournal {
            version: 1,
            active: active.to_owned(),
            safety: safety.clone(),
            replacement_sha256: crate::identity::full_sha256(
                &stage.path().join("yamantaka.sqlite3"),
                cancel,
            )?,
        },
    )?;
    if active.exists() {
        fs::rename(active, &safety).map_err(storage)?;
    }
    if let Err(error) = fs::rename(stage.path(), active) {
        if safety.exists() {
            fs::rename(&safety, active).map_err(storage)?;
        }
        fs::remove_file(&journal).map_err(storage)?;
        return Err(storage(error));
    }
    fs::remove_file(journal).map_err(storage)?;
    Ok(safety)
}

#[derive(Serialize, Deserialize)]
struct RestoreJournal {
    version: u32,
    active: std::path::PathBuf,
    safety: std::path::PathBuf,
    replacement_sha256: String,
}
fn restore_journal_path(active: &Path) -> Result<std::path::PathBuf> {
    let name = active
        .file_name()
        .ok_or_else(|| Failure::Invalid("Invalid active store".into()))?
        .to_string_lossy();
    Ok(active.with_file_name(format!(".{name}.restore-journal.json")))
}
/// Called before opening any active DB. A crash between the two renames rolls
/// back to the complete safety store. A committed new store keeps its safety copy.
pub fn recover_interrupted_restore(active: &Path) -> Result<bool> {
    let journal_path = restore_journal_path(active)?;
    if !journal_path.exists() {
        return Ok(false);
    }
    let journal: RestoreJournal =
        serde_json::from_slice(&fs::read(&journal_path).map_err(storage)?).map_err(storage)?;
    let parent = active
        .parent()
        .ok_or_else(|| Failure::Invalid("Missing recovery parent".into()))?;
    if journal.version != 1
        || journal.active != active
        || journal.safety.parent() != Some(parent)
        || journal
            .safety
            .file_name()
            .is_none_or(|name| !name.to_string_lossy().starts_with("pre-restore-"))
    {
        return Err(Failure::Invalid("Restore journal has invalid paths".into()));
    }
    if !active.exists() && journal.safety.exists() {
        fs::rename(&journal.safety, active).map_err(storage)?;
    } else if active.exists() && journal.safety.exists() {
        if crate::identity::full_sha256(
            &active.join("yamantaka.sqlite3"),
            &Cancellation::default(),
        )? != journal.replacement_sha256
        {
            return Err(Failure::Invalid(
                "Interrupted restore requires explicit recovery; both copies were retained".into(),
            ));
        }
    } else if !active.exists() {
        return Err(Failure::Invalid("Both recovery stores are missing".into()));
    }
    fs::remove_file(journal_path).map_err(storage)?;
    Ok(true)
}
