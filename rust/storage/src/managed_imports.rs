//! Explicit selection of independent imported stores; the original store stays in place.
use crate::{
    database::storage,
    migration::{self, MigrationProgress, MigrationReceipt, MigrationReview},
    preferences::atomic_json,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use yamantaka_domain::{Cancellation, Failure, Result};
pub mod repair;

#[derive(Serialize, Deserialize)]
struct Selection {
    version: u32,
    import: Option<String>,
}
#[derive(Debug, Clone)]
pub struct ImportTarget {
    root: PathBuf,
    id: String,
}
impl ImportTarget {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
    pub fn path(&self) -> PathBuf {
        self.root.join("imports").join(&self.id)
    }
    pub fn capture(
        &self,
        review: &MigrationReview,
        closed: impl Fn() -> Result<bool>,
        cancel: &Cancellation,
        progress: impl FnMut(MigrationProgress),
    ) -> Result<MigrationReceipt> {
        ordinary(&self.root, true)?;
        let imports = self.root.join("imports");
        if !imports.exists() {
            fs::create_dir(&imports).map_err(storage)?;
        }
        ordinary(&imports, true)?;
        if self.path().exists() {
            ordinary(&self.path(), true)?;
        }
        migration::capture_reviewed(review, &self.path(), closed, cancel, progress)
    }
    pub fn activate(&self, cancel: &Cancellation) -> Result<()> {
        checked_store(&self.root, &self.id)?;
        migration::verify_capture(&self.path(), cancel)?;
        cancel.check()?;
        save_selection(&self.root, Some(self.id.clone()))
    }
}
pub(crate) fn present(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(storage(error)),
    }
}
pub(crate) fn ordinary(path: &Path, directory: bool) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(storage)?;
    #[cfg(windows)]
    let reparse = {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    };
    #[cfg(not(windows))]
    let reparse = metadata.file_type().is_symlink();
    if reparse || metadata.is_dir() != directory || (!directory && !metadata.is_file()) {
        return Err(Failure::Invalid(
            "The selected library must use ordinary local files and directories".into(),
        ));
    }
    Ok(())
}
fn store_path(root: &Path, id: &str) -> Result<PathBuf> {
    let parsed = uuid::Uuid::parse_str(id)
        .map_err(|_| Failure::Invalid("Invalid imported library identity".into()))?;
    if parsed.to_string() != id {
        return Err(Failure::Invalid("Invalid imported library identity".into()));
    }
    let imports = root.join("imports");
    let store = imports.join(id);
    for path in [root, &imports] {
        ordinary(path, true)?;
    }
    Ok(store)
}
fn checked_store(root: &Path, id: &str) -> Result<PathBuf> {
    let store = store_path(root, id)?;
    let imports = root.join("imports");
    for path in [root, &imports, &store] {
        ordinary(path, true)?;
    }
    for name in ["yamantaka.sqlite3", "preferences.json"] {
        ordinary(&store.join(name), false)?;
    }
    let restored = store.join("restore-receipt.json");
    ordinary(
        &if present(&restored)? {
            restored
        } else {
            store.join("migration-receipt.json")
        },
        false,
    )?;
    Ok(store)
}
fn save_selection(root: &Path, import: Option<String>) -> Result<()> {
    if repair::pending(root)? {
        return Err(Failure::Invalid(
            "Complete pending library-selection recovery first".into(),
        ));
    }
    ordinary(root, true)?;
    let path = root.join("selected-library.json");
    if present(&path)? {
        ordinary(&path, false)?;
    }
    atomic_json(&path, &Selection { version: 1, import })
}
fn selected(root: &Path) -> Result<Option<String>> {
    if repair::pending(root)? {
        return Err(Failure::Invalid(
            "Complete pending library-selection recovery first".into(),
        ));
    }
    let path = root.join("selected-library.json");
    if !present(&path)? {
        return Ok(None);
    }
    ordinary(root, true)?;
    ordinary(&path, false)?;
    let selection: Selection =
        serde_json::from_slice(&repair::read_record(&path)?).map_err(storage)?;
    if selection.version != 1 {
        return Err(Failure::Invalid(
            "Unsupported library selection version".into(),
        ));
    }
    Ok(selection.import)
}
pub fn active_store(root: &Path) -> Result<PathBuf> {
    match selected(root)? {
        Some(id) => checked_store(root, &id),
        None => Ok(root.to_owned()),
    }
}
/// Resolve managed selection only after interrupted directory swaps have recovered.
pub fn recover_and_select(root: &Path) -> Result<PathBuf> {
    repair::recover(root)?;
    crate::backup::recover_interrupted_restore(root)?;
    if let Some(id) = selected(root)? {
        crate::backup::recover_interrupted_restore(&store_path(root, &id)?)?;
    }
    active_store(root)
}
pub enum RecoveryTarget {
    PendingOperation(PathBuf),
    LibrarySelection(PathBuf),
}
/// Locate blocked startup state without opening or switching a library.
pub fn recovery_target(root: &Path) -> Result<Option<RecoveryTarget>> {
    if repair::pending(root)? {
        return Ok(Some(RecoveryTarget::LibrarySelection(root.to_owned())));
    }
    if crate::backup::pending::present_for(root)? {
        return Ok(Some(RecoveryTarget::PendingOperation(root.to_owned())));
    }
    if let Ok(Some(id)) = selected(root) {
        let store = match store_path(root, &id) {
            Ok(store) => store,
            Err(_) => return Ok(Some(RecoveryTarget::LibrarySelection(root.to_owned()))),
        };
        if crate::backup::pending::present_for(&store)? {
            return Ok(Some(RecoveryTarget::PendingOperation(store)));
        }
    }
    if present(&root.join("selected-library.json"))? && active_store(root).is_err() {
        return Ok(Some(RecoveryTarget::LibrarySelection(root.to_owned())));
    }
    Ok(None)
}
pub fn use_original(root: &Path, cancel: &Cancellation) -> Result<()> {
    ordinary(root, true)?;
    ordinary(&root.join("yamantaka.sqlite3"), false)?;
    if present(&root.join("preferences.json"))? {
        ordinary(&root.join("preferences.json"), false)?;
    }
    cancel.check()?;
    save_selection(root, None)
}
