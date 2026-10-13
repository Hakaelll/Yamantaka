/// Opaque comparison token for one exact persisted cover decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoverRevision([u8; 32]);
impl CoverRevision {
    pub fn from_digest(digest: [u8; 32]) -> Self {
        Self(digest)
    }
}
