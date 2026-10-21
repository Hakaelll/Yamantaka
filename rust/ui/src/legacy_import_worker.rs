//! Native migration adapter. Components submit commands; storage work stays off the UI thread.
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use yamantaka_domain::{Cancellation, Failure, Result};
use yamantaka_storage::{
    managed_imports::{self, ImportTarget},
    migration::{self, LegacySource, MigrationProgress, MigrationReceipt, MigrationReview},
};

#[derive(Clone, Copy)]
pub struct Verification {
    pub closed: bool,
    pub cancel_copy: bool,
}
pub enum Command {
    Review(LegacySource),
    Copy(MigrationReview, ImportTarget),
    Activate(ImportTarget),
    Original(PathBuf),
}
pub enum Outcome {
    Reviewed(MigrationReview),
    Copied(MigrationReceipt),
    Activated,
    Original,
}
pub struct Job {
    cancel: Cancellation,
    result: Receiver<Result<Outcome>>,
    progress: Arc<Mutex<Option<MigrationProgress>>>,
    thread: Option<JoinHandle<()>>,
}
impl Job {
    pub fn start(
        command: Command,
        notify: impl Fn() + Send + 'static,
        verification: Option<Verification>,
    ) -> Result<Self> {
        let cancel = Cancellation::default();
        let token = cancel.clone();
        let (sender, result) = mpsc::channel();
        let progress = Arc::new(Mutex::new(None));
        let status = progress.clone();
        let thread = thread::Builder::new()
            .name("legacy-import".into())
            .spawn(move || {
                let closed = || {
                    verification
                        .map_or_else(yamantaka_windows::flutter_closed, |value| Ok(value.closed))
                };
                let result = match command {
                    Command::Review(source) => {
                        migration::review(&source, closed, &token).map(Outcome::Reviewed)
                    }
                    Command::Copy(review, target) => {
                        let mut last = Instant::now() - Duration::from_secs(1);
                        target
                            .capture(&review, closed, &token, |phase| {
                                *status.lock().unwrap_or_else(|error| error.into_inner()) =
                                    Some(phase);
                                if verification.is_some_and(|value| value.cancel_copy)
                                    && matches!(phase, MigrationProgress::Copying { .. })
                                {
                                    token.cancel();
                                }
                                if last.elapsed() >= Duration::from_millis(100) {
                                    notify();
                                    last = Instant::now();
                                }
                            })
                            .map(Outcome::Copied)
                    }
                    Command::Activate(target) => {
                        target.activate(&token).map(|_| Outcome::Activated)
                    }
                    Command::Original(root) => {
                        managed_imports::use_original(&root, &token).map(|_| Outcome::Original)
                    }
                };
                let _ = sender.send(result);
                notify();
            })
            .map_err(|_| Failure::Storage("Could not start the migration worker".into()))?;
        Ok(Self {
            cancel,
            result,
            progress,
            thread: Some(thread),
        })
    }
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
    pub fn progress(&self) -> Option<MigrationProgress> {
        *self
            .progress
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
    pub fn poll(&mut self) -> Option<Result<Outcome>> {
        let result = match self.result.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(Failure::Storage(
                "Migration worker ended before returning a result".into(),
            ))),
        };
        if result.is_some()
            && let Some(thread) = self.thread.take()
        {
            let _ = thread.join();
        }
        result
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
