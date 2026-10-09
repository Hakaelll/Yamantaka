use crossbeam_channel::{Sender, bounded};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use yamantaka_application::runtime::{Action, Runtime, Update};
use yamantaka_domain::{
    Cancellation, Failure, LibraryQuery, PdfEngine, PdfInfo, PdfPassword, Priority, RenderKey,
    RenderedPage, Result,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    pdf_find::{PdfFindQuery, PdfMatch, PdfPageMatches},
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
};
use yamantaka_storage::Database;

type FindReply = Arc<Mutex<Option<Sender<Result<Arc<PdfPageMatches>>>>>>;
struct HeldFind {
    entered: Sender<(String, u32, PdfFindQuery, u32, Cancellation)>,
    reply: FindReply,
}
fn unused<T>() -> Result<T> {
    Err(Failure::Unsupported("unused test operation".into()))
}
impl PdfEngine for HeldFind {
    fn open(
        &self,
        _: String,
        _: PathBuf,
        _: Option<PdfPassword>,
        _: Cancellation,
    ) -> Result<PdfInfo> {
        unused()
    }
    fn render(&self, _: RenderKey, _: Priority, _: Cancellation) -> Result<Arc<RenderedPage>> {
        unused()
    }
    fn text(&self, _: String, _: u32, _: Cancellation) -> Result<String> {
        unused()
    }
    fn close(&self, _: String) -> Result<()> {
        unused()
    }
}
impl AsyncPdfEngine for HeldFind {
    fn set_visible_documents(&self, _: &[String]) -> Result<()> {
        Ok(())
    }
    fn try_open(
        &self,
        _: String,
        _: PathBuf,
        _: Option<PdfPassword>,
        _: Cancellation,
    ) -> Result<PdfTicket<PdfInfo>> {
        unused()
    }
    fn try_render(
        &self,
        _: RenderKey,
        _: Priority,
        _: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        unused()
    }
    fn try_text(&self, _: String, _: u32, _: Cancellation) -> Result<PdfTicket<String>> {
        unused()
    }
    fn try_close(&self, _: String) -> Result<PdfTicket<()>> {
        unused()
    }
    fn try_find_page(
        &self,
        id: String,
        page: u32,
        query: PdfFindQuery,
        offset: u32,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<PdfPageMatches>>> {
        let (send, ticket) = PdfTicket::channel(7, 3, cancel.clone());
        *self.reply.lock().unwrap() = Some(send);
        self.entered
            .try_send((id, page, query, offset, cancel))
            .unwrap();
        Ok(ticket)
    }
}

#[test]
fn actor_preserves_find_identity_and_queries_sql_while_native_search_is_held() {
    for cancelled in [false, true] {
        let (entered, wait) = bounded(1);
        let reply = Arc::new(Mutex::new(None));
        let runtime = Runtime::start(
            Database::in_memory_at(11).unwrap(),
            HeldFind {
                entered,
                reply: reply.clone(),
            },
            || {},
        )
        .unwrap();
        let token = Cancellation::default();
        let query = PdfFindQuery::new("Résumé", true, true).unwrap();
        runtime
            .submit(Action::FindPage(
                "reader".into(),
                7,
                query.clone(),
                2,
                token.clone(),
            ))
            .unwrap();
        let (id, page, actual, offset, cancel) = wait.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!((id.as_str(), page, offset), ("reader", 7, 2));
        assert_eq!(actual, query);
        assert!(cancel.same_request(&token));
        runtime
            .submit(Action::Query(
                LibraryQuery::default(),
                Cancellation::default(),
            ))
            .unwrap();
        assert!(matches!(
            runtime.updates.recv_timeout(Duration::from_secs(2)),
            Ok(Update::Library(_, 0, _))
        ));
        if cancelled {
            token.cancel();
        }
        let governor = MemoryGovernor::default();
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                4096,
            ))
            .unwrap();
        let result = Arc::new(
            PdfPageMatches::from_reserved(
                7,
                vec![PdfMatch {
                    ordinal: 2,
                    snippet: "Résumé".into(),
                    rectangles: Vec::new(),
                }],
                Some(3),
                true,
                lease,
            )
            .unwrap(),
        );
        reply
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .try_send(Ok(result))
            .unwrap();
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
        {
            Update::PageMatches(id, matches, cancel) => {
                assert!(!cancelled);
                assert_eq!(id, "reader");
                assert_eq!(matches.page(), 7);
                assert_eq!(matches.next_match(), Some(3));
                assert!(cancel.same_request(&token));
            }
            Update::PageFindFailed(id, Failure::Cancelled, cancel) => {
                assert!(cancelled);
                assert_eq!(id, "reader");
                assert!(cancel.same_request(&token));
            }
            _ => panic!("find response identity"),
        }
        assert_eq!(governor.usage().text, 0);
    }
}

fn page_result(
    governor: &MemoryGovernor,
    page: u32,
    start: u32,
    count: u32,
    next: Option<u32>,
) -> Arc<PdfPageMatches> {
    page_result_with_text(governor, page, start, count, next, count > 0)
}
fn page_result_with_text(
    governor: &MemoryGovernor,
    page: u32,
    start: u32,
    count: u32,
    next: Option<u32>,
    text_present: bool,
) -> Arc<PdfPageMatches> {
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            32 * 1024,
        ))
        .unwrap();
    Arc::new(
        PdfPageMatches::from_reserved(
            page,
            (start..start + count)
                .map(|ordinal| PdfMatch {
                    ordinal,
                    snippet: format!("match {ordinal}"),
                    rectangles: Vec::new(),
                })
                .collect(),
            next,
            text_present,
            lease,
        )
        .unwrap(),
    )
}

#[test]
fn document_scan_fills_batches_across_pages_and_resumes_at_first_omitted_match() {
    use yamantaka_application::document_find::{FindCursor, FindRequest};
    let governor = MemoryGovernor::default();
    let (entered, wait) = bounded(1);
    let reply = Arc::new(Mutex::new(None));
    let runtime = Runtime::start(
        Database::in_memory_at(11).unwrap(),
        HeldFind {
            entered,
            reply: reply.clone(),
        },
        || {},
    )
    .unwrap();
    let query = PdfFindQuery::new("match", false, false).unwrap();
    let cancel = Cancellation::default();
    runtime
        .submit(Action::FindDocument(
            FindRequest::new(
                "reader".into(),
                query.clone(),
                3,
                FindCursor::default(),
                cancel.clone(),
            )
            .unwrap(),
        ))
        .unwrap();
    for (page, count, next) in [(0, 40, None), (1, 80, Some(80))] {
        let (id, requested, _, offset, token) = wait.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(id, "reader");
        assert_eq!(requested, page);
        assert_eq!(offset, 0);
        assert!(token.same_request(&cancel));
        reply
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .send(Ok(page_result(&governor, page, 0, count, next)))
            .unwrap();
    }
    let batch = match runtime
        .updates
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
    {
        Update::DocumentMatches(id, batch, token) => {
            assert_eq!(id, "reader");
            assert!(token.same_request(&cancel));
            batch
        }
        _ => panic!("expected document batch"),
    };
    let expected: Vec<_> = (0..40)
        .map(|n| (0, n))
        .chain((0..40).map(|n| (1, n)))
        .collect();
    assert_eq!(
        batch
            .matches()
            .map(|(p, m)| (p, m.ordinal))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        batch.next,
        Some(FindCursor {
            page: 1,
            ordinal: 40
        })
    );
    assert_eq!(batch.scanned_pages, 1);
    let cursor = batch.next.unwrap();
    drop(batch);
    assert_eq!(governor.usage().text, 0);
    runtime
        .submit(Action::FindDocument(
            FindRequest::new("reader".into(), query, 3, cursor, Cancellation::default()).unwrap(),
        ))
        .unwrap();
    for (page, offset, count) in [(1, 40, 60), (2, 0, 0)] {
        let (_, requested, _, ordinal, _) = wait.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!((requested, ordinal), (page, offset));
        reply
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .send(Ok(page_result(&governor, page, offset, count, None)))
            .unwrap();
    }
    match runtime
        .updates
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
    {
        Update::DocumentMatches(_, batch, _) => {
            assert_eq!(batch.matches().count(), 60);
            assert_eq!(batch.next, None);
            assert_eq!(batch.scanned_pages, 3);
            assert_eq!(batch.matches().next().unwrap().1.ordinal, 40);
        }
        _ => panic!("expected final batch"),
    }
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn document_scan_traverses_five_thousand_empty_pages_without_ui_draining() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use yamantaka_application::document_find::{FindCursor, FindRequest};
    let governor = MemoryGovernor::default();
    let (entered, wait) = bounded(1);
    let reply = Arc::new(Mutex::new(None));
    let notifications = Arc::new(AtomicUsize::new(0));
    let observed = notifications.clone();
    let runtime = Runtime::start(
        Database::in_memory_at(11).unwrap(),
        HeldFind {
            entered,
            reply: reply.clone(),
        },
        move || {
            observed.fetch_add(1, Ordering::Relaxed);
        },
    )
    .unwrap();
    let request = FindRequest::new(
        "long".into(),
        PdfFindQuery::new("missing", false, false).unwrap(),
        5000,
        FindCursor::default(),
        Cancellation::default(),
    )
    .unwrap();
    let progress = request.progress.clone();
    runtime.submit(Action::FindDocument(request)).unwrap();
    for page in 0..5000 {
        let (_, requested, _, offset, _) = wait.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!((requested, offset), (page, 0));
        assert!(governor.usage().text < 1024);
        reply
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .send(Ok(page_result(&governor, page, 0, 0, None)))
            .unwrap();
    }
    match runtime
        .updates
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
    {
        Update::DocumentMatches(_, batch, _) => {
            assert_eq!(batch.matches().count(), 0);
            assert_eq!(batch.next, None);
            assert_eq!(batch.scanned_pages, 5000);
        }
        _ => panic!("expected completed empty search"),
    }
    assert_eq!(progress.scanned_pages(), 5000);
    assert_eq!(notifications.load(Ordering::Relaxed), 313);
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn document_scan_discards_partial_results_after_cancellation_or_wrong_page() {
    use yamantaka_application::document_find::{FindCursor, FindRequest};
    for cancelled in [false, true] {
        let governor = MemoryGovernor::default();
        let (entered, wait) = bounded(1);
        let reply = Arc::new(Mutex::new(None));
        let runtime = Runtime::start(
            Database::in_memory_at(11).unwrap(),
            HeldFind {
                entered,
                reply: reply.clone(),
            },
            || {},
        )
        .unwrap();
        let cancel = Cancellation::default();
        runtime
            .submit(Action::FindDocument(
                FindRequest::new(
                    "reader".into(),
                    PdfFindQuery::new("match", false, false).unwrap(),
                    3,
                    FindCursor::default(),
                    cancel.clone(),
                )
                .unwrap(),
            ))
            .unwrap();
        wait.recv_timeout(Duration::from_secs(2)).unwrap();
        reply
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .send(Ok(page_result(&governor, 0, 0, 1, None)))
            .unwrap();
        wait.recv_timeout(Duration::from_secs(2)).unwrap();
        runtime
            .submit(Action::Query(
                LibraryQuery::default(),
                Cancellation::default(),
            ))
            .unwrap();
        assert!(matches!(
            runtime
                .updates
                .recv_timeout(Duration::from_secs(2))
                .unwrap(),
            Update::Library(..)
        ));
        if cancelled {
            cancel.cancel();
        }
        reply
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .send(Ok(page_result(
                &governor,
                if cancelled { 1 } else { 2 },
                0,
                1,
                None,
            )))
            .unwrap();
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
        {
            Update::DocumentFindFailed(id, error, token) => {
                assert_eq!(id, "reader");
                assert!(token.same_request(&cancel));
                if cancelled {
                    assert!(matches!(error, Failure::Cancelled));
                } else {
                    assert!(matches!(error, Failure::Pdf(_)));
                }
            }
            _ => panic!("expected failed scan"),
        }
        assert!(wait.try_recv().is_err());
        assert_eq!(governor.usage().text, 0);
    }
}

#[test]
fn document_scan_uses_real_pdfium_for_study_matches_and_five_thousand_pages() {
    use yamantaka_application::document_find::{FindCursor, FindRequest};
    use yamantaka_pdf::{MemoryLimits, PdfWorker};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let governor = MemoryGovernor::default();
    let engine = PdfWorker::start_with_governor(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
        governor.clone(),
    )
    .unwrap();
    let runtime =
        Runtime::start(Database::in_memory_at(11).unwrap(), engine.clone(), || {}).unwrap();
    for (name, query, expected_pages) in [
        ("outline", "reading", 3),
        ("5000-pages", "no_such_phrase_937", 5000),
    ] {
        let path = root.join(format!("build/rust-fixtures/{name}.pdf"));
        let original = std::fs::read(&path).unwrap();
        let info = engine
            .open(name.into(), path.clone(), None, Cancellation::default())
            .unwrap();
        assert_eq!(info.pages.len(), expected_pages);
        let request = FindRequest::new(
            name.into(),
            PdfFindQuery::new(query, false, false).unwrap(),
            expected_pages as u32,
            FindCursor::default(),
            Cancellation::default(),
        )
        .unwrap();
        let progress = request.progress.clone();
        runtime.submit(Action::FindDocument(request)).unwrap();
        let batch = match runtime
            .updates
            .recv_timeout(Duration::from_secs(60))
            .unwrap()
        {
            Update::DocumentMatches(id, batch, _) => {
                assert_eq!(id, name);
                batch
            }
            _ => panic!("expected real native scan"),
        };
        assert_eq!(batch.next, None);
        assert_eq!(progress.scanned_pages(), expected_pages as u32);
        let matches: Vec<_> = batch.matches().collect();
        if name == "outline" {
            assert!(matches.len() >= 3);
            assert!(matches.iter().any(|(page, _)| *page == 0));
            assert!(matches.iter().any(|(page, _)| *page == 1));
            assert!(matches.iter().any(|(page, _)| *page == 2));
            assert!(
                matches
                    .iter()
                    .all(|(page, item)| item.position(*page).is_some())
            );
        } else {
            assert!(matches.is_empty());
        }
        drop(matches);
        drop(batch);
        engine.close(name.into()).unwrap();
        assert_eq!(governor.usage().text, 0);
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
}

#[test]
fn previous_document_batches_keep_only_last_eighty_and_stop_at_exclusive_cursor() {
    use yamantaka_application::document_find::{FindCursor, FindRequest};
    for boundary in [
        None,
        Some(FindCursor {
            page: 1,
            ordinal: 90,
        }),
        Some(FindCursor {
            page: 1,
            ordinal: 80,
        }),
        Some(FindCursor {
            page: 0,
            ordinal: 20,
        }),
    ] {
        let governor = MemoryGovernor::default();
        let (entered, wait) = bounded(1);
        let reply = Arc::new(Mutex::new(None));
        let runtime = Runtime::start(
            Database::in_memory_at(11).unwrap(),
            HeldFind {
                entered,
                reply: reply.clone(),
            },
            || {},
        )
        .unwrap();
        let cancel = Cancellation::default();
        runtime
            .submit(Action::FindDocument(
                FindRequest::previous(
                    "reader".into(),
                    PdfFindQuery::new("match", false, false).unwrap(),
                    3,
                    boundary,
                    cancel.clone(),
                )
                .unwrap(),
            ))
            .unwrap();
        for (page, offset, count, next) in [
            (0, 0, 60, None),
            (1, 0, 80, Some(80)),
            (1, 80, 20, None),
            (2, 0, 5, None),
        ] {
            let (_, requested, _, ordinal, token) =
                wait.recv_timeout(Duration::from_secs(2)).unwrap();
            assert_eq!((requested, ordinal), (page, offset));
            assert!(token.same_request(&cancel));
            reply
                .lock()
                .unwrap()
                .take()
                .unwrap()
                .send(Ok(page_result(&governor, page, offset, count, next)))
                .unwrap();
            if boundary.is_some_and(|end| page == end.page && offset + count >= end.ordinal) {
                break;
            }
        }
        let batch = match runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
        {
            Update::DocumentMatches(_, batch, _) => batch,
            _ => panic!("expected previous batch"),
        };
        let mut expected: Vec<_> = (0..60)
            .map(|n| (0, n))
            .chain((0..100).map(|n| (1, n)))
            .chain((0..5).map(|n| (2, n)))
            .filter(|(p, n)| boundary.is_none_or(|end| (*p, *n) < (end.page, end.ordinal)))
            .collect();
        if expected.len() > 80 {
            expected.drain(..expected.len() - 80);
        }
        assert_eq!(
            batch
                .matches()
                .map(|(p, m)| (p, m.ordinal))
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            batch.start,
            expected.first().map(|(page, ordinal)| FindCursor {
                page: *page,
                ordinal: *ordinal
            })
        );
        assert_eq!(batch.next, boundary);
        assert_eq!(batch.scanned_pages, boundary.map_or(3, |end| end.page));
        drop(batch);
        assert_eq!(governor.usage().text, 0);
        assert!(wait.try_recv().is_err());
    }
}

#[test]
fn previous_native_search_wraps_to_last_study_match_and_excludes_current_one() {
    use yamantaka_application::document_find::{FindCursor, FindRequest};
    use yamantaka_pdf::{MemoryLimits, PdfWorker};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let governor = MemoryGovernor::default();
    let engine = PdfWorker::start_with_governor(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
        governor.clone(),
    )
    .unwrap();
    let info = engine
        .open(
            "study".into(),
            root.join("build/rust-fixtures/outline.pdf"),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let runtime =
        Runtime::start(Database::in_memory_at(11).unwrap(), engine.clone(), || {}).unwrap();
    let query = PdfFindQuery::new("reading", false, false).unwrap();
    runtime
        .submit(Action::FindDocument(
            FindRequest::previous(
                "study".into(),
                query.clone(),
                info.pages.len() as u32,
                None,
                Cancellation::default(),
            )
            .unwrap(),
        ))
        .unwrap();
    let batch = match runtime
        .updates
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
    {
        Update::DocumentMatches(_, batch, _) => batch,
        _ => panic!("expected wrapped native batch"),
    };
    let expected: Vec<_> = batch.matches().map(|(p, m)| (p, m.ordinal)).collect();
    assert_eq!(expected.last().unwrap().0, 2);
    let (page, ordinal) = *expected.last().unwrap();
    drop(batch);
    runtime
        .submit(Action::FindDocument(
            FindRequest::previous(
                "study".into(),
                query,
                info.pages.len() as u32,
                Some(FindCursor { page, ordinal }),
                Cancellation::default(),
            )
            .unwrap(),
        ))
        .unwrap();
    match runtime
        .updates
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
    {
        Update::DocumentMatches(_, batch, _) => {
            assert_eq!(
                batch
                    .matches()
                    .map(|(p, m)| (p, m.ordinal))
                    .collect::<Vec<_>>(),
                expected[..expected.len() - 1]
            );
        }
        _ => panic!("expected preceding native matches"),
    }
    engine.close("study".into()).unwrap();
    assert_eq!(governor.usage().text, 0);
}

#[test]
fn text_absence_requires_complete_inspection_and_empty_match_pages_preserve_text_evidence() {
    use yamantaka_application::document_find::{FindCursor, FindRequest};
    use yamantaka_domain::pdf_find::DocumentTextPresence;
    for (start, final_text, expected) in [
        (0, false, DocumentTextPresence::Unavailable),
        (0, true, DocumentTextPresence::Available),
        (1, false, DocumentTextPresence::Unknown),
    ] {
        let governor = MemoryGovernor::default();
        let (entered, wait) = bounded(1);
        let reply = Arc::new(Mutex::new(None));
        let runtime = Runtime::start(
            Database::in_memory_at(11).unwrap(),
            HeldFind {
                entered,
                reply: reply.clone(),
            },
            || {},
        )
        .unwrap();
        runtime
            .submit(Action::FindDocument(
                FindRequest::new(
                    "reader".into(),
                    PdfFindQuery::new("absent", false, false).unwrap(),
                    3,
                    FindCursor {
                        page: start,
                        ordinal: 0,
                    },
                    Cancellation::default(),
                )
                .unwrap(),
            ))
            .unwrap();
        for page in start..3 {
            let (_, requested, _, _, _) = wait.recv_timeout(Duration::from_secs(2)).unwrap();
            assert_eq!(requested, page);
            reply
                .lock()
                .unwrap()
                .take()
                .unwrap()
                .send(Ok(page_result_with_text(
                    &governor,
                    page,
                    0,
                    0,
                    None,
                    final_text && page == 2,
                )))
                .unwrap();
        }
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
        {
            Update::DocumentMatches(_, batch, _) => {
                assert_eq!(batch.text_presence, expected);
                assert_eq!(batch.matches().count(), 0);
            }
            _ => panic!("expected text evidence"),
        }
        assert_eq!(governor.usage().text, 0);
    }
}

#[test]
fn native_search_distinguishes_absent_phrase_from_scanned_and_blank_documents() {
    use yamantaka_application::document_find::{FindCursor, FindRequest};
    use yamantaka_domain::pdf_find::DocumentTextPresence;
    use yamantaka_pdf::{MemoryLimits, PdfWorker};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let governor = MemoryGovernor::default();
    let engine = PdfWorker::start_with_governor(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
        governor.clone(),
    )
    .unwrap();
    let runtime =
        Runtime::start(Database::in_memory_at(11).unwrap(), engine.clone(), || {}).unwrap();
    for (name, expected) in [
        ("textual", DocumentTextPresence::Available),
        ("scanned", DocumentTextPresence::Unavailable),
        ("5000-pages", DocumentTextPresence::Unavailable),
    ] {
        let path = root.join(format!("build/rust-fixtures/{name}.pdf"));
        let original = std::fs::read(&path).unwrap();
        let info = engine
            .open(name.into(), path.clone(), None, Cancellation::default())
            .unwrap();
        runtime
            .submit(Action::FindDocument(
                FindRequest::new(
                    name.into(),
                    PdfFindQuery::new("absent_phrase_731", false, false).unwrap(),
                    info.pages.len() as u32,
                    FindCursor::default(),
                    Cancellation::default(),
                )
                .unwrap(),
            ))
            .unwrap();
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(30))
            .unwrap()
        {
            Update::DocumentMatches(_, batch, _) => {
                assert_eq!(batch.text_presence, expected);
                assert_eq!(batch.matches().count(), 0);
            }
            _ => panic!("expected native text evidence"),
        }
        engine.close(name.into()).unwrap();
        assert_eq!(governor.usage().text, 0);
        assert_eq!(std::fs::read(path).unwrap(), original);
    }
}
