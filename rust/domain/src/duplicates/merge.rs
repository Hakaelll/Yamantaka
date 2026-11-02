use super::*;
use crate::{Failure, memory::ByteLease};
use std::{collections::BTreeMap, path::PathBuf};

pub struct Selection {
    pub kept: Arc<Source>,
    pub incoming: Arc<Source>,
    pub kept_proof: FullSourceHash,
    pub incoming_proof: FullSourceHash,
}
impl Selection {
    pub fn validate(&self) -> Result<()> {
        let left = &self.kept.file;
        let right = &self.incoming.file;
        if self.kept.document == self.incoming.document
            || self.kept_proof.digest != self.incoming_proof.digest
            || left.size != right.size
            || left.fingerprint != right.fingerprint
            || left.page_count != right.page_count
            || left.missing
            || right.missing
        {
            return Err(Failure::Invalid(
                "Choose two revalidated identical copies".into(),
            ));
        }
        for (file, proof) in [(left, &self.kept_proof), (right, &self.incoming_proof)] {
            if file.size != proof.source.size
                || file.fingerprint
                    != crate::source_revision::legacy_fingerprint(
                        proof.source.size,
                        proof.source.head_sha256,
                    )
            {
                return Err(Failure::Invalid(
                    "Duplicate content proof is inconsistent".into(),
                ));
            }
        }
        Ok(())
    }
    pub fn from_group(group: &Group, kept: &str, incoming: &str) -> Result<Self> {
        let find = |id: &str| {
            group
                .sources
                .iter()
                .find(|source| source.catalogue.document == id)
                .ok_or_else(|| Failure::Invalid("Choose two members of the verified group".into()))
        };
        let left = find(kept)?;
        let right = find(incoming)?;
        if kept == incoming
            || left.proof.digest != group.digest
            || right.proof.digest != group.digest
            || left.proof.source.size != group.size
            || right.proof.source.size != group.size
            || left.catalogue.file.size != group.size
            || right.catalogue.file.size != group.size
            || left.catalogue.file.fingerprint != right.catalogue.file.fingerprint
            || left.catalogue.file.page_count != right.catalogue.file.page_count
            || left.catalogue.file.missing
            || right.catalogue.file.missing
        {
            return Err(Failure::Invalid(
                "Duplicate sources require revalidation".into(),
            ));
        }
        Ok(Self {
            kept: left.catalogue.clone(),
            incoming: right.catalogue.clone(),
            kept_proof: left.proof.clone(),
            incoming_proof: right.proof.clone(),
        })
    }
}
#[derive(Clone, Copy, Default)]
pub enum Choice {
    #[default]
    Kept,
    Incoming,
}
#[derive(Default)]
pub struct Choices {
    pub title: Choice,
    pub author: Choice,
    pub note: Choice,
}
pub struct Field {
    pub kept: Option<String>,
    pub incoming: Option<String>,
}
impl Field {
    pub fn conflict(&self) -> bool {
        self.kept.is_some() && self.incoming.is_some() && self.kept != self.incoming
    }
    pub fn resolve(&self, choice: Choice) -> Option<&str> {
        match choice {
            Choice::Kept => self.kept.as_deref().or(self.incoming.as_deref()),
            Choice::Incoming => self.incoming.as_deref().or(self.kept.as_deref()),
        }
    }
}
pub struct Plan {
    pub selection: Selection,
    pub revision: [u8; 32],
    pub title: Field,
    pub author: Field,
    pub note: Field,
    pub affected: BTreeMap<&'static str, u64>,
    _lease: ByteLease,
}
impl Plan {
    pub fn new(
        selection: Selection,
        revision: [u8; 32],
        fields: [Field; 3],
        affected: BTreeMap<&'static str, u64>,
        lease: ByteLease,
    ) -> Self {
        let [title, author, note] = fields;
        Self {
            selection,
            revision,
            title,
            author,
            note,
            affected,
            _lease: lease,
        }
    }
}
pub struct Receipt {
    pub history: String,
    pub kept: String,
    pub incoming: String,
    pub backup: PathBuf,
    pub backup_sha256: String,
}
/// Keeps the selected full-hash source proofs alive through the database commit.
pub trait Guard: Send {
    fn verify(&self) -> Result<()>;
}
pub type VerifySources<'a> = dyn FnMut(&Selection) -> Result<Box<dyn Guard>> + 'a;
pub trait Store: Send {
    fn prepare(
        &self,
        selection: Selection,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
    ) -> Result<Arc<Plan>>;
    fn apply(
        &mut self,
        plan: &Plan,
        choices: &Choices,
        governor: &MemoryGovernor,
        cancel: &Cancellation,
        verify: &mut VerifySources<'_>,
    ) -> Result<Receipt>;
}
pub type Factory = Box<dyn FnOnce() -> Result<Box<dyn Store>> + Send>;
