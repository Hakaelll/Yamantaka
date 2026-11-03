//! Reviewed rollback swaps whole sibling stores; no library file is discarded.
use super::{RestoreProgress, TABLE_ORDER, restore::restore_journal_path};
use crate::{
    Database,
    database::storage,
    managed_imports::{ordinary, present},
    preferences::{Preferences, atomic_json},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
use yamantaka_domain::{Cancellation, Failure, Result};

#[derive(Debug, Clone)]
pub struct RollbackReview {
    active: PathBuf,
    source: PathBuf,
    sha256: String,
    counts: BTreeMap<String, usize>,
    bytes: u64,
}
impl RollbackReview {
    pub fn source(&self) -> &Path {
        &self.source
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn counts(&self) -> &BTreeMap<String, usize> {
        &self.counts
    }
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    active: PathBuf,
    source: PathBuf,
    safety: PathBuf,
    sha256: String,
}
struct Tree {
    hash: Sha256,
    entries: usize,
    names: usize,
    bytes: u64,
}
fn invalid(message: &str) -> Failure {
    Failure::Invalid(message.into())
}
fn active_path(active: &Path) -> Result<PathBuf> {
    let parent = active
        .parent()
        .ok_or_else(|| invalid("Missing rollback parent"))?;
    ordinary(parent, true)?;
    Ok(parent.canonicalize().map_err(storage)?.join(
        active
            .file_name()
            .ok_or_else(|| invalid("Invalid rollback store"))?,
    ))
}
fn safety_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_prefix("pre-restore-"))
        .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok())
}
pub(super) fn journal_path(active: &Path) -> Result<PathBuf> {
    let name = active
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| invalid("Invalid rollback store"))?;
    Ok(active.with_file_name(format!(".{name}.rollback-journal.json")))
}
pub(super) fn tree(path: &Path, cancel: &Cancellation) -> Result<(String, u64)> {
    ordinary(path, true)?;
    let mut state = Tree {
        hash: Sha256::new(),
        entries: 0,
        names: 0,
        bytes: 0,
    };
    state.hash.update(b"yamantaka-rollback-tree-v1");
    visit(path, path, 0, &mut state, cancel)?;
    Ok((format!("{:x}", state.hash.finalize()), state.bytes))
}
fn visit(
    root: &Path,
    path: &Path,
    depth: usize,
    state: &mut Tree,
    cancel: &Cancellation,
) -> Result<()> {
    cancel.check()?;
    if depth > 32 {
        return Err(invalid("Rollback directory nesting exceeds its limit"));
    }
    let mut children = Vec::new();
    for child in fs::read_dir(path).map_err(storage)? {
        cancel.check()?;
        let child = child.map_err(storage)?;
        state.entries += 1;
        state.names = state
            .names
            .checked_add(child.file_name().len())
            .ok_or_else(|| invalid("Rollback names exceed their limit"))?;
        if state.entries > 200_000 || state.names > 32 * 1024 * 1024 {
            return Err(invalid("Rollback directory inventory exceeds its limit"));
        }
        children.push(child);
    }
    children.sort_by_key(|e| e.file_name());
    for child in children {
        cancel.check()?;
        let child = child.path();
        let meta = child.symlink_metadata().map_err(storage)?;
        let directory = meta.is_dir();
        ordinary(&child, directory)?;
        let relative = child.strip_prefix(root).map_err(storage)?;
        let relative = relative
            .to_str()
            .ok_or_else(|| invalid("Unsupported rollback filename"))?
            .replace('\\', "/");
        state.hash.update(if directory { b"D" } else { b"F" });
        state.hash.update((relative.len() as u64).to_le_bytes());
        state.hash.update(relative.as_bytes());
        if directory {
            visit(root, &child, depth + 1, state, cancel)?;
        } else {
            let modified = meta.modified().map_err(storage)?;
            let ticks = match modified.duration_since(UNIX_EPOCH) {
                Ok(t) => i128::try_from(t.as_nanos()).map_err(storage)?,
                Err(t) => -i128::try_from(t.duration().as_nanos()).map_err(storage)?,
            };
            state.hash.update(meta.len().to_le_bytes());
            state.hash.update(ticks.to_le_bytes());
            state
                .hash
                .update(crate::identity::full_sha256(&child, cancel)?.as_bytes());
            state.bytes = state
                .bytes
                .checked_add(meta.len())
                .ok_or_else(|| invalid("Rollback size overflow"))?;
            let after = child.metadata().map_err(storage)?;
            if after.len() != meta.len() || after.modified().ok() != Some(modified) {
                return Err(Failure::BackupSourceChanged);
            }
        }
    }
    Ok(())
}
pub(super) fn validate_source(
    source: &Path,
    cancel: &Cancellation,
) -> Result<BTreeMap<String, usize>> {
    for name in ["yamantaka.sqlite3", "preferences.json"] {
        ordinary(&source.join(name), false)?;
    }
    let wal = source.join("yamantaka.sqlite3-wal");
    if present(&wal)? {
        ordinary(&wal, false)?;
    }
    // Validate independent main/WAL copies; SQLite sidecars stay in the temporary folder.
    let temp = tempfile::tempdir().map_err(storage)?;
    let copy = temp.path().join("library.sqlite3");
    copy_file(&source.join("yamantaka.sqlite3"), &copy, cancel)?;
    if present(&wal)? {
        let copy_wal = temp.path().join("library.sqlite3-wal");
        copy_file(&wal, &copy_wal, cancel)?;
        super::wal::validate(&copy_wal, &copy, cancel)?;
    }
    let db = Database::open_read_only(&copy)?;
    db.integrity()?;
    Preferences::load(&source.join("preferences.json"))?;
    let mut counts = BTreeMap::new();
    for table in TABLE_ORDER {
        cancel.check()?;
        let count: i64 = db
            .connection
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .map_err(storage)?;
        counts.insert(table.into(), usize::try_from(count).map_err(storage)?);
    }
    Ok(counts)
}
fn copy_file(source: &Path, destination: &Path, cancel: &Cancellation) -> Result<()> {
    let mut input = fs::File::open(source).map_err(storage)?;
    let mut output = fs::File::create(destination).map_err(storage)?;
    let mut buffer = vec![0; 65536];
    loop {
        cancel.check()?;
        let n = input.read(&mut buffer).map_err(storage)?;
        if n == 0 {
            break;
        }
        output.write_all(&buffer[..n]).map_err(storage)?;
    }
    Ok(())
}
pub fn review_rollback(
    active: &Path,
    source: &Path,
    cancel: &Cancellation,
) -> Result<RollbackReview> {
    cancel.check()?;
    let active = active_path(active)?;
    ordinary(&active, true)?;
    ordinary(source, true)?;
    let source = source.canonicalize().map_err(storage)?;
    if source == active || source.parent() != active.parent() || !safety_name(&source) {
        return Err(invalid(
            "Select a previous pre-restore safety folder beside the active library",
        ));
    }
    let (sha256, bytes) = tree(&source, cancel)?;
    let counts = validate_source(&source, cancel)?;
    if tree(&source, cancel)?.0 != sha256 {
        return Err(Failure::BackupSourceChanged);
    }
    Ok(RollbackReview {
        active,
        source,
        sha256,
        counts,
        bytes,
    })
}
pub fn verify_rollback(review: &RollbackReview, cancel: &Cancellation) -> Result<()> {
    cancel.check()?;
    if tree(&review.source, cancel)?.0 != review.sha256 {
        return Err(Failure::BackupSourceChanged);
    }
    Ok(())
}
pub fn install_rollback(
    review: &RollbackReview,
    active: &Path,
    cancel: &Cancellation,
) -> Result<PathBuf> {
    install_rollback_progress(review, active, cancel, |_| {})
}
pub fn install_rollback_progress(
    review: &RollbackReview,
    active: &Path,
    cancel: &Cancellation,
    mut progress: impl FnMut(RestoreProgress),
) -> Result<PathBuf> {
    let active = active_path(active)?;
    if active != review.active {
        return Err(invalid("Rollback belongs to a different library"));
    }
    ordinary(&active, true)?;
    verify_rollback(review, cancel)?;
    let journal = journal_path(&active)?;
    if present(&journal)? || present(&restore_journal_path(&active)?)? {
        return Err(invalid(
            "Recover the pending library operation before rollback",
        ));
    }
    let safety = active
        .parent()
        .unwrap()
        .join(format!("pre-restore-{}", uuid::Uuid::new_v4()));
    let record = Journal {
        version: 1,
        active: active.clone(),
        source: review.source.clone(),
        safety: safety.clone(),
        sha256: review.sha256.clone(),
    };
    progress(RestoreProgress::Installing);
    cancel.check()?;
    atomic_json(&journal, &record)?;
    if let Err(error) = cancel.check() {
        fs::remove_file(&journal).map_err(storage)?;
        return Err(error);
    }
    if let Err(error) = fs::rename(&active, &safety) {
        fs::remove_file(&journal).map_err(storage)?;
        return Err(storage(error));
    }
    progress(RestoreProgress::OriginalSaved);
    if present(&active)? {
        return Err(Failure::DestinationOccupied);
    }
    if let Err(error) = fs::rename(&review.source, &active) {
        fs::rename(&safety, &active).map_err(storage)?;
        fs::remove_file(&journal).map_err(storage)?;
        return Err(storage(error));
    }
    progress(RestoreProgress::Installed);
    if tree(&active, &Cancellation::default())?.0 != review.sha256 {
        if present(&review.source)? {
            return Err(Failure::BackupSourceChanged);
        }
        fs::rename(&active, &review.source).map_err(storage)?;
        fs::rename(&safety, &active).map_err(storage)?;
        fs::remove_file(&journal).map_err(storage)?;
        return Err(Failure::BackupSourceChanged);
    }
    fs::remove_file(journal).map_err(storage)?;
    Ok(safety)
}
pub(super) fn recover(active: &Path) -> Result<bool> {
    let active = active_path(active)?;
    let path = journal_path(&active)?;
    ordinary(&path, false)?;
    let mut bytes = Vec::new();
    fs::File::open(&path)
        .map_err(storage)?
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(storage)?;
    if bytes.len() > 65536 {
        return Err(invalid("Rollback journal exceeds its limit"));
    }
    let record: Journal = serde_json::from_slice(&bytes).map_err(storage)?;
    if record.version != 1
        || record.active != active
        || record.source == record.safety
        || record.source == active
        || record.safety == active
        || [record.source.as_path(), record.safety.as_path()]
            .iter()
            .any(|p| p.parent() != active.parent() || !safety_name(p))
        || record.sha256.len() != 64
        || !record.sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(invalid("Rollback journal has invalid paths or identity"));
    }
    let exists = [&active, &record.source, &record.safety]
        .map(|p| present(p))
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    let mut resolved = Vec::new();
    for (p, exists) in [&active, &record.source, &record.safety]
        .into_iter()
        .zip(&exists)
    {
        if *exists {
            ordinary(p, true)?;
            let canonical = p.canonicalize().map_err(storage)?;
            if resolved.contains(&canonical) {
                return Err(invalid("Rollback journal aliases a library folder"));
            }
            resolved.push(canonical);
        }
    }
    match exists.as_slice() {
        [false, true, true] => fs::rename(&record.safety, &active).map_err(storage)?,
        [true, false, true] if tree(&active, &Cancellation::default())?.0 == record.sha256 => {}
        [true, true, false] => {}
        _ => {
            return Err(invalid(
                "Rollback is ambiguous; all library folders were preserved",
            ));
        }
    }
    fs::remove_file(path).map_err(storage)?;
    Ok(true)
}
