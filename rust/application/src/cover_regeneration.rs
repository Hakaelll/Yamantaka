//! A frozen cover-decision batch using the existing governed native cover job.
use crate::{
    background_password::Challenge,
    cover_job::{CoverJob, CoverStep},
    cover_rebuild,
    cover_source::CoverSource,
    library_bulk::{BulkReply, BulkReport, normalize_ids},
};
use crossbeam_channel::{Select, Sender};
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU64, Ordering},
};
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
enum Work {
    Ordinary(Box<CoverJob>),
    Unlock(Box<cover_rebuild::Job>),
}
struct Current {
    planned: PlannedCover,
    work: Work,
}
pub(crate) struct UnlockContext<'a> {
    pub passwords: &'a Sender<Challenge>,
    pub wake: &'a Sender<()>,
    pub notify: &'a dyn Fn(),
    pub capacity: &'a AtomicU64,
}
pub(crate) struct RegenerationJob {
    cancel: Cancellation,
    queue: VecDeque<PlannedCover>,
    current: Option<Current>,
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
            .is_none_or(|current| match &current.work {
                Work::Ordinary(job) => job.ready(capacity),
                Work::Unlock(job) => job.ready(capacity),
            })
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        if let Some(current) = &self.current {
            match &current.work {
                Work::Ordinary(job) => job.register(select),
                Work::Unlock(job) => job.register(select),
            }
        }
    }
    pub(crate) fn waiting(&self) -> bool {
        self.current
            .as_ref()
            .is_some_and(|current| match &current.work {
                Work::Ordinary(job) => job.waiting(),
                Work::Unlock(job) => job.waiting(),
            })
    }
    pub(crate) fn advance(
        &mut self,
        repository: &mut impl LibraryRepository,
        engine: &impl AsyncPdfEngine,
        capacity: u64,
        unlock: &UnlockContext<'_>,
    ) -> bool {
        if self.current.is_none() {
            if self.cancel.is_cancelled() {
                return true;
            }
            let Some(planned) = self.queue.pop_front() else {
                return true;
            };
            let job = CoverJob::restore_snapshot(
                planned.source.clone(),
                0,
                self.cancel.clone(),
                planned.expected.clone(),
            );
            self.current = Some(Current {
                planned,
                work: Work::Ordinary(job),
            });
        }
        if !self.ready(capacity) {
            return false;
        }
        let Current { planned, work } = self.current.take().unwrap();
        let id = planned.id.clone();
        let result = match work {
            Work::Ordinary(job) => match job.advance(repository, engine, capacity) {
                CoverStep::Pending(job) => {
                    self.current = Some(Current {
                        planned,
                        work: Work::Ordinary(job),
                    });
                    None
                }
                CoverStep::Finished(_, Err(Failure::PasswordRequired), _) => {
                    let start = (|| {
                        let document = planned.source.read(repository, &self.cancel)?;
                        let governor = engine.memory_governor().ok_or(Failure::MemoryBudget)?;
                        cover_rebuild::Job::start(
                            cover_rebuild::Request {
                                document,
                                automatic: Some(planned.expected.clone()),
                                governor,
                                cancel: self.cancel.clone(),
                            },
                            repository,
                            engine,
                        )
                    })();
                    match start {
                        Ok(job) => {
                            self.current = Some(Current {
                                planned,
                                work: Work::Unlock(job),
                            });
                            None
                        }
                        Err(error) => Some(Err(error)),
                    }
                }
                CoverStep::Finished(_, result, _) => Some(result.map(|_| ())),
            },
            Work::Unlock(job) => match job.advance(
                repository,
                engine,
                unlock.passwords,
                unlock.wake,
                unlock.notify,
                capacity,
            ) {
                cover_rebuild::Step::Pending(job) => {
                    self.current = Some(Current {
                        planned,
                        work: Work::Unlock(job),
                    });
                    None
                }
                cover_rebuild::Step::Finished(reply, released) => {
                    if released {
                        unlock.capacity.fetch_add(1, Ordering::Release);
                    }
                    Some(reply.result)
                }
            },
        };
        if let Some(result) = result {
            match result {
                Ok(()) => self.report.succeeded.push(id),
                Err(Failure::Cancelled) if self.cancel.is_cancelled() => {
                    self.report.remaining.push(id)
                }
                Err(_) => self.report.failed.push(id),
            }
        }
        self.current.is_none() && (self.queue.is_empty() || self.cancel.is_cancelled())
    }
    pub(crate) fn snapshot(&self, finished: bool) -> RegenerationSnapshot {
        let mut report = self.report.clone();
        report.remaining.extend(
            self.current
                .iter()
                .map(|current| current.planned.id.clone()),
        );
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
        if let Some(current) = &self.current {
            match &current.work {
                Work::Ordinary(job) => job.cancel(),
                Work::Unlock(job) => job.cancel(),
            }
        }
    }
}
#[cfg(test)]
pub(crate) mod verification_test;
