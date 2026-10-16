use super::tests::{Gate, receive, selection, setup};
use crate::manual_cover::ManualCoverRequest;
use crate::{
    cover_source::CoverSource,
    runtime::{Action, Runtime, Update, WriteKind},
};
use std::time::Duration;
use yamantaka_domain::{
    Cancellation, Failure, cover_selection::ManualCover, pdf_ticket::AsyncPdfEngine,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

#[test]
fn capacity_failure_keeps_the_cover_pending_until_a_confirmed_release() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let (database, document, pdf, dll) = setup(&path);
    let native = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
    let governor = native.memory_governor();
    let detected = native
        .try_detect_cover(document.id.clone(), pdf, 10, Cancellation::default())
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
    runtime
        .submit(Action::Cover(
            CoverSource::capture(&document),
            10,
            cancel.clone(),
        ))
        .unwrap();
    requests
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .send(Err(Failure::NativeSlotsBusy))
        .unwrap();
    assert!(
        matches!(
            runtime.updates.recv_timeout(Duration::from_millis(100)),
            Err(crossbeam_channel::RecvTimeoutError::Timeout)
        ),
        "Transient capacity must retain the request instead of publishing a terminal failure"
    );
    assert!(
        matches!(
            requests.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ),
        "No retry loop without a capacity change"
    );
    runtime
        .submit(Action::VisibleDocuments(Vec::new()))
        .unwrap();
    assert!(matches!(
        receive(&runtime),
        Update::Committed(_, WriteKind::VisibleDocuments)
    ));
    requests
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .send(Ok(detected))
        .unwrap();
    assert!(
        matches!(receive(&runtime),Update::Cover(id,Ok(image),token) if id=="auto" && image.key.page==1 && image.key.generation==10 && token.same_request(&cancel))
    );
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}

fn wait_parked(runtime: &Runtime) {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while runtime.waiting_covers() != 1 {
        assert!(
            std::time::Instant::now() < deadline,
            "Cover never entered its capacity wait"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn cancelled_parked_work_finishes_without_a_capacity_release_or_native_retry() {
    for restore in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let (database, document, _, dll) = setup(&directory.path().join("library.sqlite3"));
        let native = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
        let governor = native.memory_governor();
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
        runtime
            .submit(if restore {
                Action::RestoreAutomaticCover(CoverSource::capture(&document), 10, cancel.clone())
            } else {
                Action::Cover(CoverSource::capture(&document), 10, cancel.clone())
            })
            .unwrap();
        requests
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .send(Err(Failure::NativeSlotsBusy))
            .unwrap();
        wait_parked(&runtime);
        cancel.cancel();
        runtime.wake();
        assert!(
            matches!(receive(&runtime),Update::Cover(id,Err(Failure::Cancelled),token) if id=="auto" && token.same_request(&cancel))
        );
        assert_eq!(runtime.pdf_capacity_revision(), 0);
        assert!(matches!(
            requests.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ));
        drop(runtime);
        assert_eq!(governor.usage().total, 0);
    }
}

#[test]
fn a_new_manual_choice_is_used_by_a_waiting_library_but_blocks_waiting_restoration() {
    for restore in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("library.sqlite3");
        let (database, document, pdf, dll) = setup(&path);
        let original = std::fs::read(&pdf).unwrap();
        let native = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
        let governor = native.memory_governor();
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
        requests
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .send(Err(Failure::NativeSlotsBusy))
            .unwrap();
        wait_parked(&runtime);
        runtime
            .submit(Action::SaveManualCover(Box::new(ManualCoverRequest {
                choice: ManualCover::new(&document, 2, 0.75).unwrap(),
                cancel: Cancellation::default(),
            })))
            .unwrap();
        assert!(matches!(receive(&runtime),Update::ManualCover(reply) if reply.result.is_ok()));
        runtime
            .submit(Action::VisibleDocuments(Vec::new()))
            .unwrap();
        assert!(matches!(
            receive(&runtime),
            Update::Committed(_, WriteKind::VisibleDocuments)
        ));
        match receive(&runtime) {
            Update::Cover(id, result, token) => {
                assert_eq!(id, "auto");
                assert!(token.same_request(&cancel));
                if restore {
                    assert!(matches!(result, Err(Failure::Invalid(_))));
                } else {
                    let image = result.unwrap();
                    assert_eq!((image.key.page, image.key.generation), (2, 10));
                }
            }
            _ => panic!("Expected a scoped cover response"),
        }
        assert_eq!(selection(&path).unwrap(), (2, "manual".into(), 1, 1.0));
        assert!(matches!(
            requests.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ));
        drop(runtime);
        assert_eq!(governor.usage().total, 0);
        assert_eq!(std::fs::read(pdf).unwrap(), original);
    }
}

#[test]
fn a_late_busy_reply_does_not_miss_an_already_confirmed_capacity_release() {
    let directory = tempfile::tempdir().unwrap();
    let (database, document, pdf, dll) = setup(&directory.path().join("library.sqlite3"));
    let native = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
    let governor = native.memory_governor();
    let detected = native
        .try_detect_cover(document.id.clone(), pdf, 10, Cancellation::default())
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
    runtime
        .submit(Action::Cover(
            CoverSource::capture(&document),
            10,
            cancel.clone(),
        ))
        .unwrap();
    let old_reply = requests.recv_timeout(Duration::from_secs(5)).unwrap();
    runtime
        .submit(Action::VisibleDocuments(Vec::new()))
        .unwrap();
    assert!(matches!(
        receive(&runtime),
        Update::Committed(_, WriteKind::VisibleDocuments)
    ));
    assert!(runtime.pdf_capacity_revision() > 0);
    old_reply.send(Err(Failure::NativeSlotsBusy)).unwrap();
    requests
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .send(Ok(detected))
        .unwrap();
    assert!(
        matches!(receive(&runtime),Update::Cover(id,Ok(image),token) if id=="auto" && image.key.page==1 && token.same_request(&cancel))
    );
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn a_picker_preview_retries_after_release_without_persisting_its_page() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("library.sqlite3");
    let (database, document, pdf, dll) = setup(&path);
    let native = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
    let governor = native.memory_governor();
    let image = native
        .try_cover(document.id.clone(), pdf, 2, 10, Cancellation::default())
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
        .submit(Action::PreviewCover(
            CoverSource::capture(&document),
            2,
            10,
            cancel.clone(),
        ))
        .unwrap();
    renders
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .send(Err(Failure::NativeSlotsBusy))
        .unwrap();
    wait_parked(&runtime);
    runtime
        .submit(Action::VisibleDocuments(Vec::new()))
        .unwrap();
    assert!(matches!(
        receive(&runtime),
        Update::Committed(_, WriteKind::VisibleDocuments)
    ));
    renders
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .send(Ok(image))
        .unwrap();
    assert!(
        matches!(receive(&runtime),Update::Cover(id,Ok(image),token) if id=="auto" && image.key.page==2 && token.same_request(&cancel))
    );
    assert!(selection(&path).is_none());
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn parked_covers_keep_admission_bounded_and_shutdown_cancels_every_owned_request() {
    let directory = tempfile::tempdir().unwrap();
    let (database, document, _, dll) = setup(&directory.path().join("library.sqlite3"));
    let native = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
    let governor = native.memory_governor();
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
    let mut tokens = Vec::new();
    for generation in 0..128 {
        let cancel = Cancellation::default();
        runtime
            .submit(Action::Cover(
                CoverSource::capture(&document),
                generation,
                cancel.clone(),
            ))
            .unwrap();
        tokens.push(cancel);
    }
    for _ in 0..128 {
        requests
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .send(Err(Failure::NativeSlotsBusy))
            .unwrap();
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while runtime.waiting_covers() != 128 {
        assert!(
            std::time::Instant::now() < deadline,
            "The admitted covers never reached a bounded wait"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(matches!(
        runtime.submit(Action::Query(
            yamantaka_domain::LibraryQuery::default(),
            Cancellation::default()
        )),
        Err(Failure::QueueFull)
    ));
    assert!(runtime.updates.try_recv().is_err());
    drop(runtime);
    assert!(tokens.iter().all(Cancellation::is_cancelled));
    assert_eq!(governor.usage().total, 0);
}
