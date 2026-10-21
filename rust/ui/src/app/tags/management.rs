use super::*;

pub(super) enum TagEditor {
    Rename { tag: Tag, value: String },
    Color { tag: Tag, value: String },
}
impl TagEditor {
    pub(super) fn tag(&self) -> &Tag {
        match self {
            Self::Rename { tag, .. } | Self::Color { tag, .. } => tag,
        }
    }
    pub(super) fn refresh(&mut self, entries: &[Tag]) {
        let tag = match self {
            Self::Rename { tag, .. } | Self::Color { tag, .. } => tag,
        };
        if let Some(current) = entries.iter().find(|entry| entry.id == tag.id) {
            *tag = current.clone();
        }
    }
}
pub(super) struct TagReview {
    pub(super) source: Tag,
    pub(super) target: Option<Tag>,
    pub(super) focus_cancel: bool,
}
pub(super) struct TagMergePicker {
    pub(super) source: Tag,
    pub(super) query: TagQuery,
    pub(super) entries: Vec<Tag>,
    pub(super) total: u64,
    pub(super) pending: Option<Cancellation>,
    pub(super) loaded: bool,
    pub(super) failed: bool,
}
impl Drop for TagMergePicker {
    fn drop(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
    }
}
impl YamantakaApp {
    pub(in crate::app) fn tag_dialog_open(&self) -> bool {
        self.library_tags.review.is_some() || self.library_tags.merge.is_some()
    }
    pub(super) fn can_change_shared_tags(&mut self) -> bool {
        if self.library_selection.writing()
            || self.library_tags.writing()
            || self.annotations.editing()
            || self.annotations.writing()
            || self.global_notes.writing()
        {
            self.library_tags.write_failed = true;
            self.error=Some(self.tr("Termina la edición de notas o la operación pendiente antes de cambiar etiquetas.","Finish note editing or the pending operation before changing shared tags.").into());
            return false;
        }
        true
    }
    pub(super) fn begin_tag_rename(&mut self, tag: Tag) {
        if self.can_change_shared_tags() {
            self.library_tags.editing = Some(TagEditor::Rename {
                value: tag.name.clone(),
                tag,
            });
        }
    }
    pub(super) fn begin_tag_color(&mut self, tag: Tag) {
        if self.can_change_shared_tags() {
            self.library_tags.editing = Some(TagEditor::Color {
                value: tag.color.clone(),
                tag,
            });
        }
    }
    pub(super) fn begin_tag_delete(&mut self, source: Tag) {
        if self.can_change_shared_tags() {
            self.library_tags.editing = None;
            self.library_tags.review = Some(TagReview {
                source,
                target: None,
                focus_cancel: true,
            });
        }
    }
    pub(super) fn begin_tag_merge(&mut self, source: Tag) {
        if !self.can_change_shared_tags() {
            return;
        }
        self.library_tags.editing = None;
        let mut picker = TagMergePicker {
            source,
            query: TagQuery::default(),
            entries: Vec::new(),
            total: 0,
            pending: None,
            loaded: false,
            failed: false,
        };
        self.load_tag_merge_targets(&mut picker);
        self.library_tags.merge = Some(picker);
    }
    pub(super) fn load_tag_merge_targets(&mut self, picker: &mut TagMergePicker) {
        if let Some(previous) = picker.pending.take() {
            previous.cancel();
        }
        picker.loaded = false;
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::Tags(picker.query.clone(), cancel.clone()))
        {
            Ok(()) => {
                picker.pending = Some(cancel);
                picker.failed = false;
            }
            Err(_) => picker.failed = true,
        }
    }
    pub(super) fn review_tag_merge(&mut self, target: Tag) {
        if let Some(picker) = &self.library_tags.merge
            && picker.source.id != target.id
        {
            self.library_tags.review = Some(TagReview {
                source: picker.source.clone(),
                target: Some(target),
                focus_cancel: true,
            });
        }
    }
    pub(super) fn approve_tag_review(&mut self) -> bool {
        let Some(review) = self.library_tags.review.take() else {
            return false;
        };
        let change = match &review.target {
            Some(target) => TagChange::Merge {
                source: (&review.source).into(),
                target: target.into(),
            },
            None => TagChange::Delete {
                expected: (&review.source).into(),
            },
        };
        if self.change_tag(change) {
            self.library_tags.pending_target = review.target;
            true
        } else {
            self.library_tags.review = Some(review);
            false
        }
    }
    pub(super) fn change_tag(&mut self, change: TagChange) -> bool {
        if !self.can_change_shared_tags() {
            return false;
        }
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::ChangeTag(Box::new(change.clone()), cancel.clone()))
        {
            Ok(()) => {
                self.library_tags.writing = Some((cancel, change));
                self.library_tags.write_failed = false;
                true
            }
            Err(error) => {
                self.library_tags.write_failed = true;
                self.error = Some(error.to_string());
                false
            }
        }
    }
    pub(in crate::app) fn accept_tag_change(
        &mut self,
        result: yamantaka_domain::Result<String>,
        cancel: Cancellation,
    ) {
        if !self
            .library_tags
            .writing
            .as_ref()
            .is_some_and(|(scope, _)| scope.same_request(&cancel))
        {
            return;
        }
        let (_, change) = self.library_tags.writing.take().unwrap();
        let target_name = self.library_tags.pending_target.take().map(|tag| tag.name);
        match result {
            Ok(id) => {
                if let Some(previous) = self.library_tags.filter_check.take() {
                    previous.cancel();
                }
                match &change {
                    TagChange::Rename { expected, name } => {
                        for (current, label) in &mut self.library_tags.filters {
                            if *current == expected.id {
                                *label = name.trim().into();
                            }
                        }
                    }
                    TagChange::Delete { expected } => self
                        .library_tags
                        .filters
                        .retain(|(current, _)| *current != expected.id),
                    TagChange::Merge { source, .. } => {
                        for (current, label) in &mut self.library_tags.filters {
                            if *current == source.id {
                                *current = id.clone();
                                if let Some(name) = &target_name {
                                    *label = name.clone();
                                }
                            }
                        }
                        let mut seen = std::collections::HashSet::new();
                        self.library_tags
                            .filters
                            .retain(|(current, _)| seen.insert(current.clone()));
                    }
                    _ => {}
                }
                self.library_tags.filters_checked = false;
                self.persist_tag_filters();
                self.check_tag_filters();
                if self.library_tags.filters.is_empty() {
                    self.library_tags.filters_checked = true;
                    self.persist_tag_filters();
                }
                if matches!(
                    change,
                    TagChange::Ensure { .. }
                        | TagChange::EnsureForDocument { .. }
                        | TagChange::Rename { .. }
                        | TagChange::Color { .. }
                        | TagChange::Merge { .. }
                ) {
                    self.library_tags.query.ids = vec![id];
                    self.library_tags.query.text.clear();
                    self.library_tags.query.offset = 0;
                } else if let TagChange::Delete { expected } = &change {
                    self.library_tags
                        .query
                        .ids
                        .retain(|current| *current != expected.id);
                }
                if matches!(
                    change,
                    TagChange::Ensure { .. } | TagChange::EnsureForDocument { .. }
                ) {
                    self.library_tags.draft.clear();
                }
                self.library_tags.editing = None;
                self.library_tags.merge = None;
                self.library_tags.review = None;
                self.library_tags.write_failed = false;
                self.annotations.invalidate_records();
                self.global_notes.leave();
                if matches!(self.surface, Surface::Notes) {
                    self.show_global_notes();
                }
                self.search.invalidate_tag_changes();
                self.offset = 0;
                self.refresh();
                if self.search.enabled {
                    self.send(Action::IndexStatistics);
                }
            }
            Err(_) => {
                self.library_tags.write_failed = true;
                self.library_tags.review = None;
            }
        }
        if matches!(self.surface, Surface::Tags) {
            self.load_tags();
        } else {
            self.library_tags.loaded = false;
        }
    }
}
