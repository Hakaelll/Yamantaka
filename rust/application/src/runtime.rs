use crossbeam_channel::{Receiver, RecvTimeoutError, Select, Sender, TryRecvError, bounded};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use yamantaka_domain::pdf_ticket::{AsyncPdfEngine, PdfTicket};
use yamantaka_domain::reader_preferences::ReaderPreferences;
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryQuery, LibraryRepository, NewDocument, PdfInfo,
    Priority, RenderKey, RenderedPage, Result,
};

pub enum Action {
    Query(LibraryQuery, Cancellation),
    Favorite(String, bool),
    LoadReaderPreferences(String, Cancellation),
    SaveReaderPreferences(String, ReaderPreferences),
    Open(Document, Cancellation),
    Import(Vec<PathBuf>, Cancellation),
    Render(RenderKey, Cancellation),
    Navigate(String, u32),
    Text(String, u32, Cancellation),
    Close(String),
    Shutdown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteKind {
    Favorite,
    Position,
    Close,
    ReaderPreferences,
}
pub enum Update {
    Library(Vec<Document>, u64, Cancellation),
    Opened(Box<Document>, PdfInfo, Cancellation),
    Page(Arc<RenderedPage>, Cancellation, u64, u64),
    Text(String, u32, String, Cancellation),
    Imported(usize, Cancellation),
    ReaderPreferences(String, Option<ReaderPreferences>, Cancellation),
    Committed(String, WriteKind),
    Failed(Failure, Option<Cancellation>),
}
const CAPACITY: usize = 128;
struct Permit(Arc<AtomicUsize>);
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}
struct Command {
    action: Action,
    permit: Permit,
}
struct Event {
    update: Update,
    _permit: Permit,
}
/// A paused consumer retains at most 128 data and 128 close actions/results.
/// The separate close reservations remain admissible under data saturation.
/// Capacity follows each command to its terminal event; delivery never blocks.
pub struct UpdateReceiver(Receiver<Event>);
impl UpdateReceiver {
    pub fn try_recv(&self) -> std::result::Result<Update, TryRecvError> {
        self.0.try_recv().map(|event| event.update)
    }
    pub fn recv_timeout(&self, timeout: Duration) -> std::result::Result<Update, RecvTimeoutError> {
        self.0.recv_timeout(timeout).map(|event| event.update)
    }
    pub fn try_iter(&self) -> impl Iterator<Item = Update> + '_ {
        std::iter::from_fn(|| self.try_recv().ok())
    }
}
pub struct Runtime {
    sender: Sender<Command>,
    shutdown: Sender<()>,
    outstanding: Arc<AtomicUsize>,
    control_outstanding: Arc<AtomicUsize>,
    pub updates: UpdateReceiver,
    handle: Option<JoinHandle<()>>,
}
struct ImportState {
    paths: std::collections::VecDeque<PathBuf>,
    count: usize,
    cancel: Cancellation,
    permit: Permit,
}
enum Pending {
    Open(Document, PdfTicket<PdfInfo>, Permit),
    Render(PdfTicket<Arc<RenderedPage>>, Permit),
    Text(String, u32, PdfTicket<String>, Permit),
    Close(String, PdfTicket<()>, Permit),
    ImportOpen(PathBuf, String, PdfTicket<PdfInfo>, ImportState),
    ImportClose(PathBuf, PdfInfo, PdfTicket<()>, ImportState),
}
impl Pending {
    fn register<'a>(&'a self, select: &mut Select<'a>) {
        match self {
            Self::Open(_, ticket, _) | Self::ImportOpen(_, _, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::Render(ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::Text(_, _, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::Close(_, ticket, _) | Self::ImportClose(_, _, ticket, _) => {
                select.recv(ticket.readiness());
            }
        }
    }
    fn ready(&self) -> bool {
        match self {
            Self::Open(_, ticket, _) | Self::ImportOpen(_, _, ticket, _) => ticket.is_ready(),
            Self::Render(ticket, _) => ticket.is_ready(),
            Self::Text(_, _, ticket, _) => ticket.is_ready(),
            Self::Close(_, ticket, _) | Self::ImportClose(_, _, ticket, _) => ticket.is_ready(),
        }
    }
    fn cancel(&self) {
        match self {
            Self::Open(_, ticket, _) | Self::ImportOpen(_, _, ticket, _) => {
                ticket.cancellation.cancel()
            }
            Self::Render(ticket, _) => ticket.cancellation.cancel(),
            Self::Text(_, _, ticket, _) => ticket.cancellation.cancel(),
            Self::Close(_, ticket, _) | Self::ImportClose(_, _, ticket, _) => {
                ticket.cancellation.cancel()
            }
        }
    }
}
fn emit(events: &Sender<Event>, notify: &impl Fn(), update: Update, permit: Permit) {
    // Each event owns an admission reservation; the bounded channel therefore
    // has room even if the UI has stopped draining it. Never wait on a consumer.
    if events
        .try_send(Event {
            update,
            _permit: permit,
        })
        .is_ok()
    {
        notify();
    }
}
fn failed(
    events: &Sender<Event>,
    notify: &impl Fn(),
    error: Failure,
    cancel: Option<Cancellation>,
    permit: Permit,
) {
    if !matches!(error, Failure::Cancelled) {
        emit(events, notify, Update::Failed(error, cancel), permit);
    }
}
fn advance_import<E: AsyncPdfEngine>(
    engine: &E,
    mut state: ImportState,
    pending: &mut Vec<Pending>,
    events: &Sender<Event>,
    notify: &impl Fn(),
    import_sequence: &mut u64,
) {
    if state.cancel.is_cancelled() {
        return;
    }
    if let Some(path) = state.paths.pop_front() {
        *import_sequence = import_sequence.wrapping_add(1);
        let id = format!("import-{}", *import_sequence);
        match engine.try_open(id.clone(), path.clone(), None, state.cancel.clone()) {
            Ok(ticket) => pending.push(Pending::ImportOpen(path, id, ticket, state)),
            Err(error) => failed(events, notify, error, Some(state.cancel), state.permit),
        }
    } else {
        emit(
            events,
            notify,
            Update::Imported(state.count, state.cancel),
            state.permit,
        );
    }
}
impl Runtime {
    pub fn start<R: LibraryRepository + Send + 'static, E: AsyncPdfEngine + Send + 'static>(
        mut repository: R,
        engine: E,
        notify: impl Fn() + Send + 'static,
    ) -> Result<Self> {
        let (sender, commands) = bounded::<Command>(CAPACITY * 2);
        let (events, updates) = bounded(CAPACITY * 2);
        let (shutdown, stop) = bounded(1);
        let handle = thread::Builder::new()
            .name("yamantaka-use-cases".into())
            .spawn(move || {
                let mut pending: Vec<Pending> = Vec::new();
                let mut import_sequence = 0;
                loop {
                    if stop.try_recv().is_ok() {
                        break;
                    }
                    if let Ok(Command { action, permit }) = commands.try_recv() {
                        match action {
                            Action::Shutdown => break,
                            Action::Query(query, cancel) => {
                                let result = (|| {
                                    cancel.check()?;
                                    let documents = repository.query(&query)?;
                                    let count = repository.count(&query)?;
                                    cancel.check()?;
                                    Ok(Update::Library(documents, count, cancel.clone()))
                                })();
                                match result {
                                    Ok(update) => emit(&events, &notify, update, permit),
                                    Err(error) => {
                                        failed(&events, &notify, error, Some(cancel), permit)
                                    }
                                }
                            }
                            Action::LoadReaderPreferences(id, cancel) => {
                                let result = (|| {
                                    cancel.check()?;
                                    let prefs = repository.get_reader_preferences(&id)?;
                                    cancel.check()?;
                                    Ok(prefs)
                                })();
                                match result {
                                    Ok(prefs) => emit(
                                        &events,
                                        &notify,
                                        Update::ReaderPreferences(id, prefs, cancel),
                                        permit,
                                    ),
                                    Err(error) => {
                                        failed(&events, &notify, error, Some(cancel), permit)
                                    }
                                }
                            }
                            Action::SaveReaderPreferences(id, prefs) => {
                                let result = if prefs.document_id != id {
                                    Err(Failure::Invalid(
                                        "Reader preferences belong to another document".into(),
                                    ))
                                } else {
                                    prefs.validate().and_then(|()| {
                                        repository.save_reader_preferences(&id, &prefs)
                                    })
                                };
                                match result {
                                    Ok(()) => emit(
                                        &events,
                                        &notify,
                                        Update::Committed(id, WriteKind::ReaderPreferences),
                                        permit,
                                    ),
                                    Err(error) => failed(&events, &notify, error, None, permit),
                                }
                            }
                            Action::Favorite(id, value) => {
                                match repository.set_favorite(&id, value) {
                                    Ok(()) => emit(
                                        &events,
                                        &notify,
                                        Update::Committed(id, WriteKind::Favorite),
                                        permit,
                                    ),
                                    Err(error) => failed(&events, &notify, error, None, permit),
                                }
                            }
                            Action::Navigate(id, page) => match repository.save_position(&id, page)
                            {
                                Ok(()) => emit(
                                    &events,
                                    &notify,
                                    Update::Committed(id, WriteKind::Position),
                                    permit,
                                ),
                                Err(error) => failed(&events, &notify, error, None, permit),
                            },
                            Action::Open(document, cancel) => match engine.try_open(
                                document.id.clone(),
                                document.path.clone(),
                                None,
                                cancel.clone(),
                            ) {
                                Ok(ticket) => pending.push(Pending::Open(document, ticket, permit)),
                                Err(error) => failed(&events, &notify, error, Some(cancel), permit),
                            },
                            Action::Render(key, cancel) => {
                                match engine.try_render(key, Priority::Visible, cancel.clone()) {
                                    Ok(ticket) => pending.push(Pending::Render(ticket, permit)),
                                    Err(error) => {
                                        failed(&events, &notify, error, Some(cancel), permit)
                                    }
                                }
                            }
                            Action::Text(id, page, cancel) => {
                                match engine.try_text(id.clone(), page, cancel.clone()) {
                                    Ok(ticket) => {
                                        pending.push(Pending::Text(id, page, ticket, permit))
                                    }
                                    Err(error) => {
                                        failed(&events, &notify, error, Some(cancel), permit)
                                    }
                                }
                            }
                            Action::Close(id) => match engine.try_close(id.clone()) {
                                Ok(ticket) => pending.push(Pending::Close(id, ticket, permit)),
                                Err(error) => failed(&events, &notify, error, None, permit),
                            },
                            Action::Import(paths, cancel) => advance_import(
                                &engine,
                                ImportState {
                                    paths: paths.into(),
                                    count: 0,
                                    cancel,
                                    permit,
                                },
                                &mut pending,
                                &events,
                                &notify,
                                &mut import_sequence,
                            ),
                        }
                    }
                    let mut index = 0;
                    while index < pending.len() {
                        if !pending[index].ready() {
                            index += 1;
                            continue;
                        }
                        match pending.remove(index) {
                            Pending::Open(document, ticket, permit) => {
                                let cancel = ticket.cancellation.clone();
                                match ticket.wait().and_then(|info| {
                                    cancel.check()?;
                                    Ok(info)
                                }) {
                                    Ok(info) => emit(
                                        &events,
                                        &notify,
                                        Update::Opened(Box::new(document), info, cancel),
                                        permit,
                                    ),
                                    Err(error) => {
                                        failed(&events, &notify, error, Some(cancel), permit)
                                    }
                                }
                            }
                            Pending::Render(ticket, permit) => {
                                let cancel = ticket.cancellation.clone();
                                let id = ticket.consumer_id;
                                let epoch = ticket.epoch;
                                match ticket.wait().and_then(|page| {
                                    cancel.check()?;
                                    Ok(page)
                                }) {
                                    Ok(page) => emit(
                                        &events,
                                        &notify,
                                        Update::Page(page, cancel, id, epoch),
                                        permit,
                                    ),
                                    Err(error) => {
                                        failed(&events, &notify, error, Some(cancel), permit)
                                    }
                                }
                            }
                            Pending::Text(id, page, ticket, permit) => {
                                let cancel = ticket.cancellation.clone();
                                match ticket.wait().and_then(|text| {
                                    cancel.check()?;
                                    Ok(text)
                                }) {
                                    Ok(text) => emit(
                                        &events,
                                        &notify,
                                        Update::Text(id, page, text, cancel),
                                        permit,
                                    ),
                                    Err(error) => {
                                        failed(&events, &notify, error, Some(cancel), permit)
                                    }
                                }
                            }
                            Pending::Close(id, ticket, permit) => match ticket.wait() {
                                Ok(()) => emit(
                                    &events,
                                    &notify,
                                    Update::Committed(id, WriteKind::Close),
                                    permit,
                                ),
                                Err(error) => failed(&events, &notify, error, None, permit),
                            },
                            Pending::ImportOpen(path, id, ticket, state) => match ticket.wait() {
                                Ok(info) => match engine.try_close(id) {
                                    Ok(close) => {
                                        pending.push(Pending::ImportClose(path, info, close, state))
                                    }
                                    Err(error) => failed(
                                        &events,
                                        &notify,
                                        error,
                                        Some(state.cancel),
                                        state.permit,
                                    ),
                                },
                                Err(error) => {
                                    let _ = engine.try_close(id);
                                    failed(
                                        &events,
                                        &notify,
                                        error,
                                        Some(state.cancel),
                                        state.permit,
                                    );
                                }
                            },
                            Pending::ImportClose(path, info, ticket, mut state) => {
                                match ticket.wait().and_then(|()| {
                                    state.cancel.check()?;
                                    repository.import(&NewDocument { path, info })
                                }) {
                                    Ok(_) => {
                                        state.count += 1;
                                        advance_import(
                                            &engine,
                                            state,
                                            &mut pending,
                                            &events,
                                            &notify,
                                            &mut import_sequence,
                                        );
                                    }
                                    Err(error) => failed(
                                        &events,
                                        &notify,
                                        error,
                                        Some(state.cancel),
                                        state.permit,
                                    ),
                                }
                            }
                        }
                    }
                    // A blocking select sleeps while idle and wakes on actual command
                    // or PDF completion. SQL is never held behind native rendering.
                    if !commands.is_empty() {
                        continue;
                    }
                    let mut select = Select::new();
                    select.recv(&stop);
                    select.recv(&commands);
                    for item in &pending {
                        item.register(&mut select);
                    }
                    select.ready();
                }
                for item in &pending {
                    item.cancel();
                }
                drop(pending); // Drop tickets before engine join; no UI draining required.
                drop(engine);
            })
            .map_err(|_| Failure::Storage("Use-case worker could not start".into()))?;
        Ok(Self {
            sender,
            shutdown,
            outstanding: Arc::new(AtomicUsize::new(0)),
            control_outstanding: Arc::new(AtomicUsize::new(0)),
            updates: UpdateReceiver(updates),
            handle: Some(handle),
        })
    }
    pub fn submit(&self, action: Action) -> Result<()> {
        if matches!(action, Action::Shutdown) {
            self.shutdown
                .try_send(())
                .map_err(|_| Failure::Storage("Use-case worker stopped".into()))?;
            return Ok(());
        }
        let counter = if matches!(action, Action::Close(_)) {
            &self.control_outstanding
        } else {
            &self.outstanding
        };
        counter
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < CAPACITY).then_some(count + 1)
            })
            .map_err(|_| {
                Failure::Storage("Work queue is full; drain results or cancel pending work".into())
            })?;
        self.sender
            .try_send(Command {
                action,
                permit: Permit(counter.clone()),
            })
            .map_err(|_| Failure::Storage("Use-case worker stopped".into()))
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        let _ = self.shutdown.try_send(());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct Repository;
    impl LibraryRepository for Repository {
        fn query(&self, _: &LibraryQuery) -> Result<Vec<Document>> {
            Ok(Vec::new())
        }
        fn count(&self, _: &LibraryQuery) -> Result<u64> {
            Ok(0)
        }
        fn set_favorite(&mut self, _: &str, _: bool) -> Result<()> {
            Ok(())
        }
        fn save_position(&mut self, _: &str, _: u32) -> Result<()> {
            Ok(())
        }
        fn get(&self, _: &str) -> Result<Option<Document>> {
            Ok(None)
        }
        fn import(&mut self, _: &NewDocument) -> Result<String> {
            Ok("imported".into())
        }
    }
    use yamantaka_domain::PdfEngine;
    struct Engine;
    impl PdfEngine for Engine {
        fn open(
            &self,
            _: String,
            _: PathBuf,
            _: Option<String>,
            _: Cancellation,
        ) -> Result<PdfInfo> {
            Ok(PdfInfo {
                page_sizes: vec![[400.0, 600.0]],
                title: None,
                author: None,
            })
        }
        fn render(&self, _: RenderKey, _: Priority, _: Cancellation) -> Result<Arc<RenderedPage>> {
            Err(Failure::Pdf("not used by these lifecycle tests".into()))
        }
        fn text(&self, id: String, _: u32, _: Cancellation) -> Result<String> {
            Ok(id)
        }
        fn close(&self, _: String) -> Result<()> {
            Ok(())
        }
    }

    impl AsyncPdfEngine for Engine {
        fn try_open(
            &self,
            id: String,
            path: PathBuf,
            password: Option<String>,
            cancel: Cancellation,
        ) -> Result<PdfTicket<PdfInfo>> {
            let result = self.open(id, path, password, cancel.clone());
            let (send, ticket) = PdfTicket::channel(1, 1, cancel);
            let _ = send.try_send(result);
            Ok(ticket)
        }
        fn try_render(
            &self,
            key: RenderKey,
            priority: Priority,
            cancel: Cancellation,
        ) -> Result<PdfTicket<Arc<RenderedPage>>> {
            let result = self.render(key, priority, cancel.clone());
            let (send, ticket) = PdfTicket::channel(1, 1, cancel);
            let _ = send.try_send(result);
            Ok(ticket)
        }
        fn try_text(
            &self,
            id: String,
            page: u32,
            cancel: Cancellation,
        ) -> Result<PdfTicket<String>> {
            let result = self.text(id, page, cancel.clone());
            let (send, ticket) = PdfTicket::channel(1, 1, cancel);
            let _ = send.try_send(result);
            Ok(ticket)
        }
        fn try_close(&self, id: String) -> Result<PdfTicket<()>> {
            let (send, ticket) = PdfTicket::channel(1, 1, Cancellation::default());
            let _ = send.try_send(self.close(id));
            Ok(ticket)
        }
    }
    #[test]
    fn queued_query_response_remains_cancelable_after_completion() {
        let runtime = Runtime::start(Repository, Engine, || {}).unwrap();
        let old = Cancellation::default();
        runtime
            .submit(Action::Query(LibraryQuery::default(), old.clone()))
            .unwrap();
        let response = runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        // Completion is not delivery: typing a newer query must invalidate this
        // already-queued response before it can replace the visible collection.
        old.cancel();
        let Update::Library(_, _, token) = response else {
            panic!("query response")
        };
        assert!(token.is_cancelled());
        runtime
            .submit(Action::Query(
                LibraryQuery::default(),
                Cancellation::default(),
            ))
            .unwrap();
        let Update::Library(_, _, token) = runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
        else {
            panic!("replacement response")
        };
        assert!(!token.is_cancelled());
    }

    #[test]
    fn text_response_preserves_document_page_and_cancelable_lifetime() {
        let runtime = Runtime::start(Repository, Engine, || {}).unwrap();
        let token = Cancellation::default();
        runtime
            .submit(Action::Text("book-a".into(), 7, token.clone()))
            .unwrap();
        let update = runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        token.cancel();
        let Update::Text(id, page, text, token) = update else {
            panic!("text response")
        };
        assert_eq!(id, "book-a");
        assert_eq!(page, 7);
        assert_eq!(text, "book-a");
        assert!(token.is_cancelled());
    }

    #[test]
    fn paused_ui_saturates_exactly_at_reserved_capacity_without_losing_writes() {
        let runtime = Runtime::start(Repository, Engine, || {}).unwrap();
        for n in 0..CAPACITY {
            runtime
                .submit(Action::Favorite(format!("book-{n}"), true))
                .unwrap();
        }
        assert!(runtime.submit(Action::Navigate("extra".into(), 1)).is_err());
        let mut ids = std::collections::HashSet::new();
        for _ in 0..CAPACITY {
            let Update::Committed(id, WriteKind::Favorite) = runtime
                .updates
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
            else {
                panic!("write acknowledgement")
            };
            assert!(ids.insert(id));
        }
        runtime
            .submit(Action::Query(
                LibraryQuery::default(),
                Cancellation::default(),
            ))
            .unwrap();
        assert!(matches!(
            runtime.updates.recv_timeout(Duration::from_secs(2)),
            Ok(Update::Library(_, _, _))
        ));
    }
    #[test]
    fn shutdown_joins_without_ui_drain_even_when_all_terminal_slots_are_retained() {
        let runtime = Runtime::start(Repository, Engine, || {}).unwrap();
        for n in 0..CAPACITY {
            runtime
                .submit(Action::Favorite(format!("book-{n}"), true))
                .unwrap();
        }
        let (send, recv) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            drop(runtime);
            send.send(()).unwrap();
        });
        recv.recv_timeout(Duration::from_secs(2))
            .expect("shutdown must not require UI to drain events");
    }
    type BlockedReply = Arc<std::sync::Mutex<Option<Sender<Result<Arc<RenderedPage>>>>>>;
    struct BlockedEngine {
        entered: Sender<()>,
        reply: BlockedReply,
    }
    impl PdfEngine for BlockedEngine {
        fn open(
            &self,
            id: String,
            path: PathBuf,
            password: Option<String>,
            cancel: Cancellation,
        ) -> Result<PdfInfo> {
            Engine.open(id, path, password, cancel)
        }
        fn render(&self, _: RenderKey, _: Priority, _: Cancellation) -> Result<Arc<RenderedPage>> {
            panic!("runtime must use nonblocking ticket admission")
        }
        fn text(&self, id: String, page: u32, cancel: Cancellation) -> Result<String> {
            Engine.text(id, page, cancel)
        }
        fn close(&self, id: String) -> Result<()> {
            Engine.close(id)
        }
    }
    impl AsyncPdfEngine for BlockedEngine {
        fn try_open(
            &self,
            id: String,
            path: PathBuf,
            password: Option<String>,
            cancel: Cancellation,
        ) -> Result<PdfTicket<PdfInfo>> {
            Engine.try_open(id, path, password, cancel)
        }
        fn try_render(
            &self,
            _: RenderKey,
            _: Priority,
            cancel: Cancellation,
        ) -> Result<PdfTicket<Arc<RenderedPage>>> {
            let (send, ticket) = PdfTicket::channel(42, 9, cancel);
            *self.reply.lock().unwrap() = Some(send);
            self.entered.try_send(()).unwrap();
            Ok(ticket)
        }
        fn try_text(
            &self,
            id: String,
            page: u32,
            cancel: Cancellation,
        ) -> Result<PdfTicket<String>> {
            Engine.try_text(id, page, cancel)
        }
        fn try_close(&self, id: String) -> Result<PdfTicket<()>> {
            Engine.try_close(id)
        }
    }
    #[test]
    fn sql_query_finishes_while_native_render_ticket_remains_blocked() {
        let (entered, wait) = bounded(1);
        let reply = Arc::new(std::sync::Mutex::new(None));
        let runtime = Runtime::start(
            Repository,
            BlockedEngine {
                entered,
                reply: reply.clone(),
            },
            || {},
        )
        .unwrap();
        runtime
            .submit(Action::Render(
                RenderKey {
                    document_id: "book".into(),
                    page: 0,
                    region: None,
                    physical_width: 10,
                    physical_height: 10,
                    rotation: 0,
                    generation: 1,
                },
                Cancellation::default(),
            ))
            .unwrap();
        wait.recv_timeout(Duration::from_secs(2)).unwrap();
        runtime
            .submit(Action::Query(
                LibraryQuery::default(),
                Cancellation::default(),
            ))
            .unwrap();
        assert!(matches!(
            runtime.updates.recv_timeout(Duration::from_secs(2)),
            Ok(Update::Library(_, _, _))
        ));
        reply
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .try_send(Err(Failure::MemoryBudget))
            .unwrap();
        assert!(matches!(
            runtime.updates.recv_timeout(Duration::from_secs(2)),
            Ok(Update::Failed(Failure::MemoryBudget, _))
        ));
    }

    #[test]
    fn reader_preferences_load_keeps_cancellation_and_foreign_document_save_is_rejected() {
        let runtime = Runtime::start(Repository, Engine, || {}).unwrap();
        let token = Cancellation::default();
        runtime
            .submit(Action::LoadReaderPreferences("book".into(), token.clone()))
            .unwrap();
        let Update::ReaderPreferences(id, prefs, cancel) = runtime
            .updates
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
        else {
            panic!("preferences reply")
        };
        assert_eq!(id, "book");
        assert!(prefs.is_none());
        token.cancel();
        assert!(cancel.is_cancelled());
        runtime
            .submit(Action::SaveReaderPreferences(
                "book".into(),
                ReaderPreferences::new("other"),
            ))
            .unwrap();
        assert!(matches!(
            runtime.updates.recv_timeout(Duration::from_secs(2)),
            Ok(Update::Failed(Failure::Invalid(_), None))
        ));
    }

    #[test]
    fn close_control_is_admitted_with_all_data_slots_retained_by_paused_ui() {
        let runtime = Runtime::start(Repository, Engine, || {}).unwrap();
        for n in 0..CAPACITY {
            runtime
                .submit(Action::Favorite(format!("book-{n}"), true))
                .unwrap();
        }
        runtime
            .submit(Action::Close("active".into()))
            .expect("close has a separate reservation");
        let mut writes = 0;
        let mut closed = false;
        for _ in 0..=CAPACITY {
            match runtime
                .updates
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
            {
                Update::Committed(_, WriteKind::Favorite) => writes += 1,
                Update::Committed(id, WriteKind::Close) => {
                    assert_eq!(id, "active");
                    assert!(!closed);
                    closed = true;
                }
                _ => panic!("terminal write or close acknowledgement"),
            }
        }
        assert_eq!(writes, CAPACITY);
        assert!(closed);
    }
}
