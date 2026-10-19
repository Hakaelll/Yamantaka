use crate::database::storage;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};
use yamantaka_domain::{Cancellation, Result};

pub(crate) fn registered_document(
    connection: &rusqlite::Connection,
    path: &Path,
) -> Result<Option<String>> {
    if !path.is_absolute() {
        return Err(yamantaka_domain::Failure::Invalid(
            "Import path must be absolute".into(),
        ));
    }
    let normalized = path
        .to_str()
        .ok_or_else(|| {
            yamantaka_domain::Failure::Invalid(
                "Import path is not valid Unicode; choose another file name".into(),
            )
        })?
        .to_lowercase();
    let legacy = if let Some(unc) = normalized.strip_prefix(r"\\?\unc\") {
        format!(r"\\{unc}")
    } else {
        normalized
            .strip_prefix(r"\\?\")
            .unwrap_or(&normalized)
            .to_owned()
    };
    let verbatim = if normalized.starts_with(r"\\?\") {
        normalized.clone()
    } else if let Some(unc) = normalized.strip_prefix(r"\\") {
        format!(r"\\?\unc\{unc}")
    } else {
        format!(r"\\?\{normalized}")
    };
    let ids = connection
        .prepare("SELECT id FROM documents WHERE normalized_path IN (?1,?2,?3) ORDER BY id LIMIT 2")
        .map_err(storage)?
        .query_map(rusqlite::params![normalized, legacy, verbatim], |row| {
            row.get::<_, String>(0)
        })
        .map_err(storage)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(storage)?;
    if ids.len() > 1 {
        return Err(yamantaka_domain::Failure::Invalid(
            "More than one document uses this source path; review the library before importing"
                .into(),
        ));
    }
    Ok(ids.into_iter().next())
}

pub fn legacy_fingerprint(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(storage)?;
    let size = file.metadata().map_err(storage)?.len();
    let mut bytes = Vec::with_capacity(65536);
    file.take(65536).read_to_end(&mut bytes).map_err(storage)?;
    Ok(fingerprint_from_head(size, Sha256::digest(bytes).into()))
}

use yamantaka_domain::source_revision::legacy_fingerprint as fingerprint_from_head;

pub(crate) fn verify_import_source(
    path: &Path,
    source: &yamantaka_domain::source_revision::SourceRevision,
) -> Result<String> {
    let unchanged = || -> Result<()> {
        let metadata = path.metadata().map_err(storage)?;
        if path != source.canonical_path
            || metadata.len() != source.size
            || metadata.modified().map_err(storage)? != source.modified
        {
            return Err(yamantaka_domain::Failure::Invalid(
                "PDF source changed during import".into(),
            ));
        }
        Ok(())
    };
    unchanged()?;
    let fingerprint = legacy_fingerprint(path)?;
    if fingerprint != fingerprint_from_head(source.size, source.head_sha256) {
        return Err(yamantaka_domain::Failure::Invalid(
            "PDF source changed during import".into(),
        ));
    }
    unchanged()?;
    Ok(fingerprint)
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
