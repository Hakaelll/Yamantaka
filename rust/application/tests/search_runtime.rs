use crossbeam_channel::{Receiver, Sender, bounded};
use std::{
    path::PathBuf,
    sync::Arc,
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
    Close(String, Sender<Result<()>>),
}
struct Engine(Sender<Native>);
impl PdfEngine for Engine {
    fn open(&self, _: String, _: PathBuf, _: Option<String>, _: Cancellation) -> Result<PdfInfo> {
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
    fn set_visible_documents(&self, _: &[String]) -> Result<()> {
        Ok(())
    }
    fn try_open(
        &self,
        id: String,
        _: PathBuf,
        _: Option<String>,
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
    let (send, native) = bounded(16);
    let (wake, notified) = bounded(1);
    (
        Runtime::start(db, Engine(send), move || {
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
        page_sizes: vec![[600.0, 800.0]; 3],
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
        .submit(Action::Open(reader(), Cancellation::default()))
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
    assert!(matches!(update(&runtime), Update::Opened(document, _, _) if document.id == "reader"));
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
