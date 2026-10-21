use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use yamantaka_domain::{Cancellation, Failure, LibraryQuery, LibraryRepository};
use yamantaka_storage::{
    Database,
    backup::{self, RestoreProgress},
    managed_imports,
    preferences::Preferences,
};

const CHILD_ROOT: &str = "YAMANTAKA_RESTORE_VERIFICATION_ROOT";
const CHILD_PHASE: &str = "YAMANTAKA_RESTORE_VERIFICATION_PHASE";
pub fn child() -> bool {
    let Some(root) = std::env::var_os(CHILD_ROOT) else {
        return false;
    };
    let root = Path::new(&root);
    let phase = std::env::var(CHILD_PHASE).unwrap();
    let plan = backup::review(&root.join("backup.json"), &Cancellation::default()).unwrap();
    let preview = backup::prepare_reviewed(&plan, &Cancellation::default()).unwrap();
    backup::install_preview_progress(
        preview,
        &root.join("active"),
        &Cancellation::default(),
        |progress| {
            if phase == "moved" && matches!(progress, RestoreProgress::OriginalSaved)
                || phase == "installed" && matches!(progress, RestoreProgress::Installed)
            {
                fs::write(root.join("ready"), &phase).unwrap();
                loop {
                    std::thread::park();
                }
            }
        },
    )
    .unwrap();
    panic!("The restore did not reach its requested interruption boundary");
}
fn hash(path: &Path) -> String {
    format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
}
fn fixture(path: &Path) {
    fs::write(path, include_str!("../fixtures/populated-backup-v9.json")).unwrap();
}
pub fn verify(parent: &Path) {
    let root = parent.join("reviewed-restore");
    fs::create_dir(&root).unwrap();
    let source = root.join("backup.json");
    fixture(&source);
    let original = fs::read(&source).unwrap();
    let plan = backup::review(&source, &Cancellation::default()).unwrap();
    assert_eq!(plan.version(), 9);
    assert_eq!(plan.counts()["documents"], 2);
    assert_eq!(plan.sha256(), hash(&source));
    assert_eq!(plan.source(), source.canonicalize().unwrap());
    fs::write(&source, b"changed and no longer JSON").unwrap();
    assert!(matches!(
        backup::prepare_reviewed(&plan, &Cancellation::default()),
        Err(Failure::BackupSourceChanged)
    ));
    assert!(matches!(
        backup::install_reviewed(&plan, &root.join("untouched"), &Cancellation::default()),
        Err(Failure::BackupSourceChanged)
    ));
    assert!(!root.join("untouched").exists());
    fs::write(&source, &original).unwrap();
    for phase in ["copy", "validate", "install"] {
        let active = root.join(phase);
        fs::create_dir(&active).unwrap();
        fs::write(active.join("sentinel"), b"original").unwrap();
        let cancel = Cancellation::default();
        let result = backup::install_preview_progress(
            backup::prepare_reviewed(&plan, &cancel).unwrap(),
            &active,
            &cancel,
            |progress| {
                if phase == "copy" && matches!(progress, RestoreProgress::Copying { .. })
                    || phase == "validate" && matches!(progress, RestoreProgress::Validating)
                    || phase == "install" && matches!(progress, RestoreProgress::Installing)
                {
                    cancel.cancel();
                }
            },
        );
        assert!(matches!(result, Err(Failure::Cancelled)));
        assert_eq!(fs::read(active.join("sentinel")).unwrap(), b"original");
        assert!(!root.join(format!(".{phase}.restore-journal.json")).exists());
        assert!(!fs::read_dir(&root).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".restore-")
        }));
    }
    let active = root.join("installed");
    fs::create_dir(&active).unwrap();
    fs::write(active.join("sentinel"), b"old").unwrap();
    let safety = backup::install_reviewed(&plan, &active, &Cancellation::default()).unwrap();
    assert_eq!(fs::read(safety.join("sentinel")).unwrap(), b"old");
    let receipt_hash = hash(&active.join("restore-receipt.json"));
    let database_hash = hash(&active.join("yamantaka.sqlite3"));
    let preferences_hash = hash(&active.join("preferences.json"));
    let journal = root.join(".installed.restore-journal.json");
    let mut record =
        json!({"version":1,"active":active,"safety":safety,"replacement_sha256":database_hash});
    fs::write(&journal, serde_json::to_vec(&record).unwrap()).unwrap();
    assert!(backup::recover_interrupted_restore(&active).is_err());
    assert!(journal.exists());
    record["version"] = json!(2);
    record["replacement_preferences_sha256"] = json!(preferences_hash);
    record["replacement_receipt_sha256"] = json!(receipt_hash);
    fs::write(&journal, serde_json::to_vec(&record).unwrap()).unwrap();
    let saved_preferences = fs::read(active.join("preferences.json")).unwrap();
    fs::write(active.join("preferences.json"), b"tampered").unwrap();
    assert!(backup::recover_interrupted_restore(&active).is_err());
    assert!(journal.exists());
    fs::write(active.join("preferences.json"), saved_preferences).unwrap();
    let saved_receipt = fs::read(active.join("restore-receipt.json")).unwrap();
    fs::write(active.join("restore-receipt.json"), b"changed receipt").unwrap();
    assert!(backup::recover_interrupted_restore(&active).is_err());
    assert!(journal.exists());
    fs::write(active.join("restore-receipt.json"), saved_receipt).unwrap();
    fs::write(active.join("yamantaka.sqlite3-wal"), b"unverified WAL").unwrap();
    assert!(backup::recover_interrupted_restore(&active).is_err());
    assert!(journal.exists());
    fs::remove_file(active.join("yamantaka.sqlite3-wal")).unwrap();
    assert!(backup::recover_interrupted_restore(&active).unwrap());
    assert!(!fs::read_dir(&root).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".restore-")
    }));
    assert!(!journal.exists());
    let partial = root.join(".restore-partial-cleanup");
    fs::create_dir(&partial).unwrap();
    for name in ["preferences.json", "restore-receipt.json"] {
        fs::copy(active.join(name), partial.join(name)).unwrap();
    }
    record["staging"] = json!(partial);
    fs::write(&journal, serde_json::to_vec(&record).unwrap()).unwrap();
    assert!(backup::recover_interrupted_restore(&active).unwrap());
    assert!(!partial.exists());
    assert!(!journal.exists());
    verify_managed(&root, &plan);
    for phase in ["moved", "installed"] {
        verify_killed(&root, phase);
    }
}
fn verify_managed(parent: &Path, plan: &backup::RestoreReview) {
    let root = parent.join("managed");
    let id = "00000000-0000-4000-8000-000000000001";
    let active = root.join("imports").join(id);
    fs::create_dir_all(&active).unwrap();
    let safety = backup::install_reviewed(plan, &active, &Cancellation::default()).unwrap();
    assert!(safety.is_dir());
    fs::write(
        root.join("selected-library.json"),
        serde_json::to_vec(&json!({"version":1,"import":id})).unwrap(),
    )
    .unwrap();
    assert_eq!(managed_imports::recover_and_select(&root).unwrap(), active);
    let moved = active.with_file_name("pre-restore-interrupted");
    fs::rename(&active, &moved).unwrap();
    let journal = active.with_file_name(format!(".{id}.restore-journal.json"));
    fs::write(&journal,serde_json::to_vec(&json!({"version":1,"active":active,"safety":moved,"replacement_sha256":"not-installed"})).unwrap()).unwrap();
    assert!(managed_imports::active_store(&root).is_err());
    assert_eq!(managed_imports::recover_and_select(&root).unwrap(), active);
}
fn verify_killed(parent: &Path, phase: &str) {
    let root = parent.join(format!("process-{phase}"));
    let active = root.join("active");
    fs::create_dir_all(&active).unwrap();
    {
        let database = Database::open(&active.join("yamantaka.sqlite3")).unwrap();
        assert_eq!(database.count(&LibraryQuery::default()).unwrap(), 0);
    }
    Preferences::default()
        .save(&active.join("preferences.json"))
        .unwrap();
    let original_db = hash(&active.join("yamantaka.sqlite3"));
    let original_prefs = hash(&active.join("preferences.json"));
    fixture(&root.join("backup.json"));
    let mut process = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "replacement_keeps_complete_safety_store_and_cancel_does_not_write",
            "--nocapture",
        ])
        .env(CHILD_ROOT, &root)
        .env(CHILD_PHASE, phase)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while fs::read_to_string(root.join("ready")).ok().as_deref() != Some(phase) {
        if process.try_wait().unwrap().is_some() || start.elapsed() > Duration::from_secs(15) {
            let _ = process.kill();
            let _ = process.wait();
            panic!("Restore child did not reach {phase}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    process.kill().unwrap();
    process.wait().unwrap();
    let record: Value =
        serde_json::from_slice(&fs::read(root.join(".active.restore-journal.json")).unwrap())
            .unwrap();
    assert!(backup::recover_interrupted_restore(&active).unwrap());
    assert!(!fs::read_dir(&root).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".restore-")
    }));
    if phase == "moved" {
        assert_eq!(hash(&active.join("yamantaka.sqlite3")), original_db);
        assert_eq!(hash(&active.join("preferences.json")), original_prefs);
    } else {
        let safety = Path::new(record["safety"].as_str().unwrap());
        assert_eq!(hash(&safety.join("yamantaka.sqlite3")), original_db);
        assert_eq!(hash(&safety.join("preferences.json")), original_prefs);
        let database = Database::open_read_only(&active.join("yamantaka.sqlite3")).unwrap();
        assert_eq!(database.count(&LibraryQuery::default()).unwrap(), 2);
    }
}
