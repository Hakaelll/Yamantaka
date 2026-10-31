use std::{
    io::Write,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, TryRecvError},
    },
    thread::{self, JoinHandle},
};
use yamantaka_application::diagnostics::{Renderer, Report};
use yamantaka_domain::{Cancellation, Failure, Result, memory::MemoryGovernor};
use yamantaka_storage::{Database, diagnostics::InterruptHandle};

pub struct Snapshot {
    pub report: Report,
    pub json: Arc<str>,
}
pub enum Operation {
    Collect {
        renderer: Renderer,
        governor: MemoryGovernor,
    },
    Save {
        root: PathBuf,
        destination: PathBuf,
        json: Arc<str>,
    },
}
pub enum Outcome {
    Collected(Box<Snapshot>),
    Saved,
}
pub struct Job {
    cancel: Cancellation,
    interrupt: Arc<Mutex<Option<InterruptHandle>>>,
    replies: Receiver<Result<Outcome>>,
    thread: Option<JoinHandle<()>>,
}
impl Job {
    pub fn start(
        source: PathBuf,
        operation: Operation,
        notify: impl Fn() + Send + 'static,
    ) -> Result<Self> {
        let cancel = Cancellation::default();
        let token = cancel.clone();
        let interrupt = Arc::new(Mutex::new(None));
        let shared = interrupt.clone();
        let (sender, replies) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("local-diagnostics".into())
            .spawn(move || {
                let result = (|| {
                    token.check()?;
                    let db = Database::open_read_only(&source)?;
                    *shared.lock().unwrap_or_else(|error| error.into_inner()) =
                        Some(db.diagnostics_interrupt());
                    token.check()?;
                    match operation {
                        Operation::Collect { renderer, governor } => {
                            let report = Report::collect(&db, renderer, &governor, &token)?;
                            let json = report.to_json()?;
                            Ok(Outcome::Collected(Box::new(Snapshot {
                                report,
                                json: json.into(),
                            })))
                        }
                        Operation::Save {
                            root,
                            destination,
                            json,
                        } => {
                            let parent = destination
                                .parent()
                                .ok_or_else(|| Failure::Invalid("Choose a report folder".into()))?
                                .canonicalize()
                                .map_err(|_| {
                                    Failure::Storage("Report folder unavailable".into())
                                })?;
                            let root = root.canonicalize().map_err(|_| {
                                Failure::Storage("Library folder unavailable".into())
                            })?;
                            if parent.starts_with(root)
                                || !destination
                                    .extension()
                                    .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
                            {
                                return Err(Failure::Invalid(
                                    "Choose a JSON report outside the library folder".into(),
                                ));
                            }
                            db.publish_local_export(&destination, &token, |file| {
                                file.write_all(json.as_bytes()).map_err(|_| {
                                    Failure::Storage("Report could not be saved".into())
                                })
                            })?;
                            Ok(Outcome::Saved)
                        }
                    }
                })()
                .map_err(|error| {
                    if token.is_cancelled() {
                        Failure::Cancelled
                    } else if matches!(error, Failure::DestinationOccupied) {
                        error
                    } else {
                        Failure::Storage(
                            "Diagnostic operation failed; current files were preserved".into(),
                        )
                    }
                });
                shared
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .take();
                let _ = sender.send(result);
                notify();
            })
            .map_err(|_| Failure::Storage("Diagnostic worker could not start".into()))?;
        Ok(Self {
            cancel,
            interrupt,
            replies,
            thread: Some(thread),
        })
    }
    pub fn cancel(&self) {
        self.cancel.cancel();
        if let Some(interrupt) = self
            .interrupt
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
        {
            interrupt.interrupt();
        }
    }
    pub fn poll(&mut self) -> Option<Result<Outcome>> {
        let result = match self.replies.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err(Failure::Storage("Diagnostic worker stopped".into()))
            }
        };
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        Some(
            if self.cancel.is_cancelled() && matches!(result, Ok(Outcome::Collected(_))) {
                Err(Failure::Cancelled)
            } else {
                result
            },
        )
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
