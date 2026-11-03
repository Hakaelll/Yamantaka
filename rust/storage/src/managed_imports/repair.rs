//! Confirmed selection changes retain the old record and recover publication gaps.
use super::*;
use crate::backup::{inspect_store, original_selection_tree, store_tree};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
};
use yamantaka_windows::{retained_file::RetainedFile, source_stamp::Stamp};

const POINTER: &str = "selected-library.json";
const INTENT: &str = "selected-library-repair-v1.json";
#[derive(Clone)]
pub struct Review {
    root: PathBuf,
    source: PathBuf,
    import: Option<String>,
    version: yamantaka_domain::source_revision::PhysicalSourceVersion,
    digest: String,
    tree: String,
    counts: BTreeMap<String, usize>,
}
impl Review {
    pub fn source(&self) -> &Path {
        &self.source
    }
    pub fn counts(&self) -> &BTreeMap<String, usize> {
        &self.counts
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    IntentSaved,
    PointerPreserved,
    Installed,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Intent {
    version: u32,
    original_sha256: String,
    replacement_sha256: String,
    archive_name: String,
    replacement_name: String,
    import: Option<String>,
}
fn invalid() -> Failure {
    Failure::Invalid(
        "Library selection changed or needs separate recovery; all files were preserved".into(),
    )
}
pub fn pending(root: &Path) -> Result<bool> {
    present(&root.join(INTENT))
}
pub(super) fn read_record(path: &Path) -> Result<Vec<u8>> {
    ordinary(path, false)?;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(storage)?
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(storage)?;
    if bytes.len() > 65536 {
        return Err(invalid());
    }
    Ok(bytes)
}
fn digest(lease: &RetainedFile) -> Result<String> {
    let mut bytes = Vec::new();
    lease
        .file()
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(storage)?;
    if bytes.len() > 65536 {
        return Err(invalid());
    }
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn candidate(root: &Path, import: Option<&str>) -> Result<PathBuf> {
    match import {
        Some(id) => checked_store(root, id),
        None => {
            ordinary(root, true)?;
            Ok(root.to_owned())
        }
    }
}
fn selection_tree(root: &Path, source: &Path, cancel: &Cancellation) -> Result<String> {
    if source == root {
        original_selection_tree(source, cancel).map(|v| v.0)
    } else {
        store_tree(source, cancel).map(|v| v.0)
    }
}
pub fn review(root: &Path, source: Option<&Path>, cancel: &Cancellation) -> Result<Review> {
    cancel.check()?;
    ordinary(root, true)?;
    let root = root.canonicalize().map_err(storage)?;
    if pending(&root)? {
        return Err(invalid());
    }
    let import = source
        .map(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .map(str::to_owned)
                .ok_or_else(invalid)
        })
        .transpose()?;
    let chosen = candidate(&root, import.as_deref())?;
    if let Some(source) = source
        && source.canonicalize().map_err(storage)? != chosen
    {
        return Err(invalid());
    }
    let record = RetainedFile::open(&root.join(POINTER)).map_err(|_| invalid())?;
    let version = Stamp::from_file(record.file())
        .ok_or_else(invalid)?
        .version();
    let digest = digest(&record)?;
    let tree = selection_tree(&root, &chosen, cancel)?;
    let counts = inspect_store(&chosen, cancel)?;
    if selection_tree(&root, &chosen, cancel)? != tree {
        return Err(Failure::BackupSourceChanged);
    }
    Ok(Review {
        root,
        source: chosen,
        import,
        version,
        digest,
        tree,
        counts,
    })
}
pub fn verify(review: &Review, cancel: &Cancellation) -> Result<()> {
    cancel.check()?;
    if pending(&review.root)?
        || candidate(&review.root, review.import.as_deref())? != review.source
        || selection_tree(&review.root, &review.source, cancel)? != review.tree
    {
        return Err(invalid());
    }
    let record = RetainedFile::open(&review.root.join(POINTER)).map_err(|_| invalid())?;
    if Stamp::from_file(record.file()).map(|s| s.version()) != Some(review.version)
        || digest(&record)? != review.digest
    {
        return Err(invalid());
    }
    Ok(())
}
fn publish_record(root: &Path, name: &str, value: &impl Serialize) -> Result<()> {
    let mut file = tempfile::NamedTempFile::new_in(root).map_err(storage)?;
    serde_json::to_writer_pretty(&mut file, value).map_err(storage)?;
    file.write_all(b"\n").map_err(storage)?;
    file.as_file().sync_all().map_err(storage)?;
    file.persist_noclobber(root.join(name)).map_err(storage)?;
    Ok(())
}
pub fn apply(review: &Review, root: &Path, cancel: &Cancellation) -> Result<PathBuf> {
    apply_progress(review, root, cancel, |_| {})
}
pub fn apply_progress(
    review: &Review,
    root: &Path,
    cancel: &Cancellation,
    mut progress: impl FnMut(Progress),
) -> Result<PathBuf> {
    if root.canonicalize().map_err(storage)? != review.root {
        return Err(invalid());
    }
    verify(review, cancel)?;
    let record = RetainedFile::open(&review.root.join(POINTER)).map_err(|_| invalid())?;
    if Stamp::from_file(record.file()).map(|s| s.version()) != Some(review.version)
        || digest(&record)? != review.digest
    {
        return Err(invalid());
    }
    cancel.check()?;
    // Confirmation starts a durable, non-cancellable publication before any pointer rename.
    let replacement_name = format!(
        "yamantaka-selection-replacement-{}.json",
        uuid::Uuid::new_v4()
    );
    publish_record(
        &review.root,
        &replacement_name,
        &Selection {
            version: 1,
            import: review.import.clone(),
        },
    )?;
    let replacement_sha256 = crate::identity::full_sha256(
        &review.root.join(&replacement_name),
        &Cancellation::default(),
    )?;
    let intent = Intent {
        version: 1,
        original_sha256: review.digest.clone(),
        replacement_sha256,
        archive_name: format!(
            "yamantaka-selection-preserved-{}.json",
            uuid::Uuid::new_v4()
        ),
        replacement_name,
        import: review.import.clone(),
    };
    publish_record(&review.root, INTENT, &intent)?;
    progress(Progress::IntentSaved);
    drop(record);
    finish(&review.root, &intent, &mut progress)?;
    Ok(review.root.join(intent.archive_name))
}
fn valid_name(name: &str, prefix: &str) -> bool {
    name.strip_prefix(prefix)
        .and_then(|s| s.strip_suffix(".json"))
        .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok_and(|id| id.to_string() == s))
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn finish(root: &Path, intent: &Intent, progress: &mut impl FnMut(Progress)) -> Result<()> {
    let journal = RetainedFile::open(&root.join(INTENT)).map_err(|_| invalid())?;
    if serde_json::from_slice::<Intent>(&read_record(&root.join(INTENT))?).map_err(storage)?
        != *intent
    {
        return Err(invalid());
    }
    if intent.version != 1
        || !valid_hash(&intent.original_sha256)
        || !valid_hash(&intent.replacement_sha256)
        || !valid_name(&intent.archive_name, "yamantaka-selection-preserved-")
        || !valid_name(&intent.replacement_name, "yamantaka-selection-replacement-")
    {
        return Err(invalid());
    }
    candidate(root, intent.import.as_deref())?;
    let pointer = root.join(POINTER);
    let archive = root.join(&intent.archive_name);
    let replacement = root.join(&intent.replacement_name);
    let staged = if present(&replacement)? {
        let staged = RetainedFile::open(&replacement).map_err(|_| invalid())?;
        let selection: Selection =
            serde_json::from_slice(&read_record(&replacement)?).map_err(storage)?;
        if digest(&staged)? != intent.replacement_sha256
            || selection.version != 1
            || selection.import != intent.import
        {
            return Err(invalid());
        }
        Some(staged)
    } else {
        None
    };
    if !present(&archive)? {
        if staged.is_none() {
            return Err(invalid());
        }
        let record = RetainedFile::open(&pointer).map_err(|_| invalid())?;
        if digest(&record)? != intent.original_sha256 {
            return Err(invalid());
        }
        record
            .retain_as(intent.archive_name.as_ref())
            .map_err(|_| invalid())?;
        progress(Progress::PointerPreserved);
    }
    let archived = RetainedFile::open(&archive).map_err(|_| invalid())?;
    if digest(&archived)? != intent.original_sha256 {
        return Err(invalid());
    }
    if let Some(staged) = staged {
        if present(&pointer)? {
            return Err(invalid());
        }
        staged.retain_as(POINTER.as_ref()).map_err(|_| invalid())?;
        progress(Progress::Installed);
    }
    let active = RetainedFile::open(&pointer).map_err(|_| invalid())?;
    let selection: Selection = serde_json::from_slice(&read_record(&pointer)?).map_err(storage)?;
    if digest(&active)? != intent.replacement_sha256
        || selection.version != 1
        || selection.import != intent.import
    {
        return Err(invalid());
    }
    journal.remove().map_err(|_| invalid())?;
    Ok(())
}
pub fn recover(root: &Path) -> Result<bool> {
    if !pending(root)? {
        return Ok(false);
    }
    ordinary(root, true)?;
    let root = root.canonicalize().map_err(storage)?;
    let intent: Intent =
        serde_json::from_slice(&read_record(&root.join(INTENT))?).map_err(storage)?;
    finish(&root, &intent, &mut |_| {})?;
    Ok(true)
}
