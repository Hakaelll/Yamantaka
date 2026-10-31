use super::execution::JournalEntry;
use crate::{
    memory::ByteLease,
    reader_preferences::{ReaderMode, ReaderTheme},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegacyTargetKind {
    Tag,
    Collection,
    Workspace,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyJournalState {
    pub status: String,
    pub phase: String,
    pub capability: String,
    pub error: Option<String>,
    pub interruption: Option<String>,
}
pub struct LegacyTarget {
    pub kind: LegacyTargetKind,
    pub id: String,
    pub name: String,
    pub revision: Option<[u8; 32]>,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyReadingState {
    pub page: i64,
    pub progress: f64,
    pub mode: String,
    pub theme: String,
    pub opened: String,
}
pub enum LegacyChange {
    RemoveMembership {
        target: u8,
        revision: Option<[u8; 32]>,
    },
    RestoreFavorite {
        current: bool,
        value: bool,
    },
    RestoreProgress {
        current: Arc<LegacyReadingState>,
        page: u32,
        page_count: u32,
        mode: ReaderMode,
        theme: ReaderTheme,
    },
}
pub struct LegacyDocument {
    pub id: String,
    pub title: String,
    pub revision: Option<[u8; 32]>,
    pub changes: Vec<LegacyChange>,
    _lease: ByteLease,
}
impl LegacyDocument {
    pub fn new(
        id: String,
        title: String,
        revision: Option<[u8; 32]>,
        changes: Vec<LegacyChange>,
        lease: ByteLease,
    ) -> Self {
        Self {
            id,
            title,
            revision,
            changes,
            _lease: lease,
        }
    }
}
pub struct LegacyRecoveryPlan {
    pub journal: Arc<JournalEntry>,
    pub before: LegacyJournalState,
    pub documents: Vec<Arc<LegacyDocument>>,
    pub targets: Vec<LegacyTarget>,
    pub skipped: usize,
    _leases: Vec<ByteLease>,
}
impl LegacyRecoveryPlan {
    pub fn new(
        journal: Arc<JournalEntry>,
        before: LegacyJournalState,
        documents: Vec<Arc<LegacyDocument>>,
        targets: Vec<LegacyTarget>,
        skipped: usize,
        leases: Vec<ByteLease>,
    ) -> Self {
        Self {
            journal,
            before,
            documents,
            targets,
            skipped,
            _leases: leases,
        }
    }
}
