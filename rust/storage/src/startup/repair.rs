use super::*;
use sha2::{Digest, Sha256};
use std::io::Write;
use yamantaka_domain::{Cancellation, source_revision::PhysicalSourceVersion};
use yamantaka_windows::{retained_file::RetainedFile, source_stamp::Stamp};

#[derive(Clone)]
pub struct Review {
    root: PathBuf,
    version: PhysicalSourceVersion,
    digest: [u8; 32],
    pub bytes: u64,
}
pub struct Repaired {
    pub preserved: PathBuf,
}
fn unavailable() -> Failure {
    Failure::Storage("Startup record unavailable; existing files were preserved".into())
}
fn digest(record: &RetainedFile, cancel: &Cancellation) -> Result<[u8; 32]> {
    let mut file = record.file();
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        cancel.check()?;
        let count = file.read(&mut buffer).map_err(|_| unavailable())?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hash.finalize().into())
}
pub fn review(root: &Path, cancel: &Cancellation) -> Result<Review> {
    cancel.check()?;
    let root = root.canonicalize().map_err(|_| unavailable())?;
    let path = root.join("rust-startup-v1.json");
    let record = RetainedFile::open(&path).map_err(|_| unavailable())?;
    let bytes = record.file().metadata().map_err(|_| unavailable())?.len();
    if bytes > 8 * 1024 * 1024 {
        return Err(Failure::Invalid(
            "The startup record exceeds the repair limit; file preserved".into(),
        ));
    }
    if read(&path).is_ok() {
        return Err(Failure::Invalid("The startup record is valid. Close the app and check folder permissions before retrying".into()));
    }
    let version = Stamp::from_file(record.file())
        .ok_or_else(unavailable)?
        .version();
    let digest = digest(&record, cancel)?;
    Ok(Review {
        root,
        version,
        digest,
        bytes,
    })
}
pub fn apply(review: &Review, cancel: &Cancellation) -> Result<Repaired> {
    cancel.check()?;
    let path = review.root.join("rust-startup-v1.json");
    let record = RetainedFile::open(&path).map_err(|_| unavailable())?;
    if Stamp::from_file(record.file()).map(|stamp| stamp.version()) != Some(review.version)
        || digest(&record, cancel)? != review.digest
    {
        return Err(Failure::Invalid(
            "The startup record changed. Review it again; existing files were preserved".into(),
        ));
    }
    let mut replacement =
        tempfile::NamedTempFile::new_in(&review.root).map_err(|_| unavailable())?;
    serde_json::to_writer_pretty(
        &mut replacement,
        &State {
            version: 1,
            clean: true,
            failures: 0,
            token: uuid::Uuid::new_v4().to_string(),
        },
    )
    .map_err(|_| unavailable())?;
    replacement.write_all(b"\n").map_err(|_| unavailable())?;
    replacement
        .as_file()
        .sync_all()
        .map_err(|_| unavailable())?;
    let name = format!("rust-startup-preserved-{}.json", uuid::Uuid::new_v4());
    cancel.check()?;
    record
        .retain_as(name.as_ref())
        .map_err(|error| Failure::Storage(format!("Could not retain reviewed record: {error}")))?;
    // After retention, complete the small non-cancellable publication. A crash in
    // this gap is safe: normal startup recreates a missing derived record.
    replacement
        .persist_noclobber(&path)
        .map_err(|_| unavailable())?;
    Ok(Repaired {
        preserved: review.root.join(name),
    })
}
