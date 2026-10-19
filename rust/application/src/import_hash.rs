//! Acquire the guarded full source before native parsing and retain its handle.
use crate::file_digest::HashedFile;
use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
};
pub(crate) struct ImportHashJob {
    cancel: Cancellation,
    replies: Receiver<Result<HashedFile>>,
    handle: Option<JoinHandle<()>>,
    completed: bool,
}
impl ImportHashJob {
    pub(crate) fn start(
        path: PathBuf,
        governor: MemoryGovernor,
        cancel: Cancellation,
    ) -> Result<Self> {
        cancel.check()?;
        let (send, replies) = bounded(1);
        let token = cancel.clone();
        let handle = thread::Builder::new()
            .name("yamantaka-import-hash".into())
            .spawn(move || {
                let result = (|| {
                    let mut buffer =
                        TrackedBytesBuilder::try_zeroed(&governor, MemoryCategory::Other, 65536)
                            .map_err(|_| Failure::MemoryBudget)?;
                    HashedFile::read(&path, buffer.as_mut_slice(), &token)
                })();
                let _ = send.send(result);
            })
            .map_err(|_| {
                Failure::Storage("Import source verification worker could not start".into())
            })?;
        Ok(Self {
            cancel,
            replies,
            handle: Some(handle),
            completed: false,
        })
    }
    pub(crate) fn ready(&self) -> bool {
        let mut select = Select::new();
        select.recv(&self.replies);
        select.try_ready().is_ok()
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
    pub(crate) fn cancel(&self) {
        self.cancel.cancel();
    }
    pub(crate) fn finish(&mut self) -> Result<HashedFile> {
        let result = match self.replies.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => {
                return Err(Failure::Invalid(
                    "Import source verification is not ready".into(),
                ));
            }
            Err(TryRecvError::Disconnected) => Err(Failure::Storage(
                "Import source verification worker stopped".into(),
            )),
        };
        self.completed = true;
        self.cancel.check()?;
        result
    }
}
impl Drop for ImportHashJob {
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
#[path = "import_hash/verification_test.rs"]
pub(crate) mod verification_test;
