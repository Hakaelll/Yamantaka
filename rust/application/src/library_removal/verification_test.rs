use super::*;
use crate::runtime::{Action, Runtime, Update};
use crossbeam_channel::{Receiver, Sender, bounded};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};
use yamantaka_domain::{
    LibraryQuery, PdfEngine, PdfInfo, PdfPassword, Priority, RenderKey, RenderedPage,
    pdf_ticket::PdfOpenPurpose,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;
struct CloseGate {
    id: String,
    reply: Sender<Result<()>>,
}
struct OpenGate {
    native: PdfTicket<PdfInfo>,
    reply: Sender<Result<PdfInfo>>,
}
struct Engine {
    worker: PdfWorker,
    closes: Sender<CloseGate>,
    opens: Sender<OpenGate>,
    hold_open: Arc<AtomicBool>,
    open_count: Arc<AtomicUsize>,
}
impl PdfEngine for Engine {
    fn open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<PdfPassword>,
        cancel: Cancellation,
    ) -> Result<PdfInfo> {
        self.worker.open(id, path, password, cancel)
    }
    fn render(
        &self,
        key: RenderKey,
        priority: Priority,
        cancel: Cancellation,
    ) -> Result<Arc<RenderedPage>> {
        self.worker.render(key, priority, cancel)
    }
    fn text(&self, id: String, page: u32, cancel: Cancellation) -> Result<String> {
        self.worker.text(id, page, cancel)
    }
    fn close(&self, id: String) -> Result<()> {
        self.worker.close(id)
    }
}
impl AsyncPdfEngine for Engine {
    fn try_render(
        &self,
        key: RenderKey,
        priority: Priority,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        self.worker.try_render(key, priority, cancel)
    }
    fn try_text(&self, id: String, page: u32, cancel: Cancellation) -> Result<PdfTicket<String>> {
        self.worker.try_text(id, page, cancel)
    }

    fn set_visible_documents(&self, ids: &[String]) -> Result<()> {
        self.worker.set_visible_documents(ids)
    }
    fn try_close(&self, id: String) -> Result<PdfTicket<()>> {
        let (reply, ticket) = PdfTicket::channel(700, 0, Cancellation::default());
        self.closes.send(CloseGate { id, reply }).unwrap();
        Ok(ticket)
    }
    fn try_open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<PdfPassword>,
        purpose: PdfOpenPurpose,
        cancel: Cancellation,
    ) -> Result<PdfTicket<PdfInfo>> {
        self.open_count.fetch_add(1, Ordering::AcqRel);
        let native = self
            .worker
            .try_open(id, path, password, purpose, cancel.clone())?;
        if self.hold_open.swap(false, Ordering::AcqRel) {
            let (reply, ticket) = PdfTicket::channel(701, 0, cancel);
            self.opens.send(OpenGate { native, reply }).unwrap();
            Ok(ticket)
        } else {
            Ok(native)
        }
    }
}
fn gate(receiver: &Receiver<CloseGate>, id: &str) -> CloseGate {
    let gate = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(gate.id, id);
    gate
}
fn release(gate: CloseGate, worker: &PdfWorker) {
    worker.close(gate.id).unwrap();
    gate.reply.send(Ok(())).unwrap();
}
fn remove(runtime: &Runtime, ids: &[&str]) -> Cancellation {
    let cancel = Cancellation::default();
    runtime
        .submit(Action::RemoveFromLibrary(Box::new(RemovalRequest {
            ids: ids.iter().map(|id| (*id).into()).collect(),
            cancel: cancel.clone(),
        })))
        .unwrap();
    cancel
}
fn reply(runtime: &Runtime, cancel: &Cancellation) -> BulkReport {
    loop {
        if let Update::SelectionOrganized(reply) = runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            && reply.cancel.same_request(cancel)
        {
            return reply.result.unwrap();
        }
    }
}
fn rejected(runtime: &Runtime, document: Document) {
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Open(document.clone(), None, cancel.clone()))
        .unwrap();
    loop {
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
        {
            Update::OpenFailed(failed, Failure::Invalid(_), scope)
                if scope.same_request(&cancel) =>
            {
                assert_eq!(failed.id, document.id);
                break;
            }
            Update::Opened(_, _, scope) if scope.same_request(&cancel) => {
                panic!("Stale source was reopened")
            }
            _ => {}
        }
    }
}
pub(crate) fn verify() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root
        .join("build/rust-fixtures/textual.pdf")
        .canonicalize()
        .unwrap();
    let original = std::fs::read(&source).unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("library.sqlite3");
    let db = Database::open(&path).unwrap();
    let encoded = source.to_str().unwrap().replace('\'', "''");
    for id in ["a", "b", "c", "d"] {
        db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,title_override,reading_progress) VALUES('{id}','{encoded}','{id}','fixture.pdf',2,900,'2026-10-20','2026-10-20','2026-10-20','synthetic','Saved title',0.5)")).unwrap();
    }
    let a = db.get("a").unwrap().unwrap();
    let c = db.get("c").unwrap().unwrap();
    let d = db.get("d").unwrap().unwrap();
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let governor = worker.memory_governor();
    for id in ["a", "b"] {
        worker
            .open(id.into(), source.clone(), None, Cancellation::default())
            .unwrap();
    }
    let (closes, gates) = bounded(8);
    let (opens, open_gates) = bounded(2);
    let hold_open = Arc::new(AtomicBool::new(false));
    let open_count = Arc::new(AtomicUsize::new(0));
    let runtime = Runtime::start(
        db,
        Engine {
            worker: worker.clone(),
            closes,
            opens,
            hold_open: hold_open.clone(),
            open_count: open_count.clone(),
        },
        || {},
    )
    .unwrap();
    let inspector = Database::open(&path).unwrap();
    runtime
        .submit(Action::VisibleDocuments(vec!["a".into(), "b".into()]))
        .unwrap();
    let cancel = remove(&runtime, &["a", "b", "missing", "a"]);
    let first = gate(&gates, "a");
    assert!(inspector.get("a").unwrap().is_some());
    assert!(worker.text("a".into(), 0, Cancellation::default()).is_ok());
    let import = Cancellation::default();
    runtime
        .submit(Action::Import(vec![source.clone()], import.clone()))
        .unwrap();
    let query = Cancellation::default();
    runtime
        .submit(Action::Query(LibraryQuery::default(), query.clone()))
        .unwrap();
    loop {
        if let Update::Library(_, count, scope) = runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            && scope.same_request(&query)
        {
            assert_eq!(count, 4);
            break;
        }
    }
    assert_eq!(inspector.pending_imports().unwrap().len(), 1);
    import.cancel();
    runtime.wake();
    rejected(&runtime, a.clone());
    assert_eq!(open_count.load(Ordering::Acquire), 0);
    release(first, &worker);
    let second = gate(&gates, "b");
    assert!(inspector.get("a").unwrap().is_none());
    assert!(inspector.get("b").unwrap().is_some());
    cancel.cancel();
    runtime.wake();
    release(second, &worker);
    let report = reply(&runtime, &cancel);
    assert_eq!(report.succeeded, ["a"]);
    assert_eq!(report.failed, ["missing"]);
    assert_eq!(report.remaining, ["b"]);
    assert_eq!(report.retired, ["a", "b"]);
    assert!(inspector.get("b").unwrap().is_some());
    for id in ["a", "b"] {
        assert!(worker.text(id.into(), 0, Cancellation::default()).is_err());
    }
    assert!(runtime.library_removal_progress().unwrap().finished);
    rejected(&runtime, a);
    assert_eq!(open_count.load(Ordering::Acquire), 0);
    let cancelled = Cancellation::default();
    cancelled.cancel();
    runtime
        .submit(Action::RemoveFromLibrary(Box::new(RemovalRequest {
            ids: vec!["b".into()],
            cancel: cancelled.clone(),
        })))
        .unwrap();
    loop {
        if let Update::SelectionOrganized(reply) = runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            && reply.cancel.same_request(&cancelled)
        {
            assert!(matches!(reply.result, Err(Failure::Cancelled)));
            break;
        }
    }
    assert!(gates.try_recv().is_err());
    worker
        .open("c".into(), source.clone(), None, Cancellation::default())
        .unwrap();
    let cancel = remove(&runtime, &["c"]);
    gate(&gates, "c")
        .reply
        .send(Err(Failure::Pdf("synthetic native close failure".into())))
        .unwrap();
    let failed = reply(&runtime, &cancel);
    assert_eq!(failed.failed, ["c"]);
    assert!(failed.retired.is_empty());
    assert!(inspector.get("c").unwrap().is_some());
    assert!(worker.text("c".into(), 0, Cancellation::default()).is_ok());
    inspector.execute_batch("CREATE TRIGGER reject_removal BEFORE DELETE ON documents WHEN OLD.id='c' BEGIN SELECT RAISE(ABORT,'synthetic SQL failure'); END;").unwrap();
    let cancel = remove(&runtime, &["c"]);
    release(gate(&gates, "c"), &worker);
    let failed = reply(&runtime, &cancel);
    assert_eq!(failed.failed, ["c"]);
    assert_eq!(failed.retired, ["c"]);
    assert_eq!(
        inspector.get("c").unwrap().unwrap().display_title(),
        "Saved title"
    );
    inspector
        .execute_batch("DROP TRIGGER reject_removal")
        .unwrap();
    let cancel = remove(&runtime, &["c"]);
    let changed = gate(&gates, "c");
    inspector
        .execute_batch("UPDATE documents SET fingerprint='newer source' WHERE id='c'")
        .unwrap();
    release(changed, &worker);
    let failed = reply(&runtime, &cancel);
    assert_eq!(failed.failed, ["c"]);
    assert_eq!(failed.retired, ["c"]);
    assert_eq!(
        inspector.get("c").unwrap().unwrap().fingerprint,
        "newer source"
    );
    rejected(&runtime, c);
    assert_eq!(open_count.load(Ordering::Acquire), 0);
    hold_open.store(true, Ordering::Release);
    let token = Cancellation::default();
    runtime
        .submit(Action::Open(d, None, token.clone()))
        .unwrap();
    let opening = open_gates.recv_timeout(Duration::from_secs(5)).unwrap();
    let info = opening.native.wait().unwrap();
    let mut editor = Database::open(&path).unwrap();
    editor
        .remove_from_library("d", &Cancellation::default())
        .unwrap();
    opening.reply.send(Ok(info)).unwrap();
    release(gate(&gates, "d"), &worker);
    loop {
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
        {
            Update::OpenFailed(document, Failure::Invalid(_), scope)
                if scope.same_request(&token) =>
            {
                assert_eq!(document.id, "d");
                break;
            }
            Update::Opened(_, _, scope) if scope.same_request(&token) => {
                panic!("A late native opening revived a removed record")
            }
            _ => {}
        }
    }
    assert!(worker.text("d".into(), 0, Cancellation::default()).is_err());
    assert_eq!(open_count.load(Ordering::Acquire), 1);
    assert_eq!(inspector.count(&LibraryQuery::default()).unwrap(), 2);
    drop(runtime);
    drop(worker);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(std::fs::read(source).unwrap(), original);
}
