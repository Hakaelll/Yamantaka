use super::*;
impl YamantakaApp {
    pub fn preview_rule_simulation(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.automation.simulation.preview = 1;
            self.automation.simulation.preview_mode = mode;
        }
    }
    pub(in crate::app) fn advance_rule_simulation_preview(&mut self) {
        if self.automation.simulation.preview_mode >= 11 {
            self.advance_import_rule_preview();
            return;
        }
        let phase = self.automation.simulation.preview;
        if matches!(phase, 0 | 10) {
            return;
        }
        if phase == 1 {
            self.show_automation_rules();
            self.automation.simulation.preview = 2;
            return;
        }
        if phase == 2 && !self.automation.busy() {
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
            return;
        }
        if phase == 4 && !self.automation.review.busy() {
            let summary = self.automation.review.summary();
            if self.automation.simulation.preview_mode == 6 {
                assert_eq!(summary["failed"], true);
            } else {
                assert_eq!(summary["failed"], false);
                assert_eq!(summary["documents"], 10);
                assert_eq!(summary["actions"], 7);
            }
            if self.automation.simulation.preview_mode >= 7 {
                self.execute_reviewed_rule();
                self.automation.simulation.preview = 5;
                return;
            }
            self.automation.simulation.preview = 10;
            return;
        }
        if matches!(phase, 5 | 6) && !self.automation.execution.busy() && self.library_loaded {
            let receipt = self
                .automation
                .execution
                .receipt()
                .expect("Execution reply missing");
            let mode = self.automation.simulation.preview_mode;
            use yamantaka_domain::automation::execution::ExecutionStatus;
            if phase == 5 {
                if mode == 10 {
                    assert_eq!(receipt.status, ExecutionStatus::Failed);
                    assert!(!receipt.reversible);
                } else {
                    assert_eq!(receipt.status, ExecutionStatus::Completed);
                    assert_eq!(receipt.completed, 10);
                    assert!(
                        self.documents
                            .iter()
                            .all(|document| document.reading_progress == 1.0)
                    );
                    if matches!(mode, 8 | 9) {
                        self.start_rule_execution(
                            yamantaka_application::automation_execution::ExecutionCommand::Undo(
                                receipt.revision.clone(),
                            ),
                        );
                        self.automation.simulation.preview = 6;
                        return;
                    }
                }
            } else {
                assert_eq!(receipt.status, ExecutionStatus::RolledBack);
                assert!(!receipt.reversible);
                assert!(
                    self.documents
                        .iter()
                        .any(|document| document.reading_progress < 1.0)
                );
            }
            self.automation.simulation.preview = 10;
            return;
        }
        if phase == 3 {
            if self.automation.simulation.preview_mode == 2
                && let Some(pending) = &self.automation.simulation.pending
                && pending.watch.snapshot().scanned > 0
            {
                pending.cancel.cancel();
            }
            let Some(outcome) = &self.automation.simulation.outcome else {
                return;
            };
            match outcome {
                SimulationOutcome::Complete(report) => {
                    assert_ne!(self.automation.simulation.preview_mode, 2);
                    if self.automation.simulation.preview_mode == 1 {
                        assert_eq!(report.progress.scanned, 3);
                        assert_eq!(report.progress.matched, 2);
                        assert_eq!(report.progress.unverified, 0);
                        assert_eq!(
                            report
                                .matches
                                .iter()
                                .map(|matched| matched.document.as_str())
                                .collect::<Vec<_>>(),
                            ["demo-0", "demo-1"]
                        );
                    } else {
                        assert_eq!(report.progress.scanned, 10);
                        assert_eq!(report.progress.matched, 10);
                    }
                }
                SimulationOutcome::Cancelled(progress) => {
                    assert_eq!(self.automation.simulation.preview_mode, 2);
                    assert!(progress.scanned > 0);
                }
                SimulationOutcome::Failed { error, .. } => panic!("{error}"),
                SimulationOutcome::LimitReached(_) => {
                    panic!("Synthetic simulation unexpectedly reached its limit")
                }
            }
            if self.automation.simulation.preview_mode >= 4 {
                let SimulationOutcome::Complete(report) = outcome else {
                    panic!("Review requires a complete simulation")
                };
                let report = report.clone();
                self.prepare_rule_review(report);
                self.automation.simulation.preview = 4;
                return;
            }
            self.automation.simulation.preview = 10;
        }
    }
}
