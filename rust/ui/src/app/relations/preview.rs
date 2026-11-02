use super::*;
impl YamantakaApp {
    pub fn preview_relations(&mut self, review: bool, locations: u8) {
        if self.demo && self.capture.is_some() {
            self.relations.preview = 1;
            self.relations.preview_locations = locations;
            self.relations.preview_review = review;
        }
    }
    pub(in crate::app) fn advance_relations_preview(&mut self) {
        let phase = self.relations.preview;
        if phase == 0
            || phase == 10
            || self.relations.pending.is_some()
            || self.relations.writing.is_some()
            || self.relations.context_lookup.is_some()
        {
            return;
        }
        assert!(self.relations.error.is_none(), "{:?}", self.relations.error);
        if phase == 1 {
            let Some(document) = self
                .documents
                .iter()
                .find(|doc| doc.id == "demo-0")
                .cloned()
            else {
                return;
            };
            self.show_relations(document);
            self.relations.preview = 2;
            return;
        }
        if phase == 11 {
            if !self.reader.loading
                && self
                    .reader
                    .active
                    .as_ref()
                    .is_some_and(|(document, _)| document.id == "demo-1")
            {
                assert_eq!(self.reader.page, 1);
                self.relations.preview = 10;
            }
            return;
        }
        let Some(page) = self.relations.page.as_ref() else {
            return;
        };
        match phase {
            2..=5 => {
                assert_eq!(page.total, u64::from(phase - 1));
                self.change_relation(RelationChange::Create {
                    source: "demo-0".into(),
                    target: "demo-1".into(),
                    kind: RelationKind::ALL[usize::from(phase - 2)],
                    source_page: (self.relations.preview_locations > 0).then_some(0),
                    target_page: (self.relations.preview_locations > 0).then_some(1),
                });
                self.relations.preview += 1;
            }
            6 => {
                assert_eq!(page.total, 5);
                let legacy = page
                    .entries
                    .iter()
                    .find(|entry| entry.relation.id == "legacy-relation")
                    .unwrap();
                assert!(legacy.backlink);
                assert_eq!(
                    legacy.relation.source_anchor_id.as_deref(),
                    Some("legacy-annotation")
                );
                assert_eq!(legacy.relation.target_anchor_id.as_deref(), Some("1"));
                self.change_relation(RelationChange::Create {
                    source: "demo-0".into(),
                    target: "demo-1".into(),
                    kind: RelationKind::Related,
                    source_page: None,
                    target_page: None,
                });
                self.relations.preview = 7;
            }
            7 => {
                assert_eq!(page.total, 5);
                if self.relations.preview_locations > 0 {
                    let entry = page
                        .entries
                        .iter()
                        .find(|entry| entry.relation.kind == RelationKind::Related)
                        .unwrap()
                        .clone();
                    assert_eq!(entry.destination, RelationDestination::Page(1));
                    assert_eq!(entry.relation.source_anchor_id.as_deref(), Some("0"));
                    if self.relations.preview_locations >= 3 {
                        match self.relations.preview_locations {
                            3 => {
                                self.edit_relation_location(entry.relation.clone(), "demo-0");
                                self.relations.editing.as_mut().unwrap().replace = true;
                                self.relations.preview = 10;
                            }
                            4 => {
                                self.change_relation(RelationChange::UpdateLocation {
                                    document: "demo-0".into(),
                                    expected: entry.relation.clone(),
                                    page: Some(1),
                                });
                                self.relations.preview = 12;
                            }
                            5 => {
                                self.load_relation_context("demo-1".into());
                                self.relations.preview = 13;
                            }
                            _ => {
                                self.edit_relation_location(entry.relation.clone(), "demo-0");
                                self.relations.editing = None;
                                self.relations.preview = 10;
                            }
                        }
                    } else if self.relations.preview_locations == 2 {
                        self.open_relation(&entry);
                        self.relations.preview = 11;
                    } else {
                        self.relations.target = self
                            .documents
                            .iter()
                            .find(|doc| doc.id == "demo-1")
                            .cloned();
                        self.relations.source_page = Some(0);
                        self.relations.target_page = Some(1);
                        self.relations.preview = 10;
                    }
                    return;
                }
                let expected = page
                    .entries
                    .iter()
                    .find(|entry| entry.relation.kind == RelationKind::Related)
                    .unwrap()
                    .relation
                    .clone();
                if self.relations.preview_review {
                    self.relations.removing = Some(expected);
                    self.relations.preview = 10;
                } else {
                    self.change_relation(RelationChange::Remove {
                        document: "demo-0".into(),
                        expected,
                    });
                    self.relations.preview = 8;
                }
            }
            8 => {
                assert_eq!(page.total, 4);
                self.load_relation_context("demo-1".into());
                self.relations.preview = 9;
            }
            9 => {
                assert_eq!(self.relations.context.as_ref().unwrap().id, "demo-1");
                assert_eq!(page.total, 3);
                assert!(
                    page.entries
                        .iter()
                        .all(|entry| entry.backlink && entry.other_document == "demo-0")
                );
                assert!(
                    page.entries
                        .iter()
                        .any(|entry| entry.relation.kind == RelationKind::Replaces)
                );
                self.relations.preview = 10;
            }
            12 => {
                let entry = page
                    .entries
                    .iter()
                    .find(|entry| entry.relation.kind == RelationKind::Related)
                    .unwrap();
                assert_eq!(entry.relation.source_anchor_id.as_deref(), Some("1"));
                assert_eq!(entry.relation.target_anchor_id.as_deref(), Some("1"));
                self.relations.preview = 10;
            }
            13 => {
                let entry = page
                    .entries
                    .iter()
                    .find(|entry| entry.relation.kind == RelationKind::Related)
                    .unwrap();
                assert!(entry.backlink);
                self.change_relation(RelationChange::UpdateLocation {
                    document: "demo-1".into(),
                    expected: entry.relation.clone(),
                    page: Some(0),
                });
                self.relations.preview = 14;
            }
            14 => {
                let entry = page
                    .entries
                    .iter()
                    .find(|entry| entry.relation.kind == RelationKind::Related)
                    .unwrap();
                assert_eq!(entry.relation.source_anchor_id.as_deref(), Some("0"));
                assert_eq!(entry.relation.target_anchor_id.as_deref(), Some("0"));
                self.relations.preview = 10;
            }
            _ => {}
        }
    }
}
