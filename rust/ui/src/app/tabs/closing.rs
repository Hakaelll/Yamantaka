use super::*;
use yamantaka_application::reader_close::{ReaderCloseReply, ReaderCloseRequest};

pub(super) enum CloseKind {
    Last(DocumentTabs),
    Inactive(DocumentTabs),
    Retire,
}
mod state;
pub(super) use state::CloseState;
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
        if self.library_selection.removing() || self.tabs.busy() || self.opening.busy() {
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
        if active && self.reader.annotations.editing() {
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
        if self.library_selection.removing() || self.tabs.busy() || self.opening.busy() {
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
    pub(super) fn request_empty_tab_plan(&mut self, plan: DocumentTabs) -> bool {
        if let Some((document, _)) = &self.reader.active {
            let document = document.id.clone();
            self.cache_reading_context();
            if let Err(error) = self.validate_reading_session_context() {
                self.error = Some(error.to_string());
                return false;
            }
            self.submit_tab_close(document, CloseKind::Last(plan))
        } else {
            let removed = removed_documents(&self.tabs.model, &plan);
            self.cancel_reader_opening();
            self.tabs.retire_after_switch = removed;
            self.tabs.model = plan;
            self.show_surface(Surface::Library);
            true
        }
    }
    fn submit_tab_close(&mut self, document: String, kind: CloseKind) -> bool {
        if let Err(error) = self
            .tabs
            .closing
            .can_enqueue(std::slice::from_ref(&document))
        {
            self.error = Some(error.to_string());
            return false;
        }
        let token = Cancellation::default();
        match self.runtime.submit(Action::Close(ReaderCloseRequest {
            document: document.clone(),
            cancel: token.clone(),
        })) {
            Ok(()) => {
                self.tabs.closing.bind(document, token, kind);
                true
            }
            Err(error) => {
                if matches!(kind, CloseKind::Retire) {
                    self.tabs.closing.fail(document);
                }
                self.error = Some(error.to_string());
                false
            }
        }
    }
    pub(in crate::app) fn retire_closed_reader(&mut self) {
        if !self.tabs.retire_after_switch.is_empty() {
            let documents = std::mem::take(&mut self.tabs.retire_after_switch);
            if let Err(error) = self.tabs.closing.enqueue(documents) {
                self.error = Some(error.to_string());
                return;
            }
        }
        if let Some(document) = self.tabs.closing.next() {
            self.submit_tab_close(document, CloseKind::Retire);
        }
    }
    pub(in crate::app) fn retry_reader_close(&mut self) {
        if self.library_selection.removing() || self.tabs.busy() || self.opening.busy() {
            return;
        }
        let live: Vec<_> = self
            .tabs
            .model
            .tabs()
            .iter()
            .map(ReaderTab::document)
            .collect();
        self.tabs.closing.retry(&live);
        self.retire_closed_reader();
    }
    pub(in crate::app) fn can_retry_reader_close(&self) -> bool {
        !self.tabs.busy() && !self.opening.busy() && self.tabs.closing.has_failure()
    }
    pub(in crate::app) fn accept_tab_close(&mut self, reply: &ReaderCloseReply) -> bool {
        let Some(pending) = self.tabs.closing.take(&reply.document, &reply.cancel) else {
            return false;
        };
        if let Err(error) = &reply.result {
            if matches!(pending.kind, CloseKind::Retire) {
                self.tabs.closing.fail(pending.document);
            }
            self.error = Some(error.to_string());
            return true;
        }
        self.tabs.closing.succeeded();
        self.sync_tab_context();
        match pending.kind {
            CloseKind::Last(mut plan) => {
                self.tabs.retire_after_switch = removed_documents(&self.tabs.model, &plan)
                    .into_iter()
                    .filter(|id| id != &reply.document)
                    .collect();
                self.cache_reading_context();
                if let Err(error) = self.finish_reading_session() {
                    self.error = Some(error.to_string());
                }
                self.tabs.refresh_plan_context(&mut plan);
                self.tabs.model = plan;
                self.replace_reader_state(Cancellation::default());
                self.reader.reader_preferences_loading = false;
                self.reader.restoration.clear();
                self.reader.active = None;
                self.reader.navigation = None;
                self.show_surface(Surface::Library);
                self.refresh();
            }
            CloseKind::Inactive(mut plan) => {
                self.tabs.refresh_plan_context(&mut plan);
                self.tabs.model = plan;
            }
            CloseKind::Retire => {}
        }
        true
    }
    pub(in crate::app) fn forget_failed_retirement(&mut self, document: &str) {
        self.tabs.closing.forget(document);
    }
}
