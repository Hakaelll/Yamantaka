use super::*;
impl YamantakaApp {
    pub fn preview_revision_groups(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.relations.revisions.preview = 1;
            self.relations.revisions.preview_mode = mode;
        }
    }
    pub(in crate::app) fn advance_revision_preview(&mut self) {
        let phase = self.relations.revisions.preview;
        if matches!(phase, 0 | 10) || self.relations.revisions.busy() {
            return;
        }
        assert!(
            self.relations.revisions.error.is_none(),
            "{:?}",
            self.relations.revisions.error
        );
        if phase == 1 {
            let Some(document) = self
                .documents
                .iter()
                .find(|doc| doc.id == "demo-0")
                .cloned()
            else {
                return;
            };
            self.relations.section = RelationSection::Revisions;
            self.show_relations(document);
            self.relations.revisions.preview = 2;
            return;
        }
        let Some(groups) = self.relations.revisions.groups.as_ref() else {
            return;
        };
        match phase {
            2 => {
                assert_eq!(groups.total, 1);
                let group = groups.entries[0].clone();
                assert_eq!(group.id, "legacy-revisions");
                assert_eq!(group.member_count, 2);
                assert_eq!(group.current.as_deref(), Some("demo-1"));
                self.select_revision_group(group);
                self.relations.revisions.preview = 3;
            }
            3 => {
                let Some(members) = self.relations.revisions.members.as_ref() else {
                    return;
                };
                assert_eq!(members.total, 2);
                assert_eq!(members.current.as_deref(), Some("demo-1"));
                if self.relations.revisions.preview_mode == 0 {
                    self.relations.revisions.preview = 10;
                    return;
                }
                self.change_revision_group(RevisionChange::Create {
                    name: "Study editions".into(),
                    members: vec!["demo-0".into(), "demo-1".into()],
                    current: Some("demo-0".into()),
                });
                self.relations.revisions.preview = 4;
            }
            4 => {
                let Some(members) = self.relations.revisions.members.as_ref() else {
                    return;
                };
                assert_eq!(groups.total, 2);
                assert_eq!(members.total, 2);
                assert_eq!(members.current.as_deref(), Some("demo-0"));
                let group = self.relations.revisions.selected.as_ref().unwrap();
                assert_eq!(group.name, "Study editions");
                assert!(
                    groups
                        .entries
                        .iter()
                        .any(|group| group.id == "legacy-revisions"
                            && group.current.as_deref() == Some("demo-1"))
                );
                if self.relations.revisions.preview_mode == 1 {
                    self.relations.revisions.preview = 10;
                    return;
                }
                self.change_revision_group(RevisionChange::SetCurrent {
                    expected: group.into(),
                    document: "demo-1".into(),
                });
                self.relations.revisions.preview = 5;
            }
            5 => {
                let Some(members) = self.relations.revisions.members.as_ref() else {
                    return;
                };
                assert_eq!(groups.total, 2);
                assert_eq!(members.current.as_deref(), Some("demo-1"));
                assert_eq!(
                    self.relations
                        .revisions
                        .selected
                        .as_ref()
                        .unwrap()
                        .current
                        .as_deref(),
                    Some("demo-1")
                );
                self.relations.revisions.preview = 10;
            }
            _ => {}
        }
    }
}
