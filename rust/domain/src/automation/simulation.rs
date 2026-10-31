use super::*;
use crate::{
    Cancellation,
    memory::{ByteLease, MemoryGovernor},
    source_revision::SourceRevision,
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
pub const MAX_SIMULATION_DOCUMENTS: u64 = 100_000;
pub const MAX_SIMULATION_SECONDS: u64 = 30;
pub struct ScanBound {
    pub value: Option<String>,
    _lease: ByteLease,
}
impl ScanBound {
    pub fn new(value: Option<String>, lease: ByteLease) -> Self {
        Self {
            value,
            _lease: lease,
        }
    }
}
pub struct ScanEntry {
    pub revision: [u8; 32],
    pub id: String,
    pub title: String,
    pub path: PathBuf,
    pub normalized_path: String,
    pub file_name: String,
    pub author: Option<String>,
    pub missing: bool,
    pub fingerprint: String,
    pub size: u64,
    pub modified: String,
    pub membership: bool,
    _lease: ByteLease,
}
pub struct ScanData {
    pub revision: [u8; 32],
    pub id: String,
    pub title: String,
    pub path: PathBuf,
    pub normalized_path: String,
    pub file_name: String,
    pub author: Option<String>,
    pub missing: bool,
    pub fingerprint: String,
    pub size: u64,
    pub modified: String,
    pub membership: bool,
}
impl ScanEntry {
    pub fn new(data: ScanData, lease: ByteLease) -> Self {
        Self {
            revision: data.revision,
            id: data.id,
            title: data.title,
            path: data.path,
            normalized_path: data.normalized_path,
            file_name: data.file_name,
            author: data.author,
            missing: data.missing,
            fingerprint: data.fingerprint,
            size: data.size,
            modified: data.modified,
            membership: data.membership,
            _lease: lease,
        }
    }
}
pub type RuleScanFactory = Box<dyn FnOnce() -> Result<Box<dyn RuleScanReader>> + Send>;
pub trait RuleScanReader: Send {
    fn imported(
        &self,
        _id: &str,
        _governor: &MemoryGovernor,
        _cancel: &Cancellation,
    ) -> Result<Option<Arc<ScanEntry>>> {
        Err(Failure::Unsupported("Imported-document simulation".into()))
    }
    fn prepare_legacy(
        &self,
        _journal: &super::execution::JournalRevision,
        _governor: &MemoryGovernor,
        _cancel: &Cancellation,
    ) -> Result<super::legacy::LegacyRecoveryPlan> {
        Err(Failure::Unsupported("Inherited receipt review".into()))
    }
    fn prepare(
        &self,
        _report: Arc<SimulationReport>,
        _governor: &MemoryGovernor,
        _cancel: &Cancellation,
    ) -> Result<super::plan::RulePlan> {
        Err(Failure::Unsupported("Rule action review".into()))
    }
    fn begin(
        &self,
        expected: &SavedRule,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<ScanBound>;
    fn page(
        &self,
        after: Option<&str>,
        upper: &str,
        trigger: &RuleTrigger,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Vec<Arc<ScanEntry>>>;
}
#[derive(Clone)]
pub struct ExactProof {
    pub source: SourceRevision,
    pub digest: [u8; 32],
}
pub struct SimulationMatch {
    pub revision: [u8; 32],
    pub document: String,
    pub title: String,
    pub proof: Option<ExactProof>,
    _lease: ByteLease,
}
impl SimulationMatch {
    pub fn new(
        revision: [u8; 32],
        document: String,
        title: String,
        proof: Option<ExactProof>,
        lease: ByteLease,
    ) -> Self {
        Self {
            revision,
            document,
            title,
            proof,
            _lease: lease,
        }
    }
}
#[derive(Clone, Copy, Default, Debug)]
pub struct SimulationProgress {
    pub scanned: u64,
    pub matched: u64,
    pub unverified: u64,
}
#[derive(Default)]
pub struct SimulationWatch {
    scanned: AtomicU64,
    matched: AtomicU64,
    unverified: AtomicU64,
}
impl SimulationWatch {
    pub fn publish(&self, progress: SimulationProgress) {
        self.scanned.store(progress.scanned, Ordering::Relaxed);
        self.matched.store(progress.matched, Ordering::Relaxed);
        self.unverified
            .store(progress.unverified, Ordering::Relaxed);
    }
    pub fn snapshot(&self) -> SimulationProgress {
        let scanned = self.scanned.load(Ordering::Relaxed);
        SimulationProgress {
            scanned,
            matched: self.matched.load(Ordering::Relaxed).min(scanned),
            unverified: self.unverified.load(Ordering::Relaxed).min(scanned),
        }
    }
}
pub struct SimulationReport {
    pub imported: Option<Arc<super::imported::ImportedDocuments>>,
    pub rule: Arc<SavedRule>,
    pub progress: SimulationProgress,
    pub matches: Vec<Arc<SimulationMatch>>,
}
pub enum SimulationOutcome {
    Complete(Arc<SimulationReport>),
    Cancelled(SimulationProgress),
    LimitReached(SimulationProgress),
    Failed {
        progress: SimulationProgress,
        error: Failure,
    },
}
