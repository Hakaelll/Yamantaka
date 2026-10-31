use super::{plan::RulePlan, simulation::SimulationMatch};
use crate::{
    Cancellation, Result,
    memory::{ByteLease, MemoryGovernor},
    search::IndexAccess,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Debug)]
pub struct JournalRevision {
    pub id: String,
    pub updated_at: String,
    pub hash: [u8; 32],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionStatus {
    Completed,
    Cancelled,
    Interrupted,
    RolledBack,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalOrigin {
    Native,
    Flutter {
        retry_candidate: bool,
        has_undo: bool,
    },
}
pub struct JournalEntry {
    pub origin: JournalOrigin,
    pub recovery: bool,
    pub revision: JournalRevision,
    pub rule_name: String,
    pub status: ExecutionStatus,
    pub completed: u64,
    pub total: u64,
    pub reversible: bool,
    pub error_code: Option<String>,
    _lease: ByteLease,
}
impl JournalEntry {
    pub fn new(
        revision: JournalRevision,
        rule_name: String,
        status: ExecutionStatus,
        counts: (u64, u64),
        reversible: bool,
        error_code: Option<String>,
        lease: ByteLease,
    ) -> Self {
        Self {
            origin: JournalOrigin::Native,
            recovery: false,
            revision,
            rule_name,
            status,
            completed: counts.0,
            total: counts.1,
            reversible,
            error_code,
            _lease: lease,
        }
    }
    pub fn with_recovery(mut self, value: bool) -> Self {
        self.recovery = value;
        self
    }
    pub fn with_origin(mut self, origin: JournalOrigin) -> Self {
        self.origin = origin;
        self
    }
}
pub struct JournalPage {
    pub entries: Vec<Arc<JournalEntry>>,
    pub total: u64,
}
#[derive(Default)]
pub struct ExecutionWatch {
    completed: AtomicU64,
}
impl ExecutionWatch {
    pub fn publish(&self, completed: u64) {
        self.completed.store(completed, Ordering::Relaxed);
    }
    pub fn completed(&self) -> u64 {
        self.completed.load(Ordering::Relaxed)
    }
}
pub struct ExecutionContext {
    pub governor: MemoryGovernor,
    pub cancel: Cancellation,
    pub watch: Arc<ExecutionWatch>,
    pub access: IndexAccess,
}
pub type SourceGuard = Option<Box<dyn Send>>;
pub type SourceVerifier<'a> = dyn FnMut(&SimulationMatch) -> Result<SourceGuard> + 'a;
pub type ExecutionFactory = Box<dyn FnOnce() -> Result<Box<dyn RuleExecutionStore>> + Send>;
pub trait RuleExecutionStore: Send {
    fn recover_legacy(
        &mut self,
        _plan: &super::legacy::LegacyRecoveryPlan,
        _context: &ExecutionContext,
    ) -> Result<Arc<JournalEntry>> {
        Err(crate::Failure::Unsupported(
            "Inherited receipt recovery".into(),
        ))
    }
    fn apply(
        &mut self,
        plan: &RulePlan,
        context: &ExecutionContext,
        verify: &mut SourceVerifier<'_>,
    ) -> Result<Arc<JournalEntry>>;
    fn undo(
        &mut self,
        revision: &JournalRevision,
        context: &ExecutionContext,
    ) -> Result<Arc<JournalEntry>>;
}
