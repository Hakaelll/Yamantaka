use std::{path::PathBuf, time::SystemTime};

/// Runtime comparison token for the physical file that produced a preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalSourceVersion([u8; 32]);
impl PhysicalSourceVersion {
    pub fn from_digest(digest: [u8; 32]) -> Self {
        Self(digest)
    }
}

/// Bounded source identity captured by the native document owner, not a full-file digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRevision {
    pub canonical_path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub head_sha256: [u8; 32],
}

/// A full digest bound to the same native source revision as imported metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullSourceHash {
    pub source: SourceRevision,
    pub digest: [u8; 32],
}

/// Legacy identity preserves migrated references; it does not prove full equality.
pub fn legacy_fingerprint(size: u64, head: [u8; 32]) -> String {
    use sha2::{Digest, Sha256};
    let digest: sha2::digest::Output<Sha256> = head.into();
    let partial = format!("{digest:x}");
    format!(
        "{:x}",
        Sha256::digest(format!("{size}:{partial}").as_bytes())
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CataloguedFile {
    pub path: PathBuf,
    pub fingerprint: String,
    pub page_count: u32,
    pub missing: bool,
    pub size: u64,
    pub modified: String,
}
