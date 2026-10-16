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
    pdf_links::PdfPageLinks,
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
};
use yamantaka_storage::Database;

type LinksReply = Arc<Mutex<Option<Sender<Result<Arc<PdfPageLinks>>>>>>;
struct HeldLinks {
    entered: Sender<(String, u32, Cancellation)>,
    reply: LinksReply,
}
fn unused<T>() -> Result<T> {
    Err(Failure::Unsupported("unused test operation".into()))
}
impl PdfEngine for HeldLinks {
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
impl AsyncPdfEngine for HeldLinks {
    fn set_visible_documents(&self, _: &[String]) -> Result<()> {
        Ok(())
    }
    fn try_open(
        &self,
        _: String,
        _: PathBuf,
        _: Option<PdfPassword>,
        _purpose: yamantaka_domain::pdf_ticket::PdfOpenPurpose,
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
    fn try_page_links(
        &self,
        id: String,
        page: u32,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<PdfPageLinks>>> {
        let (send, ticket) = PdfTicket::channel(7, 3, cancel.clone());
        *self.reply.lock().unwrap() = Some(send);
        self.entered.try_send((id, page, cancel)).unwrap();
        Ok(ticket)
    }
}

#[test]
fn actor_preserves_link_identity_and_remains_responsive_during_native_work() {
    for outcome in [
        "success",
        "cancelled",
        "wrong-page",
        "native-error",
        "disconnected",
    ] {
        let (entered, wait) = bounded(1);
        let reply = Arc::new(Mutex::new(None));
        let runtime = Runtime::start(
            Database::in_memory_at(11).unwrap(),
            HeldLinks {
                entered,
                reply: reply.clone(),
            },
            || {},
        )
        .unwrap();
        let token = Cancellation::default();
        runtime
            .submit(Action::PageLinks("reader".into(), 7, token.clone()))
            .unwrap();
        let (id, page, actual) = wait.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!((id.as_str(), page), ("reader", 7));
        assert!(actual.same_request(&token));
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
        let governor = MemoryGovernor::default();
        if outcome == "cancelled" {
            token.cancel();
        }
        let sender = reply.lock().unwrap().take().unwrap();
        match outcome {
            "disconnected" => drop(sender),
            "native-error" => sender
                .try_send(Err(Failure::Pdf("fixture failure".into())))
                .unwrap(),
            _ => {
                let lease = governor
                    .try_reserve_one(Cost::new(
                        MemoryCategory::Text,
                        MemoryRepresentation::Cpu,
                        4096,
                    ))
                    .unwrap();
                let page = if outcome == "wrong-page" { 8 } else { 7 };
                let payload =
                    Arc::new(PdfPageLinks::from_reserved(page, Vec::new(), 2, lease).unwrap());
                sender.try_send(Ok(payload)).unwrap();
            }
        }
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
        {
            Update::PageLinks(id, links, cancel) => {
                assert_eq!(outcome, "success");
                assert_eq!(id, "reader");
                assert_eq!(links.page(), 7);
                assert_eq!(links.unsupported_targets(), 2);
                assert!(cancel.same_request(&token));
            }
            Update::PageLinksFailed(id, page, error, cancel) => {
                assert_ne!(outcome, "success");
                assert_eq!((id.as_str(), page), ("reader", 7));
                assert!(cancel.same_request(&token));
                match outcome {
                    "cancelled" => assert!(matches!(error, Failure::Cancelled)),
                    "wrong-page" => assert!(matches!(error, Failure::Invalid(_))),
                    "native-error" => assert!(matches!(error, Failure::Pdf(_))),
                    "disconnected" => {}
                    _ => unreachable!(),
                }
            }
            _ => panic!("unexpected link response"),
        }
        assert_eq!(governor.usage().text, 0);
        assert!(runtime.updates.try_recv().is_err());
    }
}

#[test]
fn actor_shutdown_cancels_a_held_link_ticket_without_waiting_for_native_completion() {
    let (entered, wait) = bounded(1);
    let reply = Arc::new(Mutex::new(None));
    let runtime = Runtime::start(
        Database::in_memory_at(11).unwrap(),
        HeldLinks {
            entered,
            reply: reply.clone(),
        },
        || {},
    )
    .unwrap();
    let token = Cancellation::default();
    runtime
        .submit(Action::PageLinks("reader".into(), 7, token.clone()))
        .unwrap();
    wait.recv_timeout(Duration::from_secs(2)).unwrap();
    let (finished, completed) = bounded(1);
    let thread = std::thread::spawn(move || {
        drop(runtime);
        finished.send(()).unwrap();
    });
    completed.recv_timeout(Duration::from_secs(2)).unwrap();
    thread.join().unwrap();
    assert!(token.is_cancelled());
    assert!(
        reply
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .try_send(Err(Failure::Cancelled))
            .is_err()
    );
}
