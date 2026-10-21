use super::*;
impl YamantakaApp {
    pub fn preview_tags(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.library_tags.preview = Some((0, mode));
        }
    }
    pub(in crate::app) fn update_tag_preview(&mut self) {
        let Some((step, mode)) = self.library_tags.preview else {
            return;
        };
        if self.library_tags.writing.is_some()
            || self.library_tags.pending.is_some()
            || self.library_tags.filter_check.is_some()
        {
            return;
        }
        if step >= 6 {
            self.advance_tag_management_preview(step, mode);
            return;
        }
        if self.library_tags.failed || self.library_tags.write_failed {
            self.library_tags.preview = None;
            return;
        }
        if step == 0 {
            let Some(document) = self
                .documents
                .iter()
                .find(|doc| doc.id == "demo-0")
                .cloned()
            else {
                return;
            };
            self.show_tags(Some(document));
            self.change_tag(TagChange::EnsureForDocument {
                name: "Study".into(),
                document: "demo-0".into(),
            });
            self.library_tags.preview = Some((1, mode));
            return;
        }
        if step == 5 {
            self.library_tags.preview = None;
            return;
        }
        if !self.library_tags.loaded {
            return;
        }
        let study = self
            .library_tags
            .entries
            .iter()
            .find(|tag| tag.name == "Study")
            .cloned();
        let reference = self
            .library_tags
            .entries
            .iter()
            .find(|tag| tag.name == "Reference")
            .cloned();
        match step {
            1 if study.is_some() => {
                self.change_tag(TagChange::SetMembership {
                    tag: study.unwrap().id,
                    document: "demo-0".into(),
                    included: true,
                });
                self.library_tags.preview = Some((2, mode));
            }
            2 if study.as_ref().is_some_and(|tag| tag.contains_document) => {
                self.change_tag(TagChange::Ensure {
                    name: "Reference".into(),
                });
                self.library_tags.preview = Some((3, mode));
            }
            3 if reference.is_some() => {
                self.change_tag(TagChange::SetMembership {
                    tag: reference.unwrap().id,
                    document: "demo-1".into(),
                    included: true,
                });
                self.library_tags.preview = Some((4, mode));
            }
            4 if reference
                .as_ref()
                .is_some_and(|tag| tag.document_count == 1) =>
            {
                if !self.library_tags.query.ids.is_empty() {
                    self.library_tags.query.ids.clear();
                    self.load_tags();
                    return;
                }
                self.library_tags.preview = None;
                if mode >= 2 {
                    let source = study.unwrap();
                    let target = reference.unwrap();
                    self.toggle_tag_filter(&source);
                    self.toggle_tag_filter(&target);
                    self.begin_tag_rename(source.clone());
                    self.change_tag(TagChange::Rename {
                        expected: (&source).into(),
                        name: "Reading notes".into(),
                    });
                    self.library_tags.preview = Some((6, mode));
                } else if mode == 1 {
                    self.toggle_tag_filter(&study.unwrap());
                    self.toggle_tag_filter(&reference.unwrap());
                    let mut saved = self.preferences.clone();
                    saved.rust.insert(
                        "libraryTags".into(),
                        serde_json::json!(
                            self.library_tags
                                .filters
                                .iter()
                                .map(
                                    |(id, _)| serde_json::json!({"id":id,"name":"Old saved label"})
                                )
                                .collect::<Vec<_>>()
                        ),
                    );
                    let path = self.store.join("tag-preview.preferences.json");
                    match saved.save(&path).and_then(|()| Preferences::load(&path)) {
                        Ok(preferences) => {
                            let state = TagState::new(&preferences);
                            self.library_tags.filters = state.filters.clone();
                            self.check_tag_filters();
                            self.show_surface(Surface::Library);
                            self.library_tags.preview = Some((5, mode));
                        }
                        Err(_) => self.library_tags.write_failed = true,
                    }
                }
            }
            _ => {}
        }
    }
}
