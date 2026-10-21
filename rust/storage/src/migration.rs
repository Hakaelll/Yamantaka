//! Independent, WAL-aware capture of a closed Flutter installation.
use crate::{
    database::{migrate, readonly, snapshot_cancellable, storage, validate, version},
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

mod review;
pub use review::{MigrationReview, review};

#[derive(Debug, Clone)]
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
    #[serde(default)]
    pub source_schema_sha256: String,
    #[serde(default)]
    pub captured_schema_sha256: String,
    pub captured_database_sha256: String,
    #[serde(default)]
    pub captured_preferences_sha256: String,
    #[serde(default)]
    pub captured_data_sha256: BTreeMap<String, String>,
    pub imported_rows: BTreeMap<String, u64>,
    #[serde(default)]
    pub preserved_data_sha256: BTreeMap<String, String>,
    pub completed_at: String,
}

#[derive(Debug, Clone, Copy)]
pub enum MigrationProgress {
    Copying { copied_pages: u32, total_pages: u32 },
    Validating,
    Installing,
}

pub fn capture_reviewed(
    review: &MigrationReview,
    destination: &Path,
    flutter_closed: impl Fn() -> Result<bool>,
    cancel: &Cancellation,
    mut progress: impl FnMut(MigrationProgress),
) -> Result<MigrationReceipt> {
    capture_inner(
        review.source(),
        destination,
        flutter_closed,
        cancel,
        Some(review),
        &mut progress,
    )
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
    capture_inner(
        source,
        destination,
        flutter_closed,
        cancel,
        None,
        &mut |_| {},
    )
}
fn capture_inner(
    source: &LegacySource,
    destination: &Path,
    flutter_closed: impl Fn() -> Result<bool>,
    cancel: &Cancellation,
    reviewed: Option<&MigrationReview>,
    progress: &mut dyn FnMut(MigrationProgress),
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
        if receipt.version != 3 {
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
    let preferences = Preferences::from_flutter_bytes(&preferences_before)?;
    let original = readonly(&source.database)?;
    original.execute_batch("BEGIN").map_err(storage)?;
    let schema = version(&original)?;
    if schema == 0 || schema > SCHEMA_VERSION {
        return Err(Failure::Invalid("Unsupported Flutter database".into()));
    }
    validate(&original)?;
    let rows = table_counts(&original)?;
    let originals = table_identities(&original, cancel)?;
    let preferences_hash = format!("{:x}", Sha256::digest(&preferences_before));
    let source_hashes = fingerprints(&originals);
    let source_schema_sha256 = schema_hash(&original, cancel)?;
    if let Some(reviewed) = reviewed {
        reviewed.check(
            schema,
            &rows,
            &source_hashes,
            &preferences_hash,
            &source_schema_sha256,
        )?;
    }
    if let Some(receipt) = existing {
        if receipt.source_schema != schema
            || receipt.source_schema_sha256 != source_schema_sha256
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
        confirm_source_preferences(source, &preferences_before, &flutter_closed)?;
        return verify_capture(destination, cancel);
    }
    let mut expected_after_cleanup = source_hashes;
    expected_after_cleanup.remove("library_search");
    expected_after_cleanup.remove("search_index_state");
    for table in ["documents", "document_covers"] {
        if let Some(identity) = originals.get(table) {
            expected_after_cleanup.insert(
                table.into(),
                table_hash_projected(&original, table, &identity.columns, true, cancel)?,
            );
        }
    }
    let target_path = staging.path().join("yamantaka.sqlite3");
    snapshot_cancellable(
        &original,
        &target_path,
        cancel,
        |copied_pages, total_pages| {
            progress(MigrationProgress::Copying {
                copied_pages,
                total_pages,
            })
        },
    )?;
    original.execute_batch("COMMIT").map_err(storage)?;
    cancel.check()?;
    confirm_source_preferences(source, &preferences_before, &flutter_closed)?;
    progress(MigrationProgress::Validating);
    cancel.check()?;
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
    for (table, expected) in &expected_after_cleanup {
        if table_hash(&target, table, &originals[table].columns, cancel)? != *expected {
            return Err(Failure::Invalid(format!(
                "Cache cleanup changed preserved records in {table}"
            )));
        }
    }
    let captured_schema_sha256 = schema_hash(&target, cancel)?;
    let captured_data_sha256 = fingerprints(&table_identities(&target, cancel)?);
    target.close().map_err(|(_, e)| storage(e))?;
    preferences.save(&staging.path().join("preferences.json"))?;
    let receipt = MigrationReceipt {
        version: 3,
        source_schema: schema,
        target_schema: SCHEMA_VERSION,
        source_preferences_sha256: preferences_hash,
        source_schema_sha256,
        captured_schema_sha256,
        captured_database_sha256: full_sha256(&target_path, cancel)?,
        captured_preferences_sha256: full_sha256(&staging.path().join("preferences.json"), cancel)?,
        captured_data_sha256,
        imported_rows: rows,
        preserved_data_sha256: originals
            .into_iter()
            .map(|(table, identity)| (table, identity.sha256))
            .collect(),
        completed_at: chrono::Utc::now().to_rfc3339(),
    };
    atomic_json(&staging.path().join("migration-receipt.json"), &receipt)?;
    cancel.check()?;
    progress(MigrationProgress::Installing);
    cancel.check()?;
    confirm_source_preferences(source, &preferences_before, &flutter_closed)?;
    cancel.check()?;
    if destination.exists() {
        return Err(Failure::DestinationOccupied);
    }
    // Rename an entire staged store. A crash cannot expose a half-imported pair.
    fs::rename(staging.path(), destination).map_err(storage)?;
    Ok(receipt)
}

fn confirm_source_preferences(
    source: &LegacySource,
    expected: &[u8],
    flutter_closed: &impl Fn() -> Result<bool>,
) -> Result<()> {
    if !flutter_closed()? {
        return Err(Failure::LegacyRunning);
    }
    if fs::read(&source.preferences).map_err(storage)? != expected {
        return Err(Failure::MigrationSourceChanged);
    }
    Ok(())
}

pub fn verify_capture(destination: &Path, cancel: &Cancellation) -> Result<MigrationReceipt> {
    let receipt: MigrationReceipt = serde_json::from_slice(
        &fs::read(destination.join("migration-receipt.json")).map_err(storage)?,
    )
    .map_err(storage)?;
    if receipt.version != 3 || receipt.target_schema != SCHEMA_VERSION {
        return Err(Failure::Invalid("Unsupported migration receipt".into()));
    }
    if full_sha256(&destination.join("yamantaka.sqlite3"), cancel)?
        != receipt.captured_database_sha256
    {
        return Err(Failure::Invalid(
            "Captured database hash differs (the library may have been edited)".into(),
        ));
    }
    let preferences_path = destination.join("preferences.json");
    if full_sha256(&preferences_path, cancel)? != receipt.captured_preferences_sha256 {
        return Err(Failure::Invalid(
            "Captured preferences differ from the migration receipt".into(),
        ));
    }
    Preferences::load(&preferences_path)?;
    let database = readonly(&destination.join("yamantaka.sqlite3"))?;
    database.execute_batch("BEGIN").map_err(storage)?;
    validate(&database)?;
    if schema_hash(&database, cancel)? != receipt.captured_schema_sha256 {
        return Err(Failure::Invalid(
            "Captured schema differs from the migration receipt".into(),
        ));
    }
    if fingerprints(&table_identities(&database, cancel)?) != receipt.captured_data_sha256 {
        return Err(Failure::Invalid(
            "Captured records differ from the migration receipt".into(),
        ));
    }
    database.execute_batch("COMMIT").map_err(storage)?;
    cancel.check()?;
    Ok(receipt)
}

struct TableIdentity {
    columns: Vec<String>,
    sha256: String,
}
fn fingerprints(identities: &BTreeMap<String, TableIdentity>) -> BTreeMap<String, String> {
    identities
        .iter()
        .map(|(name, identity)| (name.clone(), identity.sha256.clone()))
        .collect()
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
    table_hash_projected(connection, name, columns, false, cancel)
}
fn schema_hash(connection: &Connection, cancel: &Cancellation) -> Result<String> {
    table_hash(
        connection,
        "sqlite_schema",
        &[
            "type".into(),
            "name".into(),
            "tbl_name".into(),
            "sql".into(),
        ],
        cancel,
    )
}
fn table_hash_projected(
    connection: &Connection,
    name: &str,
    columns: &[String],
    reset_cache: bool,
    cancel: &Cancellation,
) -> Result<String> {
    let column_sql = columns
        .iter()
        .map(|column| match (reset_cache, name, column.as_str()) {
            (true, "documents", "thumbnail_path") => "NULL".into(),
            (true, "document_covers", "small_thumbnail_path" | "large_thumbnail_path") => {
                "''".into()
            }
            _ => quoted(column),
        })
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
    for column in columns {
        hash.update((column.len() as u64).to_le_bytes());
        hash.update(column.as_bytes());
    }
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
