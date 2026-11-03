use super::*;
use std::time::Instant;
use yamantaka_application::background_password::Challenge;
use yamantaka_domain::LibraryQuery;

pub(super) fn challenge(runtime: &Runtime) -> Challenge {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(challenge) = runtime.next_background_password() {
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
fn metadata_evidence(database: &std::path::Path, source: &std::path::Path) -> serde_json::Value {
    let connection =
        rusqlite::Connection::open_with_flags(database, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    let text:String=connection.query_row("SELECT a.value FROM app_metadata a JOIN documents d ON a.key='rust.frontMatterEvidence.v1:'||d.id WHERE d.path=?",[source.to_string_lossy().as_ref()],|row|row.get(0)).unwrap();
    serde_json::from_str(&text).unwrap()
}
pub(super) fn verify_protected_import() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root
        .join("build/rust-fixtures/protected.pdf")
        .canonicalize()
        .unwrap();
    let original = std::fs::read(&source).unwrap();
    let mut uninterrupted_evidence = None;
    for mode in ["unlock", "drop", "cancel", "shutdown"] {
        let dir = tempfile::tempdir().unwrap();
        let dbpath = dir.path().join("library.sqlite3");
        let database = Database::open(&dbpath).unwrap();
        let cache = Arc::new(yamantaka_storage::cover_cache::CoverCache::new(
            dir.path().join("covers"),
            16 * 1024 * 1024,
        ));
        let worker = PdfWorker::start_with_cover_cache(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
            yamantaka_domain::memory::MemoryGovernor::new(Default::default()),
            Some(cache.clone()),
        )
        .unwrap();
        let runtime = Runtime::start(database, worker, || {}).unwrap();
        let cancel = Cancellation::default();
        runtime
            .submit(Action::Import(
                vec![
                    if mode == "unlock" {
                        root.join("build/rust-fixtures/protected.pdf")
                    } else {
                        source.clone()
                    },
                    root.join("build/rust-fixtures/textual.pdf"),
                ],
                cancel.clone(),
            ))
            .unwrap();
        let first = challenge(&runtime);
        assert_eq!(first.path().canonicalize().unwrap(), source);
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
                uninterrupted_evidence = Some(metadata_evidence(&dbpath, &source));
                verify_cover_cache(&db, &source, &root, cache.clone());
                super::cover_rebuild::verify(&dbpath, &source, &root, cache.clone());
                super::protected_index::verify(&dbpath, &source, &root);
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
    for text_phase in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let dbpath = dir.path().join("resume.sqlite3");
        let worker = PdfWorker::start(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
        )
        .unwrap();
        let control = worker.clone();
        let (source_send, source_receive) = bounded(1);
        let (text_send, text_receive) = bounded(1);
        let pages = Arc::new(Mutex::new(Vec::new()));
        let (opens, _opened) = crossbeam_channel::unbounded();
        let runtime = Runtime::start(
            Database::open(&dbpath).unwrap(),
            Engine {
                worker,
                hold: Arc::new(Mutex::new(if text_phase { Some(text_send) } else { None })),
                source_hold: Mutex::new(if text_phase { None } else { Some(source_send) }),
                opens,
                pages: pages.clone(),
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
        if text_phase {
            let held = text_receive.recv_timeout(Duration::from_secs(10)).unwrap();
            assert_eq!(held.page, 1);
            control.close(held.id).unwrap();
            held.reply
                .send(Err(Failure::NativeDocumentUnavailable))
                .unwrap();
        } else {
            let held = source_receive
                .recv_timeout(Duration::from_secs(10))
                .unwrap();
            control.close(held.id).unwrap();
            held.reply
                .send(Err(Failure::NativeDocumentUnavailable))
                .unwrap();
        }
        let again = challenge(&runtime);
        assert!(
            !again.rejected(),
            "reopening must not report a previously correct password as rejected"
        );
        if text_phase {
            again
                .answer("synthetic incorrect credential".into())
                .unwrap();
            let rejected = challenge(&runtime);
            assert!(rejected.rejected());
            rejected.answer("fixture-password".into()).unwrap();
        } else {
            again.answer("fixture-password".into()).unwrap();
        }
        terminal(&runtime, true);
        drop(runtime);
        drop(control);
        let db = Database::open_read_only(&dbpath).unwrap();
        assert_eq!(db.count(&LibraryQuery::default()).unwrap(), 2);
        assert!(db.pending_imports().unwrap().is_empty());
        assert_eq!(
            metadata_evidence(&dbpath, &source),
            *uninterrupted_evidence.as_ref().unwrap(),
            "reopening lost previously collected bibliographic evidence"
        );
        if text_phase {
            assert_eq!(
                *pages.lock().unwrap(),
                [0, 1, 1, 0, 1],
                "reopening must resume the interrupted metadata page"
            );
        }
    }
    assert_eq!(std::fs::read(source).unwrap(), original);
}

fn verify_cover_cache(
    db: &Database,
    source: &std::path::Path,
    root: &std::path::Path,
    cache: Arc<yamantaka_storage::cover_cache::CoverCache>,
) {
    let document = db
        .query(&LibraryQuery::default())
        .unwrap()
        .into_iter()
        .find(|doc| doc.path == source)
        .unwrap();
    assert_eq!(db.cover_source_page(&document.id).unwrap(), Some(0));
    let fresh = PdfWorker::start_with_cover_cache(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
        yamantaka_domain::memory::MemoryGovernor::new(Default::default()),
        Some(cache),
    )
    .unwrap();
    assert!(matches!(
        fresh.open(
            "locked-probe".into(),
            source.into(),
            None,
            Cancellation::default()
        ),
        Err(Failure::PasswordRequired)
    ));
    let image = fresh
        .try_cover(
            document.id.clone(),
            source.into(),
            0,
            19,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(image.key.document_id, document.id);
    assert_eq!(image.key.generation, 19);
    let plain = fresh
        .try_cover(
            "plain-reference".into(),
            root.join("build/rust-fixtures/textual.pdf"),
            0,
            19,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!([image.width, image.height], [plain.width, plain.height]);
    assert_eq!(
        image.pixels(),
        plain.pixels(),
        "cached encrypted cover differs from the original page"
    );
    assert!(
        image
            .pixels()
            .chunks_exact(4)
            .any(|pixel| pixel[0] < 200 && pixel[1] < 200 && pixel[2] < 200)
    );
    fresh
        .open(
            "guarded-cover".into(),
            source.into(),
            Some("fixture-password".into()),
            Cancellation::default(),
        )
        .unwrap();
    let expected = fresh
        .try_source_revision("guarded-cover".into(), Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    let mut changed = expected.clone();
    changed.head_sha256[0] ^= 1;
    assert!(matches!(
        fresh
            .try_capture_import_cover("guarded-cover".into(), changed, Cancellation::default())
            .unwrap()
            .wait(),
        Err(Failure::PdfSourceChanged)
    ));
    fresh.close("guarded-cover".into()).unwrap();
    assert!(matches!(
        fresh
            .try_capture_import_cover("guarded-cover".into(), expected, Cancellation::default())
            .unwrap()
            .wait(),
        Err(Failure::NativeDocumentUnavailable)
    ));
}
