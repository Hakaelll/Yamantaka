use crate::{
    Cancellation, Result,
    memory::{ByteLease, MemoryGovernor},
    source_revision::{CataloguedFile, FullSourceHash},
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub struct Source {
    pub document: String,
    pub title: String,
    pub file: CataloguedFile,
    _lease: ByteLease,
}
impl Source {
    pub fn new(document: String, title: String, file: CataloguedFile, lease: ByteLease) -> Self {
        Self {
            document,
            title,
            file,
            _lease: lease,
        }
    }
}
pub type ReaderFactory = Box<dyn FnOnce() -> Result<Box<dyn Reader>> + Send>;
/// A reader owns one independent, read-only catalogue snapshot.
pub trait Reader: Send {
    fn page(
        &self,
        after: Option<&str>,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Vec<Arc<Source>>>;
}
pub struct VerifiedSource {
    pub catalogue: Arc<Source>,
    pub proof: FullSourceHash,
    _lease: ByteLease,
}
impl VerifiedSource {
    pub fn new(catalogue: Arc<Source>, proof: FullSourceHash, lease: ByteLease) -> Self {
        Self {
            catalogue,
            proof,
            _lease: lease,
        }
    }
}
pub struct Group {
    pub size: u64,
    pub digest: [u8; 32],
    pub sources: Vec<VerifiedSource>,
}
pub struct Report {
    pub groups: Vec<Group>,
    pub progress: Progress,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub checked: u64,
    pub matched: u64,
    pub unverified: u64,
}
#[derive(Default)]
pub struct Watch {
    checked: AtomicU64,
    matched: AtomicU64,
    unverified: AtomicU64,
}
impl Watch {
    pub fn publish(&self, value: Progress) {
        self.checked.store(value.checked, Ordering::Relaxed);
        self.matched.store(value.matched, Ordering::Relaxed);
        self.unverified.store(value.unverified, Ordering::Relaxed);
    }
    pub fn snapshot(&self) -> Progress {
        let checked = self.checked.load(Ordering::Relaxed);
        Progress {
            checked,
            matched: self.matched.load(Ordering::Relaxed).min(checked),
            unverified: self.unverified.load(Ordering::Relaxed).min(checked),
        }
    }
}
