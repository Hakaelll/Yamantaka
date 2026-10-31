use crossbeam_channel::{Receiver, Select, Sender};
use std::sync::Arc;
use yamantaka_domain::{
    Cancellation, Failure, NewDocument, PdfInfo,
    front_matter::{AnalysisFailure, FrontMatter, ImportAnalysis, MAX_FRONT_MATTER_PAGES},
    pdf_text::{PdfPageText, TextGeometryPurpose},
    pdf_ticket::{AsyncPdfEngine, PdfOpenPurpose, PdfTicket},
    source_revision::SourceRevision,
};

pub(crate) enum ImportStep {
    Pending(Box<ImportEnrichment>),
    Ready(Box<NewDocument>),
    Failed(Failure),
}

enum Work {
    RequestPage,
    Text(PdfTicket<Arc<PdfPageText>>),
    Reopen(PdfTicket<PdfInfo>, bool),
    Password(Receiver<yamantaka_domain::PdfPassword>),
    Source(PdfTicket<SourceRevision>),
}

pub(crate) struct ImportEnrichment {
    pub id: String,
    document: Box<NewDocument>,
    page: u32,
    scan: FrontMatter,
    work: Work,
    cancel: Cancellation,
}

fn unavailable(mut document: Box<NewDocument>, error: Failure) -> ImportStep {
    let reason = match error {
        Failure::MemoryBudget => AnalysisFailure::MemoryBudget,
        Failure::TextWorkLimit => AnalysisFailure::TextWorkLimit,
        _ => AnalysisFailure::Unavailable,
    };
    document.analysis = ImportAnalysis::Unavailable(reason);
    ImportStep::Ready(document)
}

impl ImportEnrichment {
    pub fn start(
        engine: &impl AsyncPdfEngine,
        id: String,
        document: Box<NewDocument>,
        previous: Option<&yamantaka_domain::metadata_review::MetadataReview>,
        cancel: Cancellation,
    ) -> ImportStep {
        let result = (|| {
            cancel.check()?;
            let governor = engine
                .memory_governor()
                .ok_or_else(|| Failure::Unsupported("Governed metadata analysis".into()))?;
            let file_name = document
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let scan = FrontMatter::new(
                &file_name,
                previous
                    .filter(|previous| {
                        matches!(
                            yamantaka_domain::import_metadata::catalogue_title(
                                previous.automatic_title.as_deref(),
                                previous.source_title.as_deref(),
                                &file_name
                            ),
                            yamantaka_domain::import_metadata::MetadataValue::Accepted(_)
                        )
                    })
                    .and_then(|p| p.automatic_title.as_deref())
                    .or(document.info.title.as_deref()),
                previous
                    .and_then(|p| p.automatic_author.as_deref())
                    .filter(|value| {
                        matches!(
                            yamantaka_domain::import_metadata::author(Some(value)),
                            yamantaka_domain::import_metadata::MetadataValue::Accepted(_)
                        )
                    })
                    .or(document.info.author.as_deref()),
                &governor,
            )?;
            Ok(scan)
        })();
        match result {
            Ok(scan) => Box::new(Self {
                id,
                document,
                page: 0,
                scan,
                work: Work::RequestPage,
                cancel,
            })
            .request_page(engine),
            Err(error) => unavailable(document, error),
        }
    }

    pub fn ready(&self) -> bool {
        match &self.work {
            Work::RequestPage => true,
            Work::Text(ticket) => ticket.is_ready(),
            Work::Reopen(ticket, _) => ticket.is_ready(),
            Work::Password(receive) => {
                let mut select = Select::new();
                select.recv(receive);
                self.cancel.is_cancelled() || select.try_ready().is_ok()
            }
            Work::Source(ticket) => ticket.is_ready(),
        }
    }
    pub fn register<'a>(&'a self, select: &mut Select<'a>) {
        match &self.work {
            Work::RequestPage => {}
            Work::Text(ticket) => {
                select.recv(ticket.readiness());
            }
            Work::Reopen(ticket, _) => {
                select.recv(ticket.readiness());
            }
            Work::Source(ticket) => {
                select.recv(ticket.readiness());
            }
            Work::Password(receive) => {
                select.recv(receive);
            }
        }
    }
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
    fn request_page(mut self: Box<Self>, engine: &impl AsyncPdfEngine) -> ImportStep {
        if let Err(error) = self.cancel.check() {
            return ImportStep::Failed(error);
        }
        match engine.try_text_geometry(
            self.id.clone(),
            self.page,
            TextGeometryPurpose::FrontMatter,
            self.cancel.clone(),
        ) {
            Ok(ticket) => {
                self.work = Work::Text(ticket);
                ImportStep::Pending(self)
            }
            Err(Failure::NativeDocumentUnavailable) => self.reopen(engine),
            Err(error) => unavailable(self.document, error),
        }
    }
    fn reopen(self: Box<Self>, engine: &impl AsyncPdfEngine) -> ImportStep {
        self.open(engine, None)
    }
    fn open(
        mut self: Box<Self>,
        engine: &impl AsyncPdfEngine,
        password: Option<yamantaka_domain::PdfPassword>,
    ) -> ImportStep {
        if let Err(error) = self.cancel.check() {
            return ImportStep::Failed(error);
        }
        let attempted = password.is_some();
        match engine.try_open(
            self.id.clone(),
            self.document.path.clone(),
            password,
            PdfOpenPurpose::Background,
            self.cancel.clone(),
        ) {
            Ok(ticket) => {
                self.work = Work::Reopen(ticket, attempted);
                ImportStep::Pending(self)
            }
            Err(error) => ImportStep::Failed(error),
        }
    }
    pub fn advance(
        mut self: Box<Self>,
        engine: &impl AsyncPdfEngine,
        passwords: &Sender<crate::import_password::Challenge>,
        wake: &Sender<()>,
        notify: &impl Fn(),
    ) -> ImportStep {
        if let Err(error) = self.cancel.check() {
            return ImportStep::Failed(error);
        }
        match std::mem::replace(&mut self.work, Work::RequestPage) {
            Work::RequestPage => self.request_page(engine),
            Work::Text(ticket) => {
                let observed = (|| {
                    let text = ticket.wait()?;
                    self.cancel.check()?;
                    if text.page() != self.page {
                        return Err(Failure::Invalid(
                            "Metadata text belongs to another page".into(),
                        ));
                    }
                    let geometry = self
                        .document
                        .info
                        .pages
                        .get(self.page as usize)
                        .ok_or_else(|| {
                            Failure::Invalid("Metadata page is outside the document".into())
                        })?;
                    self.scan
                        .observe(&text, geometry.coordinates(), &self.cancel)
                })();
                match observed {
                    Err(Failure::NativeDocumentUnavailable) => return self.reopen(engine),
                    Err(error) => return unavailable(self.document, error),
                    Ok(()) => {}
                }
                let next = self.page + 1;
                if next
                    < self
                        .document
                        .info
                        .pages
                        .len()
                        .min(MAX_FRONT_MATTER_PAGES as usize) as u32
                {
                    self.page = next;
                    self.request_page(engine)
                } else {
                    match self.scan.finish(&self.cancel) {
                        Ok(evidence) => {
                            self.document.analysis = ImportAnalysis::Complete(Box::new(evidence));
                            ImportStep::Ready(self.document)
                        }
                        Err(error) => unavailable(self.document, error),
                    }
                }
            }
            Work::Password(receive) => match receive.recv() {
                Ok(password) => self.open(engine, Some(password)),
                Err(_) => ImportStep::Failed(Failure::Cancelled),
            },
            Work::Reopen(ticket, attempted) => match ticket.wait() {
                Err(Failure::PasswordRequired) => {
                    let (challenge, receive) = crate::import_password::Challenge::channel(
                        self.document.path.clone(),
                        attempted,
                        self.cancel.clone(),
                        wake.clone(),
                    );
                    match passwords.try_send(challenge) {
                        Ok(()) => {
                            self.work = Work::Password(receive);
                            notify();
                            ImportStep::Pending(self)
                        }
                        Err(_) => ImportStep::Failed(Failure::QueueFull),
                    }
                }
                Ok(info) => {
                    if info.pages[..] != self.document.info.pages[..] {
                        return ImportStep::Failed(Failure::Invalid(
                            "Reopened metadata source has different page geometry".into(),
                        ));
                    }
                    match engine.try_source_revision(self.id.clone(), self.cancel.clone()) {
                        Ok(ticket) => {
                            self.work = Work::Source(ticket);
                            ImportStep::Pending(self)
                        }
                        Err(Failure::NativeDocumentUnavailable) => self.reopen(engine),
                        Err(error) => ImportStep::Failed(error),
                    }
                }
                Err(Failure::NativeSlotsBusy | Failure::NativeDocumentUnavailable) => {
                    self.reopen(engine)
                }
                Err(error) => ImportStep::Failed(error),
            },
            Work::Source(ticket) => match ticket.wait() {
                Ok(source) if source == self.document.source => self.request_page(engine),
                Ok(_) => ImportStep::Failed(Failure::Invalid(
                    "Metadata source changed while analysis was interrupted".into(),
                )),
                Err(Failure::NativeDocumentUnavailable) => self.reopen(engine),
                Err(error) => ImportStep::Failed(error),
            },
        }
    }
}
