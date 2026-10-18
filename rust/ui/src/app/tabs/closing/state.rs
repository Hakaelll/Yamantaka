use super::*;
use std::collections::VecDeque;
use yamantaka_domain::{Failure, MAX_TABS, Result};

pub(super) struct PendingClose {
    pub(super) document: String,
    pub(super) token: Cancellation,
    pub(super) kind: CloseKind,
}
#[derive(Default)]
pub(in crate::app) struct CloseState {
    pending: Option<PendingClose>,
    queue: VecDeque<String>,
    failed_retirement: Option<String>,
    completed: u64,
}
impl CloseState {
    pub(in crate::app) fn busy(&self) -> bool {
        self.pending.is_some() || (!self.queue.is_empty() && self.failed_retirement.is_none())
    }
    pub(in crate::app) fn can_enqueue(&self, documents: &[String]) -> Result<()> {
        let mut ids: Vec<&str> = Vec::with_capacity(MAX_TABS);
        for id in self
            .queue
            .iter()
            .chain(self.failed_retirement.iter())
            .chain(self.pending.iter().map(|p| &p.document))
            .chain(documents)
        {
            if id.is_empty() {
                return Err(Failure::Invalid(
                    "A native closure requires a document identity".into(),
                ));
            }
            if !ids.contains(&id.as_str()) {
                if ids.len() == MAX_TABS {
                    return Err(Failure::Invalid(
                        "Retry pending closures before closing more readings".into(),
                    ));
                }
                ids.push(id);
            }
        }
        Ok(())
    }
    pub(super) fn enqueue(&mut self, documents: Vec<String>) -> Result<()> {
        self.can_enqueue(&documents)?;
        for document in documents {
            if self.failed_retirement.as_ref() != Some(&document)
                && !self
                    .pending
                    .as_ref()
                    .is_some_and(|p| p.document == document)
                && !self.queue.contains(&document)
            {
                self.queue.push_back(document);
            }
        }
        Ok(())
    }
    pub(super) fn next(&mut self) -> Option<String> {
        if self.pending.is_some() || self.failed_retirement.is_some() {
            return None;
        }
        self.queue.pop_front()
    }
    pub(super) fn bind(&mut self, document: String, token: Cancellation, kind: CloseKind) {
        self.pending = Some(PendingClose {
            document,
            token,
            kind,
        });
    }
    pub(super) fn take(&mut self, document: &str, token: &Cancellation) -> Option<PendingClose> {
        if !self
            .pending
            .as_ref()
            .is_some_and(|p| p.document == document && p.token.same_request(token))
        {
            return None;
        }
        self.pending.take()
    }
    pub(super) fn fail(&mut self, document: String) {
        self.failed_retirement = Some(document);
    }
    pub(super) fn retry(&mut self, live: &[&str]) {
        if let Some(document) = self.failed_retirement.take()
            && !live.contains(&document.as_str())
        {
            self.queue.push_front(document);
        }
    }
    pub(super) fn forget(&mut self, document: &str) {
        self.queue.retain(|id| id != document);
        if self.failed_retirement.as_deref() == Some(document) {
            self.failed_retirement = None;
        }
    }
    pub(super) fn has_failure(&self) -> bool {
        self.failed_retirement.is_some()
    }
    pub(super) fn succeeded(&mut self) {
        self.completed = self.completed.saturating_add(1);
    }
    pub(in crate::app) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"pending":self.pending.as_ref().map(|p|p.document.as_str()),"queued":self.queue,"failed":self.failed_retirement,"completed":self.completed})
    }
}
