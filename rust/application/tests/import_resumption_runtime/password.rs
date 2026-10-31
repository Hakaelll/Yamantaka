use super::*;
use std::time::Instant;
use yamantaka_application::import_password::Challenge;
use yamantaka_domain::LibraryQuery;

fn challenge(runtime: &Runtime) -> Challenge {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(challenge) = runtime.next_import_password() {
            return challenge;
        }
        assert!(
            Instant::now() < deadline,
            "protected import did not request a password"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn terminal(runtime: &Runtime, success: bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let update = runtime
            .updates
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap();
        match update {
            Update::Imported(count, _, _) => {
                assert!(success);
                assert_eq!(count, 2);
                return;
            }
            Update::Failed(error, _) => {
                assert!(!success);
                assert!(matches!(error, Failure::Cancelled));
                return;
            }
            _ => {}
        }
    }
}
pub(super) fn verify_protected_import() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root
        .join("build/rust-fixtures/protected.pdf")
        .canonicalize()
        .unwrap();
    let original = std::fs::read(&source).unwrap();
    for mode in ["unlock", "drop", "cancel", "shutdown"] {
        let dir = tempfile::tempdir().unwrap();
        let dbpath = dir.path().join("library.sqlite3");
        let database = Database::open(&dbpath).unwrap();
        let worker = PdfWorker::start(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
        )
        .unwrap();
        let runtime = Runtime::start(database, worker, || {}).unwrap();
        let cancel = Cancellation::default();
        runtime
            .submit(Action::Import(
                vec![source.clone(), root.join("build/rust-fixtures/textual.pdf")],
                cancel.clone(),
            ))
            .unwrap();
        let first = challenge(&runtime);
        assert_eq!(first.path(), source);
        assert!(!first.rejected());
        runtime
            .submit(Action::Query(
                LibraryQuery::default(),
                Cancellation::default(),
            ))
            .unwrap();
        loop {
            match runtime
                .updates
                .recv_timeout(Duration::from_secs(10))
                .unwrap()
            {
                Update::Library(_, count, ..) => {
                    assert_eq!(count, 0);
                    break;
                }
                Update::Failed(error, _) => panic!("pending password blocked catalogue: {error}"),
                _ => {}
            }
        }
        match mode {
            "unlock" => {
                first
                    .answer("synthetic incorrect credential".into())
                    .unwrap();
                let retry = challenge(&runtime);
                assert!(retry.rejected());
                retry.answer("fixture-password".into()).unwrap();
                terminal(&runtime, true);
                drop(runtime);
                let db = Database::open_read_only(&dbpath).unwrap();
                assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 2);
                assert!(db.pending_imports().unwrap().is_empty());
                let documents = db.query(&LibraryQuery::default()).unwrap();
                assert!(
                    documents
                        .iter()
                        .any(|d| d.path == source && d.page_count == 2)
                );
            }
            "drop" => {
                drop(first);
                terminal(&runtime, false);
                drop(runtime);
            }
            "cancel" => {
                cancel.cancel();
                assert!(first.answer("fixture-password".into()).is_err());
                terminal(&runtime, false);
                drop(runtime);
            }
            "shutdown" => {
                drop(runtime);
                assert!(first.cancelled());
                drop(first);
            }
            _ => unreachable!(),
        }
        let db = Database::open_read_only(&dbpath).unwrap();
        if mode != "unlock" {
            assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 0);
            assert_eq!(db.pending_imports().unwrap().len(), 1);
        }
        drop(db);
        for entry in std::fs::read_dir(dir.path()).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                let bytes = std::fs::read(entry.path()).unwrap();
                for secret in [
                    b"fixture-password".as_slice(),
                    b"synthetic incorrect credential".as_slice(),
                ] {
                    assert!(
                        !bytes.windows(secret.len()).any(|window| window == secret),
                        "credential persisted in an import artifact"
                    );
                }
            }
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let dbpath = dir.path().join("resume.sqlite3");
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let control = worker.clone();
    let (source_send, source_receive) = bounded(1);
    let (opens, _opened) = crossbeam_channel::unbounded();
    let runtime = Runtime::start(
        Database::open(&dbpath).unwrap(),
        Engine {
            worker,
            hold: Arc::new(Mutex::new(None)),
            source_hold: Mutex::new(Some(source_send)),
            opens,
            pages: Arc::new(Mutex::new(Vec::new())),
        },
        || {},
    )
    .unwrap();
    runtime
        .submit(Action::Import(
            vec![source.clone(), root.join("build/rust-fixtures/textual.pdf")],
            Cancellation::default(),
        ))
        .unwrap();
    challenge(&runtime)
        .answer("fixture-password".into())
        .unwrap();
    let held = source_receive
        .recv_timeout(Duration::from_secs(10))
        .unwrap();
    control.close(held.id).unwrap();
    held.reply
        .send(Err(Failure::NativeDocumentUnavailable))
        .unwrap();
    let again = challenge(&runtime);
    assert!(
        !again.rejected(),
        "reopening must not report a previously correct password as rejected"
    );
    again.answer("fixture-password".into()).unwrap();
    terminal(&runtime, true);
    drop(runtime);
    drop(control);
    let db = Database::open_read_only(&dbpath).unwrap();
    assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 2);
    assert!(db.pending_imports().unwrap().is_empty());
    assert_eq!(std::fs::read(source).unwrap(), original);
}
