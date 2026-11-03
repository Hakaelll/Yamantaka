use super::*;
use std::{collections::BTreeMap, fs};
use yamantaka_domain::Cancellation;
use yamantaka_storage::{backup, identity::full_sha256, managed_imports};

#[derive(Clone)]
pub(super) struct Verification {
    pub previous: Option<PathBuf>,
    pub books: usize,
    current: BTreeMap<String, String>,
    old_path: PathBuf,
    old: BTreeMap<String, String>,
    journal: PathBuf,
    bytes: Vec<u8>,
    install: bool,
}
fn files(root: &Path) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    fs::read_dir(root)?
        .map(|entry| {
            let entry = entry?;
            assert!(entry.file_type()?.is_file());
            Ok((
                entry.file_name().to_str().unwrap().to_owned(),
                full_sha256(&entry.path(), &Cancellation::default())?,
            ))
        })
        .collect()
}
impl Verification {
    pub(super) fn verify(
        &self,
        store: &Path,
        safety: Option<&Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.install {
            let preserved: Vec<_> = fs::read_dir(store.parent().unwrap())?
                .filter_map(Result::ok)
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with("yamantaka-recovery-preserved-")
                })
                .collect();
            assert_eq!(preserved.len(), 1);
            assert_eq!(fs::read(preserved[0].path())?, self.bytes);
            assert!(!self.journal.exists());
            if self.previous.is_some() {
                assert_eq!(files(store)?, self.old);
                assert_eq!(files(safety.unwrap())?, self.current);
            } else {
                assert_eq!(files(store)?, self.current);
                assert_eq!(files(&self.old_path)?, self.old);
            }
            assert_eq!(managed_imports::recover_and_select(store)?, store);
        } else {
            assert!(safety.is_none());
            assert_eq!(files(store)?, self.current);
            assert_eq!(files(&self.old_path)?, self.old);
            assert_eq!(fs::read(&self.journal)?, self.bytes);
        }
        Ok(())
    }
}
pub(super) fn run(args: &[OsString], capture: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mode = args
        .iter()
        .filter_map(|arg| arg.to_str())
        .find_map(|s| s.strip_prefix("--interrupted-recovery="))
        .unwrap();
    assert!(["current", "previous", "cancel"].contains(&mode));
    let root = crate::qa_ui_root()
        .join("../../build/demo-store/captures")
        .join(format!("pending-ñ-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root)?;
    let root = root.canonicalize()?;
    let store = root.join("active");
    let old = root.join(format!("pre-restore-{}", uuid::Uuid::new_v4()));
    for path in [&store, &old] {
        fs::create_dir(path)?;
        let preview = backup::preview_payload(
            serde_json::from_str(include_str!(
                "../../../../storage/tests/fixtures/populated-backup-v9.json"
            ))?,
            &Cancellation::default(),
        )?;
        preview.database.snapshot(&path.join("yamantaka.sqlite3"))?;
        preview.preferences.save(&path.join("preferences.json"))?;
    }
    fs::write(store.join("current-sentinel"), b"keep current resources")?;
    fs::write(old.join("previous-sentinel"), b"keep previous resources")?;
    // Copy a real uncheckpointed WAL into the isolated active store.
    let wal_source = root.join("wal-source.sqlite3");
    fs::copy(store.join("yamantaka.sqlite3"), &wal_source)?;
    let db = Database::open(&wal_source)?;
    db.execute_batch("PRAGMA wal_autocheckpoint=0; INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('wal-only','synthetic-wal.pdf','synthetic-wal.pdf','synthetic-wal.pdf','WAL only title',2,900,'2026-11-03','2026-11-03','2026-11-03','wal-synthetic')")?;
    fs::copy(&wal_source, store.join("yamantaka.sqlite3"))?;
    fs::copy(
        root.join("wal-source.sqlite3-wal"),
        store.join("yamantaka.sqlite3-wal"),
    )?;
    drop(db);
    let journal = root.join(".active.restore-journal.json");
    let bytes = serde_json::to_vec(
        &serde_json::json!({"version":1,"active":store.canonicalize()?,"safety":old.canonicalize()?,"replacement_sha256":"unconfirmed synthetic restore"}),
    )?;
    fs::write(&journal, &bytes)?;
    let current = files(&store)?;
    let previous = files(&old)?;
    let blocked = managed_imports::recover_and_select(&store).unwrap_err();
    assert!(blocked.to_string().contains("requires explicit recovery"));
    assert!(
        matches!(managed_imports::recovery_target(&store)?,Some(managed_imports::RecoveryTarget::PendingOperation(path)) if path==store)
    );
    let token = Cancellation::default();
    let wal = store.join("yamantaka.sqlite3-wal");
    let wal_bytes = fs::read(&wal)?;
    let mut damaged = wal_bytes.clone();
    *damaged.last_mut().unwrap() ^= 1;
    fs::write(&wal, damaged)?;
    assert!(backup::pending::review(&store, None, &token).is_err());
    fs::write(&wal, &wal_bytes)?;
    let reviewed = backup::pending::review(&store, None, &token)?;
    assert_eq!(reviewed.counts()["documents"], 3);
    fs::write(&journal, b"changed synthetic operation")?;
    assert!(backup::pending::resolve(&reviewed, &store, &token).is_err());
    assert_eq!(fs::read(&journal)?, b"changed synthetic operation");
    fs::write(&journal, &bytes)?;
    let reviewed = backup::pending::review(&store, None, &token)?;
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(backup::pending::resolve(&reviewed, &store, &cancelled).is_err());
    fs::write(store.join("late-change"), b"late change")?;
    assert!(backup::pending::verify(&reviewed, &token).is_err());
    fs::remove_file(store.join("late-change"))?;
    let other = root.join(".active.rollback-journal.json");
    fs::write(&other, b"second synthetic operation")?;
    assert!(backup::pending::review(&store, None, &token).is_err());
    fs::remove_file(other)?;
    assert_eq!(files(&store)?, current);
    assert_eq!(files(&old)?, previous);
    let pending = Verification {
        previous: if mode == "previous" {
            Some(old.clone())
        } else {
            None
        },
        books: if mode == "previous" { 2 } else { 3 },
        current,
        old_path: old,
        old: previous,
        journal,
        bytes,
        install: mode != "cancel",
    };
    let verification = super::Verification {
        selection: None,
        capture,
        backup: PathBuf::new(),
        original: Vec::new(),
        file: "",
        mode: if mode == "cancel" {
            "review".into()
        } else {
            "install".into()
        },
        requested: false,
        captured: false,
        scale: if args.iter().any(|s| s == "--text-scale=2") {
            2.0
        } else {
            1.0
        },
        pending: Some(pending),
    };
    super::super::run(
        store,
        FailedStage::InterruptedOperation,
        !args.iter().any(|arg| arg == "--es"),
        args.iter().any(|arg| arg == "--software"),
        Some(verification),
    )
    .map(|_| ())
}
