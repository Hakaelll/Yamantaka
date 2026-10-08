use crossbeam_channel::{Select, Sender};
use std::{cell::RefCell, collections::VecDeque, path::PathBuf, sync::Arc};
use yamantaka_application::search_index::{IndexCommand, IndexPhase, SearchIndexController};
use yamantaka_domain::{
    Cancellation, Failure, PdfEngine, PdfInfo, Priority, RenderKey, RenderedPage, Result,
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
    search::{IndexStatus, SearchQuery, SearchRepository},
};
use yamantaka_storage::Database;

type TextReply = (u32, Cancellation, Sender<Result<String>>);

#[derive(Default)]
struct Engine {
    opens: RefCell<VecDeque<Sender<Result<PdfInfo>>>>,
    texts: RefCell<VecDeque<TextReply>>,
    closed: RefCell<Vec<String>>,
    requested_pages: RefCell<Vec<u32>>,
}
impl Engine {
    fn open_result(&self, result: Result<PdfInfo>) {
        self.opens
            .borrow_mut()
            .pop_front()
            .unwrap()
            .send(result)
            .unwrap();
    }
    fn opened(&self) {
        self.open_result(Ok(PdfInfo {
            page_sizes: vec![[600.0, 800.0]; 3],
            title: None,
            author: None,
        }));
    }
    fn text_result(&self, text: &str) -> Cancellation {
        let (_, cancel, sender) = self.texts.borrow_mut().pop_front().unwrap();
        sender.send(Ok(text.into())).unwrap();
        cancel
    }
}
impl PdfEngine for Engine {
    fn open(&self, _: String, _: PathBuf, _: Option<String>, _: Cancellation) -> Result<PdfInfo> {
        panic!("controller must not wait for native open")
    }
    fn render(&self, _: RenderKey, _: Priority, _: Cancellation) -> Result<Arc<RenderedPage>> {
        panic!("indexing must not rasterize")
    }
    fn text(&self, _: String, _: u32, _: Cancellation) -> Result<String> {
        panic!("controller must not wait for text")
    }
    fn close(&self, _: String) -> Result<()> {
        panic!("controller must not wait for close")
    }
}
impl AsyncPdfEngine for Engine {
    fn try_open(
        &self,
        _: String,
        _: PathBuf,
        _: Option<String>,
        cancel: Cancellation,
    ) -> Result<PdfTicket<PdfInfo>> {
        let (sender, ticket) = PdfTicket::channel(1, 1, cancel);
        self.opens.borrow_mut().push_back(sender);
        Ok(ticket)
    }
    fn try_render(
        &self,
        _: RenderKey,
        _: Priority,
        _: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        panic!("indexing must not rasterize")
    }
    fn try_text(&self, _: String, page: u32, cancel: Cancellation) -> Result<PdfTicket<String>> {
        let (sender, ticket) = PdfTicket::channel(2, 1, cancel.clone());
        self.requested_pages.borrow_mut().push(page);
        self.texts.borrow_mut().push_back((page, cancel, sender));
        Ok(ticket)
    }
    fn try_close(&self, id: String) -> Result<PdfTicket<()>> {
        self.closed.borrow_mut().push(id);
        let (sender, ticket) = PdfTicket::channel(3, 1, Cancellation::default());
        sender.send(Ok(())).unwrap();
        Ok(ticket)
    }
}
fn library() -> Database {
    let db = Database::in_memory_at(11).unwrap();
    db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,title,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES ('a','a.pdf','a.pdf','a.pdf','A personal book',3,1,'now','now','now','fa');").unwrap();
    db
}
fn start(c: &mut SearchIndexController, db: &mut Database) {
    c.command(IndexCommand::Enable(true), db).unwrap();
    c.command(IndexCommand::Start { rebuild: false }, db)
        .unwrap();
}
fn step(c: &mut SearchIndexController, db: &mut Database, e: &Engine) {
    assert!(c.step(db, e).unwrap());
}
fn finish(c: &mut SearchIndexController, db: &mut Database, e: &Engine) {
    for _ in 0..40 {
        if c.needs_step() {
            step(c, db, e);
        } else if !e.opens.borrow().is_empty() {
            e.opened();
        } else if !e.texts.borrow().is_empty() {
            e.text_result("Optics and personal study notes");
        } else {
            break;
        }
    }
    assert_eq!(c.progress().phase, IndexPhase::Completed);
}
fn search(db: &Database, text: &str) -> usize {
    db.search(
        &SearchQuery {
            text: text.into(),
            ..Default::default()
        },
        &Cancellation::default(),
    )
    .unwrap()
    .len()
}

#[test]
fn disabled_paused_and_blocked_native_work_never_spin_or_index_without_consent() {
    let mut db = library();
    let e = Engine::default();
    let mut c = SearchIndexController::default();
    assert!(
        c.command(IndexCommand::Start { rebuild: true }, &mut db)
            .is_err()
    );
    assert!(!c.step(&mut db, &e).unwrap());
    assert!(db.index_state("a").unwrap().is_none());
    start(&mut c, &mut db);
    step(&mut c, &mut db, &e);
    assert!(!c.needs_step());
    let mut ready = Select::new();
    c.register_readiness(&mut ready);
    assert!(ready.try_ready().is_err());
    drop(ready);
    c.command(IndexCommand::Pause(true), &mut db).unwrap();
    e.opened();
    step(&mut c, &mut db, &e);
    assert!(!c.step(&mut db, &e).unwrap());
    assert!(e.requested_pages.borrow().is_empty());
    assert_eq!(c.take_progress().unwrap().phase, IndexPhase::Paused);
    assert!(c.take_progress().is_none());
    c.command(IndexCommand::Pause(false), &mut db).unwrap();
    finish(&mut c, &mut db, &e);
    assert_eq!(*e.requested_pages.borrow(), vec![0, 1, 2]);
    assert_eq!(
        db.index_state("a").unwrap().unwrap().status,
        IndexStatus::Indexed
    );
    assert_eq!(search(&db, "Optics"), 3);
    assert_eq!(e.closed.borrow().len(), 1);
}

#[test]
fn cancellation_discards_late_text_and_restart_resumes_the_durable_cursor() {
    let mut db = library();
    let e = Engine::default();
    let mut c = SearchIndexController::default();
    start(&mut c, &mut db);
    step(&mut c, &mut db, &e);
    e.opened();
    step(&mut c, &mut db, &e);
    step(&mut c, &mut db, &e);
    e.text_result("Committed optics");
    step(&mut c, &mut db, &e);
    step(&mut c, &mut db, &e);
    c.command(IndexCommand::Cancel, &mut db).unwrap();
    assert!(
        e.text_result("Obsolete confidential payload")
            .is_cancelled()
    );
    for _ in 0..3 {
        step(&mut c, &mut db, &e);
    }
    assert_eq!(c.progress().phase, IndexPhase::Cancelled);
    assert_eq!(db.index_state("a").unwrap().unwrap().indexed_pages, 1);
    assert_eq!(search(&db, "Obsolete"), 0);
    let mut restarted = SearchIndexController::default();
    start(&mut restarted, &mut db);
    finish(&mut restarted, &mut db, &e);
    assert_eq!(*e.requested_pages.borrow(), vec![0, 1, 1, 2]);
    assert_eq!(search(&db, "Committed"), 1);
    assert_eq!(search(&db, "Obsolete"), 0);
}

#[test]
fn exclusion_invalidates_pending_text_and_survives_a_new_controller() {
    let mut db = library();
    let e = Engine::default();
    let mut c = SearchIndexController::default();
    start(&mut c, &mut db);
    step(&mut c, &mut db, &e);
    e.opened();
    step(&mut c, &mut db, &e);
    step(&mut c, &mut db, &e);
    c.command(
        IndexCommand::Exclude {
            document_id: "a".into(),
            excluded: true,
        },
        &mut db,
    )
    .unwrap();
    assert!(e.text_result("Excluded secret").is_cancelled());
    for _ in 0..3 {
        step(&mut c, &mut db, &e);
    }
    assert_eq!(search(&db, "Excluded"), 0);
    assert_eq!(
        db.index_state("a").unwrap().unwrap().status,
        IndexStatus::Excluded
    );
    let mut restarted = SearchIndexController::default();
    start(&mut restarted, &mut db);
    finish(&mut restarted, &mut db, &e);
    assert_eq!(restarted.progress().completed_documents, 0);
    assert_eq!(e.closed.borrow().len(), 1);
}

#[test]
fn native_failure_is_redacted_and_shutdown_closes_cancelled_work_without_drain() {
    let mut db = library();
    let e = Engine::default();
    let mut c = SearchIndexController::default();
    start(&mut c, &mut db);
    step(&mut c, &mut db, &e);
    e.open_result(Err(Failure::Pdf(
        "C:/personal/private/password.pdf: secret error".into(),
    )));
    finish(&mut c, &mut db, &e);
    assert_eq!(c.progress().failed_documents, 1);
    assert_eq!(c.progress().last_error_code, Some("pdf"));
    assert_eq!(
        db.index_state("a").unwrap().unwrap().error_code.as_deref(),
        Some("pdf")
    );
    let mut restarted = SearchIndexController::default();
    start(&mut restarted, &mut db);
    step(&mut restarted, &mut db, &e);
    e.opened();
    step(&mut restarted, &mut db, &e);
    step(&mut restarted, &mut db, &e);
    let close = restarted.shutdown(&e).unwrap().unwrap();
    assert!(close.is_ready());
    close.wait().unwrap();
    let (_, cancel, _sender) = e.texts.borrow_mut().pop_front().unwrap();
    assert!(cancel.is_cancelled());
    assert_eq!(e.closed.borrow().len(), 2);
}
