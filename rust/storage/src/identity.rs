use crate::database::storage;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};
use yamantaka_domain::{Cancellation, Result};

pub fn legacy_fingerprint(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(storage)?;
    let size = file.metadata().map_err(storage)?.len();
    let mut bytes = Vec::with_capacity(65536);
    file.take(65536).read_to_end(&mut bytes).map_err(storage)?;
    let partial = format!("{:x}", Sha256::digest(bytes));
    Ok(format!(
        "{:x}",
        Sha256::digest(format!("{size}:{partial}").as_bytes())
    ))
}

pub fn full_sha256(path: &Path, cancellation: &Cancellation) -> Result<String> {
    let mut file = File::open(path).map_err(storage)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        cancellation.check()?;
        let count = file.read(&mut buffer).map_err(storage)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

/// Exact duplicate confirmation never relies on the legacy partial fingerprint.
pub fn exact_duplicate(left: &Path, right: &Path, cancellation: &Cancellation) -> Result<bool> {
    let before_left = left.metadata().map_err(storage)?;
    let before_right = right.metadata().map_err(storage)?;
    if before_left.len() != before_right.len() {
        return Ok(false);
    }
    let equal = full_sha256(left, cancellation)? == full_sha256(right, cancellation)?;
    let after_left = left.metadata().map_err(storage)?;
    let after_right = right.metadata().map_err(storage)?;
    Ok(equal
        && before_left.len() == after_left.len()
        && before_right.len() == after_right.len()
        && before_left.modified().ok() == after_left.modified().ok()
        && before_right.modified().ok() == after_right.modified().ok())
}
