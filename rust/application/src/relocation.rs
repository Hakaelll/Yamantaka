use crate::file_digest::HashedFile;
use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryRepository, Result,
    file_check::FileSnapshot,
    memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
    relocation::RelocationPlan,
    source_revision::CataloguedFile,
};

pub enum RelocationAction {
    Inspect {
        document: Box<Document>,
        candidate: PathBuf,
    },
    Confirm(Box<RelocationPlan>),
}
pub struct RelocationRequest {
    pub action: RelocationAction,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
impl RelocationRequest {
    pub fn document(&self) -> &str {
        match &self.action {
            RelocationAction::Inspect { document, .. } => &document.id,
            RelocationAction::Confirm(plan) => &plan.expected.document,
        }
    }
}
pub enum RelocationOutcome {
    Review(Box<RelocationPlan>),
    Applied,
}
pub struct RelocationReply {
    pub document: String,
    pub cancel: Cancellation,
    pub result: Result<RelocationOutcome>,
}
fn stale() -> Failure {
    Failure::Invalid(
        "The file or catalogue changed; select and review the replacement again".into(),
    )
}
fn validate(plan: &RelocationPlan, file: &HashedFile) -> Result<()> {
    file.verify()?;
    let candidate = &plan.candidate;
    if candidate.path != file.path()
        || candidate.size != file.size()
        || candidate.modified != chrono::DateTime::<chrono::Utc>::from(file.modified()).to_rfc3339()
        || candidate.fingerprint != file.fingerprint()
        || plan.digest != file.digest()
    {
        return Err(stale());
    }
    Ok(())
}
pub(crate) struct RelocationJob {
    snapshot: FileSnapshot,
    confirm: Option<Box<RelocationPlan>>,
    cancel: Cancellation,
    replies: Receiver<Result<HashedFile>>,
    handle: Option<JoinHandle<()>>,
    completed: bool,
}
impl RelocationJob {
    pub(crate) fn start(
        request: RelocationRequest,
        repository: &impl LibraryRepository,
    ) -> Result<Self> {
        request.cancel.check()?;
        let snapshot = repository.file_snapshot(request.document())?;
        if !snapshot.file.missing {
            return Err(Failure::Invalid(
                "Check the missing file before choosing its replacement".into(),
            ));
        }
        let (candidate, confirm) = match request.action {
            RelocationAction::Inspect {
                document,
                candidate,
            } => {
                if snapshot.file.path != document.path
                    || snapshot.file.fingerprint != document.fingerprint
                    || snapshot.file.page_count != document.page_count
                    || !document.missing
                {
                    return Err(stale());
                }
                (candidate, None)
            }
            RelocationAction::Confirm(plan) => {
                if plan.expected != snapshot {
                    return Err(stale());
                }
                (plan.candidate.path.clone(), Some(plan))
            }
        };
        let (send, replies) = bounded(1);
        let token = request.cancel.clone();
        let handle = thread::Builder::new()
            .name("yamantaka-relocation".into())
            .spawn(move || {
                let result = (|| {
                    let mut bytes = TrackedBytesBuilder::try_zeroed(
                        &request.governor,
                        MemoryCategory::Other,
                        65536,
                    )
                    .map_err(|_| Failure::MemoryBudget)?;
                    HashedFile::read(&candidate, bytes.as_mut_slice(), &token)
                })();
                let _ = send.send(result);
            })
            .map_err(|_| Failure::Storage("Relocation worker could not start".into()))?;
        Ok(Self {
            snapshot,
            confirm,
            cancel: request.cancel,
            replies,
            handle: Some(handle),
            completed: false,
        })
    }
    pub(crate) fn document(&self) -> &str {
        &self.snapshot.document
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
    pub(crate) fn cancel(&self) {
        self.cancel.cancel();
    }
    pub(crate) fn poll(
        &mut self,
        repository: &mut impl LibraryRepository,
    ) -> Option<RelocationReply> {
        let observed = match self.replies.try_recv() {
            Ok(value) => value,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err(Failure::Storage("Relocation worker stopped".into()))
            }
        };
        self.completed = true;
        let result = (|| {
            self.cancel.check()?;
            if repository.file_snapshot(&self.snapshot.document)? != self.snapshot {
                return Err(stale());
            }
            let file = observed?;
            file.verify()?;
            if file.size() != self.snapshot.file.size
                || file.fingerprint() != self.snapshot.file.fingerprint
                || self
                    .snapshot
                    .digest
                    .is_some_and(|digest| digest != file.digest())
            {
                return Err(Failure::Invalid(
                    "The selected file does not match the saved document".into(),
                ));
            }
            let candidate = CataloguedFile {
                path: file.path().to_owned(),
                fingerprint: file.fingerprint(),
                page_count: self.snapshot.file.page_count,
                missing: false,
                size: file.size(),
                modified: chrono::DateTime::<chrono::Utc>::from(file.modified()).to_rfc3339(),
            };
            if let Some(plan) = &self.confirm {
                validate(plan, &file)?;
                self.cancel.check()?;
                repository.relocate_file(plan, &self.cancel)?;
                Ok(RelocationOutcome::Applied)
            } else {
                Ok(RelocationOutcome::Review(Box::new(RelocationPlan {
                    expected: self.snapshot.clone(),
                    candidate,
                    digest: file.digest(),
                })))
            }
        })();
        Some(RelocationReply {
            document: self.snapshot.document.clone(),
            cancel: self.cancel.clone(),
            result,
        })
    }
}
impl Drop for RelocationJob {
    fn drop(&mut self) {
        if !self.completed {
            self.cancel.cancel();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
#[path = "relocation/verification_test.rs"]
pub(crate) mod verification_test;
