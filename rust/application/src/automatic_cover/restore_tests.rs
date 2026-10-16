use super::tests::{Gate, receive, selection, setup};
use crate::cover_source::CoverSource;
use crate::{
    manual_cover::ManualCoverRequest,
    runtime::{Action, Runtime, Update},
};
use std::time::Duration;
use yamantaka_domain::{
    Cancellation, LibraryRepository, cover_selection::ManualCover, pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;

fn seed(db: &Database) {
    db.execute_batch("INSERT INTO document_covers VALUES('auto',0,'old-small','old-large','old','old',1,'manual',4,'contain',0.75);").unwrap();
}
#[test]
fn restoration_keeps_manual_metadata_until_detection_and_then_commits_the_automatic_page() {
    let root = tempfile::tempdir().unwrap();
    let dbpath = root.path().join("library.sqlite3");
    let (db, doc, pdf, dll) = setup(&dbpath);
    seed(&db);
    let original = std::fs::read(&pdf).unwrap();
    let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
    let detected = worker
        .try_detect_cover(doc.id.clone(), pdf.clone(), 10, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    let (ready, requests) = std::sync::mpsc::channel();
    let runtime = Runtime::start(
        db,
        Gate {
            native: worker,
            ready,
            render_ready: None,
        },
        || {},
    )
    .unwrap();
    runtime
        .submit(Action::RestoreAutomaticCover(
            CoverSource::capture(&doc),
            10,
            Cancellation::default(),
        ))
        .unwrap();
    let sender = requests.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(selection(&dbpath).unwrap(), (0, "manual".into(), 4, 1.0));
    sender.send(Ok(detected)).unwrap();
    assert!(matches!(receive(&runtime),Update::Cover(_,Ok(image),_) if image.key.page==1));
    let actual = selection(&dbpath).unwrap();
    assert_eq!((actual.0, actual.1.as_str(), actual.2), (1, "automatic", 3));
    drop(runtime);
    assert_eq!(std::fs::read(pdf).unwrap(), original);
}
#[test]
fn newer_manual_choice_defeats_pending_explicit_restoration() {
    let root = tempfile::tempdir().unwrap();
    let dbpath = root.path().join("library.sqlite3");
    let (db, doc, pdf, dll) = setup(&dbpath);
    seed(&db);
    let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
    let detected = worker
        .try_detect_cover(doc.id.clone(), pdf.clone(), 10, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    let (ready, requests) = std::sync::mpsc::channel();
    let runtime = Runtime::start(
        db,
        Gate {
            native: worker,
            ready,
            render_ready: None,
        },
        || {},
    )
    .unwrap();
    runtime
        .submit(Action::RestoreAutomaticCover(
            CoverSource::capture(&doc),
            10,
            Cancellation::default(),
        ))
        .unwrap();
    let sender = requests.recv_timeout(Duration::from_secs(5)).unwrap();
    runtime
        .submit(Action::SaveManualCover(Box::new(ManualCoverRequest {
            choice: ManualCover::new(&doc, 2, 0.75).unwrap(),
            cancel: Cancellation::default(),
        })))
        .unwrap();
    assert!(matches!(receive(&runtime),Update::ManualCover(reply) if reply.result.is_ok()));
    sender.send(Ok(detected)).unwrap();
    assert!(matches!(receive(&runtime), Update::Cover(_, Err(_), _)));
    assert_eq!(selection(&dbpath).unwrap(), (2, "manual".into(), 1, 1.0));
}
#[test]
fn cancelled_failed_native_and_failed_storage_restoration_preserve_the_previous_choice() {
    for scenario in 0..3 {
        let root = tempfile::tempdir().unwrap();
        let dbpath = root.path().join("library.sqlite3");
        let (db, doc, pdf, dll) = setup(&dbpath);
        seed(&db);
        if scenario == 2 {
            db.execute_batch("CREATE TRIGGER denied BEFORE UPDATE OF thumbnail_path ON documents BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
        }
        let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
        let detected = worker
            .try_detect_cover(doc.id.clone(), pdf.clone(), 10, Cancellation::default())
            .unwrap()
            .wait()
            .unwrap();
        let (ready, requests) = std::sync::mpsc::channel();
        let runtime = Runtime::start(
            db,
            Gate {
                native: worker,
                ready,
                render_ready: None,
            },
            || {},
        )
        .unwrap();
        let cancel = Cancellation::default();
        runtime
            .submit(Action::RestoreAutomaticCover(
                CoverSource::capture(&doc),
                10,
                cancel.clone(),
            ))
            .unwrap();
        let sender = requests.recv_timeout(Duration::from_secs(5)).unwrap();
        if scenario == 0 {
            cancel.cancel();
        }
        sender
            .send(if scenario == 1 {
                Err(yamantaka_domain::Failure::Pdf(
                    "synthetic native failure".into(),
                ))
            } else {
                Ok(detected)
            })
            .unwrap();
        assert!(matches!(receive(&runtime), Update::Cover(_, Err(_), _)));
        assert_eq!(selection(&dbpath).unwrap(), (0, "manual".into(), 4, 1.0));
        drop(runtime);
        assert_eq!(
            Database::open(&dbpath)
                .unwrap()
                .get("auto")
                .unwrap()
                .unwrap()
                .thumbnail_path,
            Some("prior-cache".into())
        );
    }
}
