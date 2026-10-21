use super::RestorePreview;
use crate::managed_imports::{ordinary, present};
use crate::{
    database::{snapshot_cancellable, storage},
    preferences::atomic_json,
};
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path::Path};
use yamantaka_domain::{Cancellation, Failure, Result};
#[derive(Debug, Clone, Copy)]
pub enum RestoreProgress {
    Copying { copied: u32, total: u32 },
    Validating,
    Installing,
    OriginalSaved,
    Installed,
}
/// Installation requires closing the active database first. The pre-restore copy
/// remains in its own directory and can be selected for rollback on next launch.
pub fn install_preview(
    preview: RestorePreview,
    active: &Path,
    cancel: &Cancellation,
) -> Result<std::path::PathBuf> {
    install_preview_progress(preview, active, cancel, |_| {})
}
pub fn install_preview_progress(
    preview: RestorePreview,
    active: &Path,
    cancel: &Cancellation,
    mut progress: impl FnMut(RestoreProgress),
) -> Result<std::path::PathBuf> {
    cancel.check()?;
    let parent = active
        .parent()
        .ok_or_else(|| Failure::Invalid("Missing restore parent".into()))?;
    ordinary(parent, true)?;
    let existed = present(active)?;
    if existed {
        ordinary(active, true)?;
    }
    let journal = restore_journal_path(active)?;
    if present(&journal)? {
        return Err(Failure::Invalid(
            "Recover the interrupted restore before starting another".into(),
        ));
    }
    let stage = tempfile::Builder::new()
        .prefix(".restore-")
        .tempdir_in(parent)
        .map_err(storage)?;
    snapshot_cancellable(
        &preview.database.connection,
        &stage.path().join("yamantaka.sqlite3"),
        cancel,
        |copied, total| progress(RestoreProgress::Copying { copied, total }),
    )?;
    preview
        .preferences
        .save(&stage.path().join("preferences.json"))?;
    cancel.check()?;
    progress(RestoreProgress::Validating);
    cancel.check()?;
    let database_hash =
        crate::identity::full_sha256(&stage.path().join("yamantaka.sqlite3"), cancel)?;
    let preferences_hash =
        crate::identity::full_sha256(&stage.path().join("preferences.json"), cancel)?;
    atomic_json(
        &stage.path().join("restore-receipt.json"),
        &serde_json::json!({"version":1,"backupVersion":preview.backup_version,"databaseSha256":database_hash,"preferencesSha256":preferences_hash}),
    )?;
    let receipt_hash =
        crate::identity::full_sha256(&stage.path().join("restore-receipt.json"), cancel)?;
    let safety = parent.join(format!("pre-restore-{}", uuid::Uuid::new_v4()));
    progress(RestoreProgress::Installing);
    cancel.check()?;
    if present(&journal)? || present(&safety)? {
        return Err(Failure::Invalid(
            "Recover the interrupted restore before starting another".into(),
        ));
    }
    atomic_json(
        &journal,
        &RestoreJournal {
            version: 2,
            active: active.to_owned(),
            safety: safety.clone(),
            replacement_sha256: database_hash,
            replacement_preferences_sha256: Some(preferences_hash),
            replacement_receipt_sha256: Some(receipt_hash),
            original_existed: Some(existed),
            staging: Some(stage.path().to_owned()),
        },
    )?;
    if let Err(error) = cancel.check() {
        fs::remove_file(&journal).map_err(storage)?;
        return Err(error);
    }
    if existed {
        ordinary(active, true)?;
        fs::rename(active, &safety).map_err(storage)?;
        progress(RestoreProgress::OriginalSaved);
    } else if present(active)? {
        fs::remove_file(&journal).map_err(storage)?;
        return Err(Failure::DestinationOccupied);
    }
    if let Err(error) = fs::rename(stage.path(), active) {
        if safety.exists() {
            fs::rename(&safety, active).map_err(storage)?;
        }
        fs::remove_file(&journal).map_err(storage)?;
        return Err(storage(error));
    }
    progress(RestoreProgress::Installed);
    fs::remove_file(journal).map_err(storage)?;
    Ok(safety)
}

#[derive(Serialize, Deserialize)]
struct RestoreJournal {
    version: u32,
    active: std::path::PathBuf,
    safety: std::path::PathBuf,
    replacement_sha256: String,
    #[serde(default)]
    replacement_preferences_sha256: Option<String>,
    #[serde(default)]
    replacement_receipt_sha256: Option<String>,
    #[serde(default)]
    original_existed: Option<bool>,
    #[serde(default)]
    staging: Option<std::path::PathBuf>,
}
fn restore_journal_path(active: &Path) -> Result<std::path::PathBuf> {
    let name = active
        .file_name()
        .ok_or_else(|| Failure::Invalid("Invalid active store".into()))?
        .to_string_lossy();
    Ok(active.with_file_name(format!(".{name}.restore-journal.json")))
}
/// Called before opening any active DB. A crash between the two renames rolls
/// back to the complete safety store. A committed new store keeps its safety copy.
pub fn recover_interrupted_restore(active: &Path) -> Result<bool> {
    let journal_path = restore_journal_path(active)?;
    if !present(&journal_path)? {
        return Ok(false);
    }
    ordinary(&journal_path, false)?;
    let mut bytes = Vec::new();
    fs::File::open(&journal_path)
        .map_err(storage)?
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(storage)?;
    if bytes.len() > 65536 {
        return Err(Failure::Invalid("Restore journal is too large".into()));
    }
    let journal: RestoreJournal = serde_json::from_slice(&bytes).map_err(storage)?;
    let parent = active
        .parent()
        .ok_or_else(|| Failure::Invalid("Missing recovery parent".into()))?;
    if !matches!(journal.version, 1 | 2)
        || journal.active != active
        || journal.safety.parent() != Some(parent)
        || journal.safety == active
        || journal.staging.as_ref().is_some_and(|stage| {
            journal.version != 2
                || stage.parent() != Some(parent)
                || stage == active
                || *stage == journal.safety
                || stage
                    .file_name()
                    .is_none_or(|name| !name.to_string_lossy().starts_with(".restore-"))
        })
        || journal
            .safety
            .file_name()
            .is_none_or(|name| !name.to_string_lossy().starts_with("pre-restore-"))
    {
        return Err(Failure::Invalid("Restore journal has invalid paths".into()));
    }
    ordinary(parent, true)?;
    let active_exists = present(active)?;
    let safety_exists = present(&journal.safety)?;
    if active_exists {
        ordinary(active, true)?;
    }
    if safety_exists {
        ordinary(&journal.safety, true)?;
    }
    if !active_exists && safety_exists {
        fs::rename(&journal.safety, active).map_err(storage)?;
    } else if active_exists && (safety_exists || journal.original_existed == Some(false)) {
        let matches = || -> Result<bool> {
            if journal.version != 2 {
                return Ok(false);
            }
            let wal = active.join("yamantaka.sqlite3-wal");
            if present(&wal)? {
                ordinary(&wal, false)?;
                if fs::metadata(&wal).map_err(storage)?.len() > 0 {
                    return Ok(false);
                }
            }
            for (name, expected) in [
                ("yamantaka.sqlite3", Some(&journal.replacement_sha256)),
                (
                    "preferences.json",
                    journal.replacement_preferences_sha256.as_ref(),
                ),
                (
                    "restore-receipt.json",
                    journal.replacement_receipt_sha256.as_ref(),
                ),
            ] {
                let Some(expected) = expected else {
                    return Ok(false);
                };
                let path = active.join(name);
                ordinary(&path, false)?;
                if crate::identity::full_sha256(&path, &Cancellation::default())? != *expected {
                    return Ok(false);
                }
            }
            Ok(true)
        };
        if !matches()? {
            return Err(Failure::Invalid(
                "Interrupted restore requires explicit recovery; both copies were retained".into(),
            ));
        }
    } else if !active_exists && journal.original_existed != Some(false) {
        return Err(Failure::Invalid("Both recovery stores are missing".into()));
    }
    if let Some(stage) = &journal.staging
        && present(stage)?
    {
        ordinary(stage, true)?;
        let files = [
            ("yamantaka.sqlite3", Some(&journal.replacement_sha256)),
            (
                "preferences.json",
                journal.replacement_preferences_sha256.as_ref(),
            ),
            (
                "restore-receipt.json",
                journal.replacement_receipt_sha256.as_ref(),
            ),
        ];
        for entry in fs::read_dir(stage).map_err(storage)? {
            let entry = entry.map_err(storage)?;
            if !files.iter().any(|(name, _)| entry.file_name() == *name) {
                return Err(Failure::Invalid(
                    "Unrecognized restore staging files were retained".into(),
                ));
            }
        }
        for (name, expected) in files {
            let Some(expected) = expected else {
                return Err(Failure::Invalid("Incomplete restore journal".into()));
            };
            let path = stage.join(name);
            if !present(&path)? {
                continue;
            }
            ordinary(&path, false)?;
            if crate::identity::full_sha256(&path, &Cancellation::default())? != *expected {
                return Err(Failure::Invalid(
                    "Changed restore staging files were retained".into(),
                ));
            }
        }
        for (name, _) in files {
            match fs::remove_file(stage.join(name)) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(storage(error)),
            }
        }
        fs::remove_dir(stage).map_err(storage)?;
    }
    fs::remove_file(journal_path).map_err(storage)?;
    Ok(true)
}
