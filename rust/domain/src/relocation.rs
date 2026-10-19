use crate::{file_check::FileSnapshot, source_revision::CataloguedFile};

/// A reviewed candidate. A legacy match is not proof of historical byte equality.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelocationPlan {
    pub expected: FileSnapshot,
    pub candidate: CataloguedFile,
    pub digest: [u8; 32],
}
impl RelocationPlan {
    pub fn exact(&self) -> bool {
        self.expected.digest == Some(self.digest)
    }
}
