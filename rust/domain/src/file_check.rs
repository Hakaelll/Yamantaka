use crate::source_revision::CataloguedFile;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSnapshot {
    pub document: String,
    pub file: CataloguedFile,
    pub digest: Option<[u8; 32]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileCheckResult {
    Missing,
    Changed,
    Unverifiable,
    Available {
        digest: [u8; 32],
        modified: String,
        baseline_created: bool,
    },
}
