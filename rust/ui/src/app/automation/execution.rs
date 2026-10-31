use super::*;
use yamantaka_application::automation_execution::{
    ExecutionCommand, ExecutionReply, ExecutionRequest,
};
use yamantaka_domain::{Failure, automation::execution::*};
mod view;
struct Pending {
    cancel: Cancellation,
    watch: Arc<ExecutionWatch>,
    undo: bool,
}
#[derive(Default)]
pub(super) struct ExecutionState {
    pending: Option<Pending>,
    loading: Option<Cancellation>,
    page: Option<Arc<JournalPage>>,
    offset: usize,
    history_open: bool,
    receipt: Option<Arc<JournalEntry>>,
    review: Option<Arc<JournalEntry>>,
    error: Option<String>,
    close_after: bool,
}
impl ExecutionState {
    pub(super) fn collapse_history(&mut self) {
        self.history_open = false;
    }
    pub(super) fn busy(&self) -> bool {
        self.pending.is_some() || self.loading.is_some()
    }
    pub(super) fn leave(&mut self) {
        if let Some(pending) = &self.pending {
            pending.cancel.cancel();
        }
        if let Some(cancel) = self.loading.take() {
            cancel.cancel();
        }
        self.page = None;
        self.review = None;
    }
    pub(super) fn receipt(&self) -> Option<Arc<JournalEntry>> {
        self.receipt.clone()
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"pending":self.pending.is_some(),"loading":self.loading.is_some(),"failed":self.error.is_some(),"total":self.page.as_ref().map(|page|page.total),"status":self.receipt.as_ref().map(|entry|format!("{:?}",entry.status)),"completed":self.receipt.as_ref().map(|entry|entry.completed),"reversible":self.receipt.as_ref().map(|entry|entry.reversible)})
    }
}
impl Drop for ExecutionState {
    fn drop(&mut self) {
        self.leave();
    }
}
impl YamantakaApp {
    pub(super) fn execute_reviewed_rule(&mut self) {
        if self.automation.busy() {
            return;
        }
        let Some(plan) = self.automation.review.plan() else {
            return;
        };
        self.start_rule_execution(ExecutionCommand::Apply(plan));
    }
    pub(in crate::app::automation) fn start_rule_execution(&mut self, command: ExecutionCommand) {
        if self.automation.execution.pending.is_some() {
            return;
        }
        let undo = matches!(&command, ExecutionCommand::Undo(_));
        let request = ExecutionRequest::new(command, self.governor.clone());
        let pending = Pending {
            cancel: request.context.cancel.clone(),
            watch: request.context.watch.clone(),
            undo,
        };
        self.automation.execution.error = None;
        self.automation.execution.receipt = None;
        match self.runtime.submit(Action::ExecuteRule(Box::new(request))) {
            Ok(()) => self.automation.execution.pending = Some(pending),
            Err(error) => self.automation.execution.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_rule_execution(&mut self, reply: ExecutionReply) {
        let state = &mut self.automation.execution;
        if !state
            .pending
            .as_ref()
            .is_some_and(|pending| pending.cancel.same_request(&reply.cancel))
        {
            return;
        }
        state.pending = None;
        state.history_open = true;
        state.review = None;
        match reply.result {
            Ok(receipt) => state.receipt = Some(receipt),
            Err(error) => {
                state.error = Some(format!(
                    "{error}. {}",
                    if self.english {
                        "Reload execution history before retrying."
                    } else {
                        "Recarga el historial antes de reintentar."
                    }
                ))
            }
        }
        self.automation.review.leave();
        self.automation.simulation.leave();
        self.bulk_tags_changed();
        self.bulk_collections_changed();
        self.refresh();
        if matches!(self.surface, Surface::Automation) {
            self.load_rule_journals();
        }
    }
    pub(super) fn load_rule_journals(&mut self) {
        if let Some(cancel) = self.automation.execution.loading.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::AutomationJournals(
            self.automation.execution.offset,
            self.governor.clone(),
            cancel.clone(),
        )) {
            Ok(()) => self.automation.execution.loading = Some(cancel),
            Err(error) => self.automation.execution.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_rule_journals(
        &mut self,
        result: yamantaka_domain::Result<JournalPage>,
        cancel: Cancellation,
    ) {
        let state = &mut self.automation.execution;
        if !state
            .loading
            .as_ref()
            .is_some_and(|pending| pending.same_request(&cancel))
        {
            return;
        }
        state.loading = None;
        match result {
            Ok(page) => state.page = Some(Arc::new(page)),
            Err(error) => {
                if matches!(error, Failure::MemoryBudget) {
                    state.page = None;
                    state.review = None;
                }
                state.error = Some(error.to_string());
            }
        }
    }
    pub(in crate::app) fn poll_rule_execution(&mut self, ctx: &egui::Context) {
        let state = &mut self.automation.execution;
        if let Some(pending) = &state.pending {
            ctx.request_repaint_after(Duration::from_millis(100));
            if ctx.input(|input| input.viewport().close_requested()) {
                state.close_after = true;
                pending.cancel.cancel();
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            }
        } else if state.close_after {
            state.close_after = false;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}
