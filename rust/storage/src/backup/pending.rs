//! Explicit recovery archives a pending operation without following its paths.
use super::{restore, rollback};
use crate::{
    database::storage,
    managed_imports::{ordinary, present},
};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};
use yamantaka_domain::{Cancellation, Failure, Result};
use yamantaka_windows::{retained_file::RetainedFile, source_stamp::Stamp};

#[derive(Clone)]
pub struct Review {
    active: PathBuf,
    active_sha: String,
    journal: PathBuf,
    journal_version: yamantaka_domain::source_revision::PhysicalSourceVersion,
    journal_sha: [u8; 32],
    previous: Option<super::RollbackReview>,
    counts: BTreeMap<String, usize>,
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
}
fn invalid() -> Failure {
    Failure::Invalid(
        "The pending operation changed or cannot be reviewed; all files were preserved".into(),
    )
}
fn record(active: &Path) -> Result<PathBuf> {
    let mut paths = Vec::new();
    for path in [
        restore::restore_journal_path(active)?,
        rollback::journal_path(active)?,
    ] {
        if present(&path)? {
            paths.push(path);
        }
    }
    if paths.len() != 1 {
        return Err(invalid());
    }
    Ok(paths.remove(0))
}
pub fn present_for(active: &Path) -> Result<bool> {
    Ok(present(&restore::restore_journal_path(active)?)?
        || present(&rollback::journal_path(active)?)?)
}
fn digest(file: &RetainedFile, cancel: &Cancellation) -> Result<[u8; 32]> {
    use sha2::{Digest, Sha256};
    cancel.check()?;
    let mut bytes = Vec::new();
    file.file()
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(storage)?;
    if bytes.len() > 65536 {
        return Err(invalid());
    }
    cancel.check()?;
    Ok(Sha256::digest(bytes).into())
}
pub fn review(active: &Path, previous: Option<&Path>, cancel: &Cancellation) -> Result<Review> {
    cancel.check()?;
    ordinary(active, true)?;
    let active = active.canonicalize().map_err(storage)?;
    ordinary(active.parent().ok_or_else(invalid)?, true)?;
    let journal = record(&active)?;
    let lease = RetainedFile::open(&journal).map_err(|_| invalid())?;
    let journal_version = Stamp::from_file(lease.file())
        .ok_or_else(invalid)?
        .version();
    let journal_sha = digest(&lease, cancel)?;
    let active_sha = rollback::tree(&active, cancel)?.0;
    let previous = previous
        .map(|source| rollback::review_rollback(&active, source, cancel))
        .transpose()?;
    let counts = if let Some(previous) = &previous {
        previous.counts().clone()
    } else {
        rollback::validate_source(&active, cancel)?
    };
    if rollback::tree(&active, cancel)?.0 != active_sha {
        return Err(Failure::BackupSourceChanged);
    }
    Ok(Review {
        active,
        active_sha,
        journal,
        journal_version,
        journal_sha,
        previous,
        counts,
    })
}
pub fn verify(review: &Review, cancel: &Cancellation) -> Result<()> {
    cancel.check()?;
    if record(&review.active)? != review.journal
        || rollback::tree(&review.active, cancel)?.0 != review.active_sha
    {
        return Err(invalid());
    }
    if let Some(previous) = &review.previous {
        super::verify_rollback(previous, cancel)?;
    }
    let lease = RetainedFile::open(&review.journal).map_err(|_| invalid())?;
    check_record(review, &lease, cancel)
}
fn check_record(review: &Review, lease: &RetainedFile, cancel: &Cancellation) -> Result<()> {
    if Stamp::from_file(lease.file()).map(|s| s.version()) != Some(review.journal_version)
        || digest(lease, cancel)? != review.journal_sha
    {
        return Err(invalid());
    }
    Ok(())
}
pub fn resolve(review: &Review, active: &Path, cancel: &Cancellation) -> Result<PathBuf> {
    if active.canonicalize().map_err(storage)? != review.active {
        return Err(invalid());
    }
    verify(review, cancel)?;
    let lease = RetainedFile::open(&review.journal).map_err(|_| invalid())?;
    check_record(review, &lease, cancel)?;
    let name = format!("yamantaka-recovery-preserved-{}.json", uuid::Uuid::new_v4());
    cancel.check()?;
    lease.retain_as(name.as_ref()).map_err(|_| invalid())?;
    // The archived operation is never deleted. A new rollback has its own journal.
    match &review.previous {
        Some(previous) => {
            super::install_rollback(previous, &review.active, &Cancellation::default())
        }
        None => Ok(review.journal.parent().ok_or_else(invalid)?.join(name)),
    }
}
