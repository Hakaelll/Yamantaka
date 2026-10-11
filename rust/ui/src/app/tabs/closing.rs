use super::*;
use yamantaka_application::reader_close::{ReaderCloseReply, ReaderCloseRequest};

pub(super) enum CloseKind {
    Last(DocumentTabs),
    Inactive(DocumentTabs),
    Retire,
}
struct PendingClose {
    document: String,
    token: Cancellation,
    kind: CloseKind,
}
#[derive(Default)]
pub(in crate::app) struct CloseState {
    pending: Option<PendingClose>,
    failed_retirement: Option<String>,
}
impl CloseState {
    pub(in crate::app) fn busy(&self) -> bool {
        self.pending.is_some()
    }
    fn bind(&mut self, document: String, token: Cancellation, kind: CloseKind) {
        self.pending = Some(PendingClose {
            document,
            token,
            kind,
        });
    }
    fn take(&mut self, document: &str, token: &Cancellation) -> Option<PendingClose> {
        if !self
            .pending
            .as_ref()
            .is_some_and(|p| p.document == document && p.token.same_request(token))
        {
            return None;
        }
        self.pending.take()
    }
}
impl TabState {
    pub(super) fn refresh_plan_context(&self, plan: &mut DocumentTabs) {
        let selected = plan.active().map(|t| t.id().to_owned());
        if let Some(current) = self.model.active() {
            if plan.activate(current.id()) {
                let _ = plan.update_active(
                    current.page(),
                    current.preferences().clone(),
                    current.panel(),
                );
            } else {
                let _ = plan.refresh_closed(
                    current.id(),
                    current.page(),
                    current.preferences().clone(),
                    current.panel(),
                );
            }
        }
        if let Some(selected) = selected {
            plan.activate(&selected);
        }
    }
}
impl YamantakaApp {
    pub(in crate::app) fn close_reader_tab(&mut self, id: &str) {
        if self.tabs.busy() || self.opening.busy() {
            return;
        }
        self.sync_tab_context();
        let Some(closing) = self
            .tabs
            .model
            .tabs()
            .iter()
            .find(|t| t.id() == id)
            .cloned()
        else {
            return;
        };
        let active = self.tabs.model.active().is_some_and(|t| t.id() == id);
        if active && self.annotations.editing() {
            self.error = Some(
                self.tr(
                    "Termina o cancela la nota antes de cerrar la lectura.",
                    "Finish or cancel the note before closing the reading.",
                )
                .into(),
            );
            return;
        }
        let mut plan = self.tabs.model.clone();
        plan.close(id);
        if active && plan.active().is_some() {
            self.request_tab_plan(plan);
        } else if plan
            .tabs()
            .iter()
            .any(|t| t.document() == closing.document())
        {
            self.tabs.model = plan;
        } else {
            self.cache_reading_context();
            if active && let Err(error) = self.validate_reading_session_context() {
                self.error = Some(error.to_string());
                return;
            }
            self.submit_tab_close(
                closing.document().into(),
                if active {
                    CloseKind::Last(plan)
                } else {
                    CloseKind::Inactive(plan)
                },
            );
        }
    }
    pub(in crate::app) fn reopen_reader_tab(&mut self) {
        if self.tabs.busy() || self.opening.busy() {
            return;
        }
        self.sync_tab_context();
        let mut plan = self.tabs.model.clone();
        match plan.reopen_last_closed() {
            Ok(Some(_)) => {
                self.request_tab_plan(plan);
            }
            Ok(None) => {}
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    fn submit_tab_close(&mut self, document: String, kind: CloseKind) {
        let token = Cancellation::default();
        match self.runtime.submit(Action::Close(ReaderCloseRequest {
            document: document.clone(),
            cancel: token.clone(),
        })) {
            Ok(()) => self.tabs.closing.bind(document, token, kind),
            Err(error) => {
                if matches!(kind, CloseKind::Retire) {
                    self.tabs.closing.failed_retirement = Some(document);
                }
                self.error = Some(error.to_string());
            }
        }
    }
    pub(in crate::app) fn retire_closed_reader(&mut self) {
        if !self.tabs.closing.busy()
            && let Some(document) = self.tabs.retire_after_switch.take()
        {
            self.submit_tab_close(document, CloseKind::Retire);
        }
    }
    pub(in crate::app) fn retry_reader_close(&mut self) {
        if self.tabs.closing.busy() {
            return;
        }
        if let Some(document) = self.tabs.closing.failed_retirement.take()
            && !self
                .tabs
                .model
                .tabs()
                .iter()
                .any(|t| t.document() == document)
        {
            self.submit_tab_close(document, CloseKind::Retire);
        }
    }
    pub(in crate::app) fn can_retry_reader_close(&self) -> bool {
        !self.tabs.busy() && !self.opening.busy() && self.tabs.closing.failed_retirement.is_some()
    }
    pub(in crate::app) fn accept_tab_close(&mut self, reply: &ReaderCloseReply) -> bool {
        let Some(pending) = self.tabs.closing.take(&reply.document, &reply.cancel) else {
            return false;
        };
        if let Err(error) = &reply.result {
            if matches!(pending.kind, CloseKind::Retire) {
                self.tabs.closing.failed_retirement = Some(pending.document);
            }
            self.error = Some(error.to_string());
            return true;
        }
        self.sync_tab_context();
        match pending.kind {
            CloseKind::Last(mut plan) => {
                self.cache_reading_context();
                if let Err(error) = self.finish_reading_session() {
                    self.error = Some(error.to_string());
                }
                self.tabs.refresh_plan_context(&mut plan);
                self.tabs.model = plan;
                self.replace_reader_state(Cancellation::default());
                self.reader_preferences_loading = false;
                self.restoration.clear();
                self.active = None;
                self.navigation = None;
                self.show_surface(Surface::Library);
                self.refresh();
            }
            CloseKind::Inactive(mut plan) => {
                self.tabs.refresh_plan_context(&mut plan);
                self.tabs.model = plan;
            }
            CloseKind::Retire => {
                self.tabs.closing.failed_retirement = None;
            }
        }
        true
    }
    pub(in crate::app) fn forget_failed_retirement(&mut self, document: &str) {
        if self.tabs.closing.failed_retirement.as_deref() == Some(document) {
            self.tabs.closing.failed_retirement = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn close_reply_identity_is_exact_and_native_success_remains_actionable_after_late_cancellation()
    {
        let mut state = CloseState::default();
        let token = Cancellation::default();
        state.bind("document".into(), token.clone(), CloseKind::Retire);
        assert!(state.take("other", &token).is_none());
        assert!(state.take("document", &Cancellation::default()).is_none());
        assert!(state.busy());
        token.cancel();
        let pending = state.take("document", &token).unwrap();
        assert!(matches!(pending.kind, CloseKind::Retire));
        assert!(!state.busy());
        assert!(state.take("document", &token).is_none());
    }
}
