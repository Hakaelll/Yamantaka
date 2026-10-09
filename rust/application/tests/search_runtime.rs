use crossbeam_channel::{Receiver, Sender, bounded};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use yamantaka_application::{
    runtime::{Action, Runtime, Update, WriteKind},
    search_index::{IndexCommand, IndexPhase},
};
use yamantaka_domain::{
    Cancellation, Document, PdfEngine, PdfInfo, Priority, RenderKey, RenderedPage, Result,
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
    search::SearchQuery,
};
use yamantaka_storage::Database;

enum Native {
    Open(String, Cancellation, Sender<Result<PdfInfo>>),
    Text(u32, Cancellation, Sender<Result<String>>),
    Outline(
        Cancellation,
        Sender<Result<Arc<yamantaka_domain::outline::PdfOutline>>>,
    ),
    Close(String, Sender<Result<()>>),
}
struct Engine(Sender<Native>, Arc<Mutex<Vec<Vec<String>>>>);
impl PdfEngine for Engine {
    fn open(
        &self,
        _: String,
        _: PathBuf,
        _: Option<yamantaka_domain::PdfPassword>,
        _: Cancellation,
    ) -> Result<PdfInfo> {
        panic!("actor must not wait for open")
    }
    fn render(&self, _: RenderKey, _: Priority, _: Cancellation) -> Result<Arc<RenderedPage>> {
        panic!("indexing must not render")
    }
    fn text(&self, _: String, _: u32, _: Cancellation) -> Result<String> {
        panic!("actor must not wait for text")
    }
    fn close(&self, _: String) -> Result<()> {
        panic!("actor must not wait for close")
    }
}
impl AsyncPdfEngine for Engine {
    fn set_visible_documents(&self, ids: &[String]) -> Result<()> {
        self.1.lock().unwrap().push(ids.to_vec());
        Ok(())
    }
    fn try_open(
        &self,
        id: String,
        _: PathBuf,
        _: Option<yamantaka_domain::PdfPassword>,
        cancel: Cancellation,
    ) -> Result<PdfTicket<PdfInfo>> {
        let (send, ticket) = PdfTicket::channel(1, 1, cancel.clone());
        self.0.try_send(Native::Open(id, cancel, send)).unwrap();
        Ok(ticket)
    }
    fn try_render(
        &self,
        _: RenderKey,
        _: Priority,
        _: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        panic!("indexing must not render")
    }
    fn try_text(&self, _: String, page: u32, cancel: Cancellation) -> Result<PdfTicket<String>> {
        let (send, ticket) = PdfTicket::channel(2, 1, cancel.clone());
        self.0.try_send(Native::Text(page, cancel, send)).unwrap();
        Ok(ticket)
    }
    fn try_close(&self, id: String) -> Result<PdfTicket<()>> {
        let (send, ticket) = PdfTicket::channel(3, 1, Cancellation::default());
        self.0.try_send(Native::Close(id, send)).unwrap();
        Ok(ticket)
    }
    fn try_outline(
        &self,
        _: String,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<yamantaka_domain::outline::PdfOutline>>> {
        let (send, ticket) = PdfTicket::channel(4, 1, cancel.clone());
        self.0.try_send(Native::Outline(cancel, send)).unwrap();
        Ok(ticket)
    }
}
fn setup() -> (Runtime, Receiver<Native>, Receiver<()>) {
    setup_with_failure(false)
}
fn setup_with_failure(broken_index: bool) -> (Runtime, Receiver<Native>, Receiver<()>) {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES ('a','a.pdf','a.pdf','a.pdf','Astronomy',3,1,'now','now','now','fa');").unwrap();
    if broken_index {
        db.execute_batch("DROP TABLE search_index_state").unwrap();
    }
    start_database(db)
}
fn start_database(db: Database) -> (Runtime, Receiver<Native>, Receiver<()>) {
    start_database_with_pins(db, Arc::default())
}
fn start_database_with_pins(
    db: Database,
    pins: Arc<Mutex<Vec<Vec<String>>>>,
) -> (Runtime, Receiver<Native>, Receiver<()>) {
    let (send, native) = bounded(16);
    let (wake, notified) = bounded(1);
    (
        Runtime::start(db, Engine(send, pins), move || {
            let _ = wake.try_send(());
        })
        .unwrap(),
        native,
        notified,
    )
}

#[test]
fn result_lookup_reaches_documents_outside_the_catalogue_page_and_reads_exclusion_state() {
    let db = Database::in_memory_at(11).unwrap();
    for index in 0..81 {
        db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES ('synthetic-{index:03}','synthetic-{index:03}.pdf','synthetic-{index:03}.pdf','synthetic-{index:03}.pdf','Synthetic study document',3,1,'now','now','now','synthetic-{index:03}');")).unwrap();
    }
    let (runtime, native, _) = start_database(db);
    runtime
        .submit(Action::Query(Default::default(), Cancellation::default()))
        .unwrap();
    let Update::Library(documents, count, _) = update(&runtime) else {
        panic!("expected catalogue page")
    };
    assert_eq!(count, 81);
    assert_eq!(documents.len(), 80);
    let id = (0..81)
        .map(|index| format!("synthetic-{index:03}"))
        .find(|id| documents.iter().all(|document| document.id != *id))
        .unwrap();
    runtime
        .submit(Action::GetDocument(id.clone(), Cancellation::default()))
        .unwrap();
    assert!(
        matches!(update(&runtime), Update::Document(actual, Some(document), cancel)
        if actual == id && document.id == id && document.page_count == 3 && !cancel.is_cancelled())
    );
    runtime
        .submit(Action::IndexState(id.clone(), Cancellation::default()))
        .unwrap();
    assert!(matches!(update(&runtime), Update::IndexState(actual, None, _) if actual == id));
    runtime
        .submit(Action::Index(IndexCommand::Exclude {
            document_id: id.clone(),
            excluded: true,
        }))
        .unwrap();
    committed(&runtime, WriteKind::Index);
    runtime
        .submit(Action::IndexState(id.clone(), Cancellation::default()))
        .unwrap();
    assert!(
        matches!(update(&runtime), Update::IndexState(actual, Some(state), _)
        if actual == id && state.status == yamantaka_domain::search::IndexStatus::Excluded)
    );
    runtime.submit(Action::IndexStatistics).unwrap();
    assert!(matches!(update(&runtime), Update::IndexStatistics(stats) if stats.excluded == 1));
    runtime
        .submit(Action::GetDocument(
            "absent".into(),
            Cancellation::default(),
        ))
        .unwrap();
    assert!(matches!(update(&runtime), Update::Document(id, None, _) if id == "absent"));
    let cancel = Cancellation::default();
    cancel.cancel();
    runtime.submit(Action::GetDocument(id, cancel)).unwrap();
    runtime
        .submit(Action::Query(Default::default(), Cancellation::default()))
        .unwrap();
    assert!(matches!(update(&runtime), Update::Library(_, 81, _)));
    assert!(native.try_recv().is_err());
}

#[test]
fn storage_failure_aborts_once_and_retains_a_redacted_snapshot_without_blocking_sql() {
    let (runtime, native, notified) = setup_with_failure(true);
    start(&runtime);
    phase(&runtime, &notified, IndexPhase::Cancelled);
    for _ in 0..20 {
        query_barrier(&runtime);
    }
    runtime
        .submit(Action::VisibleDocuments(vec![
            "left".into(),
            "right".into(),
        ]))
        .unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
    query_barrier(&runtime);
    let progress = runtime.index_progress();
    assert_eq!(progress.phase, IndexPhase::Cancelled);
    assert_eq!(progress.failed_documents, 1);
    assert_eq!(progress.last_error_code, Some("storage"));
    assert!(native.try_recv().is_err());
}

#[test]
fn shutdown_cancels_index_open_and_joins_without_draining_saturated_ui_results() {
    let (runtime, native, _) = setup();
    start(&runtime);
    let Native::Open(id, cancel, open) = next(&native) else {
        panic!("expected index open")
    };
    for _ in 0..128 {
        runtime.submit(Action::Favorite("a".into(), true)).unwrap();
    }
    assert!(runtime.submit(Action::Favorite("a".into(), false)).is_err());
    let (done, joined) = bounded(1);
    let thread = std::thread::spawn(move || {
        drop(runtime);
        done.send(()).unwrap();
    });
    let Native::Close(closing, close) = next(&native) else {
        panic!("expected shutdown close")
    };
    assert_eq!(closing, id);
    assert!(cancel.is_cancelled());
    close.send(Ok(())).unwrap();
    joined.recv_timeout(Duration::from_secs(5)).unwrap();
    thread.join().unwrap();
    assert!(
        open.send(Ok(info())).is_err(),
        "shutdown must discard the pending open ticket"
    );
}
fn info() -> PdfInfo {
    PdfInfo {
        pages: {
            let governor = yamantaka_domain::memory::MemoryGovernor::default();
            let mut pages =
                yamantaka_domain::pdf_pages::PdfPagesBuilder::new(3, &governor).unwrap();
            let coordinates = yamantaka_domain::pdf_geometry::PageCoordinateSpace::new(
                [0.0, 800.0],
                [600.0, 800.0],
                [0.0, 0.0],
            )
            .unwrap();
            for _ in 0..3 {
                pages
                    .push(
                        yamantaka_domain::pdf_pages::PdfPageInfo::new([600.0, 800.0], coordinates)
                            .unwrap(),
                    )
                    .unwrap();
            }
            pages.finish().unwrap()
        },
        title: None,
        author: None,
    }
}
fn next(native: &Receiver<Native>) -> Native {
    native.recv_timeout(Duration::from_secs(5)).unwrap()
}
fn update(runtime: &Runtime) -> Update {
    runtime
        .updates
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
}
fn committed(runtime: &Runtime, kind: WriteKind) {
    assert!(matches!(update(runtime), Update::Committed(_, actual) if actual == kind));
}
fn start(runtime: &Runtime) {
    runtime
        .submit(Action::Index(IndexCommand::Enable(true)))
        .unwrap();
    committed(runtime, WriteKind::Index);
    runtime
        .submit(Action::Index(IndexCommand::Start { rebuild: false }))
        .unwrap();
    committed(runtime, WriteKind::Index);
}
fn phase(runtime: &Runtime, notified: &Receiver<()>, expected: IndexPhase) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while runtime.index_progress().phase != expected {
        notified
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap();
    }
}
fn query_barrier(runtime: &Runtime) {
    runtime
        .submit(Action::Query(Default::default(), Cancellation::default()))
        .unwrap();
    assert!(matches!(update(runtime), Update::Library(_, 1, _)));
}
fn reader() -> Document {
    Document {
        id: "reader".into(),
        path: PathBuf::from("reader.pdf"),
        file_name: "reader.pdf".into(),
        title: None,
        author: None,
        page_count: 3,
        last_page_index: 0,
        reading_progress: 0.0,
        favorite: false,
        missing: false,
        fingerprint: "reader-revision".into(),
        thumbnail_path: None,
    }
}

#[test]
fn outline_tickets_keep_catalogue_queries_responsive_and_preserve_request_identity() {
    use yamantaka_domain::{
        Failure,
        memory::{Cost, GlobalMemoryLimits, MemoryCategory, MemoryGovernor, MemoryRepresentation},
        outline::{OutlineEntry, PdfOutline},
        pdf_destination::{DestinationView, NavigationDestination},
    };
    let (runtime, native, _) = setup();
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Outline("a".into(), cancel.clone()))
        .unwrap();
    let Native::Outline(native_cancel, reply) = next(&native) else {
        panic!("outline request");
    };
    query_barrier(&runtime);
    assert!(native_cancel.same_request(&cancel));
    let governor = MemoryGovernor::new(GlobalMemoryLimits::default());
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            1024,
        ))
        .unwrap();
    let outline = Arc::new(
        PdfOutline::from_reserved(
            vec![OutlineEntry {
                title: "Chapter One".into(),
                parent: None,
                destination: Some(NavigationDestination {
                    page: 2,
                    content_bounds: None,
                    coordinates: yamantaka_domain::pdf_geometry::PageCoordinateSpace::new(
                        [0.0, 200.0],
                        [200.0, 200.0],
                        [0.0, 0.0],
                    )
                    .unwrap(),
                    view: DestinationView::FitPage,
                }),
            }],
            lease,
        )
        .unwrap(),
    );
    reply.send(Ok(outline)).unwrap();
    let Update::Outline(id, outline, token) = update(&runtime) else {
        panic!("outline response");
    };
    assert_eq!(id, "a");
    assert_eq!(outline.entries()[0].title, "Chapter One");
    assert_eq!(outline.entries()[0].destination.unwrap().page, 2);
    assert!(token.same_request(&cancel));
    drop(outline);
    assert_eq!(governor.usage().text, 0);
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Outline("a".into(), cancel.clone()))
        .unwrap();
    let Native::Outline(_, reply) = next(&native) else {
        panic!("second outline request");
    };
    cancel.cancel();
    reply.send(Err(Failure::Cancelled)).unwrap();
    assert!(
        matches!(update(&runtime), Update::OutlineFailed(id,Failure::Cancelled,token) if id=="a" && token.same_request(&cancel))
    );
    query_barrier(&runtime);
}

#[test]
fn bookmark_commands_remain_responsive_during_native_work_and_preserve_scoped_cancellation() {
    use yamantaka_domain::{Failure, bookmarks::BookmarkChange};
    let (runtime, native, notified) = setup();
    start(&runtime);
    let Native::Open(_, native_cancel, native_reply) = next(&native) else {
        panic!("held index open")
    };
    let write = Cancellation::default();
    runtime
        .submit(Action::ChangeBookmark(
            "a".into(),
            BookmarkChange::Save {
                page: 2,
                label: Some("Study landmark".into()),
            },
            0,
            write.clone(),
        ))
        .unwrap();
    let Update::Bookmarks(id, requested, page, token) = update(&runtime) else {
        panic!("bookmark write snapshot")
    };
    assert_eq!((id.as_str(), requested, page.total), ("a", 0, 1));
    assert!(token.same_request(&write));
    let bookmark = page.entries[0].clone();
    assert_eq!(bookmark.page_index, 2);
    runtime
        .submit(Action::ChangeBookmark(
            "a".into(),
            BookmarkChange::Rename {
                id: bookmark.id.clone(),
                label: Some("  Résumé · دراسة  ".into()),
            },
            0,
            Cancellation::default(),
        ))
        .unwrap();
    let Update::Bookmarks(_, _, page, _) = update(&runtime) else {
        panic!("renamed bookmark snapshot")
    };
    assert_eq!(page.entries[0].id, bookmark.id);
    assert_eq!(page.entries[0].created_at, bookmark.created_at);
    assert_eq!(page.entries[0].label.as_deref(), Some("Résumé · دراسة"));
    runtime
        .submit(Action::Search(
            SearchQuery {
                text: "Résumé".into(),
                kinds: vec![yamantaka_domain::search::SearchKind::Bookmark],
                ..Default::default()
            },
            Cancellation::default(),
        ))
        .unwrap();
    let Update::Search(results, _) = update(&runtime) else {
        panic!("immediate bookmark search");
    };
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].page, Some(2));
    let cancel = Cancellation::default();
    cancel.cancel();
    runtime
        .submit(Action::ChangeBookmark(
            "a".into(),
            BookmarkChange::Remove {
                id: bookmark.id.clone(),
            },
            0,
            cancel.clone(),
        ))
        .unwrap();
    assert!(
        matches!(update(&runtime), Update::BookmarkFailed(id,Failure::Cancelled,token) if id=="a" && token.same_request(&cancel))
    );
    runtime
        .submit(Action::Bookmarks(
            "a".into(),
            usize::MAX,
            Cancellation::default(),
        ))
        .unwrap();
    let Update::Bookmarks(id, requested, page, _) = update(&runtime) else {
        panic!("bounded bookmark query")
    };
    assert_eq!(
        (id.as_str(), requested, page.offset, page.total),
        ("a", usize::MAX, 0, 1)
    );
    assert_eq!(page.entries[0].id, bookmark.id);
    runtime
        .submit(Action::Bookmarks("a".into(), 0, cancel))
        .unwrap();
    assert!(matches!(
        update(&runtime),
        Update::BookmarkFailed(_, Failure::Cancelled, _)
    ));
    query_barrier(&runtime);
    assert!(
        native.try_recv().is_err(),
        "bookmark SQL must not issue or wait for PDF calls"
    );
    runtime
        .submit(Action::ChangeBookmark(
            "a".into(),
            BookmarkChange::Remove { id: bookmark.id },
            0,
            Cancellation::default(),
        ))
        .unwrap();
    assert!(matches!(update(&runtime), Update::Bookmarks(_,0,page,_) if page.total==0));
    runtime
        .submit(Action::Search(
            SearchQuery {
                text: "Résumé".into(),
                kinds: vec![yamantaka_domain::search::SearchKind::Bookmark],
                ..Default::default()
            },
            Cancellation::default(),
        ))
        .unwrap();
    assert!(matches!(update(&runtime), Update::Search(results, _) if results.is_empty()));
    runtime.submit(Action::Index(IndexCommand::Cancel)).unwrap();
    committed(&runtime, WriteKind::Index);
    assert!(native_cancel.is_cancelled());
    native_reply.send(Err(Failure::Cancelled)).unwrap();
    let Native::Close(_, reply) = next(&native) else {
        panic!("cancelled index close")
    };
    reply.send(Ok(())).unwrap();
    phase(&runtime, &notified, IndexPhase::Cancelled);
}

#[test]
fn open_confirmation_keeps_old_pins_and_rejects_cancelled_or_stale_acceptance() {
    let pins = Arc::new(Mutex::new(Vec::new()));
    let (runtime, native, _) =
        start_database_with_pins(Database::in_memory_at(11).unwrap(), pins.clone());
    let mut old = reader();
    old.id = "old".into();
    runtime
        .submit(Action::Open(old, None, Cancellation::default()))
        .unwrap();
    let Native::Open(_, _, reply) = next(&native) else {
        panic!("old reader open")
    };
    reply.send(Ok(info())).unwrap();
    let Update::Opened(doc, _, token) = update(&runtime) else {
        panic!("opened old reader")
    };
    runtime.submit(Action::ConfirmOpen(doc.id, token)).unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
    assert_eq!(pins.lock().unwrap().last().unwrap(), &["old"]);

    let replacement = reader();
    runtime
        .submit(Action::Open(
            replacement.clone(),
            None,
            Cancellation::default(),
        ))
        .unwrap();
    let Native::Open(_, _, reply) = next(&native) else {
        panic!("replacement open")
    };
    assert_eq!(pins.lock().unwrap().last().unwrap(), &["old", "reader"]);
    reply.send(Ok(info())).unwrap();
    let Update::Opened(doc, _, token) = update(&runtime) else {
        panic!("replacement result")
    };
    runtime
        .submit(Action::ConfirmOpen(doc.id.clone(), Cancellation::default()))
        .unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
    assert_eq!(
        pins.lock().unwrap().last().unwrap(),
        &["old", "reader"],
        "an unrelated token cannot accept an open"
    );
    token.cancel();
    runtime.submit(Action::ConfirmOpen(doc.id, token)).unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
    assert_eq!(pins.lock().unwrap().last().unwrap(), &["old"]);

    let first = Cancellation::default();
    runtime
        .submit(Action::Open(replacement.clone(), None, first.clone()))
        .unwrap();
    let Native::Open(_, _, first_reply) = next(&native) else {
        panic!("first replacement")
    };
    let second = Cancellation::default();
    runtime
        .submit(Action::Open(replacement, None, second.clone()))
        .unwrap();
    let Native::Open(_, _, second_reply) = next(&native) else {
        panic!("newer replacement")
    };
    assert!(first.is_cancelled());
    first_reply.send(Ok(info())).unwrap();
    assert!(matches!(
        update(&runtime),
        Update::OpenFailed(_, yamantaka_domain::Failure::Cancelled, _)
    ));
    assert_eq!(
        pins.lock().unwrap().last().unwrap(),
        &["old", "reader"],
        "older completion must not restore pins over newer admission"
    );
    second_reply.send(Ok(info())).unwrap();
    let Update::Opened(doc, _, token) = update(&runtime) else {
        panic!("latest result")
    };
    let mut next_document = reader();
    next_document.id = "next".into();
    runtime
        .submit(Action::Open(next_document, None, Cancellation::default()))
        .unwrap();
    runtime
        .submit(Action::Query(Default::default(), Cancellation::default()))
        .unwrap();
    assert!(matches!(update(&runtime), Update::Library(_, 0, _)));
    assert!(
        native.try_recv().is_err(),
        "a later open must await the accepted reader's confirmation"
    );
    runtime.submit(Action::ConfirmOpen(doc.id, token)).unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
    let Native::Open(_, _, reply) = next(&native) else {
        panic!("deferred next open")
    };
    assert_eq!(pins.lock().unwrap().last().unwrap(), &["reader", "next"]);
    reply
        .send(Err(yamantaka_domain::Failure::PasswordRequired))
        .unwrap();
    assert!(matches!(
        update(&runtime),
        Update::OpenFailed(_, yamantaka_domain::Failure::PasswordRequired, _)
    ));
    assert_eq!(pins.lock().unwrap().last().unwrap(), &["reader"]);
}

#[test]
fn confirmation_remains_admissible_when_data_results_fill_all_reservations() {
    let pins = Arc::new(Mutex::new(Vec::new()));
    let (runtime, native, _) =
        start_database_with_pins(Database::in_memory_at(11).unwrap(), pins.clone());
    runtime
        .submit(Action::Open(reader(), None, Cancellation::default()))
        .unwrap();
    let Native::Open(_, _, reply) = next(&native) else {
        panic!("reader open")
    };
    reply.send(Ok(info())).unwrap();
    let Update::Opened(doc, _, token) = update(&runtime) else {
        panic!("reader result")
    };
    for _ in 0..128 {
        runtime
            .submit(Action::Query(Default::default(), Cancellation::default()))
            .unwrap();
    }
    assert!(
        runtime
            .submit(Action::Query(Default::default(), Cancellation::default()))
            .is_err()
    );
    runtime
        .submit(Action::ConfirmOpen(doc.id, token))
        .expect("confirmation uses reserved control capacity");
    for _ in 0..128 {
        assert!(matches!(update(&runtime), Update::Library(_, _, _)));
    }
    committed(&runtime, WriteKind::VisibleDocuments);
    assert_eq!(pins.lock().unwrap().last().unwrap(), &["reader"]);
}

#[test]
fn import_waits_for_ui_confirmation_without_blocking_catalogue_queries() {
    let (runtime, native, _) = start_database(Database::in_memory_at(11).unwrap());
    runtime
        .submit(Action::Open(reader(), None, Cancellation::default()))
        .unwrap();
    let Native::Open(_, _, reply) = next(&native) else {
        panic!("reader open")
    };
    reply.send(Ok(info())).unwrap();
    let Update::Opened(doc, _, token) = update(&runtime) else {
        panic!("reader result")
    };
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../build/rust-fixtures/textual.pdf")
        .canonicalize()
        .unwrap();
    runtime
        .submit(Action::Import(vec![fixture], Cancellation::default()))
        .unwrap();
    runtime
        .submit(Action::Query(Default::default(), Cancellation::default()))
        .unwrap();
    assert!(matches!(update(&runtime), Update::Library(_, 0, _)));
    assert!(
        native.try_recv().is_err(),
        "import must retain its reservation until acceptance"
    );
    runtime.submit(Action::ConfirmOpen(doc.id, token)).unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
    let Native::Open(_, _, reply) = next(&native) else {
        panic!("deferred import open")
    };
    reply.send(Ok(info())).unwrap();
    let Native::Close(_, reply) = next(&native) else {
        panic!("import inspection close")
    };
    reply.send(Ok(())).unwrap();
    assert!(matches!(update(&runtime), Update::Imported(1, _)));
}

#[test]
fn shutdown_cancels_a_delivered_open_that_the_ui_has_not_confirmed() {
    let (runtime, native, _) = start_database(Database::in_memory_at(11).unwrap());
    runtime
        .submit(Action::Open(reader(), None, Cancellation::default()))
        .unwrap();
    let Native::Open(_, _, reply) = next(&native) else {
        panic!("reader open")
    };
    reply.send(Ok(info())).unwrap();
    let Update::Opened(_, _, token) = update(&runtime) else {
        panic!("reader result")
    };
    drop(runtime);
    assert!(token.is_cancelled());
}

#[test]
fn actor_requires_consent_and_keeps_sql_responsive_during_cancelled_native_open() {
    let (runtime, native, notified) = setup();
    runtime
        .submit(Action::Index(IndexCommand::Start { rebuild: false }))
        .unwrap();
    assert!(matches!(update(&runtime), Update::Failed(_, None)));
    runtime
        .submit(Action::Search(
            SearchQuery {
                text: "Astronomy".into(),
                ..Default::default()
            },
            Cancellation::default(),
        ))
        .unwrap();
    assert!(matches!(update(&runtime), Update::SearchFailed(_, _)));
    assert!(native.try_recv().is_err());
    start(&runtime);
    let Native::Open(_, cancel, send) = next(&native) else {
        panic!("expected index open")
    };
    query_barrier(&runtime);
    runtime.submit(Action::Index(IndexCommand::Cancel)).unwrap();
    committed(&runtime, WriteKind::Index);
    assert!(cancel.is_cancelled());
    send.send(Ok(info())).unwrap();
    let Native::Close(_, close) = next(&native) else {
        panic!("expected cancelled close")
    };
    close.send(Ok(())).unwrap();
    phase(&runtime, &notified, IndexPhase::Cancelled);
    assert!(native.try_recv().is_err());
    assert_eq!(runtime.index_progress().failed_documents, 0);
}

#[test]
fn foreground_open_waits_for_index_close_and_indexing_resumes_without_late_text() {
    let (runtime, native, notified) = setup();
    start(&runtime);
    let Native::Open(index_id, _, send) = next(&native) else {
        panic!("expected index open")
    };
    send.send(Ok(info())).unwrap();
    let Native::Text(0, _, send) = next(&native) else {
        panic!("expected first page")
    };
    send.send(Ok("Committed astronomy".into())).unwrap();
    let Native::Text(1, cancel, send) = next(&native) else {
        panic!("expected second page")
    };
    runtime
        .submit(Action::Open(reader(), None, Cancellation::default()))
        .unwrap();
    query_barrier(&runtime);
    assert!(cancel.is_cancelled());
    send.send(Ok("Obsolete astronomy".into())).unwrap();
    let Native::Close(id, close) = next(&native) else {
        panic!("expected index close")
    };
    assert_eq!(id, index_id);
    query_barrier(&runtime);
    assert!(
        native.try_recv().is_err(),
        "reader must wait for the close reply"
    );
    close.send(Ok(())).unwrap();
    let Native::Open(id, _, send) = next(&native) else {
        panic!("expected reader open")
    };
    assert_eq!(id, "reader");
    query_barrier(&runtime);
    assert!(
        native.try_recv().is_err(),
        "index must wait for reader inspection"
    );
    send.send(Ok(info())).unwrap();
    let Update::Opened(document, _, token) = update(&runtime) else {
        panic!("expected opened reader")
    };
    assert_eq!(document.id, "reader");
    query_barrier(&runtime);
    assert!(
        native.try_recv().is_err(),
        "index must await UI acceptance of the open result"
    );
    runtime
        .submit(Action::ConfirmOpen(document.id, token))
        .unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
    let Native::Open(id, _, send) = next(&native) else {
        panic!("expected index resume")
    };
    assert_eq!(id, index_id);
    send.send(Ok(info())).unwrap();
    for expected in [1, 2] {
        let Native::Text(page, _, send) = next(&native) else {
            panic!("expected resumed page")
        };
        assert_eq!(page, expected);
        send.send(Ok("Resumed study".into())).unwrap();
    }
    let Native::Close(_, send) = next(&native) else {
        panic!("expected final close")
    };
    send.send(Ok(())).unwrap();
    phase(&runtime, &notified, IndexPhase::Completed);
    for (text, count) in [("Committed", 1), ("Obsolete", 0), ("Resumed", 2)] {
        runtime
            .submit(Action::Search(
                SearchQuery {
                    text: text.into(),
                    ..Default::default()
                },
                Cancellation::default(),
            ))
            .unwrap();
        assert!(
            matches!(update(&runtime), Update::Search(results, cancel) if results.len() == count && !cancel.is_cancelled())
        );
    }
    assert_eq!(runtime.index_progress().completed_documents, 1);
    assert_eq!(runtime.index_progress().failed_documents, 0);
}

#[test]
fn two_visible_slots_pause_indexing_and_cancelled_import_releases_its_reservation() {
    let (runtime, native, notified) = setup();
    runtime
        .submit(Action::VisibleDocuments(vec![
            "left".into(),
            "right".into(),
        ]))
        .unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
    start(&runtime);
    phase(&runtime, &notified, IndexPhase::Paused);
    query_barrier(&runtime);
    assert!(native.try_recv().is_err());
    runtime
        .submit(Action::VisibleDocuments(vec!["left".into()]))
        .unwrap();
    committed(&runtime, WriteKind::VisibleDocuments);
    let Native::Open(_, cancel, send) = next(&native) else {
        panic!("expected index open")
    };
    let import_cancel = Cancellation::default();
    runtime
        .submit(Action::Import(
            vec![PathBuf::from("import.pdf")],
            import_cancel.clone(),
        ))
        .unwrap();
    query_barrier(&runtime);
    assert!(cancel.is_cancelled());
    import_cancel.cancel();
    send.send(Ok(info())).unwrap();
    let Native::Close(_, close) = next(&native) else {
        panic!("expected index close")
    };
    close.send(Ok(())).unwrap();
    // The cancelled importer must never open its PDF; the next request resumes
    // indexing. Cancel and complete this run before dropping the runtime.
    let Native::Open(id, _, send) = next(&native) else {
        panic!("expected resumed index")
    };
    assert!(id.starts_with("index-"));
    runtime.submit(Action::Index(IndexCommand::Cancel)).unwrap();
    committed(&runtime, WriteKind::Index);
    send.send(Ok(info())).unwrap();
    let Native::Close(_, close) = next(&native) else {
        panic!("expected cancelled index close")
    };
    close.send(Ok(())).unwrap();
    phase(&runtime, &notified, IndexPhase::Cancelled);
    query_barrier(&runtime);
    assert!(native.try_recv().is_err());
}
