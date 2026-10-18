use crate::annotation_export::worker::{AnnotationExportReply, AnnotationExportRequest, ExportJob};
use crate::annotations::{
    AnnotationMarksReply, AnnotationMarksRequest, AnnotationReply, AnnotationRequest,
};
use crate::cover_job::{CoverJob, CoverStep};
use crate::cover_source::CoverSource;
use crate::document_find::{FindBatch, FindRequest, FindScan, ScanStep};
use crate::file_comparison::{ComparisonJob, ComparisonReply, ComparisonRequest};
use crate::folder_import::FolderImport;
use crate::import_enrichment::{ImportEnrichment, ImportStep};
use crate::reader_close::{ReaderCloseReply, ReaderCloseRequest};
use crate::reader_continuity::{ContinuityReply, ContinuityRequest, ContinuityWriter};
use crate::reading_sessions::deletion::{ReadingSessionDeleteReply, ReadingSessionDeleteRequest};
use crate::reading_sessions::goals::{ReadingGoalReply, ReadingGoalRequest};
use crate::reading_sessions::{
    ReadingHistoryReply, ReadingHistoryRequest, ReadingSessionReply, ReadingSessionRequest,
};
use crate::search_index::{IndexCommand, IndexProgress, SearchIndexController};
use crossbeam_channel::{Receiver, RecvTimeoutError, Select, Sender, TryRecvError, bounded};
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use yamantaka_domain::annotations::repository::AnnotationRepository;
use yamantaka_domain::bookmarks::{BookmarkChange, BookmarkPage, BookmarkRepository};
use yamantaka_domain::pdf_ticket::{AsyncPdfEngine, PdfTicket};
use yamantaka_domain::reader_preferences::ReaderPreferences;
use yamantaka_domain::reading_sessions::ReadingSessionRepository;
use yamantaka_domain::search::{IndexStats, SearchQuery, SearchRepository, SearchResult};
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryQuery, LibraryRepository, NewDocument, PdfInfo,
    Priority, RenderKey, RenderedPage, Result,
};

pub enum Action {
    CompareFiles(Box<ComparisonRequest>),
    ReanalyzeMetadata(
        Box<yamantaka_domain::metadata_review::MetadataReview>,
        Cancellation,
    ),
    Metadata(Box<crate::metadata_review::MetadataRequest>),
    MetadataSource(Box<crate::metadata_source::MetadataSourceRequest>),
    RestoreAutomaticCover(CoverSource, u64, Cancellation),
    CoverSourcePage(String, Cancellation),
    PreviewCover(CoverSource, u32, u64, Cancellation),
    SaveManualCover(Box<crate::manual_cover::ManualCoverRequest>),
    Cover(CoverSource, u64, Cancellation),
    ReaderContinuity(Box<ContinuityRequest>),
    ReadingHistory(Box<ReadingHistoryRequest>),
    TrimReadingHistory(usize, Cancellation),
    ReadingGoal(Box<ReadingGoalRequest>),
    DeleteReadingSession(Box<ReadingSessionDeleteRequest>),
    ReadingSession(Box<ReadingSessionRequest>),
    AnnotationExport(Box<AnnotationExportRequest>),
    AnnotationMarks(Box<AnnotationMarksRequest>),
    Annotation(Box<AnnotationRequest>),
    Bookmarks(String, usize, Cancellation),
    ChangeBookmark(String, BookmarkChange, usize, Cancellation),
    GetDocument(String, Cancellation),
    IndexState(String, Cancellation),
    Search(SearchQuery, Cancellation),
    Index(IndexCommand),
    IndexStatistics,
    VisibleDocuments(Vec<String>),
    ConfirmOpen(String, Cancellation),
    Query(LibraryQuery, Cancellation),
    Favorite(String, bool),
    LoadReaderPreferences(String, Cancellation),
    SaveReaderPreferences(String, ReaderPreferences),
    Open(
        Document,
        Option<yamantaka_domain::PdfPassword>,
        Cancellation,
    ),
    Import(Vec<PathBuf>, Cancellation),
    ImportFolder(PathBuf, Cancellation),
    Render(RenderKey, Priority, Cancellation),
    Navigate(String, u32),
    Text(String, u32, Cancellation),
    Outline(String, Cancellation),
    PageLinks(String, u32, Cancellation),
    TextGeometry(String, u32, Cancellation),
    FindDocument(FindRequest),
    FindPage(
        String,
        u32,
        yamantaka_domain::pdf_find::PdfFindQuery,
        u32,
        Cancellation,
    ),
    Close(ReaderCloseRequest),
    Shutdown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteKind {
    Index,
    VisibleDocuments,
    Favorite,
    Position,
    ReaderPreferences,
}
pub enum Update {
    FileComparison(Box<ComparisonReply>),
    Metadata(Box<crate::metadata_review::MetadataReply>),
    MetadataSource(crate::metadata_source::MetadataSourceReply),
    CoverSourcePage(String, Result<Option<u32>>, Cancellation),
    ManualCover(crate::manual_cover::ManualCoverReply),
    Cover(String, Result<Arc<RenderedPage>>, Cancellation),
    ReaderContinuity(ContinuityReply),
    ReaderClosed(ReaderCloseReply),
    ReadingHistory(ReadingHistoryReply),
    ReadingHistoryTrimmed(Result<u64>, Cancellation),
    ReadingGoal(ReadingGoalReply),
    ReadingSessionDeleted(ReadingSessionDeleteReply),
    ReadingSession(ReadingSessionReply),
    AnnotationExport(AnnotationExportReply),
    AnnotationMarks(AnnotationMarksReply),
    Annotations(AnnotationReply),
    Bookmarks(String, usize, BookmarkPage, Cancellation),
    BookmarkFailed(String, Failure, Cancellation),
    SearchFailed(Failure, Cancellation),
    Document(String, Option<Document>, Cancellation),
    IndexState(
        String,
        Option<yamantaka_domain::search::IndexState>,
        Cancellation,
    ),
    Search(Vec<SearchResult>, Cancellation),
    IndexStatistics(IndexStats),
    Library(Vec<Document>, u64, Cancellation),
    Opened(Box<Document>, PdfInfo, Cancellation),
    OpenFailed(Box<Document>, Failure, Cancellation),
    Page(Arc<RenderedPage>, Cancellation, u64, u64),
    Text(String, u32, String, Cancellation),
    Outline(
        String,
        Arc<yamantaka_domain::outline::PdfOutline>,
        Cancellation,
    ),
    OutlineFailed(String, Failure, Cancellation),
    TextGeometry(
        String,
        Arc<yamantaka_domain::pdf_text::PdfPageText>,
        Cancellation,
    ),
    TextGeometryFailed(String, u32, Failure, Cancellation),
    PageLinks(
        String,
        Arc<yamantaka_domain::pdf_links::PdfPageLinks>,
        Cancellation,
    ),
    PageLinksFailed(String, u32, Failure, Cancellation),
    DocumentMatches(String, FindBatch, Cancellation),
    DocumentFindFailed(String, Failure, Cancellation),
    PageMatches(
        String,
        Arc<yamantaka_domain::pdf_find::PdfPageMatches>,
        Cancellation,
    ),
    PageFindFailed(String, Failure, Cancellation),
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
    pdf_capacity_revision: Arc<AtomicU64>,
    waiting_covers: Arc<AtomicUsize>,
    wake: Sender<()>,
    import_revision: Arc<AtomicU64>,
    index_progress: Arc<Mutex<IndexProgress>>,
    sender: Sender<Command>,
    shutdown: Sender<RuntimeControl>,
    outstanding: Arc<AtomicUsize>,
    control_outstanding: Arc<AtomicUsize>,
    pub updates: UpdateReceiver,
    handle: Option<JoinHandle<()>>,
}
enum RuntimeControl {
    Stop,
    PersistContinuity(
        u64,
        Box<yamantaka_domain::reader_tabs::DocumentTabs>,
        Sender<Result<()>>,
    ),
    PersistSessions(Vec<ReadingSessionRequest>, usize, Sender<Result<()>>),
}
struct ImportState {
    folder: Option<Box<FolderImport>>,
    paths: std::collections::VecDeque<PathBuf>,
    count: usize,
    cancel: Cancellation,
    permit: Permit,
    reanalysis: Option<Box<(yamantaka_domain::metadata_review::MetadataReview, Document)>>,
}
enum Pending {
    ImportFolder(Box<FolderImport>, ImportState),
    FindDocument(Box<FindScan>, Permit),
    Open(Document, PdfTicket<PdfInfo>, Permit, u64),
    Render(PdfTicket<Arc<RenderedPage>>, Permit),
    Text(String, u32, PdfTicket<String>, Permit),
    Outline(
        String,
        PdfTicket<Arc<yamantaka_domain::outline::PdfOutline>>,
        Permit,
    ),
    TextGeometry(
        String,
        u32,
        PdfTicket<Arc<yamantaka_domain::pdf_text::PdfPageText>>,
        Permit,
    ),
    PageLinks(
        String,
        u32,
        PdfTicket<Arc<yamantaka_domain::pdf_links::PdfPageLinks>>,
        Permit,
    ),
    FindPage(
        String,
        PdfTicket<Arc<yamantaka_domain::pdf_find::PdfPageMatches>>,
        Permit,
    ),
    Close(ReaderCloseRequest, PdfTicket<()>, Permit),
    ImportOpen(PathBuf, String, PdfTicket<PdfInfo>, ImportState),
    ImportSource(
        PathBuf,
        String,
        PdfInfo,
        PdfTicket<yamantaka_domain::source_revision::SourceRevision>,
        ImportState,
    ),
    ImportScan(Box<ImportEnrichment>, ImportState),
    ImportClose(Box<NewDocument>, PdfTicket<()>, ImportState),
    ImportAbort(Failure, PdfTicket<()>, ImportState),
}
impl Pending {
    fn register<'a>(&'a self, select: &mut Select<'a>) {
        match self {
            Self::ImportFolder(folder, _) => folder.register(select),
            Self::FindDocument(scan, _) => {
                select.recv(scan.ticket().readiness());
            }
            Self::Open(_, ticket, _, _) | Self::ImportOpen(_, _, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::Render(ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::Text(_, _, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::Outline(_, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::FindPage(_, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::TextGeometry(_, _, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::PageLinks(_, _, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::ImportSource(_, _, _, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Self::ImportScan(scan, _) => {
                scan.register(select);
            }
            Self::Close(_, ticket, _)
            | Self::ImportClose(_, ticket, _)
            | Self::ImportAbort(_, ticket, _) => {
                select.recv(ticket.readiness());
            }
        }
    }
    fn ready(&self) -> bool {
        match self {
            Self::ImportFolder(folder, state) => state.cancel.is_cancelled() || folder.ready(),
            Self::FindDocument(scan, _) => scan.ticket().is_ready(),
            Self::Open(_, ticket, _, _) | Self::ImportOpen(_, _, ticket, _) => ticket.is_ready(),
            Self::Render(ticket, _) => ticket.is_ready(),
            Self::Text(_, _, ticket, _) => ticket.is_ready(),
            Self::Outline(_, ticket, _) => ticket.is_ready(),
            Self::FindPage(_, ticket, _) => ticket.is_ready(),
            Self::PageLinks(_, _, ticket, _) => ticket.is_ready(),
            Self::TextGeometry(_, _, ticket, _) => ticket.is_ready(),
            Self::ImportSource(_, _, _, ticket, _) => ticket.is_ready(),
            Self::ImportScan(scan, _) => scan.ready(),
            Self::Close(_, ticket, _)
            | Self::ImportClose(_, ticket, _)
            | Self::ImportAbort(_, ticket, _) => ticket.is_ready(),
        }
    }
    fn cancel(&self) {
        match self {
            Self::ImportFolder(_, state) => state.cancel.cancel(),
            Self::FindDocument(scan, _) => scan.request.cancellation.cancel(),
            Self::Open(_, ticket, _, _) | Self::ImportOpen(_, _, ticket, _) => {
                ticket.cancellation.cancel()
            }
            Self::Render(ticket, _) => ticket.cancellation.cancel(),
            Self::Text(_, _, ticket, _) => ticket.cancellation.cancel(),
            Self::Outline(_, ticket, _) => ticket.cancellation.cancel(),
            Self::FindPage(_, ticket, _) => ticket.cancellation.cancel(),
            Self::PageLinks(_, _, ticket, _) => ticket.cancellation.cancel(),
            Self::TextGeometry(_, _, ticket, _) => ticket.cancellation.cancel(),
            Self::ImportSource(_, _, _, ticket, _) => ticket.cancellation.cancel(),
            Self::ImportScan(scan, _) => scan.cancel(),
            Self::Close(_, ticket, _)
            | Self::ImportClose(_, ticket, _)
            | Self::ImportAbort(_, ticket, _) => ticket.cancellation.cancel(),
        }
    }
}
fn queue_import_step(
    engine: &impl AsyncPdfEngine,
    id: String,
    step: ImportStep,
    state: ImportState,
    pending: &mut Vec<Pending>,
    events: &Sender<Event>,
    notify: &impl Fn(),
) {
    match step {
        ImportStep::Pending(scan) => pending.push(Pending::ImportScan(scan, state)),
        ImportStep::Failed(error) => {
            close_failed_import(engine, id, error, state, pending, events, notify)
        }
        ImportStep::Ready(document) => match engine.try_close(id) {
            Ok(ticket) => pending.push(Pending::ImportClose(document, ticket, state)),
            Err(error) => import_failed(events, notify, error, state),
        },
    }
}
fn close_failed_import(
    engine: &impl AsyncPdfEngine,
    id: String,
    error: Failure,
    state: ImportState,
    pending: &mut Vec<Pending>,
    events: &Sender<Event>,
    notify: &impl Fn(),
) {
    match engine.try_close(id) {
        Ok(ticket) => pending.push(Pending::ImportAbort(error, ticket, state)),
        Err(error) => import_failed(events, notify, error, state),
    }
}
fn foreground_admissible(
    action: &Action,
    pending: &[Pending],
    awaiting: &Option<(String, Cancellation)>,
) -> bool {
    match action {
        Action::Import(..) | Action::ImportFolder(..) | Action::ReanalyzeMetadata(..) => {
            awaiting.is_none() && !pending.iter().any(|item| matches!(item, Pending::Open(..)))
        }
        Action::Open(document, ..) => awaiting
            .as_ref()
            .is_none_or(|(_, token)| token.is_cancelled())
            && !pending.iter().any(
                |item| matches!(item,Pending::Close(request, ..) if request.document==document.id),
            ),
        _ => true,
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
fn import_failed(events: &Sender<Event>, notify: &impl Fn(), error: Failure, state: ImportState) {
    if let Some(target) = state.reanalysis {
        emit(
            events,
            notify,
            Update::Metadata(Box::new(crate::metadata_review::MetadataReply {
                document: target.0.document,
                writing: true,
                cancel: state.cancel,
                result: Err(error),
            })),
            state.permit,
        );
        return;
    }
    emit(
        events,
        notify,
        Update::Failed(error, Some(state.cancel)),
        state.permit,
    );
}
fn open_import(
    engine: &impl AsyncPdfEngine,
    path: PathBuf,
    id: String,
    state: ImportState,
    pending: &mut Vec<Pending>,
    events: &Sender<Event>,
    notify: &impl Fn(),
) {
    if let Err(error) = state.cancel.check() {
        import_failed(events, notify, error, state);
        return;
    }
    match engine.try_open(
        id.clone(),
        path.clone(),
        None,
        yamantaka_domain::pdf_ticket::PdfOpenPurpose::Background,
        state.cancel.clone(),
    ) {
        Ok(ticket) => pending.push(Pending::ImportOpen(path, id, ticket, state)),
        Err(error) => import_failed(events, notify, error, state),
    }
}
fn advance_import<E: AsyncPdfEngine>(
    engine: &E,
    repository: &impl LibraryRepository,
    mut state: ImportState,
    pending: &mut Vec<Pending>,
    events: &Sender<Event>,
    notify: &impl Fn(),
) {
    if state.cancel.is_cancelled() {
        import_failed(events, notify, Failure::Cancelled, state);
        return;
    }
    if let Some(path) = state.paths.pop_front() {
        let reserved = crate::temporary_document::reserve(repository, "rust-import");
        let id = match reserved {
            Ok(id) => id,
            Err(error) => {
                import_failed(events, notify, error, state);
                return;
            }
        };
        open_import(engine, path, id, state, pending, events, notify);
    } else if let Some(folder) = state.folder.take() {
        if let Err(error) = folder.request_next() {
            import_failed(events, notify, error, state);
        } else {
            pending.push(Pending::ImportFolder(folder, state));
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
    pub fn start<
        R: LibraryRepository
            + SearchRepository
            + BookmarkRepository
            + AnnotationRepository
            + ReadingSessionRepository
            + Send
            + 'static,
        E: AsyncPdfEngine + Send + 'static,
    >(
        mut repository: R,
        engine: E,
        notify: impl Fn() + Send + 'static,
    ) -> Result<Self> {
        let (sender, commands) = bounded::<Command>(CAPACITY * 2);
        let (events, updates) = bounded(CAPACITY * 2);
        let (shutdown, stop) = bounded::<RuntimeControl>(1);
        let (wake, wakeups) = bounded::<()>(1);
        let waiting_covers = Arc::new(AtomicUsize::new(0));
        let waiting_snapshot = waiting_covers.clone();
        let index_progress = Arc::new(Mutex::new(IndexProgress::default()));
        let progress_snapshot = index_progress.clone();
        let import_revision = Arc::new(AtomicU64::new(0));
        let import_progress = import_revision.clone();
        let pdf_capacity_revision = Arc::new(AtomicU64::new(0));
        let pdf_capacity = pdf_capacity_revision.clone();
        let handle = thread::Builder::new()
            .name("yamantaka-use-cases".into())
            .spawn(move || {
                let mut pending: Vec<Pending> = Vec::new();
                let mut covers: Vec<(Box<CoverJob>, Permit)> = Vec::new();
                let mut continuity = ContinuityWriter::default();
                let mut export: Option<(ExportJob, Permit)> = None;
                let mut comparison: Option<(ComparisonJob, Permit)> = None;
                let mut indexer = SearchIndexController::default();
                let mut index_cleanup: Option<PdfTicket<()>> = None;
                let mut deferred: VecDeque<Command> = VecDeque::new();
                // Only readers accepted by the UI are committed visible owners.
                let mut visible: Vec<String> = Vec::new();
                let mut open_revision = 0u64;
                let mut awaiting_open: Option<(String, Cancellation)> = None;
                loop {
                    if wakeups.try_recv().is_ok() {engine.wake_cancelled();}
                    match stop.try_recv() {
                        Ok(RuntimeControl::Stop) => break,
                        Ok(RuntimeControl::PersistContinuity(revision, tabs, completed)) => {
                            let _ = completed.try_send(continuity.save(&mut repository, revision, &tabs));
                        }
                        Ok(RuntimeControl::PersistSessions(requests, limit, completed)) => {
                            let mut result = Ok(());
                            for request in requests {
                                let outcome = request.execute(&mut repository).result;
                                if result.is_ok() { result = outcome; }
                            }
                            if result.is_ok() {
                                result = repository.trim_reading_history(limit, &Cancellation::default()).map(|_| ());
                            }
                            let _ = completed.try_send(result);
                        }
                        Err(_) => {}
                    }
                    let index_busy =
                        indexer.native_document_id().is_some() || index_cleanup.is_some();
                    let deferred_ready = !index_busy && deferred.front().is_some_and(|command| foreground_admissible(&command.action, &pending, &awaiting_open));
                    let command = if deferred_ready {
                        deferred.pop_front()
                    } else {
                        commands.try_recv().ok()
                    };
                    if let Some(Command { action, permit }) = command {
                        if (index_busy
                            && matches!(
                                &action,
                                Action::Open(..)
                                    | Action::ImportFolder(..)
                                    | Action::Import(..)
                                    | Action::ReanalyzeMetadata(..)
                                    | Action::VisibleDocuments(..)
                            )) || !foreground_admissible(&action, &pending, &awaiting_open)
                        {
                            indexer.set_slot_available(false);
                            deferred.push_back(Command { action, permit });
                        } else {
                            match action {
                                Action::ReadingHistory(request) => emit(&events, &notify, Update::ReadingHistory(request.execute(&repository)), permit),
                                Action::DeleteReadingSession(request) => emit(&events,&notify,Update::ReadingSessionDeleted(request.execute(&mut repository)),permit),
                                Action::MetadataSource(request)=>emit(&events,&notify,Update::MetadataSource(request.execute(&repository)),permit),
                                Action::Metadata(request) => {
                                    let access=if indexer.progress().enabled {yamantaka_domain::search::IndexAccess::Enabled} else {yamantaka_domain::search::IndexAccess::Disabled};
                                    emit(&events,&notify,Update::Metadata(Box::new(request.execute(&mut repository,access))),permit);
                                }
                                Action::ReadingGoal(request) => emit(&events,&notify,Update::ReadingGoal(request.execute(&mut repository)),permit),
                                Action::TrimReadingHistory(limit, cancel) => {
                                    let result = repository.trim_reading_history(limit, &cancel);
                                    emit(&events, &notify, Update::ReadingHistoryTrimmed(result, cancel), permit);
                                }
                                Action::ReadingSession(request) => emit(
                                    &events, &notify,
                                    Update::ReadingSession(request.execute(&mut repository)), permit,
                                ),
                                Action::Shutdown => break,
                                Action::CompareFiles(request) => {
                                    let document = request.document.clone();
                                    let candidate = request.candidate.clone();
                                    let cancel = request.cancel.clone();
                                    let result = if comparison.is_some() { Err(Failure::QueueFull) }
                                        else { ComparisonJob::start(*request, &repository) };
                                    match result {
                                        Ok(job) => comparison = Some((job,permit)),
                                        Err(error) => emit(&events,&notify,Update::FileComparison(Box::new(
                                            ComparisonReply {document,candidate,result:Err(error),cancel}
                                        )),permit),
                                    }
                                }
                                Action::AnnotationExport(request) => {
                                    let scope = request.scope();
                                    let cancel = request.cancel.clone();
                                    let result = if export.is_some() {
                                        Err(Failure::QueueFull)
                                    } else {
                                        cancel.check().and_then(|()| repository.export_reader())
                                            .and_then(|reader| ExportJob::start(*request, reader))
                                    };
                                    match result {
                                        Ok(job) => export = Some((job, permit)),
                                        Err(error) => emit(&events, &notify, Update::AnnotationExport(
                                            AnnotationExportReply { scope, result: Err(error), cancel }
                                        ), permit),
                                    }
                                }
                                Action::AnnotationMarks(request) => {
                                    let reply = request.execute(&repository);
                                    emit(&events, &notify, Update::AnnotationMarks(reply), permit);
                                }
                                Action::Annotation(request) => {
                                    let access = if indexer.progress().enabled {
                                        yamantaka_domain::search::IndexAccess::Enabled
                                    } else {
                                        yamantaka_domain::search::IndexAccess::Disabled
                                    };
                                    let reply = request.execute(&mut repository, access);
                                    emit(&events, &notify, Update::Annotations(reply), permit);
                                }
                                Action::Bookmarks(id, offset, cancel) => {
                                    match repository.bookmarks(&id, offset, &cancel) {
                                        Ok(page) => emit(&events, &notify, Update::Bookmarks(id, offset, page, cancel), permit),
                                        Err(error) => emit(&events, &notify, Update::BookmarkFailed(id, error, cancel), permit),
                                    }
                                }
                                Action::ChangeBookmark(id, change, offset, cancel) => {
                                    let access = if indexer.progress().enabled {
                                        yamantaka_domain::search::IndexAccess::Enabled
                                    } else {
                                        yamantaka_domain::search::IndexAccess::Disabled
                                    };
                                    let result = repository.change_bookmark(&id, &change, offset, access, &cancel);
                                    match result {
                                        Ok(page) => emit(&events, &notify, Update::Bookmarks(id, offset, page, cancel), permit),
                                        Err(error) => emit(&events, &notify, Update::BookmarkFailed(id, error, cancel), permit),
                                    }
                                }
                                Action::GetDocument(id, cancel) => {
                                    let result = cancel
                                        .check()
                                        .and_then(|()| repository.get(&id))
                                        .and_then(|document| {
                                            cancel.check()?;
                                            Ok(document)
                                        });
                                    match result {
                                        Ok(document) => emit(
                                            &events,
                                            &notify,
                                            Update::Document(id, document, cancel),
                                            permit,
                                        ),
                                        Err(error) => {
                                            failed(&events, &notify, error, Some(cancel), permit)
                                        }
                                    }
                                }
                                Action::IndexState(id, cancel) => {
                                    let result = cancel
                                        .check()
                                        .and_then(|()| repository.index_state(&id))
                                        .and_then(|state| {
                                            cancel.check()?;
                                            Ok(state)
                                        });
                                    match result {
                                        Ok(state) => emit(
                                            &events,
                                            &notify,
                                            Update::IndexState(id, state, cancel),
                                            permit,
                                        ),
                                        Err(error) => {
                                            failed(&events, &notify, error, Some(cancel), permit)
                                        }
                                    }
                                }
                                Action::Index(command) => {
                                    let result = if index_cleanup.is_some()
                                        && matches!(&command, IndexCommand::Start { .. })
                                    {
                                        Err(Failure::Invalid(
                                            "The previous index document is still closing".into(),
                                        ))
                                    } else {
                                        indexer.command(command, &mut repository)
                                    };
                                    match result {
                                        Ok(()) => emit(
                                            &events,
                                            &notify,
                                            Update::Committed(String::new(), WriteKind::Index),
                                            permit,
                                        ),
                                        Err(error) => failed(&events, &notify, error, None, permit),
                                    }
                                }
                                Action::Search(query, cancel) => {
                                    let result = cancel
                                        .check()
                                        .and_then(|()| {
                                            if indexer.progress().enabled {
                                                repository.search(&query, &cancel)
                                            } else {
                                                Err(Failure::Invalid(
                                                    "Global indexing is disabled".into(),
                                                ))
                                            }
                                        })
                                        .and_then(|results| {
                                            cancel.check()?;
                                            Ok(results)
                                        });
                                    match result {
                                        Ok(results) => emit(
                                            &events,
                                            &notify,
                                            Update::Search(results, cancel),
                                            permit,
                                        ),
                                        Err(error) => emit(
                                            &events,
                                            &notify,
                                            Update::SearchFailed(error, cancel),
                                            permit,
                                        ),
                                    }
                                }
                                Action::IndexStatistics => match repository.index_stats() {
                                    Ok(stats) => emit(
                                        &events,
                                        &notify,
                                        Update::IndexStatistics(stats),
                                        permit,
                                    ),
                                    Err(error) => failed(&events, &notify, error, None, permit),
                                },
                                Action::VisibleDocuments(ids) => {
                                    match engine.set_visible_documents(&ids) {
                                        Ok(()) => {
                                            for item in &pending {
                                                if matches!(item, Pending::Open(..)) { item.cancel(); }
                                            }
                                            if let Some((_, token)) = awaiting_open.take() { token.cancel(); }
                                            visible = ids;
                                            pdf_capacity.fetch_add(1,Ordering::Release);
                                            emit(
                                                &events,
                                                &notify,
                                                Update::Committed(
                                                    String::new(),
                                                    WriteKind::VisibleDocuments,
                                                ),
                                                permit,
                                            );
                                        }
                                        Err(error) => failed(&events, &notify, error, None, permit),
                                    }
                                }
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
                                Action::ReaderContinuity(request) => {
                                    let reply = continuity.execute(&mut repository, *request);
                                    emit(&events, &notify, Update::ReaderContinuity(reply), permit);
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
                                Action::Navigate(id, page) => {
                                    match repository.save_position(&id, page) {
                                        Ok(()) => emit(
                                            &events,
                                            &notify,
                                            Update::Committed(id, WriteKind::Position),
                                            permit,
                                        ),
                                        Err(error) => failed(&events, &notify, error, None, permit),
                                    }
                                }
                                Action::ConfirmOpen(id, cancel) => {
                                    let matches = awaiting_open.as_ref().is_some_and(|(target, token)| target == &id && token.same_request(&cancel));
                                    if matches {
                                        let ids = if cancel.is_cancelled() { visible.clone() } else { vec![id.clone()] };
                                        match engine.set_visible_documents(&ids) {
                                            Ok(()) => { visible = ids; awaiting_open = None; pdf_capacity.fetch_add(1,Ordering::Release); }
                                            Err(error) => { failed(&events, &notify, error, Some(cancel), permit); continue; }
                                        }
                                    }
                                    // A stale confirmation is an idempotent no-op.
                                    emit(&events, &notify, Update::Committed(id, WriteKind::VisibleDocuments), permit);
                                }
                                Action::Open(document, password, cancel) => {
                                    let mut protected = visible.clone();
                                    if !protected.contains(&document.id) { protected.push(document.id.clone()); }
                                    let revision = open_revision.checked_add(1);
                                    let preparation = cancel.check().and_then(|()| {
                                        if protected.len() > yamantaka_domain::MAX_HOT_DOCUMENTS {
                                            return Err(Failure::Invalid("Close a visible reader before opening another document".into()));
                                        }
                                        revision.ok_or_else(|| Failure::Invalid("Document opening revision exhausted".into()))
                                    });
                                    let revision = match preparation {
                                        Ok(revision) => revision,
                                        Err(error) => { emit(&events, &notify, Update::OpenFailed(Box::new(document), error, cancel), permit); continue; }
                                    };
                                    open_revision = revision;
                                    for item in &pending {
                                        if matches!(item, Pending::Open(..)) { item.cancel(); }
                                    }
                                    if let Some((_, token)) = awaiting_open.take() { token.cancel(); }
                                    let result = engine.set_visible_documents(&protected).and_then(|()| engine.try_open(
                                        document.id.clone(), document.path.clone(), password,yamantaka_domain::pdf_ticket::PdfOpenPurpose::Reader, cancel.clone(),
                                    ));
                                    match result {
                                        Ok(ticket) => pending.push(Pending::Open(document, ticket, permit, revision)),
                                        Err(error) => {
                                            let error = match engine.set_visible_documents(&visible) { Ok(()) => { pdf_capacity.fetch_add(1,Ordering::Release); error }, Err(error) => error };
                                            emit(&events, &notify, Update::OpenFailed(Box::new(document), error, cancel), permit);
                                        }
                                    }
                                },
                                Action::CoverSourcePage(id, cancel) => {
                                    let result = cancel.check().and_then(|_| repository.cover_source_page(&id)).and_then(|page| { cancel.check()?; Ok(page) });
                                    emit(&events,&notify,Update::CoverSourcePage(id,result,cancel),permit);
                                }
                                Action::SaveManualCover(request) => emit(&events,&notify,Update::ManualCover(request.execute(&mut repository)),permit),
                                Action::PreviewCover(source,page,generation,cancel) => covers.push((CoverJob::preview(source,page,generation,cancel),permit)),
                                Action::Cover(source,generation,cancel) => covers.push((CoverJob::library(source,generation,cancel),permit)),
                                Action::Render(key, priority, cancel) => {
                                    match engine.try_render(key, priority, cancel.clone())
                                    {
                                        Ok(ticket) => pending.push(Pending::Render(ticket, permit)),
                                        Err(error) => {
                                            failed(&events, &notify, error, Some(cancel), permit)
                                        }
                                    }
                                }
                                Action::FindDocument(request) => {
                                    let id = request.document_id.clone();
                                    let cancel = request.cancellation.clone();
                                    match FindScan::start(request, &engine) {
                                        Ok(scan) => pending.push(Pending::FindDocument(Box::new(scan), permit)),
                                        Err(error) => emit(&events, &notify, Update::DocumentFindFailed(id, error, cancel), permit),
                                    }
                                }
                                Action::TextGeometry(id, page, cancel) => {
                                    match engine.try_text_geometry(id.clone(), page, yamantaka_domain::pdf_text::TextGeometryPurpose::Reader, cancel.clone()) {
                                        Ok(ticket) => pending.push(Pending::TextGeometry(id, page, ticket, permit)),
                                        Err(error) => emit(&events, &notify, Update::TextGeometryFailed(id, page, error, cancel), permit),
                                    }
                                }
                                Action::PageLinks(id, page, cancel) => {
                                    match engine.try_page_links(id.clone(), page, cancel.clone()) {
                                        Ok(ticket) => pending.push(Pending::PageLinks(id, page, ticket, permit)),
                                        Err(error) => emit(&events, &notify, Update::PageLinksFailed(id, page, error, cancel), permit),
                                    }
                                }
                                Action::FindPage(id, page, query, offset, cancel) => {
                                    match engine.try_find_page(id.clone(), page, query, offset, cancel.clone()) {
                                        Ok(ticket) => pending.push(Pending::FindPage(id, ticket, permit)),
                                        Err(error) => emit(&events, &notify, Update::PageFindFailed(id, error, cancel), permit),
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
                                Action::Outline(id, cancel) => match engine.try_outline(id.clone(), cancel.clone()) {
                                    Ok(ticket) => pending.push(Pending::Outline(id, ticket, permit)),
                                    Err(error) => emit(&events, &notify, Update::OutlineFailed(id, error, cancel), permit),
                                },
                                Action::RestoreAutomaticCover(source,generation,cancel) => {
                                    let id=source.id().to_owned();
                                    match CoverJob::restore(source,generation,cancel.clone(),&repository) {
                                        Ok(job)=>covers.push((job,permit)),
                                        Err(error)=>emit(&events,&notify,Update::Cover(id,Err(error),cancel),permit),
                                    }
                                },
                                Action::Close(request) => {
                                    let valid=request.cancel.check().and_then(|()|if request.document.is_empty() {Err(Failure::Invalid("Reader close requires a document identity".into()))}else{Ok(())});
                                    if let Err(error)=valid {emit(&events,&notify,Update::ReaderClosed(request.reply(Err(error))),permit);continue;}
                                    match engine.try_close(request.document.clone()) {
                                        Ok(ticket) => {
                                            for item in &pending {if matches!(item,Pending::Open(document, ..) if document.id==request.document) {item.cancel();}}
                                            if awaiting_open.as_ref().is_some_and(|(id,_)|*id==request.document) && let Some((_,token))=awaiting_open.take() {token.cancel();}
                                            pending.push(Pending::Close(request,ticket,permit));
                                        }
                                        Err(error) => emit(&events,&notify,Update::ReaderClosed(request.reply(Err(error))),permit),
                                    }
                                },
                                Action::ReanalyzeMetadata(expected,cancel) => {
                                    let source=cancel.check().and_then(|()|repository.metadata_review(&expected.document,&cancel)).and_then(|fresh| {
                                        if fresh!=*expected {return Err(Failure::Invalid("Metadata changed; reload before analysis".into()));}
                                        repository.get(&expected.document)?.filter(|d|!d.missing).ok_or_else(||Failure::Invalid("Metadata source is missing".into()))
                                    });
                                    match source {
                                        Ok(document)=>advance_import(&engine,&repository,ImportState{folder:None,paths:vec![document.path.clone()].into(),count:0,cancel,permit,reanalysis:Some(Box::new((*expected,document)))},&mut pending,&events,&notify),
                                        Err(error)=>emit(&events,&notify,Update::Metadata(Box::new(crate::metadata_review::MetadataReply{document:expected.document,writing:true,cancel,result:Err(error)})),permit),
                                    }
                                }
                                Action::ImportFolder(root,cancel) => {
                                    match FolderImport::start(root,cancel.clone()) {
                                        Ok(folder) => advance_import(&engine,&repository,ImportState {
                                            folder:Some(Box::new(folder)),paths:VecDeque::new(),count:0,
                                            cancel,permit,reanalysis:None,
                                        },&mut pending,&events,&notify),
                                        Err(error) => emit(&events,&notify,Update::Failed(error,Some(cancel)),permit),
                                    }
                                }
                                Action::Import(paths, cancel) => advance_import(
                                    &engine,
                                    &repository,
                                    ImportState {
                                        folder: None,
                                        paths: paths.into(),
                                        count: 0,
                                        cancel,
                                        permit,
                                        reanalysis:None,
                                    },
                                    &mut pending,
                                    &events,
                                    &notify,
                                ),
                            }
                        }
                    }
                    if let Some(reply) = export.as_mut().and_then(|(job, _)| job.poll()) {
                        let (_, permit) = export.take().unwrap();
                        emit(&events, &notify, Update::AnnotationExport(reply), permit);
                    }
                    if let Some(reply) = comparison.as_mut().and_then(|(job,_)|job.poll(&repository)) {
                        let (_,permit) = comparison.take().unwrap();
                        emit(&events,&notify,Update::FileComparison(Box::new(reply)),permit);
                    }
                    let mut next_imports = Vec::new();
                    let mut index = 0;
                    while index < pending.len() {
                        if !pending[index].ready() {
                            index += 1;
                            continue;
                        }
                        match pending.remove(index) {
                            Pending::FindDocument(scan, permit) => {
                                let id = scan.request.document_id.clone();
                                let cancel = scan.request.cancellation.clone();
                                match scan.advance(&engine, &notify) {
                                    Ok(ScanStep::Continue(scan)) => {
                                        pending.insert(index, Pending::FindDocument(Box::new(scan), permit));
                                        index += 1;
                                    }
                                    Ok(ScanStep::Finished(batch)) => emit(&events, &notify, Update::DocumentMatches(id, batch, cancel), permit),
                                    Err(error) => emit(&events, &notify, Update::DocumentFindFailed(id, error, cancel), permit),
                                }
                            }
                            Pending::Open(document, ticket, permit, revision) => {
                                let cancel = ticket.cancellation.clone();
                                let result = ticket.wait().and_then(|info| {
                                    cancel.check()?;
                                    if revision != open_revision { return Err(Failure::Cancelled); }
                                    Ok(info)
                                });
                                match result {
                                    Ok(info) => {
                                        // Keep the old reader and replacement pinned until
                                        // the UI accepts this exact request, including races
                                        // where cancellation follows delivery of Opened.
                                        awaiting_open = Some((document.id.clone(), cancel.clone()));
                                        emit(&events, &notify, Update::Opened(Box::new(document), info, cancel), permit);
                                    }
                                    Err(error) => {
                                        let error = if revision == open_revision && awaiting_open.is_none() {
                                            match engine.set_visible_documents(&visible) { Ok(()) => { pdf_capacity.fetch_add(1,Ordering::Release); error }, Err(error) => error }
                                        } else { error };
                                        emit(&events, &notify, Update::OpenFailed(Box::new(document), error, cancel), permit);
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
                            Pending::Outline(id, ticket, permit) => {
                                let cancel = ticket.cancellation.clone();
                                match ticket.wait().and_then(|outline| { cancel.check()?; Ok(outline) }) {
                                    Ok(outline) => emit(&events, &notify, Update::Outline(id, outline, cancel), permit),
                                    Err(error) => emit(&events, &notify, Update::OutlineFailed(id, error, cancel), permit),
                                }
                            }
                            Pending::TextGeometry(id, page, ticket, permit) => {
                                let cancel = ticket.cancellation.clone();
                                let result = ticket.wait().and_then(|text| {
                                    cancel.check()?;
                                    if text.page() != page { return Err(Failure::Invalid("PDF text response belongs to a different page".into())); }
                                    Ok(text)
                                });
                                match result {
                                    Ok(text) => emit(&events, &notify, Update::TextGeometry(id, text, cancel), permit),
                                    Err(error) => emit(&events, &notify, Update::TextGeometryFailed(id, page, error, cancel), permit),
                                }
                            }
                            Pending::PageLinks(id, page, ticket, permit) => {
                                let cancel = ticket.cancellation.clone();
                                let result = ticket.wait().and_then(|links| {
                                    cancel.check()?;
                                    if links.page() != page { return Err(Failure::Invalid("PDF link response belongs to a different page".into())); }
                                    Ok(links)
                                });
                                match result {
                                    Ok(links) => emit(&events, &notify, Update::PageLinks(id, links, cancel), permit),
                                    Err(error) => emit(&events, &notify, Update::PageLinksFailed(id, page, error, cancel), permit),
                                }
                            }
                            Pending::FindPage(id, ticket, permit) => {
                                let cancel = ticket.cancellation.clone();
                                match ticket.wait().and_then(|matches| { cancel.check()?; Ok(matches) }) {
                                    Ok(matches) => emit(&events, &notify, Update::PageMatches(id, matches, cancel), permit),
                                    Err(error) => emit(&events, &notify, Update::PageFindFailed(id, error, cancel), permit),
                                }
                            }
                            Pending::Close(request, ticket, permit) => {
                                let result=ticket.wait();
                                if result.is_ok() {
                                    pdf_capacity.fetch_add(1,Ordering::Release);
                                    visible.retain(|id|*id!=request.document);
                                    if awaiting_open.as_ref().is_some_and(|(id,_)|*id==request.document) && let Some((_,token))=awaiting_open.take() {token.cancel();}
                                }
                                emit(&events,&notify,Update::ReaderClosed(request.reply(result)),permit);
                            },
                            Pending::ImportFolder(folder,mut state) => {
                                let result = state.cancel.check().and_then(|()|folder.receive());
                                match result {
                                    Ok(Some(path)) => {state.folder=Some(folder);state.paths.push_back(path);next_imports.push(state);}
                                    Ok(None) => {drop(folder);next_imports.push(state);}
                                    Err(error) => import_failed(&events,&notify,error,state),
                                }
                            }
                            Pending::ImportOpen(path, id, ticket, state) => match ticket.wait() {
                                Ok(info) => match engine.try_source_revision(id.clone(), state.cancel.clone()) {
                                    Ok(ticket) => pending.push(Pending::ImportSource(path,id,info,ticket,state)),
                                    Err(Failure::NativeDocumentUnavailable)=>open_import(&engine,path,id,state,&mut pending,&events,&notify),
                                    Err(error) => close_failed_import(&engine,id,error,state,&mut pending,&events,&notify),
                                },
                                Err(Failure::NativeSlotsBusy|Failure::NativeDocumentUnavailable)=>open_import(&engine,path,id,state,&mut pending,&events,&notify),
                                Err(error) => close_failed_import(&engine,id,error,state,&mut pending,&events,&notify),
                            },
                            Pending::ImportSource(path,id,info,ticket,state) => match ticket.wait() {
                                Ok(source) => {
                                    let document=Box::new(NewDocument {path,info,source,analysis:yamantaka_domain::front_matter::ImportAnalysis::NotRequested});
                                    let step=ImportEnrichment::start(&engine,id.clone(),document,state.reanalysis.as_ref().map(|target|&target.0),state.cancel.clone());
                                    queue_import_step(&engine,id,step,state,&mut pending,&events,&notify);
                                }
                                Err(Failure::NativeDocumentUnavailable)=>open_import(&engine,path,id,state,&mut pending,&events,&notify),
                                Err(error) => close_failed_import(&engine,id,error,state,&mut pending,&events,&notify),
                            },
                            Pending::ImportScan(scan,state) => {
                                let id=scan.id.clone();
                                let step=scan.advance(&engine);
                                queue_import_step(&engine,id,step,state,&mut pending,&events,&notify);
                            },
                            Pending::ImportClose(document,ticket,mut state) => {
                                let closed=ticket.wait();
                                if closed.is_ok() {pdf_capacity.fetch_add(1,Ordering::Release);}
                                if let Some(target)=state.reanalysis.as_ref() {
                                    let access=if indexer.progress().enabled {yamantaka_domain::search::IndexAccess::Enabled}else{yamantaka_domain::search::IndexAccess::Disabled};
                                    let result=closed.and_then(|()|state.cancel.check()).and_then(|()|repository.reanalyze_metadata(&target.0,&target.1,&document,access,&state.cancel));
                                    emit(&events,&notify,Update::Metadata(Box::new(crate::metadata_review::MetadataReply{document:target.0.document.clone(),writing:true,cancel:state.cancel,result})),state.permit);
                                    continue;
                                }
                                match closed.and_then(|()| {state.cancel.check()?;repository.import(&document)}) {
                                    Ok(_) => {
                                        state.count+=1;
                                        import_progress.fetch_add(1,Ordering::Release);
                                        notify();
                                        next_imports.push(state);
                                    }
                                    Err(error) => import_failed(&events,&notify,error,state),
                                }
                            }
                            Pending::ImportAbort(error,ticket,state)=>{
                                let closed=ticket.wait();
                                if closed.is_ok() {pdf_capacity.fetch_add(1,Ordering::Release);}
                                import_failed(&events,&notify,closed.err().unwrap_or(error),state);
                            }
                        }
                    }
                    let capacity=pdf_capacity.load(Ordering::Acquire);
                    let mut cover_index=0;
                    while cover_index<covers.len() {
                        // At most two transitions let a late busy reply observe a
                        // release before the next import is admitted.
                        let mut finished=false;
                        for _ in 0..2 {
                            if !covers[cover_index].0.ready(capacity) {break;}
                            let (job,permit)=covers.remove(cover_index);
                            match job.advance(&mut repository,&engine,capacity) {
                                CoverStep::Pending(job)=>covers.insert(cover_index,(job,permit)),
                                CoverStep::Finished(id,result,cancel)=>{
                                    emit(&events,&notify,Update::Cover(id,result,cancel),permit);
                                    finished=true;
                                    break;
                                }
                            }
                        }
                        if !finished {cover_index+=1;}
                    }
                    let waiting=covers.iter().filter(|(job,_)|job.waiting()).count();
                    if waiting_snapshot.swap(waiting,Ordering::AcqRel)!=waiting {notify();}
                    for state in next_imports {
                        advance_import(&engine,&repository,state,&mut pending,&events,&notify);
                    }
                    if index_cleanup.as_ref().is_some_and(PdfTicket::is_ready) && index_cleanup.take().unwrap().wait().is_ok() {pdf_capacity.fetch_add(1,Ordering::Release);notify();}
                    let native_admission_pending = !covers.is_empty() || pending.iter().any(|item| {
                        matches!(
                            item,
                            Pending::Open(..) | Pending::ImportFolder(..) | Pending::ImportOpen(..) | Pending::ImportSource(..) | Pending::ImportScan(..) | Pending::ImportClose(..) | Pending::ImportAbort(..)
                        )
                    });
                    indexer.set_slot_available(
                        deferred.is_empty()
                            && !native_admission_pending
                            && awaiting_open.is_none()
                            && visible.len() < yamantaka_domain::MAX_HOT_DOCUMENTS
                            && index_cleanup.is_none(),
                    );
                    let released=indexer.released_native_revision();
                    if let Err(error) = indexer.step(&mut repository, &engine) {
                        index_cleanup = indexer.abort(&error, &engine).ok().flatten();
                    }
                    if indexer.released_native_revision()!=released {pdf_capacity.fetch_add(1,Ordering::Release);notify();}
                    if let Some(progress) = indexer.take_progress() {
                        *progress_snapshot.lock().unwrap() = progress;
                        notify();
                    }
                    // A blocking select sleeps while idle and wakes on actual command
                    // or PDF completion. SQL is never held behind native rendering.
                    if covers.iter().any(|(job,_)|job.ready(pdf_capacity.load(Ordering::Acquire)))
                        || !commands.is_empty()
                        || indexer.needs_step()
                        || (deferred.front().is_some_and(|command| foreground_admissible(&command.action, &pending, &awaiting_open))
                            && indexer.native_document_id().is_none()
                            && index_cleanup.is_none())
                    {
                        continue;
                    }
                    let mut select = Select::new();
                    select.recv(&stop);
                    select.recv(&commands);
                    select.recv(&wakeups);
                    if let Some((job, _)) = &export {
                        job.register(&mut select);
                    }
                    if let Some((job,_)) = &comparison { job.register(&mut select); }
                    for item in &pending {
                        item.register(&mut select);
                    }
                    for (job,_) in &covers {job.register(&mut select);}
                    indexer.register_readiness(&mut select);
                    if let Some(ticket) = &index_cleanup {
                        select.recv(ticket.readiness());
                    }
                    select.ready();
                }
                for item in &pending {
                    item.cancel();
                }
                if let Some((_, token)) = awaiting_open.take() { token.cancel(); }
                for (job,_) in &covers {job.cancel();}
                drop(covers);
                drop(pending); // Drop tickets before engine join; no UI draining required.
                drop(export); // Cancel and join file work before releasing the actor.
                drop(comparison);
                if let Ok(Some(ticket)) = indexer.shutdown(&engine) {
                    let _ = ticket.wait();
                }
                if let Some(ticket) = index_cleanup {
                    let _ = ticket.wait();
                }
                drop(engine);
            })
            .map_err(|_| Failure::Storage("Use-case worker could not start".into()))?;
        Ok(Self {
            waiting_covers,
            wake,
            pdf_capacity_revision,
            import_revision,
            index_progress,
            sender,
            shutdown,
            outstanding: Arc::new(AtomicUsize::new(0)),
            control_outstanding: Arc::new(AtomicUsize::new(0)),
            updates: UpdateReceiver(updates),
            handle: Some(handle),
        })
    }
    /// Coalesced bounded snapshot; reading it never queries SQL or PDFium.
    pub fn index_progress(&self) -> IndexProgress {
        self.index_progress.lock().unwrap().clone()
    }

    pub fn import_revision(&self) -> u64 {
        self.import_revision.load(Ordering::Acquire)
    }
    pub fn waiting_covers(&self) -> usize {
        self.waiting_covers.load(Ordering::Acquire)
    }
    /// Wake parked work after cancelling its token without submitting a command.
    pub fn wake(&self) {
        let _ = self.wake.try_send(());
    }
    pub fn pdf_capacity_revision(&self) -> u64 {
        self.pdf_capacity_revision.load(Ordering::Acquire)
    }
    /// Blocking persistence barrier for shutdown/recovery, independent of UI event capacity.
    /// The caller must finalize activity first and retain these immutable commands on failure.
    pub fn persist_reading_sessions(
        &self,
        requests: Vec<ReadingSessionRequest>,
        history_limit: usize,
    ) -> Result<()> {
        if requests.len() > crate::reading_sessions::lifecycle::MAX_UNCOMPLETED_SESSIONS * 2 {
            return Err(Failure::Invalid(
                "Too many reading session recovery commands".into(),
            ));
        }
        if requests.is_empty() {
            return Ok(());
        }
        let (completed, reply) = bounded(1);
        self.shutdown
            .try_send(RuntimeControl::PersistSessions(
                requests,
                history_limit,
                completed,
            ))
            .map_err(|error| match error {
                crossbeam_channel::TrySendError::Full(_) => Failure::QueueFull,
                crossbeam_channel::TrySendError::Disconnected(_) => {
                    Failure::Storage("Use-case worker stopped".into())
                }
            })?;
        reply.recv().map_err(|_| {
            Failure::Storage("Use-case worker stopped before session persistence completed".into())
        })?
    }
    /// A newer revision fences queued older writes even when control overtakes data.
    pub fn persist_reader_continuity(
        &self,
        revision: u64,
        tabs: yamantaka_domain::reader_tabs::DocumentTabs,
    ) -> Result<()> {
        let (completed, reply) = bounded(1);
        self.shutdown
            .try_send(RuntimeControl::PersistContinuity(
                revision,
                Box::new(tabs),
                completed,
            ))
            .map_err(|error| match error {
                crossbeam_channel::TrySendError::Full(_) => Failure::QueueFull,
                crossbeam_channel::TrySendError::Disconnected(_) => {
                    Failure::Storage("Use-case worker stopped".into())
                }
            })?;
        reply.recv().map_err(|_| {
            Failure::Storage(
                "Use-case worker stopped before reader continuity persistence completed".into(),
            )
        })?
    }
    pub fn submit(&self, action: Action) -> Result<()> {
        if matches!(action, Action::Shutdown) {
            self.shutdown
                .try_send(RuntimeControl::Stop)
                .map_err(|_| Failure::Storage("Use-case worker stopped".into()))?;
            return Ok(());
        }
        let counter = if matches!(action, Action::Close(_) | Action::ConfirmOpen(..)) {
            &self.control_outstanding
        } else {
            &self.outstanding
        };
        counter
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < CAPACITY).then_some(count + 1)
            })
            .map_err(|_| Failure::QueueFull)?;
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
        let _ = self.shutdown.try_send(RuntimeControl::Stop);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
