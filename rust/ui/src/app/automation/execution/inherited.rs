use super::*;
impl YamantakaApp {
    pub(in crate::app::automation) fn check_inherited_rule(&mut self, journal: JournalRevision) {
        if self.automation.busy() {
            return;
        }
        let cancel = Cancellation::default();
        self.automation.execution.error = None;
        match self.runtime.submit(Action::InheritedAutomationRule(
            journal,
            self.governor.clone(),
            cancel.clone(),
        )) {
            Ok(()) => self.automation.execution.retrying = Some(cancel),
            Err(error) => self.automation.execution.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_inherited_rule(
        &mut self,
        result: yamantaka_domain::Result<Arc<SavedRule>>,
        cancel: Cancellation,
    ) {
        let state = &mut self.automation.execution;
        if !state
            .retrying
            .as_ref()
            .is_some_and(|pending| pending.same_request(&cancel))
        {
            return;
        }
        state.retrying = None;
        if cancel.is_cancelled() || !matches!(self.surface, Surface::Automation) {
            return;
        }
        match result {
            Ok(rule) => {
                state.history_open = false;
                self.simulate_automation_rule(rule);
            }
            Err(error) => state.error = Some(error.to_string()),
        }
    }
    pub fn preview_inherited_journals(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.automation.execution.inherited_preview = 1;
            self.automation.execution.inherited_mode = mode;
        }
    }
    pub(in crate::app) fn advance_inherited_journal_preview(&mut self) {
        let phase = self.automation.execution.inherited_preview;
        let mode = self.automation.execution.inherited_mode;
        if matches!(phase, 0 | 10) {
            return;
        }
        if phase == 1 {
            self.show_automation_rules();
            self.automation.execution.inherited_preview = 2;
            return;
        }
        if phase == 2 && !self.automation.busy() {
            let state = &mut self.automation.execution;
            let page = state.page.as_ref().unwrap();
            assert_eq!(page.total, if mode == 4 { 82 } else { 3 });
            state.history_open = true;
            if mode == 4 {
                state.offset = 80;
                self.load_rule_journals();
                self.automation.execution.inherited_preview = 4;
                return;
            }
            assert_eq!(page.entries.len(), 3);
            assert!(
                page.entries.iter().all(|entry| !entry.reversible
                    && matches!(entry.origin, JournalOrigin::Flutter { .. }))
            );
            if matches!(mode, 1 | 2) {
                let retry = page
                    .entries
                    .iter()
                    .find(|entry| entry.revision.id == "inherited-0-retry")
                    .unwrap()
                    .revision
                    .clone();
                self.check_inherited_rule(retry);
                self.automation.execution.inherited_preview = 3;
            } else {
                state.inherited_preview = 10;
            }
            return;
        }
        if phase == 3 && !self.automation.busy() {
            if mode == 2 {
                assert!(self.automation.execution.error.is_some());
            } else {
                let summary = self.automation.simulation_summary();
                assert_eq!(summary["state"], "complete");
                assert_eq!(summary["matched"], 10);
            }
            self.automation.execution.inherited_preview = 10;
        }
        if phase == 4 && !self.automation.busy() {
            let page = self.automation.execution.page.as_ref().unwrap();
            assert_eq!(page.total, 82);
            assert_eq!(page.entries.len(), 2);
            self.automation.execution.inherited_preview = 10;
        }
    }
}
