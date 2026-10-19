//! A frozen cover-decision batch using the existing governed native cover job.
use crate::{
    cover_job::{CoverJob, CoverStep},
    cover_source::CoverSource,
    library_bulk::{BulkReply, BulkReport, normalize_ids},
};
use crossbeam_channel::Select;
use std::collections::VecDeque;
use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository, Result, cover_snapshot::CoverSnapshot,
    pdf_ticket::AsyncPdfEngine,
};
pub struct RegenerationRequest {
    pub ids: Vec<String>,
    pub cancel: Cancellation,
}
#[derive(Clone)]
pub struct RegenerationSnapshot {
    pub cancel: Cancellation,
    pub report: BulkReport,
    pub finished: bool,
    pub waiting_for_capacity: bool,
}
struct PlannedCover {
    id: String,
    source: CoverSource,
    expected: CoverSnapshot,
}
pub(crate) struct RegenerationJob {
    cancel: Cancellation,
    queue: VecDeque<PlannedCover>,
    current: Option<(String, Box<CoverJob>)>,
    report: BulkReport,
}
impl RegenerationJob {
    pub(crate) fn start(
        request: RegenerationRequest,
        repository: &impl LibraryRepository,
    ) -> Result<Self> {
        request.cancel.check()?;
        let ids = normalize_ids(request.ids)?;
        let mut queue = VecDeque::with_capacity(ids.len());
        let mut report = BulkReport::default();
        // The actor does not interleave another command while these SQL-only
        // snapshots capture the revisions that this explicit reset may replace.
        for id in ids {
            request.cancel.check()?;
            let planned = (|| {
                let document = repository
                    .get(&id)?
                    .ok_or_else(|| Failure::Invalid("Cover document no longer exists".into()))?;
                if document.missing {
                    return Err(Failure::Invalid("Cover source is unavailable".into()));
                }
                let expected = repository.cover_snapshot(&id)?;
                Ok(PlannedCover {
                    id: id.clone(),
                    source: CoverSource::capture(&document),
                    expected,
                })
            })();
            match planned {
                Ok(planned) => queue.push_back(planned),
                Err(_) => report.failed.push(id),
            }
        }
        Ok(Self {
            cancel: request.cancel,
            queue,
            current: None,
            report,
        })
    }
    pub(crate) fn ready(&self, capacity: u64) -> bool {
        self.current
            .as_ref()
            .is_none_or(|(_, job)| job.ready(capacity))
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        if let Some((_, job)) = &self.current {
            job.register(select);
        }
    }
    pub(crate) fn waiting(&self) -> bool {
        self.current.as_ref().is_some_and(|(_, job)| job.waiting())
    }
    pub(crate) fn advance(
        &mut self,
        repository: &mut impl LibraryRepository,
        engine: &impl AsyncPdfEngine,
        capacity: u64,
    ) -> bool {
        if self.current.is_none() {
            if self.cancel.is_cancelled() {
                return true;
            }
            let Some(planned) = self.queue.pop_front() else {
                return true;
            };
            self.current = Some((
                planned.id,
                CoverJob::restore_snapshot(
                    planned.source,
                    0,
                    self.cancel.clone(),
                    planned.expected,
                ),
            ));
        }
        if !self.ready(capacity) {
            return false;
        }
        let (id, job) = self.current.take().unwrap();
        match job.advance(repository, engine, capacity) {
            CoverStep::Pending(job) => self.current = Some((id, job)),
            CoverStep::Finished(id, result, _) => match result {
                Ok(_) => self.report.succeeded.push(id),
                Err(Failure::Cancelled) if self.cancel.is_cancelled() => {
                    self.report.remaining.push(id)
                }
                Err(_) => self.report.failed.push(id),
            },
        }
        self.current.is_none() && (self.queue.is_empty() || self.cancel.is_cancelled())
    }
    pub(crate) fn snapshot(&self, finished: bool) -> RegenerationSnapshot {
        let mut report = self.report.clone();
        report
            .remaining
            .extend(self.current.iter().map(|(id, _)| id.clone()));
        report
            .remaining
            .extend(self.queue.iter().map(|planned| planned.id.clone()));
        RegenerationSnapshot {
            cancel: self.cancel.clone(),
            report,
            finished,
            waiting_for_capacity: self.waiting(),
        }
    }
    pub(crate) fn reply(&self) -> BulkReply {
        BulkReply {
            cancel: self.cancel.clone(),
            result: Ok(self.snapshot(true).report),
        }
    }
}
impl Drop for RegenerationJob {
    fn drop(&mut self) {
        if let Some((_, job)) = &self.current {
            job.cancel();
        }
    }
}
#[cfg(test)]
pub(crate) mod verification_test;
