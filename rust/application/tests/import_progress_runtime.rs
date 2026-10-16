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

type HeldGeometry = (
    PdfTicket<Arc<PdfPageText>>,
    Sender<Result<Arc<PdfPageText>>>,
);

type HeldReply = Arc<Mutex<Option<Sender<Result<PdfInfo>>>>>;
struct HeldSecondOpen {
    worker: PdfWorker,
    opens: AtomicUsize,
    held: HeldReply,
    entered: Sender<()>,
    first_text: Mutex<Option<Sender<HeldGeometry>>>,
    cover_attempts: Arc<AtomicUsize>,
    covers_before_second_open: Arc<AtomicUsize>,
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
            self.covers_before_second_open.store(
                self.cover_attempts.load(Ordering::Acquire),
                Ordering::Release,
            );
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
    fn try_cover(
        &self,
        id: String,
        path: PathBuf,
        page: u32,
        generation: u64,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        self.cover_attempts.fetch_add(1, Ordering::AcqRel);
        self.worker.try_cover(id, path, page, generation, cancel)
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
        let native = self
            .worker
            .try_text_geometry(id, page, purpose, cancel.clone())?;
        if let Some(held) = self.first_text.lock().unwrap().take() {
            let (sender, ticket) = PdfTicket::channel(100, 1, cancel);
            held.send((native, sender)).unwrap();
            Ok(ticket)
        } else {
            Ok(native)
        }
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
        first_text: Mutex::new(None),
        cover_attempts: Arc::new(AtomicUsize::new(0)),
        covers_before_second_open: Arc::new(AtomicUsize::new(0)),
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

#[test]
fn a_waiting_native_cover_is_retried_before_the_next_import_open() {
    use yamantaka_application::cover_source::CoverSource;
    use yamantaka_domain::LibraryRepository;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temporary = tempfile::tempdir().unwrap();
    let worker = PdfWorker::start(
        &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .unwrap();
    let governor = worker.memory_governor();
    let reader_path = root
        .join("build/rust-fixtures/textual.pdf")
        .canonicalize()
        .unwrap();
    let cover_path = root
        .join("build/rust-fixtures/automatic-cover.pdf")
        .canonicalize()
        .unwrap();
    let first_path = root
        .join("build/rust-fixtures/front-matter-missing.pdf")
        .canonicalize()
        .unwrap();
    let second_path = root
        .join("build/rust-fixtures/front-matter-conflict.pdf")
        .canonicalize()
        .unwrap();
    let originals = [
        reader_path.clone(),
        cover_path.clone(),
        first_path.clone(),
        second_path.clone(),
    ]
    .map(|path| {
        let bytes = std::fs::read(&path).unwrap();
        (path, bytes)
    });
    let reader = worker
        .open("reader".into(), reader_path, None, Cancellation::default())
        .unwrap();
    let database = Database::open(&temporary.path().join("library.sqlite3")).unwrap();
    let escaped = cover_path.to_string_lossy().replace('\'', "''");
    database.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('cover','{escaped}','cover','fixture.pdf',7,1,'now','now','now','synthetic'); INSERT INTO document_covers VALUES('cover',0,'','','old','old',1,'manual',1,'contain',0.75)")).unwrap();
    let cover = database.get("cover").unwrap().unwrap();
    let held = Arc::new(Mutex::new(None));
    let (entered, waiting) = bounded(1);
    let (hold_text, texts) = bounded(1);
    let cover_attempts = Arc::new(AtomicUsize::new(0));
    let covers_before_second_open = Arc::new(AtomicUsize::new(0));
    let engine = HeldSecondOpen {
        worker,
        opens: AtomicUsize::new(0),
        held: held.clone(),
        entered,
        first_text: Mutex::new(Some(hold_text)),
        cover_attempts: cover_attempts.clone(),
        covers_before_second_open: covers_before_second_open.clone(),
    };
    let runtime = Runtime::start(database, engine, || {}).unwrap();
    runtime
        .submit(Action::VisibleDocuments(vec!["reader".into()]))
        .unwrap();
    assert!(matches!(
        runtime
            .updates
            .recv_timeout(Duration::from_secs(5))
            .unwrap(),
        Update::Committed(_, WriteKind::VisibleDocuments)
    ));
    let importing = Cancellation::default();
    runtime
        .submit(Action::Import(
            vec![first_path, second_path],
            importing.clone(),
        ))
        .unwrap();
    let (native_text, release) = texts.recv_timeout(Duration::from_secs(10)).unwrap();
    let cover_cancel = Cancellation::default();
    runtime
        .submit(Action::Cover(
            CoverSource::capture(&cover),
            10,
            cover_cancel.clone(),
        ))
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while runtime.waiting_covers() != 1 {
        assert!(
            std::time::Instant::now() < deadline,
            "Real native capacity never parked the cover"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(cover_attempts.load(Ordering::Acquire), 1);
    assert_eq!(runtime.import_revision(), 0);
    release.send(native_text.wait()).unwrap();
    waiting.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(
        covers_before_second_open.load(Ordering::Acquire),
        2,
        "A waiting cover must be submitted before the next import open"
    );
    assert_eq!(runtime.import_revision(), 1);
    assert!(
        matches!(runtime.updates.recv_timeout(Duration::from_secs(10)).unwrap(),Update::Cover(id,Ok(image),token) if id=="cover" && image.key.page==0 && image.key.generation==10 && token.same_request(&cover_cancel))
    );
    runtime
        .submit(Action::Text("reader".into(), 0, Cancellation::default()))
        .unwrap();
    assert!(
        matches!(runtime.updates.recv_timeout(Duration::from_secs(5)).unwrap(),Update::Text(id,0,text,_) if id=="reader" && text.contains("Yamantaka Rust"))
    );
    importing.cancel();
    held.lock()
        .unwrap()
        .take()
        .unwrap()
        .send(Err(Failure::Cancelled))
        .unwrap();
    assert!(
        matches!(runtime.updates.recv_timeout(Duration::from_secs(5)).unwrap(),Update::Failed(Failure::Cancelled,Some(token)) if token.same_request(&importing))
    );
    drop(reader);
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
    for (path, bytes) in originals {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
