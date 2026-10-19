use super::*;
pub(in crate::app) struct OrganizationPreview {
    mode: String,
    step: u8,
    ids: Vec<String>,
}
impl OrganizationPreview {
    fn kind(&self) -> TargetKind {
        match self.mode.as_str() {
            "tag" | "tag-picker" => TargetKind::Tag,
            "remove" => TargetKind::Collection(false),
            _ => TargetKind::Collection(true),
        }
    }
    fn expected(&self) -> u64 {
        if self.mode == "remove" { 1 } else { 3 }
    }
}
impl YamantakaApp {
    pub fn preview_bulk_organization(&mut self, mode: &str) {
        if self.demo
            && self.capture.is_some()
            && [
                "tag-picker",
                "collection-picker",
                "tag",
                "collection",
                "remove",
                "archive",
            ]
            .contains(&mode)
        {
            self.preview_library_selection(false, false);
            self.bulk_organization_preview = Some(OrganizationPreview {
                mode: mode.into(),
                step: 0,
                ids: Vec::new(),
            });
        }
    }
    pub(in crate::app) fn update_bulk_organization_preview(&mut self) {
        let Some(mut preview) = self.bulk_organization_preview.take() else {
            return;
        };
        if !self.library_loaded
            || !self.library_selection.ready()
            || self
                .bulk_picker
                .as_ref()
                .is_some_and(|picker| picker.pending.is_some())
        {
            self.bulk_organization_preview = Some(preview);
            return;
        }
        match preview.step {
            0 => {
                preview.ids = self.library_selection.model.ids().to_vec();
                assert_eq!(preview.ids.len(), 3);
                if preview.mode == "archive" {
                    assert!(self.submit_library_bulk(BulkAction::Archive, preview.ids.clone()));
                    preview.step = 3;
                } else {
                    self.open_bulk_picker(preview.kind());
                    preview.step = 1;
                }
            }
            1 => {
                let mut picker = self.bulk_picker.take().unwrap();
                assert!(!picker.failed);
                assert_eq!(picker.total, 81);
                assert_eq!(picker.entries.len(), 80);
                picker.offset = 80;
                self.load_bulk_targets(&mut picker);
                self.bulk_picker = Some(picker);
                preview.step = 2;
            }
            2 => {
                let picker = self.bulk_picker.as_ref().unwrap();
                assert!(!picker.failed);
                assert_eq!(picker.entries.len(), 1);
                assert!(picker.entries[0].name.starts_with("Study"));
                if preview.mode.ends_with("-picker") {
                    self.bulk_organization_verified = Some(
                        serde_json::json!({"mode":preview.mode,"targetTotal":picker.total,"targetOffset":picker.offset,"selected":preview.ids.len(),"members":picker.entries[0].documents,"annotationUses":picker.entries[0].annotation_uses}),
                    );
                    return;
                }
                let action = picker.entries[0].action.clone();
                assert!(self.submit_library_bulk(action, preview.ids.clone()));
                self.bulk_picker = None;
                preview.step = 3;
            }
            3 => {
                assert!(self.library_selection.model.ids().is_empty());
                let mut picker = BulkPicker::new(preview.kind(), preview.ids.clone());
                picker.text = if preview.mode == "archive" {
                    "Archivado".into()
                } else {
                    "Study".into()
                };
                self.load_bulk_targets(&mut picker);
                self.bulk_picker = Some(picker);
                preview.step = 4;
            }
            4 => {
                let picker = self.bulk_picker.take().unwrap();
                assert!(!picker.failed);
                assert_eq!(picker.entries.len(), 1);
                let target = &picker.entries[0];
                assert_eq!(target.documents, preview.expected());
                self.bulk_organization_verified = Some(
                    serde_json::json!({"mode":preview.mode,"selected":preview.ids.len(),"members":target.documents,"annotationUses":target.annotation_uses}),
                );
                self.clear_tag_filters();
                self.clear_collection_filter();
                match &target.action {
                    BulkAction::Tag(id) => {
                        assert_eq!(target.annotation_uses, Some(1));
                        self.library_tags.filters = vec![(id.clone(), target.name.clone())];
                    }
                    BulkAction::Collection { expected, .. } => {
                        self.collections.filter = Some((expected.id.clone(), target.name.clone()))
                    }
                    _ => unreachable!(),
                }
                self.offset = 0;
                self.refresh();
                preview.step = 5;
            }
            5 => {
                assert_eq!(self.count, preview.expected());
                let mut actual: Vec<_> = self.documents.iter().map(|doc| doc.id.clone()).collect();
                actual.sort();
                let mut expected = if preview.mode == "remove" {
                    vec!["demo-1".to_owned()]
                } else {
                    preview.ids.clone()
                };
                expected.sort();
                assert_eq!(actual, expected);
                return;
            }
            _ => unreachable!(),
        }
        self.bulk_organization_preview = Some(preview);
    }
}
