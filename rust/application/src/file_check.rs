use crate::file_digest::HashedFile;
use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    io::ErrorKind,
    path::PathBuf,
    thread::{self, JoinHandle},
};
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryRepository, Result,
    file_check::{FileCheckResult, FileSnapshot},
    memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
};

pub struct FileCheckRequest {
    pub document: Document,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub struct FileCheckReply {
    pub document: Document,
    pub result: Result<FileCheckResult>,
    pub cancel: Cancellation,
}
enum Observation {
    Missing(PathBuf),
    Present(HashedFile),
}
impl Observation {
    fn inspect(
        snapshot: &FileSnapshot,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Self> {
        cancel.check()?;
        match snapshot.file.path.metadata() {
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(Self::Missing(snapshot.file.path.clone()));
            }
            Err(_) => {
                return Err(Failure::Storage(
                    "The file could not be inspected; check access".into(),
                ));
            }
            Ok(metadata) if !metadata.is_file() => {
                return Err(Failure::Invalid(
                    "The stored path does not contain a regular PDF file".into(),
                ));
            }
            Ok(_) => {}
        }
        let mut buffer = TrackedBytesBuilder::try_zeroed(governor, MemoryCategory::Other, 65536)
            .map_err(|_| Failure::MemoryBudget)?;
        Ok(Self::Present(HashedFile::read(
            &snapshot.file.path,
            buffer.as_mut_slice(),
            cancel,
        )?))
    }
    fn verify(&self) -> Result<()> {
        match self {
            Self::Present(file) => file.verify(),
            Self::Missing(path) => match path.metadata() {
                Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
                _ => Err(Failure::Invalid(
                    "File availability changed while checking; retry".into(),
                )),
            },
        }
    }
    fn decision(&self, snapshot: &FileSnapshot) -> Result<FileCheckResult> {
        self.verify()?;
        let Self::Present(file) = self else {
            return Ok(FileCheckResult::Missing);
        };
        let digest = file.digest();
        if let Some(previous) = snapshot.digest {
            if previous != digest {
                return Ok(FileCheckResult::Changed);
            }
        } else {
            if file.size() != snapshot.file.size || file.fingerprint() != snapshot.file.fingerprint
            {
                return Ok(FileCheckResult::Changed);
            }
            match file.modified_matches(&snapshot.file.modified) {
                None => return Ok(FileCheckResult::Unverifiable),
                Some(false) => return Ok(FileCheckResult::Changed),
                Some(true) => {}
            }
        }
        Ok(FileCheckResult::Available {
            digest,
            modified: chrono::DateTime::<chrono::Utc>::from(file.modified()).to_rfc3339(),
            baseline_created: snapshot.digest.is_none(),
        })
    }
}
pub(crate) struct FileCheckJob {
    document: Document,
    snapshot: FileSnapshot,
    cancel: Cancellation,
    replies: Receiver<Result<Observation>>,
    handle: Option<JoinHandle<()>>,
    completed: bool,
}
impl FileCheckJob {
    pub(crate) fn start(
        request: FileCheckRequest,
        repository: &impl LibraryRepository,
    ) -> Result<Self> {
        request.cancel.check()?;
        let snapshot = repository.file_snapshot(&request.document.id)?;
        if snapshot.file.path != request.document.path
            || snapshot.file.fingerprint != request.document.fingerprint
            || snapshot.file.page_count != request.document.page_count
            || snapshot.file.missing != request.document.missing
        {
            return Err(Failure::Invalid(
                "File record changed; refresh and verify again".into(),
            ));
        }
        let (send, replies) = bounded(1);
        let scope = snapshot.clone();
        let cancel = request.cancel.clone();
        let handle = thread::Builder::new()
            .name("yamantaka-file-check".into())
            .spawn(move || {
                let _ = send.send(Observation::inspect(&scope, &request.governor, &cancel));
            })
            .map_err(|_| Failure::Storage("File verification worker could not start".into()))?;
        Ok(Self {
            document: request.document,
            snapshot,
            cancel: request.cancel,
            replies,
            handle: Some(handle),
            completed: false,
        })
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
    pub(crate) fn poll(
        &mut self,
        repository: &mut impl LibraryRepository,
    ) -> Option<FileCheckReply> {
        let observed = match self.replies.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err(Failure::Storage("File verification worker stopped".into()))
            }
        };
        self.completed = true;
        let result = self.cancel.check().and_then(|()| {
            observed.and_then(|observed| {
                let decision = observed.decision(&self.snapshot)?;
                self.cancel.check()?;
                repository.apply_file_check(&self.snapshot, decision, &self.cancel)
            })
        });
        Some(FileCheckReply {
            document: self.document.clone(),
            result,
            cancel: self.cancel.clone(),
        })
    }
}
impl Drop for FileCheckJob {
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
#[path = "file_check/verification_test.rs"]
pub(crate) mod verification_test;
