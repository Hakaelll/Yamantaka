use crossbeam_channel::{Sender, bounded};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use yamantaka_application::runtime::{Action, Runtime, Update, WriteKind};
use yamantaka_domain::{
    Cancellation, Failure, LibraryQuery, PdfEngine, PdfInfo, PdfPassword, Priority, RenderKey,
    RenderedPage, Result,
    memory::MemoryGovernor,
    pdf_text::{PdfPageText, TextGeometryPurpose},
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
    source_revision::SourceRevision,
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::Database;

type HeldReply = Arc<Mutex<Option<Sender<Result<PdfInfo>>>>>;
struct HeldSecondOpen {
    worker: PdfWorker,
    opens: AtomicUsize,
    held: HeldReply,
    entered: Sender<()>,
}

impl PdfEngine for HeldSecondOpen {
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
impl AsyncPdfEngine for HeldSecondOpen {
    fn memory_governor(&self) -> Option<MemoryGovernor> {
        Some(self.worker.memory_governor())
    }
    fn try_source_revision(
        &self,
        id: String,
        cancel: Cancellation,
    ) -> Result<PdfTicket<SourceRevision>> {
        self.worker.try_source_revision(id, cancel)
    }
    fn set_visible_documents(&self, ids: &[String]) -> Result<()> {
        self.worker.set_visible_documents(ids)
    }
    fn try_open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<PdfPassword>,
        _purpose: yamantaka_domain::pdf_ticket::PdfOpenPurpose,
        cancel: Cancellation,
    ) -> Result<PdfTicket<PdfInfo>> {
        if self.opens.fetch_add(1, Ordering::AcqRel) == 1 {
            let (reply, ticket) = PdfTicket::channel(99, 1, cancel);
            *self.held.lock().unwrap() = Some(reply);
            self.entered.try_send(()).unwrap();
            Ok(ticket)
        } else {
            self.worker.try_open(id, path, password, _purpose, cancel)
        }
    }
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
    fn try_close(&self, id: String) -> Result<PdfTicket<()>> {
        self.worker.try_close(id)
    }
    fn try_text_geometry(
        &self,
        id: String,
        page: u32,
        purpose: TextGeometryPurpose,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<PdfPageText>>> {
        self.worker.try_text_geometry(id, page, purpose, cancel)
    }
}

#[test]
fn a_committed_import_is_queryable_and_editable_while_the_next_native_open_is_held() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temporary = tempfile::tempdir().unwrap();
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let governor = worker.memory_governor();
    let held = Arc::new(Mutex::new(None));
    let (entered, waiting) = bounded(1);
    let engine = HeldSecondOpen {
        worker,
        opens: AtomicUsize::new(0),
        held: held.clone(),
        entered,
    };
    let runtime = Runtime::start(
        Database::open(&temporary.path().join("library.sqlite3")).unwrap(),
        engine,
        || {},
    )
    .unwrap();
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Import(
            vec![
                root.join("build/rust-fixtures/front-matter-missing.pdf"),
                root.join("build/rust-fixtures/front-matter-conflict.pdf"),
            ],
            cancel.clone(),
        ))
        .unwrap();
    waiting.recv_timeout(Duration::from_secs(15)).unwrap();
    assert_eq!(runtime.import_revision(), 1);
    assert!(runtime.updates.try_recv().is_err());
    runtime
        .submit(Action::Query(
            LibraryQuery::default(),
            Cancellation::default(),
        ))
        .unwrap();
    let Update::Library(documents, count, _) = runtime
        .updates
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
    else {
        panic!("Expected catalogue while import is held")
    };
    assert_eq!(count, 1);
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0].author.as_deref(), Some("Ada Lovelace"));
    runtime
        .submit(Action::Favorite(documents[0].id.clone(), true))
        .unwrap();
    assert!(matches!(
        runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap(),
        Update::Committed(_, WriteKind::Favorite)
    ));
    cancel.cancel();
    held.lock()
        .unwrap()
        .take()
        .unwrap()
        .try_send(Err(Failure::Cancelled))
        .unwrap();
    assert!(
        matches!(runtime.updates.recv_timeout(Duration::from_secs(5)).unwrap(),Update::Failed(Failure::Cancelled,Some(token)) if token.same_request(&cancel))
    );
    assert_eq!(runtime.import_revision(), 1);
    runtime
        .submit(Action::Query(
            LibraryQuery::default(),
            Cancellation::default(),
        ))
        .unwrap();
    let Update::Library(documents, count, _) = runtime
        .updates
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
    else {
        panic!("Expected committed catalogue after cancellation")
    };
    assert_eq!(count, 1);
    assert!(documents[0].favorite);
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
}
