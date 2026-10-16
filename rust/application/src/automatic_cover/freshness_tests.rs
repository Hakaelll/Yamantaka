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
    db.execute_batch("INSERT INTO document_covers VALUES('auto',0,'keep-small','keep-large','old','2026-10-13',0.79,'automatic',2,'contain',0.75);").unwrap();
}

#[test]
fn outdated_version_source_date_and_page_are_redetected_by_the_real_worker() {
    for change in [
        "SELECT 1",
        "UPDATE document_covers SET algorithm_version=3,source_modified_at='old'",
        "UPDATE document_covers SET algorithm_version=3,source_page_index=7",
    ] {
        let root = tempfile::tempdir().unwrap();
        let dbpath = root.path().join("library.sqlite3");
        let (db, doc, pdf, dll) = setup(&dbpath);
        seed(&db);
        db.execute_batch(change).unwrap();
        let original = std::fs::read(&pdf).unwrap();
        let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
        let runtime = Runtime::start(db, worker, || {}).unwrap();
        runtime
            .submit(Action::Cover(
                CoverSource::capture(&doc),
                10,
                Cancellation::default(),
            ))
            .unwrap();
        assert!(
            matches!(receive(&runtime), Update::Cover(_, Ok(image), _) if image.key.page == 1 && image.key.generation == 10),
            "{change}"
        );
        let actual = selection(&dbpath).unwrap();
        assert_eq!((actual.0, actual.1.as_str(), actual.2), (1, "automatic", 3));
        assert!(actual.3 >= 0.45);
        drop(runtime);
        let db = Database::open(&dbpath).unwrap();
        assert!(db.get("auto").unwrap().unwrap().thumbnail_path.is_none());
        assert_eq!(
            db.cover_snapshot("auto").unwrap().plan,
            yamantaka_domain::cover_snapshot::CoverPlan::Render(1)
        );
        assert_eq!(std::fs::read(pdf).unwrap(), original);
    }
}

#[test]
fn fresh_automatic_and_old_manual_decisions_bypass_detection_and_remain_unchanged() {
    for manual in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let dbpath = root.path().join("library.sqlite3");
        let (db, doc, _pdf, dll) = setup(&dbpath);
        seed(&db);
        db.execute_batch(if manual {
            "UPDATE document_covers SET selection_mode='manual',source_modified_at='old'"
        } else {
            "UPDATE document_covers SET algorithm_version=3"
        })
        .unwrap();
        let revision = db.cover_revision("auto").unwrap();
        let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
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
            .submit(Action::Cover(
                CoverSource::capture(&doc),
                10,
                Cancellation::default(),
            ))
            .unwrap();
        assert!(matches!(receive(&runtime), Update::Cover(_, Ok(image), _) if image.key.page == 0));
        assert!(matches!(
            requests.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ));
        drop(runtime);
        let db = Database::open(&dbpath).unwrap();
        assert_eq!(db.cover_revision("auto").unwrap(), revision);
        assert_eq!(
            db.get("auto").unwrap().unwrap().thumbnail_path,
            Some("prior-cache".into())
        );
    }
}

#[test]
fn delayed_refresh_keeps_old_decisions_and_respects_races_cancellation_and_write_failure() {
    for scenario in 0..5 {
        let root = tempfile::tempdir().unwrap();
        let dbpath = root.path().join("library.sqlite3");
        let (db, doc, pdf, dll) = setup(&dbpath);
        seed(&db);
        if scenario == 3 {
            db.execute_batch("CREATE TRIGGER rejected BEFORE UPDATE ON document_covers BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
        }
        let prior = db.cover_revision("auto").unwrap();
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
            .submit(Action::Cover(
                CoverSource::capture(&doc),
                10,
                cancel.clone(),
            ))
            .unwrap();
        let sender = requests.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(
            selection(&dbpath).unwrap(),
            (0, "automatic".into(), 2, 0.79)
        );
        match scenario {
            0 => {
                runtime.submit(Action::SaveManualCover(Box::new(ManualCoverRequest {
                    choice: ManualCover::new(&doc, 2, 0.75).unwrap(), cancel: Cancellation::default(),
                }))).unwrap();
                assert!(matches!(receive(&runtime), Update::ManualCover(reply) if reply.result.is_ok()));
            }
            1 => Database::open(&dbpath).unwrap().execute_batch("UPDATE documents SET file_modified_at='new-source' WHERE id='auto'").unwrap(),
            2 => cancel.cancel(),
            4 => Database::open(&dbpath).unwrap().execute_batch("UPDATE document_covers SET source_page_index=2,algorithm_version=3 WHERE document_id='auto'").unwrap(),
            _ => {}
        }
        sender.send(Ok(detected)).unwrap();
        let update = receive(&runtime);
        if scenario == 0 || scenario == 4 {
            assert!(matches!(update, Update::Cover(_, Ok(image), _) if image.key.page == 2));
            let stored = selection(&dbpath).unwrap();
            assert_eq!(stored.0, 2);
            assert_eq!(stored.1, if scenario == 0 { "manual" } else { "automatic" });
        } else {
            assert!(matches!(update, Update::Cover(_, Err(_), _)), "{scenario}");
            drop(runtime);
            let db = Database::open(&dbpath).unwrap();
            assert_eq!(db.cover_revision("auto").unwrap(), prior);
            assert_eq!(
                db.get("auto").unwrap().unwrap().thumbnail_path,
                Some("prior-cache".into())
            );
        }
    }
}
