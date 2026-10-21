use super::*;

impl YamantakaApp {
    fn finish_tag_management_preview(&mut self) {
        let values: Vec<_> = self
            .library_tags
            .filters
            .iter()
            .map(|(id, name)| serde_json::json!({"id":id,"name":name}))
            .collect();
        let mut saved = self.preferences.clone();
        saved
            .rust
            .insert("libraryTags".into(), serde_json::json!(values));
        let path = self.store.join("tag-management-preview.preferences.json");
        saved.save(&path).unwrap();
        let restored = TagState::new(&Preferences::load(&path).unwrap());
        assert_eq!(restored.filters, self.library_tags.filters);
        self.library_tags.management_verified = true;
        self.library_tags.preview = None;
    }
    pub(super) fn advance_tag_management_preview(&mut self, step: u8, mode: u8) {
        if self
            .library_tags
            .merge
            .as_ref()
            .is_some_and(|picker| picker.pending.is_some())
            || !self.library_tags.loaded
        {
            return;
        }
        let source = self
            .library_tags
            .entries
            .iter()
            .find(|tag| tag.name == "Reading notes")
            .cloned();
        if step == 6 {
            let source = source.unwrap();
            assert_eq!(
                self.library_tags
                    .filters
                    .iter()
                    .map(|(_, name)| name.as_str())
                    .collect::<Vec<_>>(),
                ["Reading notes", "Reference"]
            );
            assert_eq!(source.annotation_count, 1);
            match mode {
                2 => self.finish_tag_management_preview(),
                9 => {
                    self.begin_tag_rename(source.clone());
                    if let Some(TagEditor::Rename { value, .. }) = &mut self.library_tags.editing {
                        *value = "Rejected name".into();
                    }
                    self.change_tag(TagChange::Rename {
                        expected: (&source).into(),
                        name: "Rejected name".into(),
                    });
                    self.library_tags.preview = Some((12, mode));
                }
                10 => {
                    self.begin_tag_delete(source);
                    self.library_tags.review = None;
                    self.finish_tag_management_preview();
                }
                11 => {
                    self.library_tags.preview_menu = true;
                    self.finish_tag_management_preview();
                }
                12 => {
                    self.begin_tag_rename(source);
                    self.finish_tag_management_preview();
                }
                _ => {
                    self.change_tag(TagChange::Color {
                        expected: (&source).into(),
                        color: "#AABBCC".into(),
                    });
                    self.library_tags.preview = Some((7, mode));
                }
            }
            return;
        }
        if step == 7 {
            let source = source.unwrap();
            assert_eq!(source.color, "#AABBCC");
            match mode {
                3 => {
                    self.begin_tag_color(source);
                    self.finish_tag_management_preview();
                }
                7 | 8 => {
                    self.begin_tag_delete(source);
                    if mode == 8 {
                        assert!(self.approve_tag_review());
                        self.library_tags.preview = Some((10, mode));
                    } else {
                        self.finish_tag_management_preview();
                    }
                }
                _ => {
                    self.begin_tag_merge(source);
                    if mode == 14 {
                        let mut picker = self.library_tags.merge.take().unwrap();
                        picker.query.offset = 80;
                        self.load_tag_merge_targets(&mut picker);
                        self.library_tags.merge = Some(picker);
                    }
                    self.library_tags.preview = Some((9, mode));
                }
            }
            return;
        }
        if step == 9 {
            let picker = self.library_tags.merge.as_ref().unwrap();
            if !picker.loaded || picker.failed {
                return;
            }
            if mode == 14 {
                assert_eq!(
                    (picker.total, picker.query.offset, picker.entries.len()),
                    (82, 80, 2)
                );
                self.finish_tag_management_preview();
            } else if mode == 4 {
                assert_eq!(picker.total, 2);
                self.finish_tag_management_preview();
            } else {
                let target = picker
                    .entries
                    .iter()
                    .find(|tag| tag.name == "Reference")
                    .cloned()
                    .unwrap();
                self.review_tag_merge(target);
                if mode == 5 {
                    self.finish_tag_management_preview();
                } else {
                    assert!(self.approve_tag_review());
                    self.library_tags.preview = Some((10, mode));
                }
            }
            return;
        }
        if step == 10 {
            assert!(source.is_none());
            assert_eq!(
                self.library_tags
                    .filters
                    .iter()
                    .map(|(_, name)| name.as_str())
                    .collect::<Vec<_>>(),
                ["Reference"]
            );
            let target = self
                .library_tags
                .entries
                .iter()
                .find(|tag| tag.name == "Reference")
                .unwrap();
            assert_eq!(
                (target.document_count, target.annotation_count),
                if mode == 6 { (2, 1) } else { (1, 0) }
            );
            self.finish_tag_management_preview();
        }
        if step == 12 {
            assert!(self.library_tags.write_failed);
            assert_eq!(source.unwrap().annotation_count, 1);
            assert_eq!(
                self.library_tags
                    .filters
                    .iter()
                    .map(|(_, name)| name.as_str())
                    .collect::<Vec<_>>(),
                ["Reading notes", "Reference"]
            );
            self.finish_tag_management_preview();
        }
    }
}
