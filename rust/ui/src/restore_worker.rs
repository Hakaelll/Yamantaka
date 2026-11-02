use std::{collections::BTreeMap, path::Path};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread::{self, JoinHandle},
};
use yamantaka_domain::{Cancellation, Failure, Result};
use yamantaka_storage::backup::{self, RestorePreview, RestoreReview, RollbackReview};

#[derive(Clone)]
pub enum Review {
    Json(RestoreReview),
    Rollback(RollbackReview),
}
impl Review {
    pub fn source(&self) -> &Path {
        match self {
            Self::Json(r) => r.source(),
            Self::Rollback(r) => r.source(),
        }
    }
    pub fn counts(&self) -> &BTreeMap<String, usize> {
        match self {
            Self::Json(r) => r.counts(),
            Self::Rollback(r) => r.counts(),
        }
    }
    pub fn version(&self) -> u64 {
        match self {
            Self::Json(r) => r.version(),
            Self::Rollback(_) => yamantaka_domain::SCHEMA_VERSION.into(),
        }
    }
    pub fn rollback(&self) -> bool {
        matches!(self, Self::Rollback(_))
    }
}
pub enum Prepared {
    Json(Box<RestorePreview>),
    Rollback(RollbackReview),
}
impl Prepared {
    pub fn install(self, active: &Path, cancel: &Cancellation) -> Result<PathBuf> {
        match self {
            Self::Json(p) => backup::install_preview(*p, active, cancel),
            Self::Rollback(r) => backup::install_rollback(&r, active, cancel),
        }
    }
}
pub enum Command {
    Review(PathBuf),
    ReviewExpected { path: PathBuf, sha256: String },
    ReviewRollback { active: PathBuf, source: PathBuf },
    Prepare(Review),
}
pub enum Outcome {
    Reviewed(Review),
    Prepared(Box<Prepared>),
}
pub struct Job {
    cancel: Cancellation,
    replies: Receiver<Result<Outcome>>,
    thread: Option<JoinHandle<()>>,
}
impl Job {
    pub fn start(command: Command, notify: impl FnOnce() + Send + 'static) -> Result<Self> {
        let cancel = Cancellation::default();
        let token = cancel.clone();
        let (sender, replies) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("restore-review".into())
            .spawn(move || {
                let result = match command {
                    Command::Review(path) => backup::review(&path, &token)
                        .map(Review::Json)
                        .map(Outcome::Reviewed),
                    Command::ReviewExpected { path, sha256 } => backup::review(&path, &token)
                        .and_then(|review| {
                            if review.sha256() == sha256 {
                                Ok(Outcome::Reviewed(Review::Json(review)))
                            } else {
                                Err(Failure::BackupSourceChanged)
                            }
                        }),
                    Command::ReviewRollback { active, source } => {
                        backup::review_rollback(&active, &source, &token)
                            .map(Review::Rollback)
                            .map(Outcome::Reviewed)
                    }
                    Command::Prepare(Review::Json(review)) => {
                        backup::prepare_reviewed(&review, &token)
                            .map(|p| Prepared::Json(Box::new(p)))
                            .map(|p| Outcome::Prepared(Box::new(p)))
                    }
                    Command::Prepare(Review::Rollback(review)) => {
                        backup::verify_rollback(&review, &token)
                            .map(|_| Outcome::Prepared(Box::new(Prepared::Rollback(review))))
                    }
                };
                let _ = sender.send(result);
                notify();
            })
            .map_err(|_| Failure::Storage("Restore worker could not start".into()))?;
        Ok(Self {
            cancel,
            replies,
            thread: Some(thread),
        })
    }
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
    pub fn poll(&mut self) -> Option<Result<Outcome>> {
        let result = match self.replies.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err(Failure::Storage("Restore worker stopped".into()))
            }
        };
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        Some(self.cancel.check().and(result))
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
