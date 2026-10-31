use crate::file_digest::HashedFile;
use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    automation::{execution::*, plan::RulePlan},
    memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
    search::IndexAccess,
};

pub enum ExecutionCommand {
    Apply(Arc<RulePlan>),
    Undo(JournalRevision),
}
pub struct ExecutionRequest {
    pub command: ExecutionCommand,
    pub context: ExecutionContext,
}
impl ExecutionRequest {
    pub fn new(command: ExecutionCommand, governor: MemoryGovernor) -> Self {
        Self {
            command,
            context: ExecutionContext {
                governor,
                cancel: Cancellation::default(),
                watch: Arc::new(ExecutionWatch::default()),
                access: IndexAccess::Disabled,
            },
        }
    }
}
pub struct ExecutionReply {
    pub cancel: Cancellation,
    pub result: Result<Arc<JournalEntry>>,
}
fn execute(request: &ExecutionRequest, factory: ExecutionFactory) -> Result<Arc<JournalEntry>> {
    request.context.cancel.check()?;
    let mut store = factory()?;
    match &request.command {
        ExecutionCommand::Undo(revision) => store.undo(revision, &request.context),
        ExecutionCommand::Apply(plan) => {
            let exact = plan.report().rule.definition().is_some_and(|definition| {
                definition.trigger.kind == yamantaka_domain::automation::TriggerKind::ExactDuplicate
            });
            let mut buffer = if exact {
                Some(
                    TrackedBytesBuilder::try_zeroed(
                        &request.context.governor,
                        MemoryCategory::Other,
                        65536,
                    )
                    .map_err(|_| Failure::MemoryBudget)?,
                )
            } else {
                None
            };
            let start = Instant::now();
            store.apply(plan, &request.context, &mut |matched| {
                request.context.cancel.check()?;
                if !exact {
                    return Ok(None);
                }
                let proof = matched.proof.as_ref().ok_or_else(|| {
                    Failure::Invalid("A duplicate source has no full-content proof".into())
                })?;
                let file = HashedFile::read_checked(
                    &proof.source.canonical_path,
                    buffer.as_mut().unwrap().as_mut_slice(),
                    &|| {
                        request.context.cancel.check()?;
                        if start.elapsed() >= Duration::from_secs(30) {
                            Err(Failure::Invalid("Rule execution time limit reached".into()))
                        } else {
                            Ok(())
                        }
                    },
                )?;
                if file.reference(&proof.source)?.digest != proof.digest {
                    return Err(Failure::Invalid(
                        "A duplicate source changed; simulate again".into(),
                    ));
                }
                Ok(Some(Box::new(file) as Box<dyn Send>))
            })
        }
    }
}
pub(crate) struct ExecutionJob {
    cancel: Cancellation,
    replies: Receiver<ExecutionReply>,
    handle: Option<JoinHandle<()>>,
    completed: bool,
}
impl ExecutionJob {
    pub(crate) fn start(request: ExecutionRequest, factory: ExecutionFactory) -> Result<Self> {
        request.context.cancel.check()?;
        let cancel = request.context.cancel.clone();
        let (send, replies) = bounded(1);
        let handle = thread::Builder::new()
            .name("yamantaka-rule-execution".into())
            .spawn(move || {
                let result = execute(&request, factory);
                let _ = send.send(ExecutionReply {
                    cancel: request.context.cancel,
                    result,
                });
            })
            .map_err(|_| Failure::Storage("Rule execution worker could not start".into()))?;
        Ok(Self {
            cancel,
            replies,
            handle: Some(handle),
            completed: false,
        })
    }
    pub(crate) fn register<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.replies);
    }
    pub(crate) fn poll(&mut self) -> Option<ExecutionReply> {
        let reply = match self.replies.try_recv() {
            Ok(reply) => reply,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => ExecutionReply {
                cancel: self.cancel.clone(),
                result: Err(Failure::Storage(
                    "Rule execution worker stopped; review its journal".into(),
                )),
            },
        };
        self.completed = true;
        Some(reply)
    }
}
impl Drop for ExecutionJob {
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
#[path = "automation_execution/inherited_verification.rs"]
pub(crate) mod inherited_verification;
#[cfg(test)]
#[path = "automation_execution/verification_test.rs"]
pub(crate) mod verification_test;
