use super::*;
use std::{path::Path, time::Instant};
use yamantaka_application::{
    background_password::Purpose,
    search_index::{IndexCommand, IndexPhase, SearchIndexController},
};
use yamantaka_domain::{
    LibraryQuery,
    search::{IndexAccess, SearchKind, SearchQuery, SearchRepository},
};

pub(super) fn verify(path: &Path, source: &Path, root: &Path) {
    let mut db = Database::open(path).unwrap();
    let documents = db.query(&LibraryQuery::default()).unwrap();
    let document = documents.iter().find(|d| d.path == source).unwrap();
    for other in documents.iter().filter(|d| d.id != document.id) {
        db.exclude_from_index(&other.id, true, &Cancellation::default())
            .unwrap();
    }
    for mode in ["cancel", "reopen", "yield"] {
        db.clear_index(&Cancellation::default()).unwrap();
        let worker = PdfWorker::start(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
        )
        .unwrap();
        let (prompts, receive) = bounded(8);
        let (wake, _wakeups) = bounded(8);
        let mut controller = SearchIndexController::default().with_passwords(prompts, wake);
        controller
            .command(IndexCommand::Enable(true), &mut db)
            .unwrap();
        controller
            .command(IndexCommand::Start { rebuild: false }, &mut db)
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut prompt_count = 0;
        let mut preempted = false;
        let mut resuming = false;
        loop {
            assert!(
                Instant::now() < deadline,
                "protected indexing stalled in {mode}: {:?}",
                controller.progress()
            );
            if let Ok(prompt) = receive.try_recv() {
                assert!(prompt.purpose() == Purpose::Index);
                prompt_count += 1;
                if mode == "cancel" {
                    drop(prompt);
                } else if prompt_count == 1 {
                    assert!(!prompt.rejected());
                    prompt
                        .answer("synthetic incorrect credential".into())
                        .unwrap();
                } else {
                    assert_eq!(prompt.rejected(), prompt_count == 2);
                    prompt.answer("fixture-password".into()).unwrap();
                }
            }
            if !preempted && controller.progress().current_page == 1 {
                preempted = true;
                let id = controller.native_document_id().unwrap().to_owned();
                if mode == "yield" {
                    controller.set_slot_available(false);
                    resuming = true;
                } else {
                    worker.close(id).unwrap();
                }
            }
            controller.step(&mut db, &worker).unwrap();
            if resuming && controller.native_document_id().is_none() {
                assert_eq!(controller.progress().current_page, 1);
                controller.set_slot_available(true);
                resuming = false;
            }
            match controller.progress().phase {
                IndexPhase::Cancelled => {
                    assert_eq!(mode, "cancel");
                    assert_eq!(prompt_count, 1);
                    let connection = rusqlite::Connection::open(path).unwrap();
                    let pages:u32=connection.query_row("SELECT count(*) FROM library_search WHERE document_id=? AND kind='pageText'",[&document.id],|r|r.get(0)).unwrap();
                    assert_eq!(pages, 0);
                    break;
                }
                IndexPhase::Completed => {
                    assert_ne!(mode, "cancel");
                    assert!(preempted);
                    assert_eq!(prompt_count, 3, "mode={mode}");
                    assert_eq!(controller.progress().completed_documents, 1);
                    assert_eq!(controller.progress().failed_documents, 0);
                    let connection = rusqlite::Connection::open(path).unwrap();
                    let pages: Vec<(u32,String)> = connection.prepare("SELECT page_index,body FROM library_search WHERE document_id=? AND kind='pageText' ORDER BY page_index").unwrap().query_map([&document.id], |r|Ok((r.get(0)?,r.get(1)?))).unwrap().map(|r|r.unwrap()).collect();
                    worker
                        .open(
                            "expected-index".into(),
                            source.to_owned(),
                            Some("fixture-password".into()),
                            Cancellation::default(),
                        )
                        .unwrap();
                    let expected = (0..2)
                        .map(|page| {
                            (
                                page,
                                worker
                                    .text("expected-index".into(), page, Cancellation::default())
                                    .unwrap(),
                            )
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(pages, expected);
                    worker.close("expected-index".into()).unwrap();
                    break;
                }
                _ => {}
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(controller.native_document_id().is_none());
    }
    db.clear_index(&Cancellation::default()).unwrap();
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let runtime = Runtime::start(db, worker, || {}).unwrap();
    runtime
        .submit(Action::Index(IndexCommand::Enable(true)))
        .unwrap();
    runtime
        .submit(Action::Index(IndexCommand::Start { rebuild: false }))
        .unwrap();
    let prompt = password::challenge(&runtime);
    assert!(prompt.purpose() == Purpose::Index);
    prompt.answer("fixture-password".into()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while runtime.index_progress().phase != IndexPhase::Completed {
        assert!(Instant::now() < deadline, "runtime indexing did not finish");
        while runtime.updates.try_recv().is_ok() {}
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(runtime.index_progress().failed_documents, 0);
    drop(runtime);
    let mut db = Database::open(path).unwrap();
    db.refresh_search_entries(IndexAccess::Enabled, &Cancellation::default())
        .unwrap();
    let query = SearchQuery {
        text: "Yamantaka".into(),
        kinds: vec![SearchKind::PageText],
        document_id: Some(document.id.clone()),
        ..Default::default()
    };
    assert!(
        !db.search(&query, &Cancellation::default())
            .unwrap()
            .is_empty()
    );
}
