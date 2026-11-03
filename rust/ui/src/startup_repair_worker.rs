use std::{path::PathBuf, sync::mpsc, thread};
use yamantaka_domain::{Cancellation, Failure, Result};
use yamantaka_storage::startup::repair::{self, Repaired, Review};

pub enum Operation {
    Review(PathBuf),
    Apply(Review),
}
pub enum Outcome {
    Reviewed(Review),
    Repaired(Repaired),
}
pub struct Job {
    cancel: Cancellation,
    reply: mpsc::Receiver<Result<Outcome>>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Job {
    pub fn start(operation: Operation, notify: impl Fn() + Send + 'static) -> Result<Self> {
        let cancel = Cancellation::default();
        let token = cancel.clone();
        let (sender, reply) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("startup-record-repair".into())
            .spawn(move || {
                let result = match operation {
                    Operation::Review(root) => repair::review(&root, &token).map(Outcome::Reviewed),
                    Operation::Apply(review) => {
                        repair::apply(&review, &token).map(Outcome::Repaired)
                    }
                };
                let _ = sender.send(result);
                notify();
            })
            .map_err(|_| Failure::Storage("Startup record worker could not start".into()))?;
        Ok(Self {
            cancel,
            reply,
            thread: Some(thread),
        })
    }
    pub fn poll(&mut self) -> Option<Result<Outcome>> {
        let result = match self.reply.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(Failure::Storage("Startup record worker stopped".into()))
            }
        };
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        Some(result)
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
