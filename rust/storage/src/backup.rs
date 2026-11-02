//! Versioned JSON compatibility; restoration is staged before installation.
use crate::{
    Database,
    database::{storage, validate},
    preferences::Preferences,
};
use rusqlite::{params_from_iter, types::Value as SqlValue};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::Read, path::Path};
use yamantaka_domain::{Cancellation, Failure, Result};

mod extensions;

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

pub fn exported_tables() -> impl Iterator<Item = &'static str> {
    TABLE_ORDER
        .into_iter()
        .chain(std::iter::once("document_covers"))
}
const COVER_COLUMNS: [&str; 11] = [
    "document_id",
    "source_page_index",
    "small_thumbnail_path",
    "large_thumbnail_path",
    "generated_at",
    "source_modified_at",
    "confidence",
    "selection_mode",
    "algorithm_version",
    "crop_mode",
    "page_aspect_ratio",
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
    let (payload, _) = read_payload(path, cancel, None)?;
    preview_payload(payload, cancel)
}
fn read_payload(
    path: &Path,
    cancel: &Cancellation,
    expected: Option<&str>,
) -> Result<(Value, String)> {
    cancel.check()?;
    let mut file = fs::File::open(path).map_err(storage)?;
    let metadata = file.metadata().map_err(storage)?;
    if !metadata.is_file() || metadata.len() > MAX_BACKUP_BYTES {
        return Err(Failure::Invalid(
            "Backup exceeds 128 MiB or is not a file".into(),
        ));
    }
    let mut bytes = Vec::new();
    let mut digest = Sha256::new();
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
        digest.update(&buffer[..count]);
    }
    cancel.check()?;
    let digest = format!("{:x}", digest.finalize());
    if expected.is_some_and(|expected| expected != digest) {
        return Err(Failure::BackupSourceChanged);
    }
    let payload: Value = serde_json::from_slice(&bytes).map_err(storage)?;
    Ok((payload, digest))
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
    if (payload["tables"].get("document_covers").is_some()
        || payload.get("rustPreferences").is_some())
        && payload.get("manifest").is_none()
    {
        return Err(Failure::Invalid("Missing optional backup manifest".into()));
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
        for name in ["document_covers", "rustPreferences"] {
            let present = if name == "document_covers" {
                payload["tables"].get(name).is_some()
            } else {
                payload.get(name).is_some()
            };
            let count = categories.iter().filter(|c| c["name"] == name).count();
            if present && count != 1 || !present && count != 0 {
                return Err(Failure::Invalid(
                    "Missing or duplicate optional backup category".into(),
                ));
            }
        }
        for category in categories {
            let name = category["name"]
                .as_str()
                .ok_or_else(|| Failure::Invalid("Invalid category name".into()))?;
            let rows = if name == "settings" {
                json!([payload["settings"]])
            } else if name == "rustPreferences" {
                json!([payload["rustPreferences"]])
            } else {
                payload["tables"][name].clone()
            };
            if name == "document_covers" && !rows.is_array() {
                return Err(Failure::Invalid("Missing cover backup rows".into()));
            }
            if let Some(count) = rows.as_array().map(Vec::len)
                && (category["rows"].as_u64() != Some(count as u64)
                    || category["sha256"].as_str() != Some(hash_json(&rows)?.as_str()))
            {
                return Err(Failure::Invalid(format!("Backup hash mismatch: {name}")));
            }
        }
    }
    let preferences = extensions::preferences(&payload)?;
    let tables = payload["tables"]
        .as_object_mut()
        .ok_or_else(|| Failure::Invalid("Invalid backup tables".into()))?;
    // Optional Rust category: older JSON backups have no saved cover decisions.
    tables.entry("document_covers").or_insert_with(|| json!([]));
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
    for table in exported_tables() {
        cancel.check()?;
        let rows = tables
            .get(table)
            .and_then(Value::as_array)
            .filter(|rows| rows.len() <= 100000)
            .ok_or_else(|| Failure::Invalid(format!("Invalid backup table: {table}")))?;
        counts.insert(table.into(), rows.len());
        let mut columns = if table == "document_covers" {
            COVER_COLUMNS.iter().map(|s| (*s).to_owned()).collect()
        } else {
            current[table].clone()
        };
        // Flutter's historical JSON contract omits these existing schema-v11 fields.
        if table == "documents" {
            columns.extend(["is_modified".into(), "last_validated_at".into()]);
        }
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
            if required
                .get(table)
                .unwrap_or(&columns)
                .iter()
                .any(|c| !map.contains_key(c))
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
                    let value = if table == "document_covers"
                        && matches!(
                            column.as_str(),
                            "small_thumbnail_path" | "large_thumbnail_path"
                        ) {
                        Value::String(String::new())
                    } else {
                        map.get(column)
                            .filter(|v| !v.is_null())
                            .cloned()
                            .unwrap_or_else(|| default_value(table, column))
                    };
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
    Ok(RestorePreview {
        database,
        preferences,
        counts,
        backup_version: version,
    })
}

fn default_value(table: &str, column: &str) -> Value {
    match (table, column) {
        ("documents", "is_favorite" | "restore_blocked" | "is_modified")
        | ("workspaces", "focus_mode") => json!(0),
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

mod restore;
pub use restore::{
    RestoreProgress, install_preview, install_preview_progress, recover_interrupted_restore,
};
mod review;
pub use review::{RestoreReview, install_reviewed, prepare_reviewed, review};

mod rollback;
pub use rollback::{
    RollbackReview, install_rollback, install_rollback_progress, review_rollback, verify_rollback,
};
