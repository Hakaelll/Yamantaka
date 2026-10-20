use super::*;
impl YamantakaApp {
    pub fn preview_collections(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.collections.preview = Some((0, mode));
        }
    }
    pub(in crate::app) fn update_collection_preview(&mut self) {
        let Some((step, mode)) = self.collections.preview else {
            return;
        };
        if self.collections.writing.is_some()
            || self.collections.pending.is_some()
            || self.collections.filter_check.is_some()
        {
            return;
        }
        if self.collections.failed || self.collections.write_failed {
            self.collections.preview = None;
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
            self.show_collections(Some(document));
            self.collections.preview = Some((1, mode));
            self.change_collection(CollectionChange::Create {
                name: "Reading seminar".into(),
                icon: "study".into(),
            });
            return;
        }
        if step == 4 {
            self.collections.preview = None;
            return;
        }
        if !self.collections.loaded {
            return;
        }
        let Some(entry) = self
            .collections
            .entries
            .iter()
            .find(|entry| entry.name == "Reading seminar")
            .cloned()
        else {
            return;
        };
        match step {
            1 => {
                self.change_collection(CollectionChange::SetMembership {
                    expected: (&entry).into(),
                    document: "demo-0".into(),
                    included: true,
                });
                self.collections.preview = Some((2, mode));
            }
            2 if entry.contains_document => {
                self.change_collection(CollectionChange::Pin {
                    expected: (&entry).into(),
                    pinned: true,
                });
                self.collections.preview = Some((3, mode));
            }
            3 if entry.pinned => {
                self.collections.preview = None;
                if mode == 1 {
                    self.apply_collection_filter(&entry);
                    let path = self.store.join("collection-preview.preferences.json");
                    let mut saved = self.preferences.clone();
                    saved.rust.insert(
                        "libraryCollection".into(),
                        serde_json::json!({"id":entry.id,"name":"Old saved label"}),
                    );
                    match saved.save(&path).and_then(|()| Preferences::load(&path)) {
                        Ok(restored) => {
                            let state = CollectionState::new(&restored);
                            self.collections.filter = state.filter.clone();
                            self.check_collection_filter();
                            self.collections.preview = Some((4, mode));
                        }
                        Err(_) => self.collections.write_failed = true,
                    }
                } else if mode == 2 {
                    self.collections.deleting = Some(entry);
                } else if mode >= 3 {
                    self.collections.preview = Some((5, mode));
                    self.change_collection(CollectionChange::Pin {
                        expected: (&entry).into(),
                        pinned: false,
                    });
                }
            }
            5 => {
                self.collections.preview = Some((6, mode));
                self.change_collection(CollectionChange::Create {
                    name: "Archive notes".into(),
                    icon: "archive".into(),
                });
            }
            6 => {
                self.collections.preview = Some((7, mode));
                self.change_collection(CollectionChange::Icon {
                    expected: (&entry).into(),
                    icon: "reference".into(),
                });
            }
            7 if entry.icon == "reference" => {
                self.collections.preview = Some((8, mode));
                self.change_collection(CollectionChange::Move {
                    expected: (&entry).into(),
                    direction: CollectionDirection::Later,
                });
            }
            8 => {
                assert_eq!(
                    self.collections
                        .entries
                        .iter()
                        .map(|entry| entry.name.as_str())
                        .collect::<Vec<_>>(),
                    ["Archive notes", "Reading seminar"]
                );
                assert!(entry.contains_document);
                if mode == 5 {
                    self.collections.query.text = "Reading seminar".into();
                    self.load_collections();
                    self.collections.preview = Some((9, mode));
                } else {
                    self.collections.order_verified = true;
                    self.collections.preview_menu = mode == 6;
                    self.collections.preview = None;
                    if mode == 3 {
                        self.collections.icon_editing = Some(entry.id);
                    }
                }
            }
            9 => {
                assert_eq!(self.collections.entries.len(), 1);
                self.collections.preview = Some((10, mode));
                self.change_collection(CollectionChange::Move {
                    expected: (&entry).into(),
                    direction: CollectionDirection::Earlier,
                });
            }
            10 => {
                self.collections.query.text.clear();
                self.load_collections();
                self.collections.preview = Some((11, mode));
            }
            11 => {
                assert_eq!(
                    self.collections
                        .entries
                        .iter()
                        .map(|entry| entry.name.as_str())
                        .collect::<Vec<_>>(),
                    ["Reading seminar", "Archive notes"]
                );
                assert!(entry.contains_document);
                assert_eq!(entry.icon, "reference");
                self.collections.order_verified = true;
                self.collections.preview = None;
            }
            _ => {}
        }
    }
}
