use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread::{self, JoinHandle},
};
use yamantaka_domain::{Cancellation, Failure, Result};
use yamantaka_storage::backup::{self, RestorePreview, RestoreReview};

pub enum Command {
    Review(PathBuf),
    Prepare(RestoreReview),
}
pub enum Outcome {
    Reviewed(RestoreReview),
    Prepared(Box<RestorePreview>),
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
                    Command::Review(path) => backup::review(&path, &token).map(Outcome::Reviewed),
                    Command::Prepare(review) => backup::prepare_reviewed(&review, &token)
                        .map(|preview| Outcome::Prepared(Box::new(preview))),
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
