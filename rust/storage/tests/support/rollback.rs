use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use yamantaka_domain::{Cancellation, Failure, LibraryQuery, LibraryRepository};
use yamantaka_storage::{
    Database,
    backup::{self, RestoreProgress},
    preferences::Preferences,
};
const NAME: &str = "pre-restore-00000000-0000-4000-8000-000000000001";
const ROOT: &str = "YAMANTAKA_ROLLBACK_VERIFICATION_ROOT";
const PHASE: &str = "YAMANTAKA_ROLLBACK_VERIFICATION_PHASE";
fn seed(path: &Path, title: &str) {
    fs::create_dir_all(path.join("future-data/empty")).unwrap();
    let db = Database::open(&path.join("yamantaka.sqlite3")).unwrap();
    db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('book','synthetic.pdf','synthetic.pdf','synthetic.pdf','{title}',2,1,'now','now','now','fixture');")).unwrap();
    drop(db);
    Preferences::default()
        .save(&path.join("preferences.json"))
        .unwrap();
    fs::write(path.join("future-data/preserved.bin"), title.as_bytes()).unwrap();
    fs::write(
        path.join("reader-continuity-v6.json"),
        format!("synthetic continuity for {title}"),
    )
    .unwrap();
}
fn book(path: &Path) -> String {
    let temp = tempfile::tempdir().unwrap();
    let copy = temp.path().join("read.sqlite3");
    fs::copy(path.join("yamantaka.sqlite3"), &copy).unwrap();
    Database::open_read_only(&copy)
        .unwrap()
        .query(&LibraryQuery::default())
        .unwrap()[0]
        .display_title()
        .into()
}

pub fn child() -> bool {
    let Some(root) = std::env::var_os(ROOT) else {
        return false;
    };
    let root = PathBuf::from(root);
    let phase = std::env::var(PHASE).unwrap();
    let review = backup::review_rollback(
        &root.join("active"),
        &root.join(NAME),
        &Cancellation::default(),
    )
    .unwrap();
    backup::install_rollback_progress(
        &review,
        &root.join("active"),
        &Cancellation::default(),
        |p| {
            if phase == "moved" && matches!(p, RestoreProgress::OriginalSaved)
                || phase == "installed" && matches!(p, RestoreProgress::Installed)
            {
                fs::write(root.join("ready"), &phase).unwrap();
                loop {
                    std::thread::park();
                }
            }
        },
    )
    .unwrap();
    panic!("Rollback did not reach its interruption boundary");
}
pub fn verify(parent: &Path) {
    let root = parent.join("whole-store-rollback");
    fs::create_dir(&root).unwrap();
    let active = root.join("active");
    let source = root.join(NAME);
    seed(&active, "Current");
    seed(&source, "Previous");
    let cancel = Cancellation::default();
    let review = backup::review_rollback(&active, &source, &cancel).unwrap();
    assert_eq!(review.counts()["documents"], 1);
    assert_eq!(review.sha256().len(), 64);
    assert!(review.bytes() > 0);
    assert_eq!(book(&active), "Current");
    assert!(backup::review_rollback(&active, &active, &cancel).is_err());
    assert!(backup::review_rollback(&active, parent, &cancel).is_err());
    fs::write(source.join("future-data/added.bin"), b"newer file").unwrap();
    assert!(matches!(
        backup::verify_rollback(&review, &cancel),
        Err(Failure::BackupSourceChanged)
    ));
    assert!(matches!(
        backup::install_rollback(&review, &active, &cancel),
        Err(Failure::BackupSourceChanged)
    ));
    assert_eq!(book(&active), "Current");
    fs::remove_file(source.join("future-data/added.bin")).unwrap();
    let review = backup::review_rollback(&active, &source, &cancel).unwrap();
    let cancelled = Cancellation::default();
    let result = backup::install_rollback_progress(&review, &active, &cancelled, |p| {
        if matches!(p, RestoreProgress::Installing) {
            cancelled.cancel();
        }
    });
    assert!(matches!(result, Err(Failure::Cancelled)));
    assert_eq!(book(&active), "Current");
    assert_eq!(book(&source), "Previous");
    let safety = backup::install_rollback(&review, &active, &cancel).unwrap();
    assert!(!source.exists());
    assert_eq!(book(&active), "Previous");
    assert_eq!(book(&safety), "Current");
    assert_eq!(
        fs::read(active.join("future-data/preserved.bin")).unwrap(),
        b"Previous"
    );
    assert_eq!(
        fs::read(safety.join("future-data/preserved.bin")).unwrap(),
        b"Current"
    );
    assert!(active.join("future-data/empty").is_dir());
    assert_eq!(
        fs::read_to_string(active.join("reader-continuity-v6.json")).unwrap(),
        "synthetic continuity for Previous"
    );
    let back = backup::review_rollback(&active, &safety, &cancel).unwrap();
    let previous = backup::install_rollback(&back, &active, &cancel).unwrap();
    assert_eq!(book(&active), "Current");
    assert_eq!(book(&previous), "Previous");
    fs::write(
        previous.join("yamantaka.sqlite3-wal"),
        b"pending WAL must not be ignored",
    )
    .unwrap();
    assert!(backup::review_rollback(&active, &previous, &cancel).is_err());
    fs::remove_file(previous.join("yamantaka.sqlite3-wal")).unwrap();
    for phase in ["moved", "installed"] {
        let root = root.join(phase);
        fs::create_dir(&root).unwrap();
        seed(&root.join("active"), "Current");
        seed(&root.join(NAME), "Previous");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "replacement_keeps_complete_safety_store_and_cancel_does_not_write",
                "--nocapture",
            ])
            .env(ROOT, &root)
            .env(PHASE, phase)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(15);
        while !root.join("ready").exists() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "Rollback child exited before its boundary"
            );
            if Instant::now() > deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("Rollback child did not reach its boundary");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(backup::recover_interrupted_restore(&root.join("active")).unwrap());
        assert!(!backup::recover_interrupted_restore(&root.join("active")).unwrap());
        assert_eq!(
            book(&root.join("active")),
            if phase == "moved" {
                "Current"
            } else {
                "Previous"
            }
        );
        let stores = fs::read_dir(&root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .collect::<Vec<_>>();
        assert_eq!(stores.len(), 2);
        let mut titles = stores.iter().map(|p| book(p)).collect::<Vec<_>>();
        titles.sort();
        assert_eq!(titles, vec!["Current", "Previous"]);
    }
}
