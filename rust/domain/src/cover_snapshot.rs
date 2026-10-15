use crate::cover_revision::CoverRevision;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoverPlan {
    Render(u32),
    Detect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverSnapshot {
    pub plan: CoverPlan,
    pub revision: Option<CoverRevision>,
    pub source_modified: String,
}
