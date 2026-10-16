use super::tests::{Gate, receive, selection, setup};
use crate::cover_source::CoverSource;
use crate::manual_cover::ManualCoverRequest;
use crate::runtime::{Action, Runtime, Update};
use std::time::Duration;
use yamantaka_domain::{
    Cancellation, Failure, cover_selection::ManualCover, memory::MemoryGovernor,
    pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;

#[test]
fn an_obsolete_visible_source_cannot_start_native_cover_detection() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let (database, document, _, dll) = setup(&path);
    database
        .execute_batch("UPDATE documents SET fingerprint='new-source-revision' WHERE id='auto'")
        .unwrap();
    let (ready, requests) = std::sync::mpsc::channel();
    let runtime = Runtime::start(
        database,
        Gate {
            native: PdfWorker::start(&dll, MemoryLimits::default()).unwrap(),
            ready,
            render_ready: None,
        },
        || {},
    )
    .unwrap();
    for operation in 0..3 {
        let cancel = Cancellation::default();
        let source = CoverSource::capture(&document);
        runtime
            .submit(match operation {
                0 => Action::Cover(source, 10, cancel.clone()),
                1 => Action::PreviewCover(source, 0, 10, cancel.clone()),
                _ => Action::RestoreAutomaticCover(source, 10, cancel.clone()),
            })
            .unwrap();
        assert!(
            matches!(receive(&runtime),Update::Cover(id,Err(Failure::Invalid(_)),token) if id=="auto" && token.same_request(&cancel))
        );
    }
    assert!(matches!(
        requests.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    ));
}

#[test]
fn delayed_native_library_and_preview_images_reject_changed_catalogue_sources() {
    for preview in [false, true] {
        for change in [
            "UPDATE documents SET fingerprint='replacement'",
            "UPDATE documents SET path='replacement.pdf'",
            "UPDATE documents SET page_count=6",
            "UPDATE documents SET is_missing=1",
        ] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("library.sqlite3");
            let (database, document, pdf, dll) = setup(&path);
            database.execute_batch("INSERT INTO document_covers VALUES('auto',0,'','','old','old',1,'manual',1,'contain',0.75)").unwrap();
            let original = std::fs::read(&pdf).unwrap();
            let governor = MemoryGovernor::default();
            let native =
                PdfWorker::start_with_governor(&dll, MemoryLimits::default(), governor.clone())
                    .unwrap();
            let image = native
                .try_cover(
                    document.id.clone(),
                    pdf.clone(),
                    0,
                    10,
                    Cancellation::default(),
                )
                .unwrap()
                .wait()
                .unwrap();
            let (ready, _requests) = std::sync::mpsc::channel();
            let (render_ready, renders) = std::sync::mpsc::channel();
            let runtime = Runtime::start(
                database,
                Gate {
                    native,
                    ready,
                    render_ready: Some(render_ready),
                },
                || {},
            )
            .unwrap();
            let cancel = Cancellation::default();
            let source = CoverSource::capture(&document);
            runtime
                .submit(if preview {
                    Action::PreviewCover(source, 0, 10, cancel.clone())
                } else {
                    Action::Cover(source, 10, cancel.clone())
                })
                .unwrap();
            let completion = renders.recv_timeout(Duration::from_secs(5)).unwrap();
            Database::open(&path)
                .unwrap()
                .execute_batch(change)
                .unwrap();
            completion.send(Ok(image)).unwrap();
            assert!(
                matches!(receive(&runtime),Update::Cover(id,Err(Failure::Invalid(_)),token) if id=="auto" && token.same_request(&cancel)),
                "preview={preview}, {change}"
            );
            assert_eq!(selection(&path).unwrap(), (0, "manual".into(), 1, 1.0));
            drop(runtime);
            assert_eq!(governor.usage().total, 0);
            assert_eq!(std::fs::read(pdf).unwrap(), original);
        }
    }
}

#[test]
fn delayed_library_raster_cannot_replace_a_newer_manual_choice() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let (database, document, pdf, dll) = setup(&path);
    database.execute_batch("INSERT INTO document_covers VALUES('auto',0,'','','old','old',1,'manual',1,'contain',0.75)").unwrap();
    let original = std::fs::read(&pdf).unwrap();
    let governor = MemoryGovernor::default();
    let native =
        PdfWorker::start_with_governor(&dll, MemoryLimits::default(), governor.clone()).unwrap();
    let image = native
        .try_cover(
            document.id.clone(),
            pdf.clone(),
            0,
            10,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    let (ready, _requests) = std::sync::mpsc::channel();
    let (render_ready, renders) = std::sync::mpsc::channel();
    let runtime = Runtime::start(
        database,
        Gate {
            native,
            ready,
            render_ready: Some(render_ready),
        },
        || {},
    )
    .unwrap();
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Cover(
            CoverSource::capture(&document),
            10,
            cancel.clone(),
        ))
        .unwrap();
    let completion = renders.recv_timeout(Duration::from_secs(5)).unwrap();
    runtime
        .submit(Action::SaveManualCover(Box::new(ManualCoverRequest {
            choice: ManualCover::new(&document, 2, 0.75).unwrap(),
            cancel: Cancellation::default(),
        })))
        .unwrap();
    assert!(matches!(receive(&runtime),Update::ManualCover(reply) if reply.result.is_ok()));
    completion.send(Ok(image)).unwrap();
    assert!(
        matches!(receive(&runtime),Update::Cover(id,Err(Failure::Invalid(_)),token) if id=="auto" && token.same_request(&cancel))
    );
    assert_eq!(selection(&path).unwrap(), (2, "manual".into(), 1, 1.0));
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(std::fs::read(pdf).unwrap(), original);
}

#[test]
fn preview_completion_validates_native_identity_page_generation_and_cancellation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let (database, document, pdf, dll) = setup(&path);
    let governor = MemoryGovernor::default();
    let native =
        PdfWorker::start_with_governor(&dll, MemoryLimits::default(), governor.clone()).unwrap();
    let mut images = Vec::new();
    for scenario in 0..6 {
        let mut image = native
            .try_cover(
                document.id.clone(),
                pdf.clone(),
                0,
                10,
                Cancellation::default(),
            )
            .unwrap()
            .wait()
            .unwrap();
        let frame = std::sync::Arc::get_mut(&mut image).unwrap();
        match scenario {
            0 => frame.key.document_id = "other".into(),
            1 => frame.key.page = 1,
            2 => frame.key.generation = 11,
            3 => frame.key.rotation = 90,
            _ => {}
        }
        images.push(image);
    }
    let (ready, _requests) = std::sync::mpsc::channel();
    let (render_ready, renders) = std::sync::mpsc::channel();
    let runtime = Runtime::start(
        database,
        Gate {
            native,
            ready,
            render_ready: Some(render_ready),
        },
        || {},
    )
    .unwrap();
    for (scenario, image) in images.into_iter().enumerate() {
        let cancel = Cancellation::default();
        runtime
            .submit(Action::PreviewCover(
                CoverSource::capture(&document),
                0,
                10,
                cancel.clone(),
            ))
            .unwrap();
        let completion = renders.recv_timeout(Duration::from_secs(5)).unwrap();
        if scenario == 4 {
            cancel.cancel();
        }
        completion.send(Ok(image)).unwrap();
        match receive(&runtime) {
            Update::Cover(id, result, token) => {
                assert_eq!(id, "auto");
                assert!(token.same_request(&cancel));
                match scenario {
                    0..=3 => assert!(matches!(result, Err(Failure::Invalid(_)))),
                    4 => assert!(matches!(result, Err(Failure::Cancelled))),
                    _ => {
                        let image = result.unwrap();
                        assert_eq!((image.key.page, image.key.generation), (0, 10));
                    }
                }
            }
            _ => panic!("Expected a scoped cover response"),
        }
    }
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
    assert!(
        selection(&path).is_none(),
        "Previewing must not persist a choice"
    );
}

#[test]
fn a_missing_source_rejects_delayed_detection_and_restoration_without_writing() {
    for restore in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("library.sqlite3");
        let (database, document, pdf, dll) = setup(&path);
        if restore {
            database.execute_batch("INSERT INTO document_covers VALUES('auto',0,'','','old','old',1,'manual',1,'contain',0.75)").unwrap();
        }
        let prior = selection(&path);
        let original = std::fs::read(&pdf).unwrap();
        let governor = MemoryGovernor::default();
        let native =
            PdfWorker::start_with_governor(&dll, MemoryLimits::default(), governor.clone())
                .unwrap();
        let detected = native
            .try_detect_cover(
                document.id.clone(),
                pdf.clone(),
                10,
                Cancellation::default(),
            )
            .unwrap()
            .wait()
            .unwrap();
        let (ready, requests) = std::sync::mpsc::channel();
        let runtime = Runtime::start(
            database,
            Gate {
                native,
                ready,
                render_ready: None,
            },
            || {},
        )
        .unwrap();
        let cancel = Cancellation::default();
        let source = CoverSource::capture(&document);
        runtime
            .submit(if restore {
                Action::RestoreAutomaticCover(source, 10, cancel.clone())
            } else {
                Action::Cover(source, 10, cancel.clone())
            })
            .unwrap();
        let completion = requests.recv_timeout(Duration::from_secs(5)).unwrap();
        Database::open(&path)
            .unwrap()
            .execute_batch("UPDATE documents SET is_missing=1 WHERE id='auto'")
            .unwrap();
        completion.send(Ok(detected)).unwrap();
        assert!(
            matches!(receive(&runtime),Update::Cover(id,Err(Failure::Invalid(_)),token) if id=="auto" && token.same_request(&cancel))
        );
        assert_eq!(selection(&path), prior);
        drop(runtime);
        assert_eq!(governor.usage().total, 0);
        assert_eq!(std::fs::read(pdf).unwrap(), original);
    }
}
