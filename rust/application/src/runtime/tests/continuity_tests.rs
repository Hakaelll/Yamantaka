use super::*;
use crate::reader_continuity::{ContinuityChange, ContinuityRequest};
use yamantaka_domain::reader_tabs::{DocumentTabs, ReaderTab};

fn tabs(page: u32) -> DocumentTabs {
    let mut tabs = DocumentTabs::default();
    tabs.open(
        ReaderTab::new(
            "reading".into(),
            "document".into(),
            page,
            ReaderPreferences::new("document"),
            None,
        )
        .unwrap(),
        true,
    )
    .unwrap();
    tabs
}
#[test]
fn shutdown_barrier_survives_full_ui_capacity_and_fences_an_older_queued_snapshot() {
    let repo = Repository::default();
    let stored = repo.continuity.clone();
    let runtime = Runtime::start(repo, Engine, || {}).unwrap();
    for index in 0..CAPACITY - 1 {
        runtime
            .submit(Action::Favorite(format!("book-{index}"), true))
            .unwrap();
    }
    let older = Cancellation::default();
    runtime
        .submit(Action::ReaderContinuity(Box::new(ContinuityRequest {
            change: ContinuityChange::Save {
                revision: 1,
                tabs: Box::new(tabs(1)),
            },
            cancel: older.clone(),
        })))
        .unwrap();
    assert!(matches!(
        runtime.submit(Action::Query(
            LibraryQuery::default(),
            Cancellation::default()
        )),
        Err(Failure::QueueFull)
    ));
    runtime.persist_reader_continuity(2, tabs(42)).unwrap();
    for _ in 0..CAPACITY {
        let update = runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        if let Update::ReaderContinuity(reply) = update {
            assert!(reply.cancel.same_request(&older));
            assert_eq!(reply.revision, Some(1));
            assert!(reply.result.is_ok() || matches!(reply.result, Err(Failure::Cancelled)));
        }
    }
    assert_eq!(
        stored
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .active()
            .unwrap()
            .page(),
        42
    );
    assert!(matches!(
        runtime.persist_reader_continuity(1, tabs(3)),
        Err(Failure::Cancelled)
    ));
    assert_eq!(
        stored
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .active()
            .unwrap()
            .page(),
        42
    );
}

#[test]
fn accepted_write_keeps_its_terminal_success_after_cancellation_and_load_is_scoped() {
    let repo = Repository::default();
    let stored = repo.continuity.clone();
    let runtime = Runtime::start(repo, Engine, || {}).unwrap();
    let cancel = Cancellation::default();
    runtime
        .submit(Action::ReaderContinuity(Box::new(ContinuityRequest {
            change: ContinuityChange::Save {
                revision: 1,
                tabs: Box::new(tabs(9)),
            },
            cancel: cancel.clone(),
        })))
        .unwrap();
    cancel.cancel();
    let Update::ReaderContinuity(reply) = runtime
        .updates
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
    else {
        panic!("continuity reply");
    };
    assert!(reply.result.is_ok());
    assert!(reply.cancel.same_request(&cancel));
    assert_eq!(
        stored
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .active()
            .unwrap()
            .page(),
        9
    );
    runtime
        .submit(Action::ReaderContinuity(Box::new(ContinuityRequest {
            change: ContinuityChange::Load,
            cancel: Cancellation::default(),
        })))
        .unwrap();
    let Update::ReaderContinuity(reply) = runtime
        .updates
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
    else {
        panic!("continuity reply");
    };
    assert_eq!(reply.result.unwrap().unwrap().active().unwrap().page(), 9);
    runtime
        .submit(Action::ReaderContinuity(Box::new(ContinuityRequest {
            change: ContinuityChange::Load,
            cancel,
        })))
        .unwrap();
    let Update::ReaderContinuity(reply) = runtime
        .updates
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
    else {
        panic!("continuity reply");
    };
    assert!(matches!(reply.result, Err(Failure::Cancelled)));
}
