//! A cooperative catalogue scan with one hashing job and at most 80 queued records.
use crate::file_check::{FileCheckJob, FileCheckRequest};
use crossbeam_channel::Select;
use std::collections::VecDeque;
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryRepository, MAX_LIBRARY_PAGE, Result,
    file_check::FileCheckResult, memory::MemoryGovernor,
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CheckCounts {
    pub available: u64,
    pub missing: u64,
    pub changed: u64,
    pub unverifiable: u64,
    pub failed: u64,
}
impl CheckCounts {
    pub fn checked(self) -> u64 {
        self.available + self.missing + self.changed + self.unverifiable + self.failed
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LibraryCheckPhase {
    #[default]
    Idle,
    Running,
    Completed,
    Cancelled,
    Failed,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LibraryCheckProgress {
    pub phase: LibraryCheckPhase,
    pub counts: CheckCounts,
}
#[derive(Clone)]
pub struct LibraryCheckSnapshot {
    pub cancel: Cancellation,
    pub progress: LibraryCheckProgress,
}
pub struct LibraryCheckRequest {
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub struct LibraryCheckReply {
    pub cancel: Cancellation,
    pub progress: LibraryCheckProgress,
    pub error: Option<Failure>,
}
pub(crate) struct LibraryCheckJob {
    request: LibraryCheckRequest,
    upper: Option<String>,
    after: Option<String>,
    queue: VecDeque<Document>,
    current: Option<FileCheckJob>,
    exhausted: bool,
    progress: LibraryCheckProgress,
    error: Option<Failure>,
}
impl LibraryCheckJob {
    pub(crate) fn start(
        request: LibraryCheckRequest,
        repository: &impl LibraryRepository,
    ) -> Result<Self> {
        request.cancel.check()?;
        let upper = repository.file_check_upper_bound()?;
        Ok(Self {
            request,
            upper,
            after: None,
            queue: VecDeque::new(),
            current: None,
            exhausted: false,
            progress: LibraryCheckProgress {
                phase: LibraryCheckPhase::Running,
                counts: CheckCounts::default(),
            },
            error: None,
        })
    }
    pub(crate) fn cancel(&self) -> &Cancellation {
        &self.request.cancel
    }
    pub(crate) fn progress(&self) -> LibraryCheckProgress {
        self.progress
    }
    pub(crate) fn needs_step(&self) -> bool {
        self.current.is_none() || self.request.cancel.is_cancelled()
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        if let Some(job) = &self.current {
            job.register(select);
        }
    }
    fn advance(&mut self, repository: &mut impl LibraryRepository) -> Result<()> {
        if self.request.cancel.is_cancelled() {
            self.current = None;
            self.progress.phase = LibraryCheckPhase::Cancelled;
            return Ok(());
        }
        if let Some(job) = &mut self.current {
            let Some(reply) = job.poll(repository) else {
                return Ok(());
            };
            self.current = None;
            match reply.result {
                Ok(FileCheckResult::Available { .. }) => self.progress.counts.available += 1,
                Ok(FileCheckResult::Missing) => self.progress.counts.missing += 1,
                Ok(FileCheckResult::Changed) => self.progress.counts.changed += 1,
                Ok(FileCheckResult::Unverifiable) => self.progress.counts.unverifiable += 1,
                Err(Failure::Cancelled) => {
                    self.progress.phase = LibraryCheckPhase::Cancelled;
                    return Ok(());
                }
                Err(_) => self.progress.counts.failed += 1,
            }
            return Ok(());
        }
        if self.queue.is_empty() && !self.exhausted {
            let page = if let Some(upper) = &self.upper {
                repository.file_check_page(self.after.as_deref(), upper)?
            } else {
                Vec::new()
            };
            if page.len() > MAX_LIBRARY_PAGE
                || page.windows(2).any(|pair| pair[0].id >= pair[1].id)
                || page
                    .first()
                    .is_some_and(|doc| self.after.as_ref().is_some_and(|after| doc.id <= *after))
                || page
                    .last()
                    .is_some_and(|doc| self.upper.as_ref().is_none_or(|upper| doc.id > *upper))
            {
                return Err(Failure::Invalid(
                    "File verification page is outside its ordered bounds".into(),
                ));
            }
            self.exhausted = page.len() < MAX_LIBRARY_PAGE;
            if let Some(last) = page.last() {
                self.after = Some(last.id.clone());
            }
            self.queue = page.into();
        }
        let Some(document) = self.queue.pop_front() else {
            self.progress.phase = LibraryCheckPhase::Completed;
            return Ok(());
        };
        match FileCheckJob::start(
            FileCheckRequest {
                document,
                governor: self.request.governor.clone(),
                cancel: self.request.cancel.clone(),
            },
            repository,
        ) {
            Ok(job) => self.current = Some(job),
            Err(Failure::Cancelled) => self.progress.phase = LibraryCheckPhase::Cancelled,
            Err(_) => self.progress.counts.failed += 1,
        }
        Ok(())
    }
    pub(crate) fn step(&mut self, repository: &mut impl LibraryRepository) -> bool {
        if let Err(error) = self.advance(repository) {
            self.error = Some(error);
            self.progress.phase = LibraryCheckPhase::Failed;
        }
        self.progress.phase != LibraryCheckPhase::Running
    }
    pub(crate) fn reply(&mut self) -> LibraryCheckReply {
        LibraryCheckReply {
            cancel: self.request.cancel.clone(),
            progress: self.progress,
            error: self.error.take(),
        }
    }
}
impl Drop for LibraryCheckJob {
    fn drop(&mut self) {
        if self.current.is_some() {
            self.request.cancel.cancel();
        }
        self.current = None;
    }
}
#[cfg(test)]
#[path = "library_check/verification_test.rs"]
pub(crate) mod verification_test;
