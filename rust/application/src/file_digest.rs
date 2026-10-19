use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    time::SystemTime,
};
use yamantaka_domain::{Cancellation, Failure, Result, source_revision::legacy_fingerprint};
fn changed() -> Failure {
    Failure::Invalid("File changed during verification; refresh and retry".into())
}
fn io_error(_: std::io::Error) -> Failure {
    Failure::Storage("Could not read the file; check access and close other editors".into())
}
pub(crate) struct HashedFile {
    file: File,
    path: PathBuf,
    requested_path: PathBuf,
    size: u64,
    modified: SystemTime,
    digest: [u8; 32],
    head: [u8; 32],
}
impl HashedFile {
    pub(crate) fn read(path: &Path, buffer: &mut [u8], cancel: &Cancellation) -> Result<Self> {
        cancel.check()?;
        let requested_path = path.to_owned();
        let path = path.canonicalize().map_err(io_error)?;
        let mut options = OpenOptions::new();
        options.read(true);
        // Deny concurrent writes/deletion while the proof is computed and delivered.
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(1);
        }
        let mut file = options.open(&path).map_err(io_error)?;
        let before = file.metadata().map_err(io_error)?;
        if !before.is_file() {
            return Err(Failure::Invalid(
                "Verification requires regular files".into(),
            ));
        }
        let modified = before.modified().map_err(io_error)?;
        let mut hash = Sha256::new();
        let mut head = Sha256::new();
        let mut count = 0u64;
        loop {
            cancel.check()?;
            let read = file.read(buffer).map_err(io_error)?;
            if read == 0 {
                break;
            }
            let head_bytes = (65536u64.saturating_sub(count)).min(read as u64) as usize;
            head.update(&buffer[..head_bytes]);
            hash.update(&buffer[..read]);
            count = count.checked_add(read as u64).ok_or_else(changed)?;
        }
        if count != before.len() {
            return Err(changed());
        }
        let result = Self {
            file,
            path,
            requested_path,
            size: count,
            modified,
            digest: hash.finalize().into(),
            head: head.finalize().into(),
        };
        result.verify()?;
        Ok(result)
    }
    pub(crate) fn reference(
        &self,
        source: &yamantaka_domain::source_revision::SourceRevision,
    ) -> Result<yamantaka_domain::source_revision::FullSourceHash> {
        self.verify()?;
        if self.path != source.canonical_path
            || self.size != source.size
            || self.modified != source.modified
            || self.head != source.head_sha256
        {
            return Err(changed());
        }
        Ok(yamantaka_domain::source_revision::FullSourceHash {
            source: source.clone(),
            digest: self.digest,
        })
    }
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub(crate) fn size(&self) -> u64 {
        self.size
    }
    pub(crate) fn modified(&self) -> SystemTime {
        self.modified
    }
    pub(crate) fn fingerprint(&self) -> String {
        legacy_fingerprint(self.size, self.head)
    }
    pub(crate) fn verify(&self) -> Result<()> {
        let held = self.file.metadata().map_err(io_error)?;
        let current = self.path.metadata().map_err(io_error)?;
        if self.requested_path.canonicalize().map_err(io_error)? != self.path
            || [held, current]
                .iter()
                .any(|m| m.len() != self.size || m.modified().ok() != Some(self.modified))
        {
            return Err(changed());
        }
        Ok(())
    }
}
