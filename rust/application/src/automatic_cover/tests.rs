use crate::{
    manual_cover::ManualCoverRequest,
    runtime::{Action, Runtime, Update},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use yamantaka_domain::{
    Cancellation, Document, LibraryRepository, PdfEngine, PdfInfo, PdfPassword, Priority,
    RenderKey, RenderedPage, Result,
    cover_policy::DetectedCover,
    cover_selection::ManualCover,
    memory::MemoryGovernor,
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
};
use yamantaka_pdf::{MemoryLimits, PdfWorker};
use yamantaka_storage::{Database, cover_cache::CoverCache};

pub(super) fn setup(path: &Path) -> (Database, Document, PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let pdf = root
        .join("build/rust-fixtures/automatic-cover.pdf")
        .canonicalize()
        .unwrap();
    let dll = root.join("build/native/pdfium-7881/bin/pdfium.dll");
    let db = Database::open(path).unwrap();
    let escaped = pdf.to_string_lossy().replace('\'', "''");
    db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,thumbnail_path) VALUES('auto','{escaped}','auto','fixture.pdf',7,1,'2026-10-13','now','now','synthetic','prior-cache');")).unwrap();
    let doc = db.get("auto").unwrap().unwrap();
    (db, doc, pdf, dll)
}
pub(super) fn selection(path: &Path) -> Option<(u32, String, u32, f64)> {
    use rusqlite::OptionalExtension;
    let db =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    db.query_row("SELECT source_page_index,selection_mode,algorithm_version,confidence FROM document_covers WHERE document_id='auto'",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).optional().unwrap()
}
pub(super) fn receive(runtime: &Runtime) -> Update {
    runtime
        .updates
        .recv_timeout(Duration::from_secs(10))
        .unwrap()
}

pub(super) struct Gate {
    pub(super) native: PdfWorker,
    pub(super) ready: std::sync::mpsc::Sender<crossbeam_channel::Sender<Result<DetectedCover>>>,
}
impl PdfEngine for Gate {
    fn open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<PdfPassword>,
        cancel: Cancellation,
    ) -> Result<PdfInfo> {
        self.native.open(id, path, password, cancel)
    }
    fn render(
        &self,
        key: RenderKey,
        priority: Priority,
        cancel: Cancellation,
    ) -> Result<Arc<RenderedPage>> {
        self.native.render(key, priority, cancel)
    }
    fn text(&self, id: String, page: u32, cancel: Cancellation) -> Result<String> {
        self.native.text(id, page, cancel)
    }
    fn close(&self, id: String) -> Result<()> {
        self.native.close(id)
    }
}
impl AsyncPdfEngine for Gate {
    fn set_visible_documents(&self, ids: &[String]) -> Result<()> {
        self.native.set_visible_documents(ids)
    }
    fn try_open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<PdfPassword>,
        cancel: Cancellation,
    ) -> Result<PdfTicket<PdfInfo>> {
        self.native.try_open(id, path, password, cancel)
    }
    fn try_render(
        &self,
        key: RenderKey,
        priority: Priority,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        self.native.try_render(key, priority, cancel)
    }
    fn try_text(&self, id: String, page: u32, cancel: Cancellation) -> Result<PdfTicket<String>> {
        self.native.try_text(id, page, cancel)
    }
    fn try_close(&self, id: String) -> Result<PdfTicket<()>> {
        self.native.try_close(id)
    }
    fn try_cover(
        &self,
        id: String,
        path: PathBuf,
        page: u32,
        generation: u64,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        self.native.try_cover(id, path, page, generation, cancel)
    }
    fn try_detect_cover(
        &self,
        _id: String,
        _path: PathBuf,
        _generation: u64,
        cancel: Cancellation,
    ) -> Result<PdfTicket<DetectedCover>> {
        let (sender, ticket) = PdfTicket::channel(1, 0, cancel);
        self.ready.send(sender).unwrap();
        Ok(ticket)
    }
}

#[test]
fn actor_persists_automatic_choice_then_restarts_using_saved_page_and_cache() {
    let root = tempfile::tempdir().unwrap();
    let dbpath = root.path().join("library.sqlite3");
    let (db, doc, pdf, dll) = setup(&dbpath);
    let original = std::fs::read(&pdf).unwrap();
    let governor = MemoryGovernor::default();
    let cache = Arc::new(CoverCache::new(root.path().join("covers"), 8 * 1024 * 1024));
    let worker = PdfWorker::start_with_cover_cache(
        &dll,
        MemoryLimits::default(),
        governor.clone(),
        Some(cache.clone()),
    )
    .unwrap();
    let runtime = Runtime::start(db, worker, || {}).unwrap();
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Cover(
            doc.id.clone(),
            pdf.clone(),
            10,
            cancel.clone(),
        ))
        .unwrap();
    let expected = match receive(&runtime) {
        Update::Cover(id, Ok(image), token) => {
            assert_eq!(id, "auto");
            assert!(token.same_request(&cancel));
            assert_eq!((image.key.page, image.key.generation), (1, 10));
            image.pixels().to_vec()
        }
        _ => panic!("Expected an automatically selected native cover"),
    };
    let actual = selection(&dbpath).unwrap();
    assert_eq!((actual.0, actual.1.as_str(), actual.2), (1, "automatic", 3));
    assert!(actual.3 >= 0.45);
    drop(runtime);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(
        std::fs::read_dir(root.path().join("covers"))
            .unwrap()
            .count(),
        1
    );
    let worker = PdfWorker::start_with_cover_cache(
        &dll,
        MemoryLimits::default(),
        governor.clone(),
        Some(cache),
    )
    .unwrap();
    let (ready, requests) = std::sync::mpsc::channel();
    let restarted = Runtime::start(
        Database::open(&dbpath).unwrap(),
        Gate {
            native: worker,
            ready,
        },
        || {},
    )
    .unwrap();
    restarted
        .submit(Action::Cover(
            doc.id,
            pdf.clone(),
            20,
            Cancellation::default(),
        ))
        .unwrap();
    match receive(&restarted) {
        Update::Cover(_, Ok(image), _) => {
            assert_eq!((image.key.page, image.key.generation), (1, 20));
            assert_eq!(image.pixels(), expected);
        }
        _ => panic!("Expected a saved-page cover after restart"),
    }
    assert!(
        matches!(
            requests.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ),
        "Saved selections must bypass detection"
    );
    drop(restarted);
    assert_eq!(governor.usage().total, 0);
    assert_eq!(std::fs::read(pdf).unwrap(), original);
}

#[test]
fn manual_choice_wins_over_delayed_detection_and_actor_renders_its_page() {
    let root = tempfile::tempdir().unwrap();
    let dbpath = root.path().join("library.sqlite3");
    let (db, doc, pdf, dll) = setup(&dbpath);
    let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
    let detected = worker
        .try_detect_cover(doc.id.clone(), pdf.clone(), 10, Cancellation::default())
        .unwrap()
        .wait()
        .unwrap();
    let (ready, requests) = std::sync::mpsc::channel();
    let runtime = Runtime::start(
        db,
        Gate {
            native: worker,
            ready,
        },
        || {},
    )
    .unwrap();
    runtime
        .submit(Action::Cover(
            doc.id.clone(),
            pdf,
            10,
            Cancellation::default(),
        ))
        .unwrap();
    let sender = requests.recv_timeout(Duration::from_secs(5)).unwrap();
    runtime
        .submit(Action::SaveManualCover(Box::new(ManualCoverRequest {
            choice: ManualCover::new(&doc, 0, 0.7142857142857143).unwrap(),
            cancel: Cancellation::default(),
        })))
        .unwrap();
    assert!(matches!(receive(&runtime),Update::ManualCover(reply) if reply.result.is_ok()));
    sender.send(Ok(detected)).unwrap();
    assert!(
        matches!(receive(&runtime),Update::Cover(_,Ok(image),_) if image.key.page==0 && image.key.generation==10)
    );
    assert_eq!(selection(&dbpath).unwrap(), (0, "manual".into(), 1, 1.0));
}

#[test]
fn cancellation_before_completion_and_failed_storage_keep_original_selection_state() {
    for fail_storage in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let dbpath = root.path().join("library.sqlite3");
        let (db, doc, pdf, dll) = setup(&dbpath);
        if fail_storage {
            db.execute_batch("CREATE TRIGGER denied BEFORE INSERT ON document_covers BEGIN SELECT RAISE(ABORT,'synthetic rejection'); END;").unwrap();
        }
        let worker = PdfWorker::start(&dll, MemoryLimits::default()).unwrap();
        let detected = worker
            .try_detect_cover(doc.id.clone(), pdf.clone(), 10, Cancellation::default())
            .unwrap()
            .wait()
            .unwrap();
        let (ready, requests) = std::sync::mpsc::channel();
        let runtime = Runtime::start(
            db,
            Gate {
                native: worker,
                ready,
            },
            || {},
        )
        .unwrap();
        let cancel = Cancellation::default();
        runtime
            .submit(Action::Cover(doc.id, pdf, 10, cancel.clone()))
            .unwrap();
        let sender = requests.recv_timeout(Duration::from_secs(5)).unwrap();
        if !fail_storage {
            cancel.cancel();
        }
        sender.send(Ok(detected)).unwrap();
        assert!(matches!(receive(&runtime), Update::Cover(_, Err(_), _)));
        assert!(selection(&dbpath).is_none());
        drop(runtime);
        assert_eq!(
            Database::open(&dbpath)
                .unwrap()
                .get("auto")
                .unwrap()
                .unwrap()
                .thumbnail_path,
            Some("prior-cache".into())
        );
    }
}
