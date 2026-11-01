//! Explicit cache repair keeps catalogue decisions separate from native PDF ownership.
use crate::{
    background_password::{Challenge, Purpose},
    cover_source::CoverSource,
    file_digest::HashedFile,
    import_hash::ImportHashJob,
};
use crossbeam_channel::{Receiver, Select, Sender};
use std::sync::Arc;
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryRepository, PdfInfo, PdfPassword, RenderedPage, Result,
    cover_cache::{CoverCacheRequest, CoverImageCache},
    cover_policy::DetectedCover,
    cover_snapshot::{CoverPlan, CoverSnapshot},
    memory::MemoryGovernor,
    pdf_ticket::{AsyncPdfEngine, PdfOpenPurpose, PdfTicket},
};

pub struct Request {
    pub document: Document,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub struct Reply {
    pub document: String,
    pub result: Result<()>,
    pub cancel: Cancellation,
}
enum Work {
    Hash(ImportHashJob),
    Open(PdfTicket<PdfInfo>),
    Password(Receiver<PdfPassword>),
    Source(PdfTicket<yamantaka_domain::source_revision::SourceRevision>),
    Detect(PdfTicket<DetectedCover>),
    Page(PdfTicket<Arc<RenderedPage>>),
    Close(PdfTicket<()>),
}
enum Image {
    Detected(DetectedCover),
    Page(Arc<RenderedPage>),
}
impl Image {
    fn page(&self) -> &Arc<RenderedPage> {
        match self {
            Self::Detected(d) => &d.image,
            Self::Page(p) => p,
        }
    }
}
pub(crate) enum Step {
    Pending(Box<Job>),
    Finished(Box<Reply>, bool),
}
pub(crate) struct Job {
    request: Request,
    source: CoverSource,
    snapshot: CoverSnapshot,
    id: String,
    work: Option<Work>,
    guard: Option<HashedFile>,
    cache: Arc<dyn CoverImageCache>,
    captured: Option<Image>,
    owner: bool,
    attempted: bool,
    error: Option<Failure>,
}
impl Job {
    pub fn start(
        request: Request,
        repository: &impl LibraryRepository,
        engine: &impl AsyncPdfEngine,
    ) -> Result<Box<Self>> {
        request.cancel.check()?;
        let source = CoverSource::capture(&request.document);
        source.read(repository, &request.cancel)?;
        let snapshot = repository.cover_snapshot(&request.document.id)?;
        let cache = engine
            .cover_cache_handle()
            .ok_or_else(|| Failure::Unsupported("Cover cache is disabled".into()))?;
        let id = crate::temporary_document::reserve(repository, "rust-cover-rebuild")?;
        let hash = ImportHashJob::start(
            request.document.path.clone(),
            request.governor.clone(),
            request.cancel.clone(),
        )?;
        Ok(Box::new(Self {
            request,
            source,
            snapshot,
            id,
            work: Some(Work::Hash(hash)),
            guard: None,
            cache,
            captured: None,
            owner: false,
            attempted: false,
            error: None,
        }))
    }
    pub fn ready(&self) -> bool {
        match self.work.as_ref().unwrap() {
            Work::Hash(job) => job.ready(),
            Work::Open(t) => t.is_ready(),
            Work::Source(t) => t.is_ready(),
            Work::Detect(t) => t.is_ready(),
            Work::Page(t) => t.is_ready(),
            Work::Close(t) => t.is_ready(),
            Work::Password(r) => {
                let mut select = Select::new();
                select.recv(r);
                self.request.cancel.is_cancelled() || select.try_ready().is_ok()
            }
        }
    }
    pub fn register<'a>(&'a self, select: &mut Select<'a>) {
        match self.work.as_ref().unwrap() {
            Work::Hash(job) => job.register(select),
            Work::Open(t) => {
                select.recv(t.readiness());
            }
            Work::Source(t) => {
                select.recv(t.readiness());
            }
            Work::Detect(t) => {
                select.recv(t.readiness());
            }
            Work::Page(t) => {
                select.recv(t.readiness());
            }
            Work::Close(t) => {
                select.recv(t.readiness());
            }
            Work::Password(r) => {
                select.recv(r);
            }
        }
    }
    pub fn cancel(&self) {
        self.request.cancel.cancel();
    }
    fn finish(self: Box<Self>, result: Result<()>, released: bool) -> Step {
        Step::Finished(
            Box::new(Reply {
                document: self.request.document.id.clone(),
                result,
                cancel: self.request.cancel.clone(),
            }),
            released,
        )
    }
    fn pending(mut self: Box<Self>, work: Work) -> Step {
        self.work = Some(work);
        Step::Pending(self)
    }
    fn close(mut self: Box<Self>, engine: &impl AsyncPdfEngine, error: Option<Failure>) -> Step {
        self.error = error;
        if !self.owner {
            let error = self.error.take().unwrap_or(Failure::Cancelled);
            return self.finish(Err(error), false);
        }
        match engine.try_close(self.id.clone()) {
            Ok(ticket) => self.pending(Work::Close(ticket)),
            Err(error) => self.finish(Err(error), false),
        }
    }
    fn open(
        mut self: Box<Self>,
        engine: &impl AsyncPdfEngine,
        password: Option<PdfPassword>,
    ) -> Step {
        if let Err(error) = self.request.cancel.check() {
            return self.close(engine, Some(error));
        }
        self.attempted = password.is_some();
        match engine.try_open(
            self.id.clone(),
            self.request.document.path.clone(),
            password,
            PdfOpenPurpose::Background,
            self.request.cancel.clone(),
        ) {
            Ok(ticket) => {
                self.owner = true;
                self.pending(Work::Open(ticket))
            }
            Err(error) => self.close(engine, Some(error)),
        }
    }
    fn validate(&self, repository: &impl LibraryRepository) -> Result<()> {
        self.source.read(repository, &self.request.cancel)?;
        if repository.cover_snapshot(&self.request.document.id)? != self.snapshot {
            return Err(Failure::Invalid("Cover selection changed; retry".into()));
        }
        let guard = self
            .guard
            .as_ref()
            .ok_or_else(|| Failure::Invalid("Cover source guard is missing".into()))?;
        guard.verify()?;
        if guard.fingerprint() != self.request.document.fingerprint {
            return Err(Failure::Invalid(
                "PDF content changed; check the file first".into(),
            ));
        }
        if repository
            .file_snapshot(&self.request.document.id)?
            .digest
            .is_some_and(|digest| digest != guard.digest())
        {
            return Err(Failure::Invalid(
                "PDF content changed; check the file first".into(),
            ));
        }
        Ok(())
    }
    pub fn advance(
        mut self: Box<Self>,
        repository: &mut impl LibraryRepository,
        engine: &impl AsyncPdfEngine,
        passwords: &Sender<Challenge>,
        wake: &Sender<()>,
        notify: &impl Fn(),
    ) -> Step {
        let work = self.work.take().unwrap();
        if self.request.cancel.is_cancelled() && !matches!(work, Work::Close(_)) {
            return self.close(engine, Some(Failure::Cancelled));
        }
        match work {
            Work::Hash(mut hash) => match hash.finish() {
                Ok(guard) => {
                    self.guard = Some(guard);
                    match self.validate(repository) {
                        Ok(()) => self.open(engine, None),
                        Err(error) => self.close(engine, Some(error)),
                    }
                }
                Err(error) => self.close(engine, Some(error)),
            },
            Work::Password(receive) => match receive.recv() {
                Ok(password) => self.open(engine, Some(password)),
                Err(_) => self.close(engine, Some(Failure::Cancelled)),
            },
            Work::Open(ticket) => match ticket.wait() {
                Err(Failure::PasswordRequired) => {
                    let (challenge, receive) = Challenge::channel(
                        Purpose::Cover,
                        self.request.document.path.clone(),
                        self.attempted,
                        self.request.cancel.clone(),
                        wake.clone(),
                    );
                    match passwords.try_send(challenge) {
                        Ok(()) => {
                            notify();
                            self.pending(Work::Password(receive))
                        }
                        Err(_) => self.close(engine, Some(Failure::QueueFull)),
                    }
                }
                Err(Failure::NativeDocumentUnavailable) => self.open(engine, None),
                Err(error) => self.close(engine, Some(error)),
                Ok(info) if info.pages.len() != self.request.document.page_count as usize => self
                    .close(
                        engine,
                        Some(Failure::Invalid(
                            "PDF page count changed; check the file first".into(),
                        )),
                    ),
                Ok(_) => {
                    match engine.try_source_revision(self.id.clone(), self.request.cancel.clone()) {
                        Ok(ticket) => self.pending(Work::Source(ticket)),
                        Err(Failure::NativeDocumentUnavailable) => self.open(engine, None),
                        Err(error) => self.close(engine, Some(error)),
                    }
                }
            },
            Work::Source(ticket) => match ticket.wait() {
                Err(Failure::NativeDocumentUnavailable) => self.open(engine, None),
                Err(error) => self.close(engine, Some(error)),
                Ok(source) => {
                    if let Err(error) = self
                        .guard
                        .as_ref()
                        .unwrap()
                        .reference(&source)
                        .and_then(|_| self.validate(repository))
                    {
                        return self.close(engine, Some(error));
                    }
                    match self.snapshot.plan {
                        CoverPlan::Render(page) => match engine.try_capture_import_cover_page(
                            self.id.clone(),
                            source,
                            page,
                            self.request.cancel.clone(),
                        ) {
                            Ok(ticket) => self.pending(Work::Page(ticket)),
                            Err(Failure::NativeDocumentUnavailable) => self.open(engine, None),
                            Err(error) => self.close(engine, Some(error)),
                        },
                        CoverPlan::Detect => match engine.try_capture_import_cover(
                            self.id.clone(),
                            source,
                            self.request.cancel.clone(),
                        ) {
                            Ok(ticket) => self.pending(Work::Detect(ticket)),
                            Err(Failure::NativeDocumentUnavailable) => self.open(engine, None),
                            Err(error) => self.close(engine, Some(error)),
                        },
                    }
                }
            },
            Work::Page(ticket) => match ticket.wait() {
                Ok(image) => {
                    self.captured = Some(Image::Page(image));
                    self.close(engine, None)
                }
                Err(Failure::NativeDocumentUnavailable) => self.open(engine, None),
                Err(error) => self.close(engine, Some(error)),
            },
            Work::Detect(ticket) => match ticket.wait() {
                Ok(image) => {
                    self.captured = Some(Image::Detected(image));
                    self.close(engine, None)
                }
                Err(Failure::NativeDocumentUnavailable) => self.open(engine, None),
                Err(error) => self.close(engine, Some(error)),
            },
            Work::Close(ticket) => {
                let closed = ticket.wait();
                let released = closed.is_ok();
                self.owner = !released;
                let result = closed.and_then(|()| {
                    if let Some(error) = self.error.take() {
                        return Err(error);
                    }
                    self.validate(repository)?;
                    let captured = self
                        .captured
                        .take()
                        .ok_or_else(|| Failure::Invalid("Cover capture is missing".into()))?;
                    let image = captured.page();
                    let source = self.guard.as_ref().unwrap().source_revision();
                    let identity = self.cache.identity(
                        &source.canonical_path,
                        image.key.page,
                        &self.request.cancel,
                    )?;
                    let cached = self
                        .cache
                        .load(
                            CoverCacheRequest {
                                identity,
                                document: &self.request.document.id,
                                page: image.key.page,
                                generation: 0,
                            },
                            &self.request.governor,
                            &self.request.cancel,
                        )?
                        .ok_or_else(|| Failure::Storage("Cover cache could not be saved".into()))?;
                    if cached.width != image.width
                        || cached.height != image.height
                        || cached.pixels() != image.pixels()
                    {
                        return Err(Failure::Invalid(
                            "Cover cache changed during rebuilding".into(),
                        ));
                    }
                    self.validate(repository)?;
                    if let Image::Detected(detected) = captured {
                        let mut image = detected.image.at_generation(0);
                        image.key.document_id = self.request.document.id.clone();
                        crate::automatic_cover::complete(
                            repository,
                            self.request.document.clone(),
                            0,
                            DetectedCover {
                                image: Arc::new(image),
                                choice: detected.choice,
                            },
                            self.snapshot.clone(),
                        )?;
                    }
                    Ok(())
                });
                self.finish(result, released)
            }
        }
    }
}
