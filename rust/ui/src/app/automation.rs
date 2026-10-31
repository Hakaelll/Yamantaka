use super::*;
use yamantaka_domain::automation::*;
mod preview;
mod review;
mod simulation;
mod view;
pub(super) struct RuleEditor {
    name: String,
    trigger: TriggerKind,
    trigger_value: String,
    action: ActionKind,
    action_value: String,
}
impl Default for RuleEditor {
    fn default() -> Self {
        Self {
            name: String::new(),
            trigger: TriggerKind::Extension,
            trigger_value: String::new(),
            action: ActionKind::AddTag,
            action_value: String::new(),
        }
    }
}
impl RuleEditor {
    fn draft(&self) -> RuleDraft {
        RuleDraft {
            name: self.name.trim().into(),
            enabled: true,
            definition: RuleDefinition {
                trigger: RuleTrigger {
                    kind: self.trigger,
                    value: self
                        .trigger
                        .needs_value()
                        .then(|| self.trigger_value.trim().into()),
                },
                actions: vec![RuleAction {
                    kind: self.action,
                    value: self.action_value.trim().into(),
                }],
            },
        }
    }
}
#[derive(Default)]
pub(super) struct AutomationState {
    review: review::ReviewState,
    simulation: simulation::SimulationState,
    page: Option<Arc<RulePage>>,
    pending: Option<Cancellation>,
    writing: Option<(Cancellation, bool)>,
    offset: usize,
    editor: Option<RuleEditor>,
    deleting: Option<Arc<SavedRule>>,
    error: Option<String>,
    saved: bool,
    preview: u8,
    preview_mode: u8,
}
impl AutomationState {
    pub(super) fn leave(&mut self) {
        self.review.leave();
        self.simulation.leave();
        if let Some(cancel) = self.pending.take() {
            cancel.cancel();
        }
        self.page = None;
        self.deleting = None;
    }
    fn busy(&self) -> bool {
        self.pending.is_some()
            || self.writing.is_some()
            || self.simulation.busy()
            || self.review.busy()
    }
    pub(super) fn ready(&self) -> bool {
        !self.busy() && self.simulation.ready() && matches!(self.preview, 0 | 10)
    }
    pub(super) fn simulation_summary(&self) -> serde_json::Value {
        self.simulation.summary()
    }
    pub(super) fn review_summary(&self) -> serde_json::Value {
        self.review.summary()
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"phase":self.preview,"pending":self.busy(),"failed":self.error.is_some(),"saved":self.saved,"editing":self.editor.is_some(),"confirmDelete":self.deleting.is_some(),"total":self.page.as_ref().map(|page|page.total),"offset":self.offset,"entries":self.page.as_ref().map(|page|page.entries.iter().map(|rule|serde_json::json!({"id":rule.revision.id,"name":rule.name,"enabled":rule.enabled,"available":rule.definition().is_some(),"actions":rule.definition().map(|definition|definition.actions.len()),"created":rule.created_at,"updated":rule.updated_at})).collect::<Vec<_>>())})
    }
}
impl Drop for AutomationState {
    fn drop(&mut self) {
        self.leave();
        if let Some((cancel, _)) = &self.writing {
            cancel.cancel();
        }
    }
}
impl YamantakaApp {
    pub(in crate::app) fn show_automation_rules(&mut self) {
        self.show_surface(Surface::Automation);
        if matches!(self.surface, Surface::Automation) {
            self.load_automation_rules();
        }
    }
    fn load_automation_rules(&mut self) {
        if let Some(cancel) = self.automation.pending.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        self.automation.error = None;
        match self.runtime.submit(Action::AutomationRules(
            RuleQuery {
                offset: self.automation.offset,
            },
            self.governor.clone(),
            cancel.clone(),
        )) {
            Ok(()) => self.automation.pending = Some(cancel),
            Err(error) => self.automation.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_automation_rules(
        &mut self,
        result: yamantaka_domain::Result<RulePage>,
        cancel: Cancellation,
    ) {
        if !self
            .automation
            .pending
            .as_ref()
            .is_some_and(|pending| pending.same_request(&cancel))
        {
            return;
        }
        self.automation.pending = None;
        match result {
            Ok(page)
                if self.automation.offset > 0 && self.automation.offset as u64 >= page.total =>
            {
                self.automation.offset =
                    usize::try_from(page.total.saturating_sub(1) / 80 * 80).unwrap_or(0);
                self.load_automation_rules();
            }
            Ok(page) => self.automation.page = Some(Arc::new(page)),
            Err(error) => {
                if matches!(error, yamantaka_domain::Failure::MemoryBudget) {
                    self.automation.page = None;
                    self.automation.deleting = None;
                }
                self.automation.error = Some(error.to_string());
            }
        }
    }
    fn change_automation_rule(&mut self, change: RuleChange) {
        if self.automation.writing.is_some() || self.automation.simulation.busy() {
            return;
        }
        if let Some(cancel) = self.automation.pending.take() {
            cancel.cancel();
        }
        let creating = matches!(change, RuleChange::Create(_));
        let cancel = Cancellation::default();
        self.automation.saved = false;
        self.automation.error = None;
        match self.runtime.submit(Action::ChangeAutomationRule(
            Box::new(change),
            cancel.clone(),
        )) {
            Ok(()) => self.automation.writing = Some((cancel, creating)),
            Err(error) => self.automation.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_automation_change(
        &mut self,
        result: yamantaka_domain::Result<String>,
        cancel: Cancellation,
    ) {
        if !self
            .automation
            .writing
            .as_ref()
            .is_some_and(|(pending, _)| pending.same_request(&cancel))
        {
            return;
        }
        let (_, creating) = self.automation.writing.take().unwrap();
        match result {
            Ok(_) => {
                self.automation.saved = true;
                self.automation.deleting = None;
                if creating {
                    self.automation.editor = None;
                    self.automation.offset = 0;
                }
                if matches!(self.surface, Surface::Automation) {
                    self.load_automation_rules();
                }
            }
            Err(error) => {
                self.automation.error = Some(error.to_string());
                self.error = Some(error.to_string());
            }
        }
    }
}
