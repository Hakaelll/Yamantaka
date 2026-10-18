use crossbeam_channel::{Receiver, Sender};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use yamantaka_application::runtime::{Action, Runtime, Update, WriteKind};
use yamantaka_domain::{
    Cancellation, Document, PdfEngine, PdfInfo, PdfPassword, Priority, RenderKey, RenderedPage,
    Result,
    memory::MemoryGovernor,
    pdf_text::{PdfPageText, TextGeometryPurpose},
    pdf_ticket::{AsyncPdfEngine, PdfOpenPurpose, PdfTicket},
    source_revision::SourceRevision,
};
use yamantaka_pdf::PdfWorker;

pub(super) struct HeldText {
    pub(super) id: String,
    pub(super) page: u32,
    pub(super) purpose: TextGeometryPurpose,
    pub(super) cancel: Cancellation,
    pub(super) reply: Sender<Result<Arc<PdfPageText>>>,
}
pub(super) struct HeldSource {
    pub id: String,
    pub cancel: Cancellation,
    pub reply: Sender<Result<SourceRevision>>,
}

pub(super) struct Engine {
    pub(super) worker: PdfWorker,
    pub(super) hold: Arc<Mutex<Option<Sender<HeldText>>>>,
    pub(super) source_hold: Mutex<Option<Sender<HeldSource>>>,
    pub(super) opens: Sender<String>,
    pub(super) pages: Arc<Mutex<Vec<u32>>>,
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
    fn wake_cancelled(&self) {
        self.worker.wake_cancelled();
    }
    fn memory_governor(&self) -> Option<MemoryGovernor> {
        Some(self.worker.memory_governor())
    }
    fn set_visible_documents(&self, ids: &[String]) -> Result<()> {
        self.worker.set_visible_documents(ids)
    }
    fn try_source_revision(
        &self,
        id: String,
        cancel: Cancellation,
    ) -> Result<PdfTicket<SourceRevision>> {
        if let Some(hold) = self.source_hold.lock().unwrap().take() {
            let (reply, ticket) = PdfTicket::channel(998, 0, cancel.clone());
            hold.send(HeldSource { id, cancel, reply }).unwrap();
            Ok(ticket)
        } else {
            self.worker.try_source_revision(id, cancel)
        }
    }
    fn try_open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<PdfPassword>,
        purpose: PdfOpenPurpose,
        cancel: Cancellation,
    ) -> Result<PdfTicket<PdfInfo>> {
        self.opens.send(id.clone()).unwrap();
        self.worker.try_open(id, path, password, purpose, cancel)
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
        self.pages.lock().unwrap().push(page);
        if page == 1
            && let Some(hold) = self.hold.lock().unwrap().take()
        {
            let (reply, ticket) = PdfTicket::channel(999, 0, cancel.clone());
            hold.send(HeldText {
                id,
                page,
                purpose,
                cancel,
                reply,
            })
            .unwrap();
            Ok(ticket)
        } else {
            self.worker.try_text_geometry(id, page, purpose, cancel)
        }
    }
}
pub(super) fn wait(runtime: &Runtime) -> Update {
    runtime
        .updates
        .recv_timeout(Duration::from_secs(10))
        .unwrap()
}
pub(super) fn open_reader(runtime: &Runtime, document: Document) {
    let id = document.id.clone();
    let cancel = Cancellation::default();
    runtime
        .submit(Action::Open(document, None, cancel.clone()))
        .unwrap();
    assert!(
        matches!(wait(runtime),Update::Opened(document,_,token) if document.id==id && token.same_request(&cancel))
    );
    runtime.submit(Action::ConfirmOpen(id, cancel)).unwrap();
    assert!(matches!(
        wait(runtime),
        Update::Committed(_, WriteKind::VisibleDocuments)
    ));
}
pub(super) fn visible(runtime: &Runtime, ids: &[&str]) {
    runtime
        .submit(Action::VisibleDocuments(
            ids.iter().map(|id| id.to_string()).collect(),
        ))
        .unwrap();
    assert!(matches!(
        wait(runtime),
        Update::Committed(_, WriteKind::VisibleDocuments)
    ));
}
pub(super) fn requested(opens: &Receiver<String>, id: &str) {
    loop {
        if opens
            .recv_timeout(Duration::from_secs(5))
            .expect("Interrupted analysis did not reopen its native document")
            == id
        {
            break;
        }
    }
}
