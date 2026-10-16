use std::{path::PathBuf, time::SystemTime};

/// Bounded source identity captured by the native document owner, not a full-file digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRevision {
    pub canonical_path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub head_sha256: [u8; 32],
}
