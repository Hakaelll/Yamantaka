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
    pdf_find::{PdfFindQuery, PdfMatch, PdfPageMatches},
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
};
use yamantaka_storage::Database;

type FindReply = Arc<Mutex<Option<Sender<Result<Arc<PdfPageMatches>>>>>>;
struct HeldFind {
    entered: Sender<(String, u32, PdfFindQuery, u32, Cancellation)>,
    reply: FindReply,
}
fn unused<T>() -> Result<T> {
    Err(Failure::Unsupported("unused test operation".into()))
}
impl PdfEngine for HeldFind {
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
impl AsyncPdfEngine for HeldFind {
    fn set_visible_documents(&self, _: &[String]) -> Result<()> {
        Ok(())
    }
    fn try_open(
        &self,
        _: String,
        _: PathBuf,
        _: Option<PdfPassword>,
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
    fn try_find_page(
        &self,
        id: String,
        page: u32,
        query: PdfFindQuery,
        offset: u32,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<PdfPageMatches>>> {
        let (send, ticket) = PdfTicket::channel(7, 3, cancel.clone());
        *self.reply.lock().unwrap() = Some(send);
        self.entered
            .try_send((id, page, query, offset, cancel))
            .unwrap();
        Ok(ticket)
    }
}

#[test]
fn actor_preserves_find_identity_and_queries_sql_while_native_search_is_held() {
    for cancelled in [false, true] {
        let (entered, wait) = bounded(1);
        let reply = Arc::new(Mutex::new(None));
        let runtime = Runtime::start(
            Database::in_memory_at(11).unwrap(),
            HeldFind {
                entered,
                reply: reply.clone(),
            },
            || {},
        )
        .unwrap();
        let token = Cancellation::default();
        let query = PdfFindQuery::new("Résumé", true, true).unwrap();
        runtime
            .submit(Action::FindPage(
                "reader".into(),
                7,
                query.clone(),
                2,
                token.clone(),
            ))
            .unwrap();
        let (id, page, actual, offset, cancel) = wait.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!((id.as_str(), page, offset), ("reader", 7, 2));
        assert_eq!(actual, query);
        assert!(cancel.same_request(&token));
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
        if cancelled {
            token.cancel();
        }
        let governor = MemoryGovernor::default();
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                4096,
            ))
            .unwrap();
        let result = Arc::new(
            PdfPageMatches::from_reserved(
                7,
                vec![PdfMatch {
                    ordinal: 2,
                    snippet: "Résumé".into(),
                    rectangles: Vec::new(),
                }],
                Some(3),
                lease,
            )
            .unwrap(),
        );
        reply
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .try_send(Ok(result))
            .unwrap();
        match runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
        {
            Update::PageMatches(id, matches, cancel) => {
                assert!(!cancelled);
                assert_eq!(id, "reader");
                assert_eq!(matches.page(), 7);
                assert_eq!(matches.next_match(), Some(3));
                assert!(cancel.same_request(&token));
            }
            Update::PageFindFailed(id, Failure::Cancelled, cancel) => {
                assert!(cancelled);
                assert_eq!(id, "reader");
                assert!(cancel.same_request(&token));
            }
            _ => panic!("find response identity"),
        }
        assert_eq!(governor.usage().text, 0);
    }
}
