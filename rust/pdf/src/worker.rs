use crate::cache::{Cache, RasterKey};
use crate::source::{LoadedPdf, SourceStamp};
use crossbeam_channel::{Receiver, Sender, bounded};
use pdfium_render::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
};
use yamantaka_domain::memory::{
    GlobalMemoryLimits, MemoryCategory, MemoryError, MemoryGovernor, TrackedBytesBuilder,
};
use yamantaka_domain::pdf_ticket::{AsyncPdfEngine, PdfTicket};
use yamantaka_domain::{
    Cancellation, Failure, MAX_HOT_DOCUMENTS, PdfEngine, PdfInfo, Priority, RenderKey,
    RenderedPage, Result,
};

#[derive(Clone, Copy)]
pub struct MemoryLimits {
    pub pages: usize,
    pub tiles: usize,
    pub text: usize,
    pub max_dimension: u32,
}
impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            pages: 72 * 1024 * 1024,
            tiles: 24 * 1024 * 1024,
            text: 8 * 1024 * 1024,
            max_dimension: 16384,
        }
    }
}
type Reply<T> = Response<T>;
struct Response<T> {
    sender: Sender<Result<T>>,
    _permit: Option<Arc<Permit>>,
}
impl<T> Response<T> {
    fn send(self, value: Result<T>) {
        let _ = self.sender.try_send(value);
    }
}
struct Permit(Arc<AtomicUsize>);
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}
type Consumers = Arc<Mutex<Vec<(Cancellation, Reply<Arc<RenderedPage>>)>>>;
struct InspectInfo {
    pages: yamantaka_domain::pdf_pages::PdfPagesBuilder,
    title: Option<String>,
    author: Option<String>,
}
enum Request {
    SourceRevision {
        id: String,
        epoch: u64,
        cancel: Cancellation,
        reply: Reply<yamantaka_domain::source_revision::SourceRevision>,
    },
    DetectCover {
        id: String,
        path: PathBuf,
        generation: u64,
        expected: Option<yamantaka_domain::source_revision::SourceRevision>,
        cancel: Cancellation,
        reply: Reply<yamantaka_domain::cover_policy::DetectedCover>,
    },
    Cover {
        id: String,
        path: PathBuf,
        page: u32,
        generation: u64,
        expected: Option<yamantaka_domain::source_revision::SourceRevision>,
        cancel: Cancellation,
        reply: Reply<Arc<RenderedPage>>,
    },
    Open {
        id: String,
        epoch: u64,
        path: PathBuf,
        password: Option<yamantaka_domain::PdfPassword>,
        purpose: yamantaka_domain::pdf_ticket::PdfOpenPurpose,
        cancel: Cancellation,
        reply: Reply<PdfInfo>,
    },
    Inspect {
        id: String,
        epoch: u64,
        next_page: u32,
        purpose: yamantaka_domain::pdf_ticket::PdfOpenPurpose,
        info: InspectInfo,
        cancel: Cancellation,
        reply: Reply<PdfInfo>,
    },
    Render {
        key: RenderKey,
        epoch: u64,
        priority: Priority,
        consumers: Consumers,
    },
    Text {
        id: String,
        epoch: u64,
        page: u32,
        cancel: Cancellation,
        reply: Reply<String>,
    },
    Outline {
        id: String,
        epoch: u64,
        cancel: Cancellation,
        reply: Reply<Arc<yamantaka_domain::outline::PdfOutline>>,
    },
    TextGeometry {
        id: String,
        epoch: u64,
        page: u32,
        purpose: yamantaka_domain::pdf_text::TextGeometryPurpose,
        cancel: Cancellation,
        reply: Reply<Arc<yamantaka_domain::pdf_text::PdfPageText>>,
    },
    PageLinks {
        id: String,
        epoch: u64,
        page: u32,
        cancel: Cancellation,
        reply: Reply<Arc<yamantaka_domain::pdf_links::PdfPageLinks>>,
    },
    FindPage {
        id: String,
        epoch: u64,
        page: u32,
        query: yamantaka_domain::pdf_find::PdfFindQuery,
        offset: u32,
        cancel: Cancellation,
        reply: Reply<Arc<yamantaka_domain::pdf_find::PdfPageMatches>>,
    },
    Close {
        id: String,
        epoch: u64,
        reply: Reply<()>,
    },
    Shutdown,
}
impl Request {
    fn document_id(&self) -> Option<&str> {
        match self {
            Self::DetectCover { id, .. }
            | Self::SourceRevision { id, .. }
            | Self::Cover { id, .. }
            | Self::Open { id, .. }
            | Self::Inspect { id, .. }
            | Self::Text { id, .. }
            | Self::Outline { id, .. }
            | Self::FindPage { id, .. }
            | Self::PageLinks { id, .. }
            | Self::TextGeometry { id, .. }
            | Self::Close { id, .. } => Some(id),
            Self::Render { key, .. } => Some(&key.document_id),
            Self::Shutdown => None,
        }
    }
    fn priority(&self) -> u8 {
        match self {
            Self::Shutdown | Self::Close { .. } => 0,
            Self::SourceRevision { .. } => 1,
            Self::Open { purpose, .. } | Self::Inspect { purpose, .. } => match purpose {
                yamantaka_domain::pdf_ticket::PdfOpenPurpose::Reader => 1,
                yamantaka_domain::pdf_ticket::PdfOpenPurpose::Background => 7,
            },
            Self::Cover { .. } | Self::DetectCover { .. } => 7,
            Self::Render { priority, .. } => 2 + *priority as u8,
            Self::Text { .. }
            | Self::Outline { .. }
            | Self::FindPage { .. }
            | Self::PageLinks { .. } => 4,
            Self::TextGeometry { purpose, .. } => match purpose {
                yamantaka_domain::pdf_text::TextGeometryPurpose::Reader => 4,
                yamantaka_domain::pdf_text::TextGeometryPurpose::FrontMatter => 7,
                yamantaka_domain::pdf_text::TextGeometryPurpose::Recovery => 6,
            },
        }
    }
}
#[derive(Default)]
struct Scheduler {
    queue: Vec<Request>,
    in_flight: Option<(RenderKey, u64, Consumers)>,
    epochs: HashMap<String, u64>,
    visible: HashSet<String>,
    shutdown: bool,
}
struct Inner {
    cover_cache: Option<Arc<dyn yamantaka_domain::cover_cache::CoverImageCache>>,
    wake: Sender<()>,
    scheduler: Arc<Mutex<Scheduler>>,
    handle: Mutex<Option<JoinHandle<()>>>,
    outstanding: Arc<AtomicUsize>,
    control_outstanding: Arc<AtomicUsize>,
    sequence: AtomicUsize,
    governor: MemoryGovernor,
}
const MAX_PENDING_REQUESTS: usize = 128;
impl Drop for Inner {
    fn drop(&mut self) {
        {
            let mut scheduler = self.scheduler.lock().unwrap();
            scheduler.shutdown = true;
            scheduler.queue.push(Request::Shutdown);
        }
        let _ = self.wake.try_send(());
        if let Ok(handle) = self.handle.get_mut()
            && let Some(handle) = handle.take()
        {
            let _ = handle.join();
        }
    }
}
#[derive(Clone)]
pub struct PdfWorker(Arc<Inner>);
impl PdfWorker {
    pub fn start(library: &Path, limits: MemoryLimits) -> Result<Self> {
        let total = limits
            .pages
            .checked_add(limits.tiles)
            .and_then(|bytes| bytes.checked_add(limits.text))
            .ok_or_else(|| Failure::Invalid("Memory limits overflow".into()))?;
        let governor = MemoryGovernor::new(GlobalMemoryLimits {
            total,
            pages: limits.pages,
            tiles: limits.tiles,
            text: limits.text,
            ..GlobalMemoryLimits::default()
        });
        Self::start_with_governor(library, limits, governor)
    }
    pub fn memory_governor(&self) -> MemoryGovernor {
        self.0.governor.clone()
    }
    pub fn start_with_governor(
        library: &Path,
        limits: MemoryLimits,
        governor: MemoryGovernor,
    ) -> Result<Self> {
        Self::start_with_cover_cache(library, limits, governor, None)
    }
    pub fn start_with_cover_cache(
        library: &Path,
        limits: MemoryLimits,
        governor: MemoryGovernor,
        cover_cache: Option<Arc<dyn yamantaka_domain::cover_cache::CoverImageCache>>,
    ) -> Result<Self> {
        let maintenance_cache = cover_cache.clone();
        let library = library.to_owned();
        let (wake, receiver) = bounded(1);
        let (ready, wait) = bounded(1);
        let scheduler = Arc::new(Mutex::new(Scheduler::default()));
        let worker_scheduler = scheduler.clone();
        let worker_governor = governor.clone();
        let handle = thread::Builder::new()
            .name("yamantaka-pdfium".into())
            .spawn(move || match crate::bindings::open(&library) {
                Ok(pdfium) => {
                    let _ = ready.send(Ok(()));
                    run(
                        &pdfium,
                        receiver,
                        worker_scheduler,
                        limits,
                        worker_governor,
                        cover_cache,
                    );
                }
                Err(error) => {
                    let _ = ready.send(Err(error));
                }
            })
            .map_err(|_| Failure::Pdf("PDF worker could not start".into()))?;
        if let Err(error) = wait.recv().map_err(|_| stopped())? {
            let _ = handle.join();
            return Err(error);
        }
        Ok(Self(Arc::new(Inner {
            cover_cache: maintenance_cache,
            wake,
            scheduler,
            handle: Mutex::new(Some(handle)),
            outstanding: Arc::new(AtomicUsize::new(0)),
            control_outstanding: Arc::new(AtomicUsize::new(0)),
            sequence: AtomicUsize::new(1),
            governor,
        })))
    }
    fn reply<T>(
        &self,
        epoch: u64,
        cancel: Cancellation,
        control: bool,
    ) -> Result<(Response<T>, PdfTicket<T>)> {
        let counter = if control {
            &self.0.control_outstanding
        } else {
            &self.0.outstanding
        };
        counter
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < MAX_PENDING_REQUESTS).then_some(count + 1)
            })
            .map_err(|_| Failure::Pdf("PDF queue is full; cancel pending work or retry".into()))?;
        let permit = Some(Arc::new(Permit(counter.clone())));
        let id = self.0.sequence.fetch_add(1, Ordering::Relaxed) as u64;
        let (sender, mut ticket) = PdfTicket::channel(id, epoch, cancel);
        if let Some(permit) = &permit {
            ticket = ticket.retain(permit.clone());
        }
        Ok((
            Response {
                sender,
                _permit: permit,
            },
            ticket,
        ))
    }
    fn enqueue(&self, request: Request) -> Result<()> {
        let mut scheduler = self.0.scheduler.lock().unwrap();
        if scheduler.shutdown {
            return Err(stopped());
        }
        if let Request::Open {
            id,
            purpose: yamantaka_domain::pdf_ticket::PdfOpenPurpose::Background,
            ..
        } = &request
            && protected_documents(&scheduler).contains(id)
        {
            return Err(Failure::Invalid(
                "Background work cannot replace a protected reader".into(),
            ));
        }
        if let Request::Open {
            id,
            purpose: yamantaka_domain::pdf_ticket::PdfOpenPurpose::Reader,
            ..
        } = &request
            && scheduler.visible.len() == MAX_HOT_DOCUMENTS
            && !scheduler.visible.contains(id)
        {
            return Err(hot_slots_protected());
        }
        if let Request::Render {
            key,
            epoch,
            priority,
            consumers,
        } = request
        {
            if let Some((old_key, old_epoch, old)) = &scheduler.in_flight
                && *old_key == key
                && *old_epoch == epoch
            {
                old.lock()
                    .unwrap()
                    .extend(consumers.lock().unwrap().drain(..));
            } else {
                let mut merged = false;
                for old in &mut scheduler.queue {
                    if let Request::Render {
                        key: old_key,
                        epoch: old_epoch,
                        priority: old_priority,
                        consumers: old_consumers,
                    } = old
                        && *old_key == key
                        && *old_epoch == epoch
                    {
                        *old_priority = (*old_priority).min(priority);
                        old_consumers
                            .lock()
                            .unwrap()
                            .extend(consumers.lock().unwrap().drain(..));
                        merged = true;
                        break;
                    }
                }
                if !merged {
                    scheduler.queue.push(Request::Render {
                        key,
                        epoch,
                        priority,
                        consumers,
                    });
                }
            }
        } else {
            if let Request::Open { id, epoch, .. } | Request::Close { id, epoch, .. } = &request {
                scheduler.epochs.insert(id.clone(), *epoch);
            }
            scheduler.queue.push(request);
        }
        drop(scheduler);
        let _ = self.0.wake.try_send(());
        Ok(())
    }
    fn epoch(&self, id: &str, advance: bool) -> u64 {
        if advance {
            self.0.sequence.fetch_add(1, Ordering::Relaxed) as u64
        } else {
            self.0
                .scheduler
                .lock()
                .unwrap()
                .epochs
                .get(id)
                .copied()
                .unwrap_or_default()
        }
    }
}
fn stopped() -> Failure {
    Failure::Pdf("PDF worker stopped".into())
}
impl AsyncPdfEngine for PdfWorker {
    fn cover_cache_handle(
        &self,
    ) -> Option<Arc<dyn yamantaka_domain::cover_cache::CoverImageCache>> {
        self.0.cover_cache.clone()
    }

    fn wake_cancelled(&self) {
        let _ = self.0.wake.try_send(());
    }
    fn memory_governor(&self) -> Option<MemoryGovernor> {
        Some(PdfWorker::memory_governor(self))
    }
    fn try_source_revision(
        &self,
        id: String,
        cancel: Cancellation,
    ) -> Result<PdfTicket<yamantaka_domain::source_revision::SourceRevision>> {
        let epoch = self.epoch(&id, false);
        let (reply, ticket) = self.reply(epoch, cancel.clone(), false)?;
        self.enqueue(Request::SourceRevision {
            id,
            epoch,
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
    fn try_capture_import_cover(
        &self,
        id: String,
        source: yamantaka_domain::source_revision::SourceRevision,
        cancel: Cancellation,
    ) -> Result<PdfTicket<yamantaka_domain::cover_policy::DetectedCover>> {
        let (reply, ticket) = self.reply(0, cancel.clone(), false)?;
        self.enqueue(Request::DetectCover {
            id,
            path: source.canonical_path.clone(),
            generation: 0,
            expected: Some(source),
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
    fn try_detect_cover(
        &self,
        id: String,
        path: PathBuf,
        generation: u64,
        cancel: Cancellation,
    ) -> Result<PdfTicket<yamantaka_domain::cover_policy::DetectedCover>> {
        let (reply, ticket) = self.reply(0, cancel.clone(), false)?;
        self.enqueue(Request::DetectCover {
            id,
            path,
            generation,
            expected: None,
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
    fn set_visible_documents(&self, ids: &[String]) -> Result<()> {
        if ids.len() > MAX_HOT_DOCUMENTS || ids.iter().any(|id| id.is_empty()) {
            return Err(Failure::Invalid("Invalid visible PDF set".into()));
        }
        let visible: HashSet<_> = ids.iter().cloned().collect();
        if visible.len() != ids.len() {
            return Err(Failure::Invalid("Duplicate visible PDF ID".into()));
        }
        let mut state = self.0.scheduler.lock().unwrap();
        if state.shutdown {
            return Err(stopped());
        }
        state.visible = visible;
        drop(state);
        let _ = self.0.wake.try_send(());
        Ok(())
    }
    fn try_open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<yamantaka_domain::PdfPassword>,
        purpose: yamantaka_domain::pdf_ticket::PdfOpenPurpose,
        cancel: Cancellation,
    ) -> Result<PdfTicket<PdfInfo>> {
        let (reply, mut ticket) = self.reply(0, cancel.clone(), false)?;
        let epoch = self.epoch(&id, true);
        ticket.epoch = epoch;
        self.enqueue(Request::Open {
            id,
            epoch,
            path,
            password,
            purpose,
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
    fn try_capture_import_cover_page(
        &self,
        id: String,
        source: yamantaka_domain::source_revision::SourceRevision,
        page: u32,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        let (reply, ticket) = self.reply(0, cancel.clone(), false)?;
        self.enqueue(Request::Cover {
            id,
            path: source.canonical_path.clone(),
            page,
            generation: 0,
            expected: Some(source),
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
    fn try_cover(
        &self,
        id: String,
        path: PathBuf,
        page: u32,
        generation: u64,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        let (reply, ticket) = self.reply(0, cancel.clone(), false)?;
        self.enqueue(Request::Cover {
            id,
            path,
            page,
            generation,
            expected: None,
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
    fn try_render(
        &self,
        key: RenderKey,
        priority: Priority,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>> {
        key.byte_size(16384, 192 * 1024 * 1024)?;
        let epoch = self.epoch(&key.document_id, false);
        let (reply, ticket) = self.reply(epoch, cancel.clone(), false)?;
        self.enqueue(Request::Render {
            key,
            epoch,
            priority,
            consumers: Arc::new(Mutex::new(vec![(cancel, reply)])),
        })?;
        Ok(ticket)
    }
    fn try_text(&self, id: String, page: u32, cancel: Cancellation) -> Result<PdfTicket<String>> {
        let epoch = self.epoch(&id, false);
        let (reply, ticket) = self.reply(epoch, cancel.clone(), false)?;
        self.enqueue(Request::Text {
            id,
            epoch,
            page,
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
    fn try_close(&self, id: String) -> Result<PdfTicket<()>> {
        let epoch = self.epoch(&id, true);
        let (reply, ticket) = self.reply(epoch, Cancellation::default(), true)?;
        self.enqueue(Request::Close { id, epoch, reply })?;
        Ok(ticket)
    }
    fn try_outline(
        &self,
        id: String,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<yamantaka_domain::outline::PdfOutline>>> {
        let epoch = self.epoch(&id, false);
        let (reply, ticket) = self.reply(epoch, cancel.clone(), false)?;
        self.enqueue(Request::Outline {
            id,
            epoch,
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
    fn try_text_geometry(
        &self,
        id: String,
        page: u32,
        purpose: yamantaka_domain::pdf_text::TextGeometryPurpose,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<yamantaka_domain::pdf_text::PdfPageText>>> {
        let epoch = self.epoch(&id, false);
        let (reply, ticket) = self.reply(epoch, cancel.clone(), false)?;
        self.enqueue(Request::TextGeometry {
            id,
            epoch,
            page,
            purpose,
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
    fn try_page_links(
        &self,
        id: String,
        page: u32,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<yamantaka_domain::pdf_links::PdfPageLinks>>> {
        let epoch = self.epoch(&id, false);
        let (reply, ticket) = self.reply(epoch, cancel.clone(), false)?;
        self.enqueue(Request::PageLinks {
            id,
            epoch,
            page,
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
    fn try_find_page(
        &self,
        id: String,
        page: u32,
        query: yamantaka_domain::pdf_find::PdfFindQuery,
        offset: u32,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<yamantaka_domain::pdf_find::PdfPageMatches>>> {
        let epoch = self.epoch(&id, false);
        let (reply, ticket) = self.reply(epoch, cancel.clone(), false)?;
        self.enqueue(Request::FindPage {
            id,
            epoch,
            page,
            query,
            offset,
            cancel,
            reply,
        })?;
        Ok(ticket)
    }
}
impl PdfEngine for PdfWorker {
    fn open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<yamantaka_domain::PdfPassword>,
        cancel: Cancellation,
    ) -> Result<PdfInfo> {
        self.try_open(
            id,
            path,
            password,
            yamantaka_domain::pdf_ticket::PdfOpenPurpose::Reader,
            cancel,
        )?
        .wait()
    }
    fn render(
        &self,
        key: RenderKey,
        priority: Priority,
        cancel: Cancellation,
    ) -> Result<Arc<RenderedPage>> {
        self.try_render(key, priority, cancel)?.wait()
    }
    fn text(&self, id: String, page: u32, cancel: Cancellation) -> Result<String> {
        self.try_text(id, page, cancel)?.wait()
    }
    fn close(&self, id: String) -> Result<()> {
        self.try_close(id)?.wait()
    }
    fn outline(
        &self,
        id: String,
        cancel: Cancellation,
    ) -> Result<Arc<yamantaka_domain::outline::PdfOutline>> {
        self.try_outline(id, cancel)?.wait()
    }
}
fn epoch_matches(scheduler: &Arc<Mutex<Scheduler>>, id: &str, epoch: u64) -> Result<()> {
    if scheduler
        .lock()
        .unwrap()
        .epochs
        .get(id)
        .copied()
        .unwrap_or_default()
        != epoch
    {
        Err(Failure::Cancelled)
    } else {
        Ok(())
    }
}
fn fail_request(request: Request) {
    match request {
        Request::Open { reply, .. } | Request::Inspect { reply, .. } => {
            reply.send(Err(Failure::Cancelled))
        }
        Request::Cover { reply, .. } => reply.send(Err(Failure::Cancelled)),
        Request::DetectCover { reply, .. } => reply.send(Err(Failure::Cancelled)),
        Request::SourceRevision { reply, .. } => reply.send(Err(Failure::Cancelled)),
        Request::Text { reply, .. } => reply.send(Err(Failure::Cancelled)),
        Request::Outline { reply, .. } => reply.send(Err(Failure::Cancelled)),
        Request::FindPage { reply, .. } => reply.send(Err(Failure::Cancelled)),
        Request::PageLinks { reply, .. } => reply.send(Err(Failure::Cancelled)),
        Request::TextGeometry { reply, .. } => reply.send(Err(Failure::Cancelled)),
        Request::Close { reply, .. } => reply.send(Err(Failure::Cancelled)),
        Request::Render { consumers, .. } => {
            for (_, reply) in consumers.lock().unwrap().drain(..) {
                reply.send(Err(Failure::Cancelled));
            }
        }
        Request::Shutdown => {}
    }
}

fn run(
    pdfium: &Pdfium,
    receiver: Receiver<()>,
    scheduler: Arc<Mutex<Scheduler>>,
    limits: MemoryLimits,
    governor: MemoryGovernor,
    cover_cache: Option<Arc<dyn yamantaka_domain::cover_cache::CoverImageCache>>,
) {
    let mut documents: HashMap<String, LoadedPdf<'_>> = HashMap::new();
    let mut recency: Vec<String> = Vec::new();
    let mut pages = Cache::new(limits.pages);
    let mut tiles = Cache::new(limits.tiles);
    let mut text = Cache::new(limits.text);
    let mut foreground_streak = 0;
    let mut last_document: Option<String> = None;
    loop {
        let request = {
            let mut state = scheduler.lock().unwrap();
            let mut live: HashSet<String> = documents.keys().cloned().collect();
            live.extend(
                state
                    .queue
                    .iter()
                    .filter_map(Request::document_id)
                    .map(str::to_owned),
            );
            if let Some((key, _, _)) = &state.in_flight {
                live.insert(key.document_id.clone());
            }
            state.epochs.retain(|id, _| live.contains(id));
            let protected = protected_documents(&state);
            let open_allowed = documents.len() < MAX_HOT_DOCUMENTS
                || documents.keys().any(|id| !protected.contains(id))
                || state.visible.len() == MAX_HOT_DOCUMENTS;
            let background_allowed = documents.len() < MAX_HOT_DOCUMENTS
                || documents.iter().any(|(id, loaded)| {
                    !protected.contains(id)
                        && loaded.purpose == yamantaka_domain::pdf_ticket::PdfOpenPurpose::Reader
                });
            if let Some(index) = next_request(
                &state.queue,
                foreground_streak,
                last_document.as_deref(),
                |item| match item {
                    Request::Open {
                        id,
                        epoch,
                        cancel,
                        purpose,
                        ..
                    } => {
                        (if *purpose == yamantaka_domain::pdf_ticket::PdfOpenPurpose::Reader {
                            open_allowed
                        } else {
                            background_allowed
                        }) || documents.contains_key(id)
                            || cancel.is_cancelled()
                            || state.epochs.get(id) != Some(epoch)
                    }
                    _ => true,
                },
            ) {
                let request = state.queue.remove(index);
                if let Request::Render {
                    key,
                    epoch,
                    consumers,
                    ..
                } = &request
                {
                    state.in_flight = Some((key.clone(), *epoch, consumers.clone()));
                }
                Some(request)
            } else {
                None
            }
        };
        let Some(request) = request else {
            if receiver.recv().is_err() {
                break;
            }
            continue;
        };
        last_document = request.document_id().map(str::to_owned);
        foreground_streak = if request.priority() <= 2 {
            (foreground_streak + 1).min(8)
        } else {
            0
        };
        match request {
            Request::SourceRevision {
                id,
                epoch,
                cancel,
                reply,
            } => {
                let result = (|| {
                    cancel.check()?;
                    epoch_matches(&scheduler, &id, epoch)?;
                    let loaded = documents
                        .get(&id)
                        .ok_or(Failure::NativeDocumentUnavailable)?;
                    Ok(loaded.source.snapshot())
                })();
                reply.send(result);
            }
            Request::DetectCover {
                id,
                path,
                generation,
                expected,
                cancel,
                reply,
            } => {
                let result = (|| {
                    cancel.check()?;
                    if let Some(expected) = expected {
                        let loaded = documents
                            .get(&id)
                            .ok_or(Failure::NativeDocumentUnavailable)?;
                        return super::cover_detection::capture_import(
                            pdfium,
                            loaded,
                            id,
                            &expected,
                            &cancel,
                            &governor,
                            cover_cache.as_deref(),
                        );
                    }

                    let revision = SourceStamp::read(&path, &cancel)?;
                    if let Some((old, handle)) =
                        take_cover_victim(&mut documents, &recency, &scheduler, &id, &revision)?
                    {
                        drop(handle);
                        clear_document_state(&old, &mut recency, &mut pages, &mut tiles, &mut text);
                    }
                    let detected = crate::cover_detection::detect(
                        pdfium,
                        documents.get(&id).map(|loaded| &loaded.document),
                        crate::covers::CoverSource {
                            id: id.clone(),
                            path: &path,
                            page: 0,
                            generation,
                        },
                        &cancel,
                        &governor,
                        cover_cache.as_deref(),
                    )?;
                    revision.verify(&path, &cancel)?;
                    Ok(detected)
                })();
                reply.send(result);
            }
            Request::Cover {
                id,
                path,
                page,
                generation,
                expected,
                cancel,
                reply,
            } => {
                let result = (|| {
                    cancel.check()?;
                    if let Some(expected) = expected {
                        let loaded = documents
                            .get(&id)
                            .ok_or(Failure::NativeDocumentUnavailable)?;
                        return super::cover_detection::capture_page(
                            pdfium,
                            loaded,
                            super::covers::CoverSource {
                                id,
                                path: &path,
                                page,
                                generation,
                            },
                            &expected,
                            &cancel,
                            &governor,
                            cover_cache.as_deref(),
                        );
                    }

                    let revision = SourceStamp::read(&path, &cancel)?;
                    let source = super::covers::CoverSource {
                        id: id.clone(),
                        path: &path,
                        page,
                        generation,
                    };
                    let matching = documents
                        .get(&id)
                        .is_some_and(|loaded| loaded.source == revision);
                    if !matching
                        && (documents.contains_key(&id) || documents.len() >= MAX_HOT_DOCUMENTS)
                        && let Some(cache) = cover_cache.as_deref()
                        && let Ok(identity) = cache.identity(&path, page, &cancel)
                        && let Some(image) = super::covers::load_cached(
                            &source, cache, identity, &cancel, &governor,
                        )?
                    {
                        revision.verify(&path, &cancel)?;
                        return Ok(image);
                    }
                    cancel.check()?;
                    if let Some((old, handle)) =
                        take_cover_victim(&mut documents, &recency, &scheduler, &id, &revision)?
                    {
                        drop(handle);
                        clear_document_state(&old, &mut recency, &mut pages, &mut tiles, &mut text);
                    }
                    let image = super::covers::render(
                        pdfium,
                        documents.get(&id).map(|loaded| &loaded.document),
                        source,
                        &cancel,
                        &governor,
                        cover_cache.as_deref(),
                    )?;
                    revision.verify(&path, &cancel)?;
                    Ok(image)
                })();
                reply.send(result);
            }
            Request::Shutdown => {
                let pending = std::mem::take(&mut scheduler.lock().unwrap().queue);
                for request in pending {
                    fail_request(request);
                }
                break;
            }
            Request::Open {
                id,
                epoch,
                path,
                password,
                purpose,
                cancel,
                reply,
            } => {
                let result = (|| {
                    cancel.check()?;
                    epoch_matches(&scheduler, &id, epoch)?;
                    let source = SourceStamp::read(&path, &cancel)?;
                    // Remove stale caches even when the same ID is reopened for a
                    // changed file. Close a victim before opening the next handle:
                    // the native hot-document limit also applies during opening.
                    // Pins can change after admission. Check again before any
                    // eviction; never exceed the two-handle native limit.
                    let victim = take_unprotected_victim(
                        &mut documents,
                        &recency,
                        &scheduler,
                        &id,
                        purpose == yamantaka_domain::pdf_ticket::PdfOpenPurpose::Reader,
                    )?;
                    documents.remove(&id);
                    clear_document_state(&id, &mut recency, &mut pages, &mut tiles, &mut text);
                    if let Some((old, handle)) = victim {
                        drop(handle);
                        clear_document_state(&old, &mut recency, &mut pages, &mut tiles, &mut text);
                    }
                    let opened = pdfium.load_pdf_from_file(
                        &path,
                        password.as_ref().map(yamantaka_domain::PdfPassword::expose),
                    );
                    // Release our credential immediately after the FFI call,
                    // before metadata inspection or any terminal event.
                    drop(password);
                    let document = opened.map_err(pdf_failure)?;
                    source.verify(&path, &cancel)?;
                    let info = InspectInfo {
                        pages: yamantaka_domain::pdf_pages::PdfPagesBuilder::new(
                            document.pages().len() as usize,
                            &governor,
                        )?,
                        title: document
                            .metadata()
                            .get(PdfDocumentMetadataTagType::Title)
                            .map(|t| t.value().to_owned()),
                        author: document
                            .metadata()
                            .get(PdfDocumentMetadataTagType::Author)
                            .map(|t| t.value().to_owned()),
                    };
                    cancel.check()?;
                    epoch_matches(&scheduler, &id, epoch)?;
                    documents.insert(
                        id.clone(),
                        LoadedPdf {
                            purpose,
                            document,
                            path,
                            source,
                        },
                    );
                    recency.retain(|old| old != &id);
                    recency.push(id.clone());
                    Ok(info)
                })();
                // Enumerate only sixteen pages per scheduling unit. The native
                // handle stays on this worker while SQL and other PDF jobs run.
                match result {
                    Ok(info) => scheduler.lock().unwrap().queue.push(Request::Inspect {
                        id,
                        epoch,
                        next_page: 0,
                        purpose,
                        info,
                        cancel,
                        reply,
                    }),
                    Err(error) => reply.send(Err(error)),
                }
            }
            Request::Inspect {
                id,
                epoch,
                mut next_page,
                purpose,
                mut info,
                cancel,
                reply,
            } => {
                let result = (|| {
                    cancel.check()?;
                    epoch_matches(&scheduler, &id, epoch)?;
                    let loaded = documents
                        .get(&id)
                        .ok_or_else(|| Failure::Pdf("Document was closed or evicted".into()))?;
                    let document = &loaded.document;
                    let count = u32::try_from(document.pages().len())
                        .map_err(|_| Failure::Pdf("Invalid page count".into()))?;
                    let end = next_page.saturating_add(16).min(count);
                    while next_page < end {
                        cancel.check()?;
                        let page =
                            document
                                .pages()
                                .get(next_page.try_into().map_err(|_| {
                                    Failure::Invalid("Page index out of range".into())
                                })?)
                                .map_err(pdf_failure)?;
                        info.pages.push(super::page_geometry::read(&page)?)?;
                        next_page += 1;
                    }
                    epoch_matches(&scheduler, &id, epoch)?;
                    if next_page == count {
                        loaded.source.verify(&loaded.path, &cancel)?;
                    }
                    Ok(next_page == count)
                })();
                match result {
                    Ok(true) => reply.send(info.pages.finish().map(|pages| PdfInfo {
                        pages,
                        title: info.title,
                        author: info.author,
                    })),
                    Ok(false) => scheduler.lock().unwrap().queue.push(Request::Inspect {
                        id,
                        epoch,
                        next_page,
                        purpose,
                        info,
                        cancel,
                        reply,
                    }),
                    Err(error) => reply.send(Err(error)),
                }
            }
            Request::Render {
                key,
                epoch,
                consumers,
                ..
            } => {
                let cancelled = {
                    let mut state = scheduler.lock().unwrap();
                    let mut waiting = consumers.lock().unwrap();
                    if waiting.iter().all(|(cancel, _)| cancel.is_cancelled()) {
                        state.in_flight = None;
                        Some(std::mem::take(&mut *waiting))
                    } else {
                        None
                    }
                };
                if let Some(cancelled) = cancelled {
                    for (_, reply) in cancelled {
                        reply.send(Err(Failure::Cancelled));
                    }
                    continue;
                }
                let cache = if key.region.is_some() {
                    &mut tiles
                } else {
                    &mut pages
                };
                let result = (|| {
                    epoch_matches(&scheduler, &key.document_id, epoch)?;
                    key.byte_size(
                        limits.max_dimension,
                        if key.region.is_some() {
                            limits.tiles
                        } else {
                            limits.pages
                        },
                    )?;
                    let raster_key = RasterKey::new(&key, epoch);
                    if let Some(cached) = cache.get(&raster_key) {
                        return Ok(if cached.key.generation == key.generation {
                            cached
                        } else {
                            Arc::new(cached.at_generation(key.generation))
                        });
                    }
                    let document = documents
                        .get(&key.document_id)
                        .map(|loaded| &loaded.document)
                        .ok_or_else(|| Failure::Pdf("Document was closed or evicted".into()))?;
                    let page = document
                        .pages()
                        .get(
                            key.page
                                .try_into()
                                .map_err(|_| Failure::Invalid("Page index out of range".into()))?,
                        )
                        .map_err(pdf_failure)?;
                    let rotation = match key.rotation % 360 {
                        90 => PdfPageRenderRotation::Degrees90,
                        180 => PdfPageRenderRotation::Degrees180,
                        270 => PdfPageRenderRotation::Degrees270,
                        _ => PdfPageRenderRotation::None,
                    };
                    let [width, height] = key.output_dimensions()?;
                    let [x, y, _, _] = key.region.unwrap_or([0, 0, width, height]);
                    // Keep the full device canvas scale and rotation, but translate
                    // its origin into a tile-sized native bitmap. PDFium clips to
                    // that bitmap: no full-page raster or post-render crop exists.
                    // The origin path also preserves form overlays, unlike the
                    // pdfium-render matrix/explicit-clip path, which disables them.
                    let config = PdfRenderConfig::new()
                        .set_fixed_size(key.physical_width as i32, key.physical_height as i32)
                        .set_origin(-(x as i32), -(y as i32))
                        .rotate(rotation, false)
                        .set_reverse_byte_order(false);
                    let bytes = key.byte_size(limits.max_dimension, usize::MAX)?;
                    let category = if key.region.is_some() {
                        MemoryCategory::Tiles
                    } else {
                        MemoryCategory::Pages
                    };
                    let mut buffer = loop {
                        match TrackedBytesBuilder::try_zeroed(&governor, category, bytes) {
                            Ok(buffer) => break buffer,
                            Err(MemoryError::BudgetDenied { .. }) if cache.evict_oldest() => {
                                continue;
                            }
                            Err(error) => return Err(memory_failure(error)),
                        }
                    };
                    {
                        let mut bitmap = PdfBitmap::from_bytes(
                            width as i32,
                            height as i32,
                            PdfBitmapFormat::BGRA,
                            buffer.as_mut_slice(),
                        )
                        .map_err(pdf_failure)?;
                        page.render_into_bitmap_with_config(&mut bitmap, &config)
                            .map_err(pdf_failure)?;
                    } // Destroy the borrowed native bitmap before touching its bytes.
                    for pixel in buffer.as_mut_slice().chunks_exact_mut(4) {
                        pixel.swap(0, 2);
                    }
                    let rendered =
                        Arc::new(RenderedPage::from_pixels(key.clone(), buffer.finish())?);
                    cache.insert(raster_key, rendered.clone(), rendered.pixels().len());
                    Ok(rendered)
                })();
                let active = {
                    let mut state = scheduler.lock().unwrap();
                    state.in_flight = None;
                    std::mem::take(&mut *consumers.lock().unwrap())
                };
                let result = epoch_matches(&scheduler, &key.document_id, epoch).and(result);
                for (cancel, reply) in active {
                    reply.send(if cancel.is_cancelled() {
                        Err(Failure::Cancelled)
                    } else {
                        result.clone()
                    });
                }
            }
            Request::Text {
                id,
                epoch,
                page,
                cancel,
                reply,
            } => {
                let result = (|| {
                    cancel.check()?;
                    epoch_matches(&scheduler, &id, epoch)?;
                    let key = (id.clone(), page);
                    if let Some(cached) = text.get(&key) {
                        return Ok(cached);
                    }
                    let document = documents
                        .get(&id)
                        .map(|loaded| &loaded.document)
                        .ok_or_else(|| Failure::Pdf("Document was closed or evicted".into()))?;
                    let page = document
                        .pages()
                        .get(
                            page.try_into()
                                .map_err(|_| Failure::Invalid("Page index out of range".into()))?,
                        )
                        .map_err(pdf_failure)?;
                    let page_text = page.text().map_err(pdf_failure)?;
                    if page_text.len() as usize > limits.text / 4 {
                        return Err(Failure::Pdf("Page text exceeds memory budget".into()));
                    }
                    let result = page_text.all();
                    cancel.check()?;
                    epoch_matches(&scheduler, &id, epoch)?;
                    text.insert(key, result.clone(), result.len());
                    Ok(result)
                })();
                reply.send(epoch_matches(&scheduler, &id, epoch).and(result));
            }
            Request::Outline {
                id,
                epoch,
                cancel,
                reply,
            } => {
                let result = (|| {
                    cancel.check()?;
                    epoch_matches(&scheduler, &id, epoch)?;
                    let document = documents
                        .get(&id)
                        .map(|loaded| &loaded.document)
                        .ok_or_else(|| Failure::Pdf("Document was closed or evicted".into()))?;
                    crate::outline::read(document, &governor, || {
                        cancel.check()?;
                        epoch_matches(&scheduler, &id, epoch)
                    })
                })();
                reply.send(
                    cancel
                        .check()
                        .and_then(|()| epoch_matches(&scheduler, &id, epoch))
                        .and(result),
                );
            }
            Request::TextGeometry {
                id,
                epoch,
                page,
                purpose,
                cancel,
                reply,
            } => {
                let result = (|| {
                    cancel.check()?;
                    epoch_matches(&scheduler, &id, epoch)?;
                    let document = documents
                        .get(&id)
                        .map(|loaded| &loaded.document)
                        .ok_or(Failure::NativeDocumentUnavailable)?;
                    crate::text_geometry::read(
                        document,
                        page,
                        purpose.character_limit(),
                        &governor,
                        || {
                            cancel.check()?;
                            epoch_matches(&scheduler, &id, epoch)
                        },
                    )
                })();
                reply.send(
                    cancel
                        .check()
                        .and_then(|()| epoch_matches(&scheduler, &id, epoch))
                        .and(result),
                );
            }
            Request::PageLinks {
                id,
                epoch,
                page,
                cancel,
                reply,
            } => {
                let result = (|| {
                    cancel.check()?;
                    epoch_matches(&scheduler, &id, epoch)?;
                    let document = documents
                        .get(&id)
                        .map(|loaded| &loaded.document)
                        .ok_or_else(|| Failure::Pdf("Document was closed or evicted".into()))?;
                    crate::links::read(document, page, &governor, || {
                        cancel.check()?;
                        epoch_matches(&scheduler, &id, epoch)
                    })
                })();
                reply.send(
                    cancel
                        .check()
                        .and_then(|()| epoch_matches(&scheduler, &id, epoch))
                        .and(result),
                );
            }
            Request::FindPage {
                id,
                epoch,
                page,
                query,
                offset,
                cancel,
                reply,
            } => {
                let result = (|| {
                    cancel.check()?;
                    epoch_matches(&scheduler, &id, epoch)?;
                    let document = documents
                        .get(&id)
                        .map(|loaded| &loaded.document)
                        .ok_or_else(|| Failure::Pdf("Document was closed or evicted".into()))?;
                    crate::find::read(document, page, &query, offset, &governor, || {
                        cancel.check()?;
                        epoch_matches(&scheduler, &id, epoch)
                    })
                })();
                reply.send(
                    cancel
                        .check()
                        .and_then(|()| epoch_matches(&scheduler, &id, epoch))
                        .and(result),
                );
            }
            Request::Close { id, epoch, reply } => {
                let handle = {
                    let mut state = scheduler.lock().unwrap();
                    if state.epochs.get(&id) != Some(&epoch) {
                        reply.send(Err(Failure::Cancelled));
                        continue;
                    }
                    state.visible.remove(&id);
                    documents.remove(&id)
                };
                drop(handle);
                clear_document_state(&id, &mut recency, &mut pages, &mut tiles, &mut text);
                reply.send(Ok(()));
            }
        }
    }
    // Documents drop before the borrowed Pdfium instance; outstanding native work finished.
}

fn take_cover_victim<'a>(
    documents: &mut HashMap<String, LoadedPdf<'a>>,
    recency: &[String],
    scheduler: &Arc<Mutex<Scheduler>>,
    requested: &str,
    source: &SourceStamp,
) -> Result<Option<(String, LoadedPdf<'a>)>> {
    if documents
        .get(requested)
        .is_some_and(|loaded| loaded.source != *source)
    {
        let state = scheduler.lock().unwrap();
        if protected_documents(&state).contains(requested)
            || documents.get(requested).is_some_and(|loaded| {
                loaded.purpose == yamantaka_domain::pdf_ticket::PdfOpenPurpose::Background
            })
        {
            return Err(crate::source::source_changed());
        }
        return Ok(Some((
            requested.to_owned(),
            documents
                .remove(requested)
                .expect("stale handle checked before detachment"),
        )));
    }
    take_unprotected_victim(documents, recency, scheduler, requested, false)
}

fn take_unprotected_victim<'a>(
    documents: &mut HashMap<String, LoadedPdf<'a>>,
    recency: &[String],
    scheduler: &Arc<Mutex<Scheduler>>,
    requested: &str,
    allow_background: bool,
) -> Result<Option<(String, LoadedPdf<'a>)>> {
    if documents.contains_key(requested) || documents.len() < MAX_HOT_DOCUMENTS {
        return Ok(None);
    }
    let state = scheduler.lock().unwrap();
    let protected = protected_documents(&state);
    let old = recency
        .iter()
        .find(|id| {
            documents.get(*id).is_some_and(|loaded| {
                allow_background
                    || loaded.purpose == yamantaka_domain::pdf_ticket::PdfOpenPurpose::Reader
            }) && !protected.contains(*id)
        })
        .cloned()
        .ok_or_else(hot_slots_protected)?;
    // Select and detach under the pin lock; callers drop the handle before FFI.
    let handle = documents
        .remove(&old)
        .expect("victim selected from live handles");
    Ok(Some((old, handle)))
}

fn clear_document_state(
    id: &str,
    recency: &mut Vec<String>,
    pages: &mut Cache<RasterKey, Arc<RenderedPage>>,
    tiles: &mut Cache<RasterKey, Arc<RenderedPage>>,
    text: &mut Cache<(String, u32), String>,
) {
    recency.retain(|old| old != id);
    pages.retain(|key| !key.document_is(id));
    tiles.retain(|key| !key.document_is(id));
    text.retain(|key| key.0 != id);
}

/// Close/shutdown always win. At most eight foreground requests can bypass a
/// waiting background request, preventing endless visible-page traffic from
/// starving text, prefetch or cover work. Equal priorities preserve queue order.
fn protected_documents(state: &Scheduler) -> HashSet<String> {
    state
        .visible
        .iter()
        .cloned()
        .chain(state.queue.iter().filter_map(|request| match request {
            Request::Inspect {
                id,
                epoch,
                purpose: yamantaka_domain::pdf_ticket::PdfOpenPurpose::Reader,
                ..
            } if state.epochs.get(id) == Some(epoch) => Some(id.clone()),
            _ => None,
        }))
        .collect()
}
fn hot_slots_protected() -> Failure {
    Failure::NativeSlotsBusy
}
fn next_request(
    queue: &[Request],
    foreground_streak: usize,
    last_document: Option<&str>,
    eligible: impl Fn(&Request) -> bool,
) -> Option<usize> {
    if let Some(index) = queue.iter().position(|item| item.priority() == 0) {
        return Some(index);
    }
    let priority = if foreground_streak >= 8 {
        queue
            .iter()
            .filter(|item| eligible(item) && item.priority() > 2)
            .map(Request::priority)
            .min()
    } else {
        None
    }
    .or_else(|| {
        queue
            .iter()
            .filter(|item| eligible(item))
            .map(Request::priority)
            .min()
    })?;
    queue
        .iter()
        .position(|item| {
            eligible(item) && item.priority() == priority && item.document_id() != last_document
        })
        .or_else(|| {
            queue
                .iter()
                .position(|item| eligible(item) && item.priority() == priority)
        })
}
pub(crate) fn pdf_failure(error: PdfiumError) -> Failure {
    if matches!(
        error,
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError)
    ) {
        Failure::PasswordRequired
    } else {
        Failure::Pdf("PDFium could not process this document".into())
    }
}
fn memory_failure(error: MemoryError) -> Failure {
    match error {
        MemoryError::BudgetDenied { .. } => Failure::MemoryBudget,
        _ => Failure::Invalid(error.to_string()),
    }
}
