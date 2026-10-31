mod compatibility;
pub mod execution;
pub use compatibility::flutter_compatibility_hash;
pub mod plan;
pub mod simulation;
use crate::{Failure, Result, memory::ByteLease};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriggerKind {
    DocumentImported,
    SourceFolder,
    Extension,
    Author,
    Tag,
    Collection,
    ExactDuplicate,
    Inaccessible,
}
impl TriggerKind {
    pub const ALL: [Self; 8] = [
        Self::DocumentImported,
        Self::SourceFolder,
        Self::Extension,
        Self::Author,
        Self::Tag,
        Self::Collection,
        Self::ExactDuplicate,
        Self::Inaccessible,
    ];
    pub fn needs_value(self) -> bool {
        !matches!(
            self,
            Self::DocumentImported | Self::ExactDuplicate | Self::Inaccessible
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionKind {
    AddTag,
    AddCollection,
    AddWorkspace,
    SetReadingState,
    SetPriority,
    AddReadingQueue,
}
impl ActionKind {
    pub const ALL: [Self; 6] = [
        Self::AddTag,
        Self::AddCollection,
        Self::AddWorkspace,
        Self::SetReadingState,
        Self::SetPriority,
        Self::AddReadingQueue,
    ];
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleTrigger {
    pub kind: TriggerKind,
    pub value: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleAction {
    pub kind: ActionKind,
    pub value: String,
}
#[derive(Clone, Debug)]
pub struct RuleDefinition {
    pub trigger: RuleTrigger,
    pub actions: Vec<RuleAction>,
}
impl RuleDefinition {
    pub fn validate(&self) -> Result<()> {
        let value = self.trigger.value.as_deref().unwrap_or("");
        if self.trigger.kind.needs_value() && value.trim().is_empty() {
            return Err(Failure::Invalid("Choose a trigger value".into()));
        }
        self.validate_actions()
    }
    fn validate_actions(&self) -> Result<()> {
        if self.actions.is_empty()
            || self.actions.len() > 8
            || self
                .actions
                .iter()
                .any(|action| action.value.trim().is_empty())
        {
            return Err(Failure::Invalid(
                "Use between one and eight actions with nonempty values".into(),
            ));
        }
        Ok(())
    }
    pub fn decode(trigger: &str, actions: &str) -> Option<Self> {
        let definition = Self {
            trigger: serde_json::from_str(trigger).ok()?,
            actions: serde_json::from_str(actions).ok()?,
        };
        definition.validate_actions().ok()?;
        Some(definition)
    }
}
pub enum RuleContent {
    Supported(RuleDefinition),
    Unavailable,
}
#[derive(Clone, Debug)]
pub struct RuleRevision {
    pub id: String,
    pub hash: [u8; 32],
}
pub struct SavedRule {
    pub revision: RuleRevision,
    pub name: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    pub content: RuleContent,
    _lease: ByteLease,
}
impl SavedRule {
    pub fn new(
        revision: RuleRevision,
        name: String,
        enabled: bool,
        created_at: String,
        updated_at: String,
        content: RuleContent,
        lease: ByteLease,
    ) -> Self {
        Self {
            revision,
            name,
            enabled,
            created_at,
            updated_at,
            content,
            _lease: lease,
        }
    }
    pub fn definition(&self) -> Option<&RuleDefinition> {
        match &self.content {
            RuleContent::Supported(definition) => Some(definition),
            RuleContent::Unavailable => None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct RuleQuery {
    pub offset: usize,
}
pub struct RulePage {
    pub entries: Vec<Arc<SavedRule>>,
    pub total: u64,
}
#[derive(Clone, Debug)]
pub struct RuleDraft {
    pub name: String,
    pub enabled: bool,
    pub definition: RuleDefinition,
}
impl RuleDraft {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() || self.name.encode_utf16().count() > 120 {
            return Err(Failure::Invalid(
                "Use a rule name between 1 and 120 UTF-16 units".into(),
            ));
        }
        self.definition.validate()
    }
}
pub enum RuleChange {
    Create(RuleDraft),
    SetEnabled {
        expected: Arc<SavedRule>,
        enabled: bool,
    },
    Delete {
        expected: Arc<SavedRule>,
    },
}
