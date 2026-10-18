use crate::{
    automatic_cover::{self, CoverCompletion},
    cover_source::{CoverFrame, CoverSource},
};
use crossbeam_channel::Select;
use std::sync::Arc;
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryRepository, RenderedPage, Result,
    cover_policy::DetectedCover,
    cover_snapshot::{CoverPlan, CoverSnapshot},
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
};

enum Intent {
    Library,
    Preview(u32),
    Restore(CoverSnapshot),
}
enum Admission {
    Render(u32, Option<CoverSnapshot>),
    Detect(CoverSnapshot),
}
enum Work {
    Start,
    Waiting(u64),
    Render(CoverFrame, PdfTicket<Arc<RenderedPage>>, u64),
    Detect(Document, CoverSnapshot, PdfTicket<DetectedCover>, u64),
}
pub(crate) struct CoverJob {
    source: CoverSource,
    generation: u64,
    cancel: Cancellation,
    intent: Intent,
    work: Work,
}
pub(crate) enum CoverStep {
    Pending(Box<CoverJob>),
    Finished(String, Result<Arc<RenderedPage>>, Cancellation),
}
impl CoverJob {
    pub fn library(source: CoverSource, generation: u64, cancel: Cancellation) -> Box<Self> {
        Box::new(Self {
            source,
            generation,
            cancel,
            intent: Intent::Library,
            work: Work::Start,
        })
    }
    pub fn preview(
        source: CoverSource,
        page: u32,
        generation: u64,
        cancel: Cancellation,
    ) -> Box<Self> {
        Box::new(Self {
            source,
            generation,
            cancel,
            intent: Intent::Preview(page),
            work: Work::Start,
        })
    }
    pub fn restore(
        source: CoverSource,
        generation: u64,
        cancel: Cancellation,
        repository: &impl LibraryRepository,
    ) -> Result<Box<Self>> {
        cancel.check()?;
        source.read(repository, &cancel)?;
        let expected = repository.cover_snapshot(source.id())?;
        Ok(Box::new(Self {
            source,
            generation,
            cancel,
            intent: Intent::Restore(expected),
            work: Work::Start,
        }))
    }
    pub fn waiting(&self) -> bool {
        matches!(self.work, Work::Waiting(_))
    }
    pub fn ready(&self, capacity: u64) -> bool {
        match &self.work {
            Work::Start => true,
            Work::Waiting(revision) => self.cancel.is_cancelled() || *revision != capacity,
            Work::Render(_, ticket, _) => ticket.is_ready(),
            Work::Detect(_, _, ticket, _) => ticket.is_ready(),
        }
    }
    pub fn register<'a>(&'a self, select: &mut Select<'a>) {
        match &self.work {
            Work::Render(_, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Work::Detect(_, _, ticket, _) => {
                select.recv(ticket.readiness());
            }
            Work::Start | Work::Waiting(_) => {}
        }
    }
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
    fn start(
        &mut self,
        repository: &impl LibraryRepository,
        engine: &impl AsyncPdfEngine,
        capacity: u64,
    ) -> Result<()> {
        self.cancel.check()?;
        let document = self.source.read(repository, &self.cancel)?;
        let admission = match &self.intent {
            Intent::Preview(page) => Admission::Render(*page, None),
            Intent::Library => {
                let snapshot = repository.cover_snapshot(self.source.id())?;
                match snapshot.plan {
                    CoverPlan::Render(page) => Admission::Render(page, Some(snapshot)),
                    CoverPlan::Detect => Admission::Detect(snapshot),
                }
            }
            Intent::Restore(expected) => {
                if repository.cover_snapshot(self.source.id())? != *expected {
                    return Err(Failure::Invalid(
                        "Cover changed while waiting to restore; reopen the picker".into(),
                    ));
                }
                Admission::Detect(expected.clone())
            }
        };
        self.work = match admission {
            Admission::Render(page, selection) => {
                if page >= document.page_count {
                    return Err(Failure::Invalid(
                        "Cover page is outside its document".into(),
                    ));
                }
                let ticket = engine.try_cover(
                    self.source.id().to_owned(),
                    self.source.path(),
                    page,
                    self.generation,
                    self.cancel.clone(),
                )?;
                Work::Render(
                    CoverFrame::new(self.source.clone(), page, self.generation, selection),
                    ticket,
                    capacity,
                )
            }
            Admission::Detect(snapshot) => {
                let ticket = engine.try_detect_cover(
                    self.source.id().to_owned(),
                    self.source.path(),
                    self.generation,
                    self.cancel.clone(),
                )?;
                Work::Detect(document, snapshot, ticket, capacity)
            }
        };
        Ok(())
    }
    pub fn advance(
        mut self: Box<Self>,
        repository: &mut impl LibraryRepository,
        engine: &impl AsyncPdfEngine,
        capacity: u64,
    ) -> CoverStep {
        let mut admitted_at = capacity;
        let result = (|| {
            self.cancel.check()?;
            match std::mem::replace(&mut self.work, Work::Start) {
                Work::Start | Work::Waiting(_) => {
                    self.start(repository, engine, capacity)?;
                    Ok(None)
                }
                Work::Render(frame, ticket, revision) => {
                    admitted_at = revision;
                    ticket
                        .wait()
                        .and_then(|image| frame.finish(repository, image, &self.cancel))
                        .map(Some)
                }
                Work::Detect(document, snapshot, ticket, revision) => {
                    admitted_at = revision;
                    let detected = ticket.wait()?;
                    self.cancel.check()?;
                    self.source.read(repository, &self.cancel)?;
                    let result = match &self.intent {
                        Intent::Restore(_) => automatic_cover::restore(
                            repository,
                            document,
                            self.generation,
                            detected,
                            snapshot,
                        )
                        .map(CoverCompletion::Image),
                        Intent::Library => automatic_cover::complete(
                            repository,
                            document,
                            self.generation,
                            detected,
                            snapshot,
                        ),
                        Intent::Preview(_) => Err(Failure::Invalid(
                            "Preview cannot complete cover detection".into(),
                        )),
                    }?;
                    match result {
                        CoverCompletion::Image(image) => Ok(Some(image)),
                        CoverCompletion::RetrySelection => {
                            admitted_at = capacity;
                            self.start(repository, engine, capacity)?;
                            Ok(None)
                        }
                    }
                }
            }
        })();
        match result {
            Err(Failure::NativeSlotsBusy) => {
                self.work = Work::Waiting(admitted_at);
                CoverStep::Pending(self)
            }
            Ok(None) => CoverStep::Pending(self),
            Ok(Some(image)) => {
                CoverStep::Finished(self.source.id().to_owned(), Ok(image), self.cancel)
            }
            Err(error) => CoverStep::Finished(self.source.id().to_owned(), Err(error), self.cancel),
        }
    }
}
