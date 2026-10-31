use super::simulation::SimulationReport;
use crate::memory::ByteLease;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct PlannedTarget {
    pub id: String,
    pub name: String,
    pub revision: Option<[u8; 32]>,
}
impl PlannedTarget {
    pub fn will_create(&self) -> bool {
        self.revision.is_none()
    }
}
#[derive(Clone, Debug)]
pub enum PlannedAction {
    AddTag(PlannedTarget),
    AddCollection(PlannedTarget),
    AddWorkspace(PlannedTarget),
    AddReadingQueue(PlannedTarget),
    SetReadingState { finished: bool },
    SetFavorite(bool),
}
pub struct RulePlan {
    report: Arc<SimulationReport>,
    actions: Vec<PlannedAction>,
    document_revisions: Vec<[u8; 32]>,
    _leases: Vec<ByteLease>,
}
impl RulePlan {
    pub fn new(
        report: Arc<SimulationReport>,
        actions: Vec<PlannedAction>,
        document_revisions: Vec<[u8; 32]>,
        leases: Vec<ByteLease>,
    ) -> Self {
        Self {
            report,
            actions,
            document_revisions,
            _leases: leases,
        }
    }
    pub fn report(&self) -> &Arc<SimulationReport> {
        &self.report
    }
    pub fn actions(&self) -> &[PlannedAction] {
        &self.actions
    }
    pub fn document_revisions(&self) -> &[[u8; 32]] {
        &self.document_revisions
    }
}
