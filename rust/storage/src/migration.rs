//! Independent, WAL-aware capture of a closed Flutter installation.
use crate::{
    database::{migrate, readonly, snapshot, storage, validate, version},
    identity::full_sha256,
    preferences::{Preferences, atomic_json},
};
use rusqlite::Connection;
use rusqlite::types::ValueRef;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use yamantaka_domain::{Cancellation, Failure, Result, SCHEMA_VERSION};

#[derive(Clone)]
pub struct LegacySource {
    pub database: PathBuf,
    pub preferences: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReceipt {
    pub version: u32,
    pub source_schema: u32,
    pub target_schema: u32,
    pub source_preferences_sha256: String,
    pub captured_database_sha256: String,
    pub imported_rows: BTreeMap<String, u64>,
    #[serde(default)]
    pub preserved_data_sha256: BTreeMap<String, String>,
    pub completed_at: String,
}

/// Caller supplies a fresh native process check, repeated around the capture.
/// A directory is installed only after both database and preferences validate.
/// No existing destination is replaced. An unchanged repeated import returns its
/// verified receipt; different source data or an edited destination is refused.
pub fn capture(
    source: &LegacySource,
    destination: &Path,
    flutter_closed: impl Fn() -> Result<bool>,
    cancel: &Cancellation,
) -> Result<MigrationReceipt> {
    cancel.check()?;
    if !flutter_closed()? {
        return Err(Failure::LegacyRunning);
    }
    let existing = if destination.exists() {
        let bytes = fs::read(destination.join("migration-receipt.json"))
            .map_err(|_| Failure::DestinationOccupied)?;
        let receipt: MigrationReceipt =
            serde_json::from_slice(&bytes).map_err(|_| Failure::DestinationOccupied)?;
        if receipt.version != 2 {
            return Err(Failure::DestinationOccupied);
        }
        Some(receipt)
    } else {
        None
    };
    let parent = destination
        .parent()
        .ok_or_else(|| Failure::Invalid("Missing import parent".into()))?;
    fs::create_dir_all(parent).map_err(storage)?;
    let staging = tempfile::Builder::new()
        .prefix(".yamantaka-import-")
        .tempdir_in(parent)
        .map_err(storage)?;
    let preferences_before = fs::read(&source.preferences).map_err(storage)?;
    let preferences = Preferences::from_flutter(&source.preferences)?;
    let original = readonly(&source.database)?;
    let schema = version(&original)?;
    if schema == 0 || schema > SCHEMA_VERSION {
        return Err(Failure::Invalid("Unsupported Flutter database".into()));
    }
    original.execute_batch("BEGIN").map_err(storage)?;
    validate(&original)?;
    let rows = table_counts(&original)?;
    let originals = table_identities(&original, cancel)?;
    let preferences_hash = format!("{:x}", Sha256::digest(&preferences_before));
    if let Some(receipt) = existing {
        if receipt.source_schema != schema
            || receipt.source_preferences_sha256 != preferences_hash
            || receipt.preserved_data_sha256
                != originals
                    .iter()
                    .map(|(name, identity)| (name.clone(), identity.sha256.clone()))
                    .collect()
        {
            return Err(Failure::DestinationOccupied);
        }
        original.execute_batch("COMMIT").map_err(storage)?;
        if !flutter_closed()?
            || preferences_before != fs::read(&source.preferences).map_err(storage)?
        {
            return Err(Failure::LegacyRunning);
        }
        return verify_capture(destination, cancel);
    }
    let target_path = staging.path().join("yamantaka.sqlite3");
    snapshot(&original, &target_path)?;
    original.execute_batch("COMMIT").map_err(storage)?;
    cancel.check()?;
    if preferences_before != fs::read(&source.preferences).map_err(storage)? || !flutter_closed()? {
        return Err(Failure::LegacyRunning);
    }
    let mut target = Connection::open(&target_path).map_err(storage)?;
    migrate(&mut target, SCHEMA_VERSION)?;
    let target_rows = table_counts(&target)?;
    for (table, count) in &rows {
        if target_rows.get(table) != Some(count) {
            return Err(Failure::Invalid(format!(
                "Row preservation failed for {table}"
            )));
        }
        let identity = &originals[table];
        if table_hash(&target, table, &identity.columns, cancel)? != identity.sha256 {
            return Err(Failure::Invalid(format!(
                "Original column preservation failed for {table}"
            )));
        }
    }
    // Rebuildable files stay independent. Preserve manual page selections in document_covers.
    // Never point Rust thumbnails at the Flutter installation.
    target.execute_batch("UPDATE documents SET thumbnail_path=NULL; UPDATE document_covers SET small_thumbnail_path='',large_thumbnail_path=''; DELETE FROM library_search; DELETE FROM search_index_state;").map_err(storage)?;
    validate(&target)?;
    target.close().map_err(|(_, e)| storage(e))?;
    preferences.save(&staging.path().join("preferences.json"))?;
    let receipt = MigrationReceipt {
        version: 2,
        source_schema: schema,
        target_schema: SCHEMA_VERSION,
        source_preferences_sha256: preferences_hash,
        captured_database_sha256: full_sha256(&target_path, cancel)?,
        imported_rows: rows,
        preserved_data_sha256: originals
            .into_iter()
            .map(|(table, identity)| (table, identity.sha256))
            .collect(),
        completed_at: chrono::Utc::now().to_rfc3339(),
    };
    atomic_json(&staging.path().join("migration-receipt.json"), &receipt)?;
    cancel.check()?;
    if !flutter_closed()? {
        return Err(Failure::LegacyRunning);
    }
    if destination.exists() {
        return Err(Failure::DestinationOccupied);
    }
    // Rename an entire staged store. A crash cannot expose a half-imported pair.
    fs::rename(staging.path(), destination).map_err(storage)?;
    Ok(receipt)
}

pub fn verify_capture(destination: &Path, cancel: &Cancellation) -> Result<MigrationReceipt> {
    let receipt: MigrationReceipt = serde_json::from_slice(
        &fs::read(destination.join("migration-receipt.json")).map_err(storage)?,
    )
    .map_err(storage)?;
    if !(1..=2).contains(&receipt.version) || receipt.target_schema != SCHEMA_VERSION {
        return Err(Failure::Invalid("Unsupported migration receipt".into()));
    }
    if full_sha256(&destination.join("yamantaka.sqlite3"), cancel)?
        != receipt.captured_database_sha256
    {
        return Err(Failure::Invalid(
            "Captured database hash differs (the library may have been edited)".into(),
        ));
    }
    Preferences::load(&destination.join("preferences.json"))?;
    validate(&readonly(&destination.join("yamantaka.sqlite3"))?)?;
    Ok(receipt)
}

struct TableIdentity {
    columns: Vec<String>,
    sha256: String,
}
fn quoted(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
fn table_identities(
    connection: &Connection,
    cancel: &Cancellation,
) -> Result<BTreeMap<String, TableIdentity>> {
    let mut identities = BTreeMap::new();
    for name in table_counts(connection)?.into_keys() {
        cancel.check()?;
        let columns = connection
            .prepare(&format!("PRAGMA table_info({})", quoted(&name)))
            .map_err(storage)?
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(storage)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage)?;
        let sha256 = table_hash(connection, &name, &columns, cancel)?;
        identities.insert(name, TableIdentity { columns, sha256 });
    }
    Ok(identities)
}
fn table_hash(
    connection: &Connection,
    name: &str,
    columns: &[String],
    cancel: &Cancellation,
) -> Result<String> {
    let column_sql = columns
        .iter()
        .map(|column| quoted(column))
        .collect::<Vec<_>>()
        .join(",");
    let mut statement = connection
        .prepare(&format!(
            "SELECT {column_sql} FROM {} ORDER BY {column_sql}",
            quoted(name)
        ))
        .map_err(storage)?;
    let mut rows = statement.query([]).map_err(storage)?;
    let mut hash = Sha256::new();
    hash.update((columns.len() as u64).to_le_bytes());
    while let Some(row) = rows.next().map_err(storage)? {
        cancel.check()?;
        hash.update([0xff]);
        for index in 0..columns.len() {
            match row.get_ref(index).map_err(storage)? {
                ValueRef::Null => hash.update([0]),
                ValueRef::Integer(value) => {
                    hash.update([1]);
                    hash.update(value.to_le_bytes());
                }
                ValueRef::Real(value) => {
                    hash.update([2]);
                    hash.update(value.to_bits().to_le_bytes());
                }
                ValueRef::Text(value) | ValueRef::Blob(value) => {
                    hash.update([
                        if matches!(row.get_ref(index).map_err(storage)?, ValueRef::Text(_)) {
                            3
                        } else {
                            4
                        },
                    ]);
                    hash.update((value.len() as u64).to_le_bytes());
                    hash.update(value);
                }
            }
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub(crate) fn table_counts(connection: &Connection) -> Result<BTreeMap<String, u64>> {
    let names = connection.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'library_search_%' ORDER BY name").map_err(storage)?.query_map([], |r| r.get::<_, String>(0)).map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;
    names
        .into_iter()
        .map(|name| {
            let count = connection
                .query_row(
                    &format!("SELECT COUNT(*) FROM \"{}\"", name.replace('"', "\"\"")),
                    [],
                    |r| r.get::<_, i64>(0),
                )
                .map_err(storage)? as u64;
            Ok((name, count))
        })
        .collect()
}
