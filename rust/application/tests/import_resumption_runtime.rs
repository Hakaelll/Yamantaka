#[path = "../../storage/tests/support/import.rs"]
mod verified_import;
use crossbeam_channel::bounded;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use yamantaka_application::runtime::{Action, Runtime, Update};
use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository, PdfEngine, pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;
#[path = "import_resumption_runtime/engine.rs"]
mod engine;
use engine::{Engine, open_reader, requested, visible, wait};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operation {
    Import,
    Reanalyze,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Resume,
    Cancel,
    SourceChanged,
    PreemptAgain,
}

#[test]
fn native_reader_preemption_resumes_the_interrupted_page_and_keeps_prior_evidence() {
    run(Operation::Import, Outcome::Resume);
}
#[test]
fn cancelled_import_reopen_finishes_without_unpinning_either_reader() {
    run(Operation::Import, Outcome::Cancel);
}
#[test]
fn an_import_source_cannot_be_replaced_while_analysis_waits() {
    run(Operation::Import, Outcome::SourceChanged);
}
#[test]
fn two_reader_preemptions_repeat_only_the_interrupted_page() {
    run(Operation::Import, Outcome::PreemptAgain);
}
#[test]
fn resumed_reanalysis_preserves_manual_values_and_refreshes_page_zero_evidence() {
    run(Operation::Reanalyze, Outcome::Resume);
}
#[test]
fn cancelled_reanalysis_reopen_keeps_the_complete_previous_review() {
    run(Operation::Reanalyze, Outcome::Cancel);
}
#[test]
fn changed_source_during_reanalysis_reopen_keeps_the_previous_review() {
    run(Operation::Reanalyze, Outcome::SourceChanged);
}

fn run(operation: Operation, outcome: Outcome) {
    use yamantaka_domain::{NewDocument, front_matter::ImportAnalysis};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = tempfile::tempdir().unwrap();
    let dbpath = dir.path().join("library.sqlite3");
    let mut database = Database::open(&dbpath).unwrap();
    let fixtures = ["front-matter-missing", "textual", "front-matter-conflict"].map(|name| {
        root.join(format!("build/rust-fixtures/{name}.pdf"))
            .canonicalize()
            .unwrap()
    });
    let originals = fixtures
        .iter()
        .map(|path| std::fs::read(path).unwrap())
        .collect::<Vec<_>>();
    let target = dir.path().join("resume-target.pdf");
    std::fs::write(&target, &originals[0]).unwrap();
    let target = target.canonicalize().unwrap();
    for (id, path) in [("reader-a", &fixtures[1]), ("reader-b", &fixtures[2])] {
        let escaped = path.to_string_lossy().replace('\'', "''");
        database.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('{id}','{escaped}','{id}','fixture.pdf',2,1,'now','now','now','synthetic')")).unwrap();
    }
    let reader_a = database.get("reader-a").unwrap().unwrap();
    let reader_b = database.get("reader-b").unwrap().unwrap();
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let governor = worker.memory_governor();
    let control = worker.clone();
    let expected = if operation == Operation::Reanalyze {
        let info = worker
            .open("seed".into(), target.clone(), None, Cancellation::default())
            .unwrap();
        let source = worker
            .try_source_revision("seed".into(), Cancellation::default())
            .unwrap()
            .wait()
            .unwrap();
        worker.close("seed".into()).unwrap();
        let id = verified_import::save(
            &mut database,
            &NewDocument {
                path: target.clone(),
                info,
                source,
                analysis: ImportAnalysis::NotRequested,
            },
        )
        .unwrap();
        database.execute_batch("UPDATE documents SET title_override='My geometry',author_override='My author' WHERE id NOT IN ('reader-a','reader-b')").unwrap();
        Some(
            database
                .metadata_review(&id, &Cancellation::default())
                .unwrap(),
        )
    } else {
        None
    };
    let (hold, held) = bounded(1);
    let hold_again = hold.clone();
    let hold = Arc::new(Mutex::new(Some(hold)));
    let (opens, opened) = crossbeam_channel::unbounded();
    let pages = Arc::new(Mutex::new(Vec::new()));
    let runtime = Runtime::start(
        database,
        Engine {
            worker,
            hold: hold.clone(),
            source_hold: Mutex::new(None),
            opens,
            pages: pages.clone(),
        },
        || {},
    )
    .unwrap();
    let cancel = Cancellation::default();
    runtime
        .submit(match &expected {
            Some(review) => Action::ReanalyzeMetadata(Box::new(review.clone()), cancel.clone()),
            None => Action::Import(vec![target.clone()], cancel.clone()),
        })
        .unwrap();
    let request = held.recv_timeout(Duration::from_secs(10)).unwrap();
    requested(&opened, &request.id);
    if operation == Operation::Import && outcome == Outcome::Resume {
        let removal = Cancellation::default();
        runtime
            .submit(Action::RemoveFromLibrary(Box::new(
                yamantaka_application::library_removal::RemovalRequest {
                    ids: vec!["reader-a".into()],
                    cancel: removal.clone(),
                },
            )))
            .unwrap();
        assert!(
            matches!(wait(&runtime),Update::SelectionOrganized(reply) if reply.cancel.same_request(&removal) && matches!(reply.result,Err(Failure::QueueFull)))
        );
        assert!(!cancel.is_cancelled());
    }
    open_reader(&runtime, reader_a);
    open_reader(&runtime, reader_b.clone());
    visible(&runtime, &["reader-a", "reader-b"]);
    let lost = control
        .try_text_geometry(
            request.id.clone(),
            request.page,
            request.purpose,
            request.cancel,
        )
        .unwrap()
        .wait();
    assert!(matches!(lost, Err(Failure::NativeDocumentUnavailable)));
    request.reply.send(lost).unwrap();
    requested(&opened, &request.id);
    assert!(
        matches!(
            runtime.updates.recv_timeout(Duration::from_millis(100)),
            Err(crossbeam_channel::RecvTimeoutError::Timeout)
        ),
        "Analysis must remain pending while its source cannot be reopened"
    );
    assert_eq!(runtime.import_revision(), 0);
    runtime
        .submit(Action::Query(Default::default(), Cancellation::default()))
        .unwrap();
    let count = if expected.is_some() { 3 } else { 2 };
    assert!(
        matches!(wait(&runtime),Update::Library(documents,total,_) if total==count && documents.len()==count as usize)
    );
    let mut target_bytes = originals[0].clone();
    if outcome == Outcome::SourceChanged {
        let mut changed = target_bytes.clone();
        changed.extend_from_slice(b"\n% Synthetic replacement while analysis waits\n");
        let result = std::fs::write(&target, &changed);
        if operation == Operation::Import {
            assert_eq!(result.unwrap_err().raw_os_error(), Some(32));
        } else {
            result.unwrap();
            target_bytes = changed;
        }
    }
    if outcome == Outcome::Cancel {
        cancel.cancel();
        runtime.wake();
    } else {
        if outcome == Outcome::PreemptAgain {
            *hold.lock().unwrap() = Some(hold_again);
        }
        visible(&runtime, &["reader-a"]);
        if outcome == Outcome::PreemptAgain {
            let repeated = held.recv_timeout(Duration::from_secs(10)).unwrap();
            assert_eq!(
                (repeated.id.as_str(), repeated.page),
                (request.id.as_str(), 1)
            );
            open_reader(&runtime, reader_b);
            visible(&runtime, &["reader-a", "reader-b"]);
            let lost = control
                .try_text_geometry(
                    repeated.id.clone(),
                    repeated.page,
                    repeated.purpose,
                    repeated.cancel,
                )
                .unwrap()
                .wait();
            assert!(matches!(lost, Err(Failure::NativeDocumentUnavailable)));
            repeated.reply.send(lost).unwrap();
            requested(&opened, &request.id);
            visible(&runtime, &["reader-a"]);
        }
    }
    let result = wait(&runtime);
    let succeeds = matches!(outcome, Outcome::Resume | Outcome::PreemptAgain)
        || operation == Operation::Import && outcome == Outcome::SourceChanged;
    if let Some(expected) = &expected {
        let Update::Metadata(reply) = result else {
            panic!("Expected a scoped metadata result");
        };
        assert!(reply.cancel.same_request(&cancel) && reply.writing);
        if succeeds {
            let review = reply.result.unwrap();
            assert_eq!(
                (review.title.as_str(), review.author.as_str()),
                ("My geometry", "My author")
            );
            assert!(review.suggested_title.is_none() && review.suggested_author.is_none());
        } else {
            assert!(
                matches!(reply.result, Err(Failure::Cancelled)) && outcome == Outcome::Cancel
                    || matches!(reply.result, Err(Failure::Invalid(_)))
                        && outcome == Outcome::SourceChanged
            );
        }
        if !succeeds {
            assert_eq!(
                Database::open(&dbpath)
                    .unwrap()
                    .metadata_review(&expected.document, &Cancellation::default())
                    .unwrap(),
                *expected
            );
        }
    } else if succeeds {
        assert!(matches!(result,Update::Imported(1,token) if token.same_request(&cancel)));
    } else {
        assert!(
            matches!(&result,Update::Failed(Failure::Cancelled,Some(token)) if outcome==Outcome::Cancel&&token.same_request(&cancel))
                || matches!(&result,Update::Failed(Failure::Invalid(_),Some(token)) if outcome==Outcome::SourceChanged&&token.same_request(&cancel))
        );
    }
    let catalogue = Database::open(&dbpath).unwrap();
    let documents = catalogue.query(&Default::default()).unwrap();
    if succeeds {
        assert_eq!(
            *pages.lock().unwrap(),
            if outcome == Outcome::PreemptAgain {
                vec![0, 1, 1, 1]
            } else {
                vec![0, 1, 1]
            },
            "Only the interrupted page may be repeated"
        );
        let imported = documents.iter().find(|d| d.path == target).unwrap();
        assert_eq!(
            imported.author.as_deref(),
            Some(if expected.is_some() {
                "My author"
            } else {
                "Ada Lovelace"
            })
        );
        let connection = rusqlite::Connection::open(&dbpath).unwrap();
        let title: Option<String> = connection
            .query_row(
                "SELECT suggested_title FROM document_enrichment WHERE document_id=?",
                [&imported.id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(title.is_none());
        if expected.is_none() {
            assert_eq!(imported.display_title(), "Geometry of Light");
        }
        let reference = catalogue
            .file_snapshot(&imported.id)
            .unwrap()
            .digest
            .unwrap();
        let actual =
            yamantaka_storage::identity::full_sha256(&target, &Cancellation::default()).unwrap();
        assert_eq!(
            reference
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            actual
        );

        let evidence: String = connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key=?",
                [format!("rust.frontMatterEvidence.v1:{}", imported.id)],
                |r| r.get(0),
            )
            .unwrap();
        let evidence: serde_json::Value = serde_json::from_str(&evidence).unwrap();
        assert_eq!(evidence["title"]["candidate"]["page"], 0);
    } else if expected.is_none() {
        assert!(!documents.iter().any(|d| d.path == target));
    }
    if succeeds && operation == Operation::Import {
        let revision = runtime.import_revision();
        for _ in opened.try_iter() {}
        runtime
            .submit(Action::Import(
                vec![target.clone()],
                Cancellation::default(),
            ))
            .unwrap();
        assert!(matches!(wait(&runtime), Update::Imported(1, _)));
        assert_eq!(runtime.import_revision(), revision);
        assert!(
            matches!(
                opened.try_recv(),
                Err(crossbeam_channel::TryRecvError::Empty)
            ),
            "A registered path must not reopen PDFium"
        );
    }
    runtime
        .submit(Action::Text("reader-a".into(), 0, Cancellation::default()))
        .unwrap();
    assert!(
        matches!(wait(&runtime),Update::Text(id,0,text,_) if id=="reader-a"&&text.contains("Yamantaka Rust"))
    );
    drop(runtime);
    drop(control);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(std::fs::read(&target).unwrap(), target_bytes);
    for (path, original) in fixtures.iter().zip(originals) {
        assert_eq!(std::fs::read(path).unwrap(), original);
    }
}

#[test]
fn the_import_read_guard_survives_losing_and_reloading_the_initial_native_owner() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = tempfile::tempdir().unwrap();
    let dbpath = dir.path().join("library.sqlite3");
    let database = Database::open(&dbpath).unwrap();
    let fixtures = [
        "front-matter-missing",
        "textual",
        "front-matter-conflict",
        "automatic-cover",
    ]
    .map(|name| {
        root.join(format!("build/rust-fixtures/{name}.pdf"))
            .canonicalize()
            .unwrap()
    });
    let originals = fixtures
        .iter()
        .map(|p| std::fs::read(p).unwrap())
        .collect::<Vec<_>>();
    let target = dir.path().join("replaced-before-source.pdf");
    std::fs::write(&target, &originals[0]).unwrap();
    let target = target.canonicalize().unwrap();
    for (id, path) in [("reader-a", &fixtures[1]), ("reader-b", &fixtures[2])] {
        let escaped = path.to_string_lossy().replace('\'', "''");
        database.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('{id}','{escaped}','{id}','fixture.pdf',2,1,'now','now','now','synthetic')")).unwrap();
    }
    let a = database.get("reader-a").unwrap().unwrap();
    let b = database.get("reader-b").unwrap().unwrap();
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let control = worker.clone();
    let governor = worker.memory_governor();
    let (source_hold, held) = bounded(1);
    let (opens, opened) = crossbeam_channel::unbounded();
    let pages = Arc::new(Mutex::new(Vec::new()));
    let runtime = Runtime::start(
        database,
        Engine {
            worker,
            hold: Arc::new(Mutex::new(None)),
            source_hold: Mutex::new(Some(source_hold)),
            opens,
            pages: pages.clone(),
        },
        || {},
    )
    .unwrap();
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Import(vec![target.clone()], cancel.clone()))
        .unwrap();
    let source = held.recv_timeout(Duration::from_secs(10)).unwrap();
    requested(&opened, &source.id);
    open_reader(&runtime, a);
    open_reader(&runtime, b);
    visible(&runtime, &["reader-a", "reader-b"]);
    let lost = control
        .try_source_revision(source.id.clone(), source.cancel)
        .unwrap()
        .wait();
    assert!(matches!(lost, Err(Failure::NativeDocumentUnavailable)));
    source.reply.send(lost).unwrap();
    requested(&opened, &source.id);
    assert_eq!(runtime.import_revision(), 0);
    // The application guard survives native eviction, even before its first snapshot.
    assert_eq!(
        std::fs::write(&target, &originals[3])
            .unwrap_err()
            .raw_os_error(),
        Some(32)
    );
    visible(&runtime, &["reader-a"]);
    assert!(matches!(wait(&runtime),Update::Imported(1,token) if token.same_request(&cancel)));
    let connection = rusqlite::Connection::open(&dbpath).unwrap();
    let (pages_count, size): (u32, i64) = connection
        .query_row(
            "SELECT page_count,file_size FROM documents WHERE id NOT IN ('reader-a','reader-b')",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(pages_count, 2);
    assert_eq!(u64::try_from(size).unwrap(), originals[0].len() as u64);
    assert_eq!(*pages.lock().unwrap(), vec![0, 1]);
    drop(runtime);
    drop(control);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(std::fs::read(&target).unwrap(), originals[0]);
    for (path, original) in fixtures.iter().zip(originals) {
        assert_eq!(std::fs::read(path).unwrap(), original);
    }
}
