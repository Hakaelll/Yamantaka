//! Explicit recovery archives a pending operation without following its paths.
use super::{restore, rollback};
use crate::{
    database::storage,
    managed_imports::{ordinary, present},
};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use yamantaka_domain::{Cancellation, Failure, Result};
use yamantaka_windows::{retained_file::RetainedFile, source_stamp::Stamp};

#[derive(Clone)]
pub struct Review {
    active: PathBuf,
    active_sha: Option<String>,
    records: Vec<Record>,
    previous: Option<super::RollbackReview>,
    counts: BTreeMap<String, usize>,
}
#[derive(Clone)]
struct Record {
    path: PathBuf,
    version: yamantaka_domain::source_revision::PhysicalSourceVersion,
    sha: [u8; 32],
}
impl Review {
    pub fn source(&self) -> &Path {
        self.previous
            .as_ref()
            .map_or(self.active.as_path(), |r| r.source())
    }
    pub fn counts(&self) -> &BTreeMap<String, usize> {
        &self.counts
    }
    pub fn previous(&self) -> bool {
        self.previous.is_some()
    }
    pub fn current_missing(&self) -> bool {
        self.active_sha.is_none()
    }
}
fn invalid() -> Failure {
    Failure::Invalid(
        "The pending operation changed or cannot be reviewed; all files were preserved".into(),
    )
}
pub(super) fn recovery_guard_path(active: &Path) -> Result<PathBuf> {
    let name = active.file_name().ok_or_else(invalid)?.to_string_lossy();
    Ok(active.with_file_name(format!(".{name}.recovery-review.json")))
}
fn records(active: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for path in [
        restore::restore_journal_path(active)?,
        rollback::journal_path(active)?,
        recovery_guard_path(active)?,
    ] {
        if present(&path)? {
            paths.push(path);
        }
    }
    Ok(paths)
}
pub fn present_for(active: &Path) -> Result<bool> {
    Ok(!records(active)?.is_empty())
}
fn digest(file: &RetainedFile, cancel: &Cancellation) -> Result<[u8; 32]> {
    use sha2::{Digest, Sha256};
    cancel.check()?;
    let mut input = file.file();
    let mut hash = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        cancel.check()?;
        let read = input.read(&mut buffer).map_err(storage)?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    cancel.check()?;
    Ok(hash.finalize().into())
}
pub fn review(active: &Path, previous: Option<&Path>, cancel: &Cancellation) -> Result<Review> {
    cancel.check()?;
    let active = rollback::active_path(active)?;
    ordinary(active.parent().ok_or_else(invalid)?, true)?;
    let paths = records(&active)?;
    if paths.is_empty() {
        return Err(invalid());
    }
    let records = paths
        .into_iter()
        .map(|path| {
            let lease = RetainedFile::open(&path).map_err(|_| invalid())?;
            Ok(Record {
                path,
                version: Stamp::from_file(lease.file())
                    .ok_or_else(invalid)?
                    .version(),
                sha: digest(&lease, cancel)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let active_sha = active_snapshot(&active, cancel)?;
    let previous = previous
        .map(|source| rollback::review_for_recovery(&active, source, cancel))
        .transpose()?;
    let counts = if let Some(previous) = &previous {
        previous.counts().clone()
    } else {
        rollback::validate_source(&active, cancel)?
    };
    if active_snapshot(&active, cancel)? != active_sha {
        return Err(Failure::BackupSourceChanged);
    }
    Ok(Review {
        active,
        active_sha,
        records,
        previous,
        counts,
    })
}
fn active_snapshot(active: &Path, cancel: &Cancellation) -> Result<Option<String>> {
    if !present(active)? {
        return Ok(None);
    }
    ordinary(active, true)?;
    Ok(Some(rollback::tree(active, cancel)?.0))
}
pub fn verify(review: &Review, cancel: &Cancellation) -> Result<()> {
    cancel.check()?;
    if records(&review.active)?
        != review
            .records
            .iter()
            .map(|r| r.path.clone())
            .collect::<Vec<_>>()
        || active_snapshot(&review.active, cancel)? != review.active_sha
    {
        return Err(invalid());
    }
    if let Some(previous) = &review.previous {
        super::verify_rollback(previous, cancel)?;
    }
    for record in &review.records {
        let lease = RetainedFile::open(&record.path).map_err(|_| invalid())?;
        check_record(record, &lease, cancel)?;
    }
    Ok(())
}
fn check_record(record: &Record, lease: &RetainedFile, cancel: &Cancellation) -> Result<()> {
    if Stamp::from_file(lease.file()).map(|s| s.version()) != Some(record.version)
        || digest(lease, cancel)? != record.sha
    {
        return Err(invalid());
    }
    Ok(())
}
pub fn resolve(review: &Review, active: &Path, cancel: &Cancellation) -> Result<PathBuf> {
    if rollback::active_path(active)? != review.active {
        return Err(invalid());
    }
    verify(review, cancel)?;
    let leases = review
        .records
        .iter()
        .map(|record| {
            let lease = RetainedFile::open(&record.path).map_err(|_| invalid())?;
            check_record(record, &lease, cancel)?;
            Ok((record, lease))
        })
        .collect::<Result<Vec<_>>>()?;
    cancel.check()?;
    let guard = recovery_guard_path(&review.active)?;
    let existing_guard = leases
        .iter()
        .find(|(record, _)| record.path == guard)
        .map(|(_, lease)| lease);
    let created_guard = if leases.len() > 1 && existing_guard.is_none() {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&guard)
            .map_err(|_| invalid())?;
        file.write_all(b"{\"version\":1,\"state\":\"preserving-operation-records\"}\n")
            .map_err(storage)?;
        file.sync_all().map_err(storage)?;
        drop(file);
        Some(RetainedFile::open(&guard).map_err(|_| invalid())?)
    } else {
        None
    };
    let parent = review.active.parent().ok_or_else(invalid)?;
    let retain = |lease: &RetainedFile, prefix: &str| -> Result<PathBuf> {
        let name = format!("{prefix}-{}.json", uuid::Uuid::new_v4());
        lease.retain_as(name.as_ref()).map_err(|_| invalid())?;
        Ok(parent.join(name))
    };
    let mut preserved = None;
    for (record, lease) in &leases {
        if record.path != guard {
            preserved = Some(retain(lease, "yamantaka-recovery-preserved")?);
        }
    }
    if let Some(lease) = existing_guard.or(created_guard.as_ref()) {
        // Keep automatic recovery blocked until every reviewed operation is archived.
        if records(&review.active)? != [guard] {
            return Err(invalid());
        }
        preserved = Some(retain(lease, "yamantaka-recovery-barrier-preserved")?);
    }
    // The archived operation is never deleted. A new rollback has its own journal.
    match &review.previous {
        Some(previous) => {
            super::install_rollback(previous, &review.active, &Cancellation::default())
        }
        None => preserved.ok_or_else(invalid),
    }
}
