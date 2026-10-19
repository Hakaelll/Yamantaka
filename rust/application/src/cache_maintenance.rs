use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    cover_cache::{CoverCacheOperation, CoverCacheReport, CoverImageCache},
};

pub struct CacheMaintenanceRequest {
    pub operation: CoverCacheOperation,
    pub cancel: Cancellation,
}
pub struct CacheMaintenanceReply {
    pub operation: CoverCacheOperation,
    pub cancel: Cancellation,
    pub result: Result<CoverCacheReport>,
}
pub(crate) struct CacheMaintenanceJob {
    operation: CoverCacheOperation,
    cancel: Cancellation,
    replies: Receiver<Result<CoverCacheReport>>,
    handle: Option<JoinHandle<()>>,
    completed: bool,
}
impl CacheMaintenanceJob {
    pub(crate) fn start(
        request: CacheMaintenanceRequest,
        cache: Arc<dyn CoverImageCache>,
    ) -> Result<Self> {
        let (send, replies) = bounded(1);
        let operation = request.operation;
        let cancel = request.cancel.clone();
        let handle = thread::Builder::new()
            .name("yamantaka-cover-cache".into())
            .spawn(move || {
                let _ = send.send(cache.maintain(operation, &cancel));
            })
            .map_err(|_| Failure::Storage("Cover cache worker could not start".into()))?;
        Ok(Self {
            operation,
            cancel: request.cancel,
            replies,
            handle: Some(handle),
            completed: false,
        })
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
    pub(crate) fn poll(&mut self) -> Option<CacheMaintenanceReply> {
        let result = match self.replies.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err(Failure::Storage("Cover cache worker stopped".into()))
            }
        };
        self.completed = true;
        Some(CacheMaintenanceReply {
            operation: self.operation,
            cancel: self.cancel.clone(),
            result,
        })
    }
}
impl Drop for CacheMaintenanceJob {
    fn drop(&mut self) {
        if !self.completed {
            self.cancel.cancel();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
