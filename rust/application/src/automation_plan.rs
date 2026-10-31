use crate::file_digest::HashedFile;
use crossbeam_channel::{Receiver, Select, TryRecvError, bounded};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    automation::{plan::RulePlan, simulation::*},
    memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
};

pub struct PlanRequest {
    pub report: Arc<SimulationReport>,
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
}
impl PlanRequest {
    pub fn new(report: Arc<SimulationReport>, governor: MemoryGovernor) -> Self {
        Self {
            report,
            governor,
            cancel: Cancellation::default(),
        }
    }
}
pub struct PlanReply {
    pub cancel: Cancellation,
    pub result: Result<Arc<RulePlan>>,
}
fn prepare(request: &PlanRequest, factory: RuleScanFactory) -> Result<Arc<RulePlan>> {
    request.cancel.check()?;
    if request.report.rule.definition().is_some_and(|definition| {
        definition.trigger.kind == yamantaka_domain::automation::TriggerKind::ExactDuplicate
    }) {
        let mut buffer =
            TrackedBytesBuilder::try_zeroed(&request.governor, MemoryCategory::Other, 65536)
                .map_err(|_| Failure::MemoryBudget)?;
        for matched in &request.report.matches {
            request.cancel.check()?;
            let proof = matched.proof.as_ref().ok_or_else(|| {
                Failure::Invalid("The duplicate source has no full-content proof".into())
            })?;
            let file = HashedFile::read(
                &proof.source.canonical_path,
                buffer.as_mut_slice(),
                &request.cancel,
            )?;
            let verified = file.reference(&proof.source)?;
            if verified.digest != proof.digest {
                return Err(Failure::Invalid(
                    "A duplicate source changed; simulate again".into(),
                ));
            }
        }
    }
    let reader = factory()?;
    Ok(Arc::new(reader.prepare(
        request.report.clone(),
        &request.governor,
        &request.cancel,
    )?))
}
pub(crate) struct PlanJob {
    cancel: Cancellation,
    replies: Receiver<PlanReply>,
    handle: Option<JoinHandle<()>>,
    completed: bool,
}
impl PlanJob {
    pub(crate) fn start(request: PlanRequest, factory: RuleScanFactory) -> Result<Self> {
        request.cancel.check()?;
        let cancel = request.cancel.clone();
        let (send, replies) = bounded(1);
        let handle = thread::Builder::new()
            .name("yamantaka-rule-review".into())
            .spawn(move || {
                let result = prepare(&request, factory);
                let _ = send.send(PlanReply {
                    cancel: request.cancel,
                    result,
                });
            })
            .map_err(|_| Failure::Storage("Rule review worker could not start".into()))?;
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
    pub(crate) fn poll(&mut self) -> Option<PlanReply> {
        let reply = match self.replies.try_recv() {
            Ok(reply) => reply,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => PlanReply {
                cancel: self.cancel.clone(),
                result: Err(Failure::Storage("Rule review worker stopped".into())),
            },
        };
        self.completed = true;
        Some(reply)
    }
}
impl Drop for PlanJob {
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
#[path = "automation_plan/verification_test.rs"]
pub(crate) mod verification_test;
