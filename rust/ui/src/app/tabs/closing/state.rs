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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retirement_is_serial_deduplicated_and_paused_for_exact_failure_retry() {
        let mut state = CloseState::default();
        state
            .enqueue(vec!["a".into(), "b".into(), "a".into()])
            .unwrap();
        assert_eq!(state.next().as_deref(), Some("a"));
        let token = Cancellation::default();
        state.bind("a".into(), token.clone(), CloseKind::Retire);
        assert!(state.next().is_none());
        assert!(state.take("b", &token).is_none());
        state.take("a", &token).unwrap();
        state.fail("a".into());
        assert!(state.next().is_none());
        assert!(!state.busy());
        state.retry(&[]);
        assert_eq!(state.next().as_deref(), Some("a"));
        state.bind("a".into(), token.clone(), CloseKind::Retire);
        token.cancel();
        state.take("a", &token).unwrap();
        state.succeeded();
        assert_eq!(state.next().as_deref(), Some("b"));
        assert_eq!(state.completed, 1);
    }
    #[test]
    fn capacity_failure_is_atomic_and_reopened_documents_are_removed_from_retirement() {
        let mut state = CloseState::default();
        state
            .enqueue((0..MAX_TABS).map(|n| format!("doc-{n}")).collect())
            .unwrap();
        let original = state.queue.clone();
        assert!(state.enqueue(vec!["extra".into()]).is_err());
        assert_eq!(state.queue, original);
        let first = state.next().unwrap();
        state.fail(first);
        state.forget("doc-0");
        state.forget("doc-1");
        assert!(!state.has_failure());
        assert_eq!(state.next().as_deref(), Some("doc-2"));
        assert!(state.enqueue(vec![String::new()]).is_err());
    }
}
