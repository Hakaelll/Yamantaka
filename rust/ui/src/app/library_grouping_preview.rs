use super::library_grouping::*;
use super::*;
use std::collections::HashSet;

impl YamantakaApp {
    pub fn preview_grouping(&mut self, mode: &str, fold: bool) {
        if self.demo && self.capture.is_some() {
            self.grouping = GroupingState::from_key(Some(mode));
            self.grouping.preview = Some(fold);
            self.refresh();
        }
    }
    pub(super) fn verify_grouping_preview(&mut self) {
        let Some(fold) = self.grouping.preview.take() else {
            return;
        };
        let groups = self.library_groups();
        assert!(groups.len() >= 2);
        let ids = self.library_visible_ids();
        assert_eq!(ids.len(), self.documents.len());
        assert_eq!(
            ids.iter().collect::<HashSet<_>>().len(),
            self.documents.len()
        );
        for group in &groups {
            let source: Vec<_> = self
                .documents
                .iter()
                .filter(|doc| group.documents.iter().any(|member| member.id == doc.id))
                .map(|doc| &doc.id)
                .collect();
            assert_eq!(
                source,
                group
                    .documents
                    .iter()
                    .map(|doc| &doc.id)
                    .collect::<Vec<_>>()
            );
        }
        self.library_selection
            .model
            .select(&ids, &ids[0], true, false);
        self.library_selection
            .model
            .select(&ids, &ids[ids.len() - 1], false, true);
        assert_eq!(self.library_selection.model.ids(), ids);
        if fold {
            self.selected = Some(groups[0].documents[0].id.clone());
            self.grouping.collapsed.insert(groups[0].key.clone());
            let visible = self.library_visible_ids();
            self.reconcile_group_selection();
            assert!(self.selected.is_none());
            assert!(visible.len() < ids.len());
            assert!(
                self.library_selection
                    .model
                    .ids()
                    .iter()
                    .all(|id| visible.contains(id))
            );
        }
        self.library_selection.model.clear();
        let visible = self.library_visible_ids();
        self.library_selection.columns = 3;
        let rows: Vec<_> = groups
            .iter()
            .filter(|group| !self.grouping.collapsed.contains(&group.key))
            .flat_map(|group| group.documents.chunks(3))
            .collect();
        if rows.len() > 1 {
            let expected = &rows[1][0].id;
            assert_eq!(
                &visible[self.library_vertical_index(&visible, 0, true)],
                expected
            );
        }

        assert_eq!(
            GroupingState::from_key(Some(self.grouping.mode.key()))
                .mode
                .key(),
            self.grouping.mode.key()
        );
        self.grouping.verified = true;
    }
    pub(super) fn grouping_summary(&self) -> serde_json::Value {
        serde_json::json!({"mode":self.grouping.mode.key(),"verified":self.grouping.verified,"visible":self.library_visible_ids(),"groups":self.library_groups().iter().map(|group|serde_json::json!({"label":group.label,"count":group.documents.len(),"collapsed":self.grouping.collapsed.contains(&group.key)})).collect::<Vec<_>>()})
    }
}
