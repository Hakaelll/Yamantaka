use super::*;
use std::collections::VecDeque;
use yamantaka_domain::{
    duplicates::merge::{Choices, Factory, Plan, Receipt, Selection},
    pdf_ticket::{AsyncPdfEngine, PdfTicket},
};
pub enum Command {
    Review(Box<Selection>),
    Apply { plan: Arc<Plan>, choices: Choices },
}
pub struct MergeRequest {
    pub command: Command,
    pub preferences: serde_json::Value,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
pub enum Outcome {
    Reviewed(Arc<Plan>),
    Merged(Receipt),
}
pub struct MergeReply {
    pub cancel: Cancellation,
    pub result: Result<Outcome>,
}
impl MergeRequest {
    pub fn writing(&self) -> bool {
        matches!(self.command, Command::Apply { .. })
    }
    pub fn documents(&self) -> [&str; 2] {
        let selection = match &self.command {
            Command::Review(selection) => selection,
            Command::Apply { plan, .. } => &plan.selection,
        };
        [&selection.kept.document, &selection.incoming.document]
    }
}
struct Worker {
    replies: Receiver<MergeReply>,
    handle: Option<JoinHandle<()>>,
    cancel: Cancellation,
    completed: bool,
}
impl Worker {
    fn start(request: MergeRequest, factory: Factory) -> Result<Self> {
        let cancel = request.cancel.clone();
        let (send, replies) = bounded(1);
        let handle = thread::Builder::new()
            .name("yamantaka-duplicate-merge".into())
            .spawn(move || {
                let result = factory().and_then(|mut store| match request.command {
                    Command::Review(selection) => store
                        .prepare(*selection, &request.governor, &request.cancel)
                        .map(Outcome::Reviewed),
                    Command::Apply { plan, choices } => store
                        .apply(
                            &plan,
                            &choices,
                            &request.governor,
                            &request.cancel,
                            &mut |selection| {
                                super::merge_sources::freeze(
                                    selection,
                                    &request.governor,
                                    &request.cancel,
                                )
                            },
                        )
                        .map(Outcome::Merged),
                });
                let _ = send.send(MergeReply {
                    cancel: request.cancel,
                    result,
                });
            })
            .map_err(|_| Failure::Storage("Consolidation worker could not start".into()))?;
        Ok(Self {
            replies,
            handle: Some(handle),
            cancel,
            completed: false,
        })
    }
    fn poll(&mut self) -> Option<MergeReply> {
        let reply = match self.replies.try_recv() {
            Ok(reply) => reply,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => MergeReply {
                cancel: self.cancel.clone(),
                result: Err(Failure::Storage("Consolidation worker stopped".into())),
            },
        };
        self.completed = true;
        Some(reply)
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        if !self.completed {
            self.cancel.cancel();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
pub(crate) struct MergeJob {
    request: Option<MergeRequest>,
    factory: Option<Factory>,
    cancel: Cancellation,
    writing: bool,
    terminal: bool,
    closes: VecDeque<String>,
    closing: Option<PdfTicket<()>>,
    worker: Option<Worker>,
}
impl MergeJob {
    pub(crate) fn start(request: MergeRequest, factory: Factory) -> Result<Self> {
        request.cancel.check()?;
        let writing = request.writing();
        let closes = if writing {
            request.documents().map(str::to_owned).into()
        } else {
            VecDeque::new()
        };
        Ok(Self {
            cancel: request.cancel.clone(),
            request: Some(request),
            factory: Some(factory),
            writing,
            terminal: false,
            closes,
            closing: None,
            worker: None,
        })
    }
    pub(crate) fn writing(&self) -> bool {
        self.writing
    }
    pub(crate) fn needs_step(&self) -> bool {
        self.worker.is_none() && self.closing.is_none()
    }
    pub(crate) fn step(&mut self, engine: &impl AsyncPdfEngine) -> Option<MergeReply> {
        if let Some(worker) = &mut self.worker {
            let reply = worker.poll();
            self.terminal = reply.is_some();
            return reply;
        }
        let outcome = (|| -> Result<()> {
            if let Some(ticket) = self.closing.take() {
                if !ticket.is_ready() {
                    self.closing = Some(ticket);
                    return Ok(());
                }
                ticket.wait()?;
            }
            self.cancel.check()?;
            if let Some(id) = self.closes.pop_front() {
                self.closing = Some(engine.try_close(id)?);
            } else {
                self.worker = Some(Worker::start(
                    self.request.take().unwrap(),
                    self.factory.take().unwrap(),
                )?);
            }
            Ok(())
        })();
        self.terminal = outcome.is_err();
        outcome.err().map(|error| MergeReply {
            cancel: self.cancel.clone(),
            result: Err(error),
        })
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        if let Some(ticket) = &self.closing {
            select.recv(ticket.readiness());
        }
        if let Some(worker) = &self.worker {
            select.recv(&worker.replies);
        }
    }
}
impl Drop for MergeJob {
    fn drop(&mut self) {
        if !self.terminal {
            self.cancel.cancel();
        }
    }
}
