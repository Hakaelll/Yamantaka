use super::*;
impl YamantakaApp {
    pub fn preview_rule_simulation(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.automation.simulation.preview = 1;
            self.automation.simulation.preview_mode = mode;
        }
    }
    pub(in crate::app) fn advance_rule_simulation_preview(&mut self) {
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
