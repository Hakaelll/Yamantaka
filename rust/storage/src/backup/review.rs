use super::{RestorePreview, install_preview, preview_payload, read_payload};
use crate::database::storage;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use yamantaka_domain::{Cancellation, Result};

#[derive(Debug, Clone)]
pub struct RestoreReview {
    source: PathBuf,
    sha256: String,
    counts: BTreeMap<String, usize>,
    version: u64,
}
impl RestoreReview {
    pub fn source(&self) -> &Path {
        &self.source
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn counts(&self) -> &BTreeMap<String, usize> {
        &self.counts
    }
    pub fn version(&self) -> u64 {
        self.version
    }
}
pub fn review(path: &Path, cancel: &Cancellation) -> Result<RestoreReview> {
    cancel.check()?;
    let source = path.canonicalize().map_err(storage)?;
    let (payload, sha256) = read_payload(&source, cancel, None)?;
    let preview = preview_payload(payload, cancel)?;
    Ok(RestoreReview {
        source,
        sha256,
        counts: preview.counts,
        version: preview.backup_version,
    })
}
pub fn prepare_reviewed(review: &RestoreReview, cancel: &Cancellation) -> Result<RestorePreview> {
    let (payload, _) = read_payload(review.source(), cancel, Some(&review.sha256))?;
    preview_payload(payload, cancel)
}
/// Call only after closing the active library and all of its database readers.
pub fn install_reviewed(
    review: &RestoreReview,
    active: &Path,
    cancel: &Cancellation,
) -> Result<PathBuf> {
    install_preview(prepare_reviewed(review, cancel)?, active, cancel)
}
