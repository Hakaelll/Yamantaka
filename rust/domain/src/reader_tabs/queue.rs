use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueueStatus {
    #[default]
    Pending,
    InReview,
    Blocked,
    Reviewed,
    Archived,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueEntry {
    status: QueueStatus,
    blocking_reason: Option<String>,
    next_action: Option<String>,
}
impl QueueEntry {
    pub fn new(status: QueueStatus, blocking_reason: &str, next_action: &str) -> Self {
        fn clean(value: &str) -> Option<String> {
            let value: String = value.trim().chars().take(240).collect();
            (!value.is_empty()).then_some(value)
        }
        Self {
            status,
            blocking_reason: clean(blocking_reason),
            next_action: clean(next_action),
        }
    }
    pub fn status(&self) -> QueueStatus {
        self.status
    }
    pub fn blocking_reason(&self) -> Option<&str> {
        self.blocking_reason.as_deref()
    }
    pub fn next_action(&self) -> Option<&str> {
        self.next_action.as_deref()
    }
    pub(super) fn validate(&self) -> Result<()> {
        if [&self.blocking_reason, &self.next_action]
            .into_iter()
            .flatten()
            .any(|text| text.is_empty() || text.trim() != text || text.chars().count() > 240)
        {
            return Err(Failure::Invalid(
                "Reading queue notes must contain at most 240 trimmed characters".into(),
            ));
        }
        Ok(())
    }
}
impl ReaderTab {
    pub fn queue(&self) -> &QueueEntry {
        &self.queue
    }
    pub fn with_queue(mut self, queue: QueueEntry) -> Result<Self> {
        queue.validate()?;
        self.queue = queue;
        Ok(self)
    }
}
impl DocumentTabs {
    pub fn update_queue(&mut self, id: &str, queue: QueueEntry) -> Result<()> {
        queue.validate()?;
        let tab = self
            .tabs
            .iter_mut()
            .find(|tab| tab.id() == id)
            .ok_or_else(|| Failure::Invalid("Reading queue entry no longer exists".into()))?;
        tab.queue = queue;
        Ok(())
    }
    pub fn next_queued(&self) -> Option<&ReaderTab> {
        self.tabs.iter().find(|tab| {
            matches!(
                tab.queue.status,
                QueueStatus::Pending | QueueStatus::InReview
            )
        })
    }
}
