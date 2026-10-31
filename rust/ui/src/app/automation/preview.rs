use super::*;
impl YamantakaApp {
    pub fn preview_automation_rules(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.automation.preview = 1;
            self.automation.preview_mode = mode;
        }
    }
    pub(in crate::app) fn advance_automation_preview(&mut self) {
        let phase = self.automation.preview;
        if matches!(phase, 0 | 10) || self.automation.busy() {
            return;
        }
        assert!(
            self.automation.error.is_none(),
            "{:?}",
            self.automation.error
        );
        if phase == 1 {
            self.show_automation_rules();
            self.automation.preview = 2;
            return;
        }
        let Some(page) = self.automation.page.as_ref() else {
            return;
        };
        match phase {
            2 => {
                if self.automation.preview_mode == 3 {
                    assert_eq!(page.total, 82);
                    assert_eq!(page.entries.len(), 80);
                    self.automation.offset = 80;
                    self.load_automation_rules();
                    self.automation.preview = 7;
                    return;
                }
                assert_eq!(page.total, 2);
                let inherited = page
                    .entries
                    .iter()
                    .find(|rule| rule.revision.id == "legacy-rule")
                    .unwrap();
                assert_eq!(inherited.definition().unwrap().actions.len(), 8);
                assert!(
                    page.entries
                        .iter()
                        .find(|rule| rule.revision.id == "future-rule")
                        .unwrap()
                        .definition()
                        .is_none()
                );
                if self.automation.preview_mode == 0 {
                    self.automation.preview = 10;
                    return;
                }
                let editor = RuleEditor {
                    name: "Reading PDFs".into(),
                    trigger: TriggerKind::Extension,
                    trigger_value: ".pdf".into(),
                    action: ActionKind::AddTag,
                    action_value: "Study".into(),
                };
                self.change_automation_rule(RuleChange::Create(editor.draft()));
                self.automation.preview = 3;
            }
            3 => {
                assert_eq!(page.total, 3);
                let expected = page
                    .entries
                    .iter()
                    .find(|rule| rule.name == "Reading PDFs")
                    .unwrap()
                    .clone();
                assert!(expected.enabled);
                self.change_automation_rule(RuleChange::SetEnabled {
                    expected,
                    enabled: false,
                });
                self.automation.preview = 4;
            }
            4 => {
                assert_eq!(page.total, 3);
                assert!(
                    !page
                        .entries
                        .iter()
                        .find(|rule| rule.name == "Reading PDFs")
                        .unwrap()
                        .enabled
                );
                let expected = page
                    .entries
                    .iter()
                    .find(|rule| rule.revision.id == "future-rule")
                    .unwrap()
                    .clone();
                self.change_automation_rule(RuleChange::SetEnabled {
                    expected,
                    enabled: false,
                });
                self.automation.preview = 5;
            }
            5 => {
                assert!(
                    !page
                        .entries
                        .iter()
                        .find(|rule| rule.revision.id == "future-rule")
                        .unwrap()
                        .enabled
                );
                assert_eq!(
                    page.entries
                        .iter()
                        .find(|rule| rule.revision.id == "legacy-rule")
                        .unwrap()
                        .definition()
                        .unwrap()
                        .actions
                        .len(),
                    8
                );
                let expected = page
                    .entries
                    .iter()
                    .find(|rule| rule.name == "Reading PDFs")
                    .unwrap()
                    .clone();
                if self.automation.preview_mode == 2 {
                    self.automation.deleting = Some(expected);
                    self.automation.preview = 10;
                } else {
                    self.change_automation_rule(RuleChange::Delete { expected });
                    self.automation.preview = 6;
                }
            }
            6 => {
                assert_eq!(page.total, 2);
                assert!(page.entries.iter().all(|rule| rule.name != "Reading PDFs"));
                self.automation.preview = 10;
            }
            7 => {
                assert_eq!(page.total, 82);
                assert_eq!(page.entries.len(), 2);
                assert_eq!(self.automation.offset, 80);
                self.automation.preview = 10;
            }
            _ => {}
        }
    }
}
