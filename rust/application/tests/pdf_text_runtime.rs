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
    pdf_geometry::NormalizedPageRect,
    pdf_text::{PdfPageText, TextCharacter},
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
};
use yamantaka_storage::Database;

type TextReply = Arc<Mutex<Option<Sender<Result<Arc<PdfPageText>>>>>>;
struct HeldText {
    entered: Sender<(String, u32, Cancellation)>,
    reply: TextReply,
    reject: bool,
}
fn unused<T>() -> Result<T> {
    Err(Failure::Unsupported("unused test operation".into()))
}
impl PdfEngine for HeldText {
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
impl AsyncPdfEngine for HeldText {
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
    fn try_text_geometry(
        &self,
        id: String,
        page: u32,
        _: yamantaka_domain::pdf_text::TextGeometryPurpose,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<PdfPageText>>> {
        if self.reject {
            return Err(Failure::QueueFull);
        }
        let (send, ticket) = PdfTicket::channel(7, 3, cancel.clone());
        *self.reply.lock().unwrap() = Some(send);
        self.entered.try_send((id, page, cancel)).unwrap();
        Ok(ticket)
    }
}

#[test]
fn actor_preserves_text_identity_and_remains_responsive_during_native_work() {
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
            HeldText {
                entered,
                reply: reply.clone(),
                reject: false,
            },
            || {},
        )
        .unwrap();
        let token = Cancellation::default();
        runtime
            .submit(Action::TextGeometry("reader".into(), 7, token.clone()))
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
                let payload = Arc::new(
                    PdfPageText::from_reserved(
                        page,
                        "café".into(),
                        [0..1, 1..2, 2..3, 3..5]
                            .into_iter()
                            .map(|bytes| TextCharacter {
                                bytes,
                                region: Some(
                                    NormalizedPageRect::new([0.1, 0.2], [0.3, 0.4]).unwrap(),
                                ),
                            })
                            .collect(),
                        lease,
                    )
                    .unwrap(),
                );
                sender.try_send(Ok(payload)).unwrap();
            }
        }
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
        {
            Update::TextGeometry(id, text, cancel) => {
                assert_eq!(outcome, "success");
                assert_eq!(id, "reader");
                assert_eq!(text.page(), 7);
                assert_eq!(text.selected_text(3..4).unwrap(), "é");
                assert_eq!(text.characters()[0].region.unwrap().min(), [0.1, 0.2]);
                assert!(cancel.same_request(&token));
            }
            Update::TextGeometryFailed(id, page, error, cancel) => {
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
            _ => panic!("unexpected text response"),
        }
        assert_eq!(governor.usage().text, 0);
        assert!(runtime.updates.try_recv().is_err());
    }
}

#[test]
fn actor_shutdown_cancels_a_held_text_ticket_without_waiting_for_native_completion() {
    let (entered, wait) = bounded(1);
    let reply = Arc::new(Mutex::new(None));
    let runtime = Runtime::start(
        Database::in_memory_at(11).unwrap(),
        HeldText {
            entered,
            reply: reply.clone(),
            reject: false,
        },
        || {},
    )
    .unwrap();
    let token = Cancellation::default();
    runtime
        .submit(Action::TextGeometry("reader".into(), 7, token.clone()))
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

#[test]
fn immediate_native_admission_failure_emits_one_identified_terminal_result() {
    let (entered, _wait) = bounded(1);
    let runtime = Runtime::start(
        Database::in_memory_at(11).unwrap(),
        HeldText {
            entered,
            reply: Arc::new(Mutex::new(None)),
            reject: true,
        },
        || {},
    )
    .unwrap();
    let token = Cancellation::default();
    runtime
        .submit(Action::TextGeometry("reader".into(), 7, token.clone()))
        .unwrap();
    match runtime
        .updates
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
    {
        Update::TextGeometryFailed(id, page, Failure::QueueFull, cancel) => {
            assert_eq!((id.as_str(), page), ("reader", 7));
            assert!(cancel.same_request(&token));
        }
        _ => panic!("unexpected text admission result"),
    }
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
    assert!(runtime.updates.try_recv().is_err());
}
