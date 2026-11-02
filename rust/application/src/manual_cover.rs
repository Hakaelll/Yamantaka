use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::thread::{self, JoinHandle};
use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository, Result, cover_selection::ManualCover,
    source_revision::PhysicalSourceVersion,
};
use yamantaka_windows::source_stamp::Lease;

pub struct ManualCoverRequest {
    pub choice: ManualCover,
    pub source_version: PhysicalSourceVersion,
    pub cancel: Cancellation,
}
pub struct ManualCoverReply {
    pub document: String,
    pub result: Result<()>,
    pub cancel: Cancellation,
}
pub(crate) struct ManualCoverJob {
    request: ManualCoverRequest,
    replies: Receiver<Result<Lease>>,
    worker: Option<JoinHandle<()>>,
}
impl ManualCoverJob {
    pub fn start(request: ManualCoverRequest) -> Result<Self> {
        request.cancel.check()?;
        let path = request.choice.path().to_owned();
        let version = request.source_version;
        let cancel = request.cancel.clone();
        let (send, replies) = bounded(1);
        let worker = thread::Builder::new()
            .name("manual-cover-source".into())
            .spawn(move || {
                let result = cancel.check().and_then(|()| Lease::acquire(&path, version));
                let _ = send.send(result);
            })
            .map_err(|_| Failure::Storage("Cover source verification could not start".into()))?;
        Ok(Self {
            request,
            replies,
            worker: Some(worker),
        })
    }
    pub fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
    pub fn poll(&mut self, repository: &mut impl LibraryRepository) -> Option<ManualCoverReply> {
        let received = match self.replies.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err(Failure::Storage("Cover source verification stopped".into()))
            }
        };
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        let result = self.request.cancel.check().and_then(|()| {
            let lease = received?;
            lease.verify()?;
            self.request.cancel.check()?;
            let result = repository.save_manual_cover(&self.request.choice);
            drop(lease);
            result
        });
        // Preserve the actual commit outcome even after presentation cancellation.
        Some(ManualCoverReply {
            document: self.request.choice.document().into(),
            result,
            cancel: self.request.cancel.clone(),
        })
    }
}
impl Drop for ManualCoverJob {
    fn drop(&mut self) {
        self.request.cancel.cancel();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
