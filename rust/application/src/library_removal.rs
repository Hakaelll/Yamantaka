//! Close one native reader before committing removal of its frozen source record.
use crate::library_bulk::{BulkReply, BulkReport, normalize_ids};
use crossbeam_channel::Select;
use std::collections::VecDeque;
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryRepository, Result,
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
};

pub struct RemovalRequest {
    pub ids: Vec<String>,
    pub cancel: Cancellation,
}
#[derive(Clone)]
pub struct RemovalSnapshot {
    pub cancel: Cancellation,
    pub report: BulkReport,
    pub finished: bool,
}
pub(crate) struct RemovalJob {
    ids: Vec<String>,
    queue: VecDeque<Document>,
    closing: Option<(Document, PdfTicket<()>)>,
    cancel: Cancellation,
    report: BulkReport,
}
fn unchanged(repository: &impl LibraryRepository, expected: &Document) -> Result<()> {
    let current = repository
        .get(&expected.id)?
        .ok_or_else(|| Failure::Invalid("Document is no longer in the library".into()))?;
    if current.path != expected.path
        || current.fingerprint != expected.fingerprint
        || current.page_count != expected.page_count
    {
        return Err(Failure::Invalid(
            "Document source changed during removal".into(),
        ));
    }
    Ok(())
}
impl RemovalJob {
    pub(crate) fn start(
        request: RemovalRequest,
        repository: &impl LibraryRepository,
    ) -> Result<Self> {
        request.cancel.check()?;
        let ids = normalize_ids(request.ids)?;
        let mut queue = VecDeque::new();
        let mut report = BulkReport::default();
        for id in &ids {
            request.cancel.check()?;
            match repository.get(id) {
                Ok(Some(document)) => queue.push_back(document),
                _ => report.failed.push(id.clone()),
            }
        }
        Ok(Self {
            ids,
            queue,
            closing: None,
            cancel: request.cancel,
            report,
        })
    }
    pub(crate) fn contains(&self, id: &str) -> bool {
        self.ids.iter().any(|candidate| candidate == id)
    }
    pub(crate) fn ready(&self) -> bool {
        self.closing
            .as_ref()
            .is_none_or(|(_, ticket)| ticket.is_ready())
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        if let Some((_, ticket)) = &self.closing {
            select.recv(ticket.readiness());
        }
    }
    pub(crate) fn snapshot(&self, finished: bool) -> RemovalSnapshot {
        let mut report = self.report.clone();
        report
            .remaining
            .extend(self.closing.iter().map(|(document, _)| document.id.clone()));
        report
            .remaining
            .extend(self.queue.iter().map(|document| document.id.clone()));
        RemovalSnapshot {
            cancel: self.cancel.clone(),
            report,
            finished,
        }
    }
    pub(crate) fn reply(&self) -> BulkReply {
        BulkReply {
            cancel: self.cancel.clone(),
            result: Ok(self.snapshot(true).report),
        }
    }
    pub(crate) fn advance(
        &mut self,
        repository: &mut impl LibraryRepository,
        engine: &impl AsyncPdfEngine,
    ) -> bool {
        if let Some((document, ticket)) = self.closing.take() {
            if !ticket.is_ready() {
                self.closing = Some((document, ticket));
                return false;
            }
            if ticket.wait().is_err() {
                self.report.failed.push(document.id);
            } else {
                // A completed close is real even when cancellation or SQL failure follows.
                self.report.retired.push(document.id.clone());
                if self.cancel.is_cancelled() {
                    self.report.remaining.push(document.id);
                } else {
                    match unchanged(repository, &document)
                        .and_then(|()| repository.remove_from_library(&document.id, &self.cancel))
                    {
                        Ok(()) => self.report.succeeded.push(document.id),
                        Err(Failure::Cancelled) if self.cancel.is_cancelled() => {
                            self.report.remaining.push(document.id)
                        }
                        Err(_) => self.report.failed.push(document.id),
                    }
                }
            }
        } else if !self.cancel.is_cancelled()
            && let Some(document) = self.queue.pop_front()
        {
            match unchanged(repository, &document)
                .and_then(|()| engine.try_close(document.id.clone()))
            {
                Ok(ticket) => self.closing = Some((document, ticket)),
                Err(_) => self.report.failed.push(document.id),
            }
        }
        self.closing.is_none() && (self.cancel.is_cancelled() || self.queue.is_empty())
    }
}

#[cfg(test)]
pub(crate) mod verification_test;
