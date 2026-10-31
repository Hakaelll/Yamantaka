use super::*;
use yamantaka_application::automation_execution::ExecutionCommand;
use yamantaka_domain::automation::execution::ExecutionStatus;
impl YamantakaApp {
    pub(super) fn advance_import_rule_preview(&mut self) {
        let phase = self.automation.simulation.preview;
        let mode = self.automation.simulation.preview_mode;
        if matches!(phase, 0 | 10)
            || self.importing.busy()
            || self.automation.busy()
            || !self.library_loaded
        {
            return;
        }
        assert!(
            self.automation.error.is_none(),
            "{:?}",
            self.automation.error
        );
        if phase == 1 {
            if mode != 14 {
                let Some(batch) = &self.importing.last_batch else {
                    return;
                };
                assert_eq!(batch.len(), 3);
                assert!(batch.ids().all(|id| !id.starts_with("demo-")));
            }
            self.show_automation_rules();
            self.automation.simulation.preview = 2;
        } else if phase == 2 {
            let Some(page) = &self.automation.page else {
                return;
            };
            let rule = page
                .entries
                .iter()
                .find(|rule| rule.revision.id == "legacy-rule")
                .unwrap()
                .clone();
            self.simulate_automation_rule(rule);
            self.automation.simulation.preview = 3;
        } else if phase == 3 {
            let Some(SimulationOutcome::Complete(report)) = &self.automation.simulation.outcome
            else {
                return;
            };
            let count = if mode == 14 { 0 } else { 3 };
            assert_eq!(report.progress.scanned, count);
            assert_eq!(report.progress.matched, count);
            assert!(
                report
                    .matches
                    .iter()
                    .all(|entry| !entry.document.starts_with("demo-"))
            );
            if mode == 11 || mode == 14 {
                self.automation.simulation.preview = 10;
            } else {
                self.prepare_rule_review(report.clone());
                self.automation.simulation.preview = 4;
            }
        } else if phase == 4 {
            assert_eq!(self.automation.review.summary()["documents"], 3);
            self.execute_reviewed_rule();
            self.automation.simulation.preview = 5;
        } else if matches!(phase, 5 | 6) {
            let receipt = self.automation.execution.receipt().unwrap();
            assert_eq!(
                receipt.status,
                if phase == 5 {
                    ExecutionStatus::Completed
                } else {
                    ExecutionStatus::RolledBack
                }
            );
            let batch = self.importing.last_batch.as_ref().unwrap();
            assert_eq!(
                self.documents
                    .iter()
                    .filter(|doc| batch.contains(&doc.id))
                    .count(),
                3
            );
            assert!(
                self.documents
                    .iter()
                    .filter(|doc| batch.contains(&doc.id))
                    .all(|doc| doc.favorite == (phase == 5))
            );
            if phase == 5 && mode == 13 {
                self.start_rule_execution(ExecutionCommand::Undo(receipt.revision.clone()));
                self.automation.simulation.preview = 6;
            } else {
                self.automation.simulation.preview = 10;
            }
        }
    }
}
