use super::*;

impl YamantakaApp {
    pub fn preview_reference_regions(&mut self) {
        self.reference_region_preview = self.demo;
    }
    pub fn preview_reference_sync(&mut self) {
        if self.demo {
            self.reference_sync_demo = 1;
        }
    }

    pub(in crate::app) fn advance_reference_sync(&mut self) {
        if !self.demo {
            return;
        }
        let Some(reference) = &mut self.reference else {
            return;
        };
        match self.reference_sync_demo {
            1 if !reference.busy()
                && reference
                    .pane
                    .viewport
                    .pages
                    .values()
                    .any(|slot| slot.image.is_some()) =>
            {
                reference.follow_page = true;
                reference.follow_zoom = true;
                reference.follow_position = true;
                self.set_reader_zoom(1.6);
                self.navigate(1);
                self.reference_sync_demo = 2;
            }
            2 if reference.pane.page == 1
                && (reference.pane.viewport.scale - 1.6).abs() < 0.0001
                && !reference.pane.loading =>
            {
                assert!(reference.follow_zoom);
                assert!((self.reader.viewport.scale - 1.6).abs() < 0.0001);
                self.reference_sync_linked = true;
                self.reader.navigation.as_mut().unwrap().update_anchor(
                    yamantaka_domain::reader::ReaderPosition::at(1, [0.5, 0.05]).unwrap(),
                );
                self.reader.viewport.scroll_anchor = true;
                self.reference_sync_demo = 3;
            }
            3 if reference
                .pane
                .navigation
                .as_ref()
                .is_some_and(|nav| (nav.position().normalized_offset() - 0.05).abs() < 0.0001)
                && !reference.pane.loading =>
            {
                assert!(
                    (self
                        .reader
                        .navigation
                        .as_ref()
                        .unwrap()
                        .position()
                        .normalized_offset()
                        - 0.05)
                        .abs()
                        < 0.0001
                );
                self.reference_sync_position = true;
                let generation = reference.pane.viewport.generation;
                let position = self.reader.navigation.as_ref().map(|nav| nav.position());
                reference.follow_primary_position(position);
                reference.follow_primary_position(position);
                assert_eq!(reference.pane.viewport.generation, generation);
                reference.set_zoom(ZoomMode::Custom, 0.8);
                self.reference_sync_demo = 4;
            }
            4 if (reference.pane.viewport.scale - 0.8).abs() < 0.0001
                && !reference.pane.loading =>
            {
                assert!(!reference.follow_zoom);
                assert!((self.reader.viewport.scale - 1.6).abs() < 0.0001);
                assert_eq!(self.reader.page, 1);
                reference.bidirectional = true;
                reference.follow_zoom = true;
                reference.set_zoom(ZoomMode::Custom, 0.9);
                reference.set_page(0);
                self.reference_sync_demo = 5;
            }
            5 if self.reader.page == 0
                && reference.pane.page == 0
                && (self.reader.viewport.scale - 0.9).abs() < 0.0001
                && (reference.pane.viewport.scale - 0.9).abs() < 0.0001
                && !reference.pane.loading =>
            {
                assert!(reference.follow_page && reference.follow_zoom);
                assert!(!reference.follow_position);
                assert!(
                    reference.pending_primary_page.is_none() && !reference.pending_primary_zoom
                );
                reference.set_zoom(ZoomMode::FitPage, reference.pane.zoom);
                self.reference_sync_demo = 6;
            }
            6 if (self.reader.viewport.scale - reference.pane.viewport.scale).abs() < 0.0001
                && !reference.pending_primary_zoom
                && !reference.pane.loading =>
            {
                assert!(reference.follow_zoom);
                self.reference_sync_reverse = true;
                self.set_reader_zoom(1.1);
                self.navigate(1);
                self.reference_sync_demo = 7;
            }
            7 if self.reader.page == 1
                && reference.pane.page == 1
                && (self.reader.viewport.scale - 1.1).abs() < 0.0001
                && (reference.pane.viewport.scale - 1.1).abs() < 0.0001
                && !reference.pane.loading =>
            {
                assert!(
                    reference.pending_primary_page.is_none() && !reference.pending_primary_zoom
                );
                self.reference_sync_demo = 8;
                reference.toggle_difference(1);
                reference.toggle_difference(0);
                let target =
                    reference.adjacent_difference(self.reader.navigation.as_ref().unwrap(), false);
                assert_eq!(target, Some(0));
                reference.difference_target = target;
            }
            8 if self.reader.page == 0 && reference.pane.page == 0 && !reference.pane.loading => {
                let target =
                    reference.adjacent_difference(self.reader.navigation.as_ref().unwrap(), true);
                assert_eq!(target, Some(1));
                reference.difference_target = target;
                self.reference_sync_demo = 9;
            }
            9 if self.reader.page == 1 && reference.pane.page == 1 && !reference.pane.loading => {
                reference.toggle_difference(0);
                assert!(!reference.differences.contains(&0));
                reference.toggle_difference(1);
                assert!(reference.differences.is_empty());
                assert!(
                    reference
                        .adjacent_difference(self.reader.navigation.as_ref().unwrap(), true)
                        .is_none()
                );
                assert!(
                    reference
                        .adjacent_difference(self.reader.navigation.as_ref().unwrap(), false)
                        .is_none()
                );
                reference.toggle_difference(1);
                self.reference_sync_differences = true;
                self.reference_sync_demo = 10;
                reference.sync_preview = self.capture.is_some();
            }
            _ => {}
        }
    }

    pub fn preview_reference_chooser(&mut self) {
        if self.demo {
            self.reference_chooser_demo = 1;
        }
    }

    pub(in crate::app) fn advance_reference_chooser(&mut self) {
        if !self.demo || self.reference_chooser_demo == 0 {
            return;
        }
        let Some(reference) = &mut self.reference else {
            return;
        };
        reference.chooser.preview = true;
        match self.reference_chooser_demo {
            1 if !reference.busy()
                && reference
                    .pane
                    .viewport
                    .pages
                    .values()
                    .any(|slot| slot.image.is_some()) =>
            {
                reference.chooser.schedule(80, Duration::ZERO);
                self.reference_chooser_demo = 2;
            }
            2 if reference.chooser.loaded && reference.chooser.offset == 80 => {
                assert_eq!(reference.chooser.total, 110);
                assert_eq!(reference.chooser.entries.len(), 30);
                self.reference_chooser_paged = reference.chooser.entries.len();
                let index = (0..100)
                    .find(|index| {
                        !self
                            .documents
                            .iter()
                            .any(|doc| doc.id == format!("chooser-{index:03}"))
                    })
                    .expect("Catalogue fixture has more books than the main page");
                reference.chooser.text = format!("Reference catalogue entry {index:03}");
                reference.chooser.schedule(0, Duration::ZERO);
                self.reference_chooser_demo = 3;
            }
            _ => {}
        }
    }

    pub fn preview_reference_password(&mut self, cancel: bool, import: bool) {
        if self.demo {
            self.reference_password_demo = if import {
                5
            } else if cancel {
                4
            } else {
                3
            };
        }
    }
    pub fn preview_reference_book(&mut self, close: bool, swap: bool, find: bool) {
        if self.demo {
            self.reference_demo = 1;
            self.reference_close_demo = close;
            self.reference_swap_demo = swap;
            self.reference_find_demo = find;
        }
    }

    pub fn preview_reference_links(&mut self) {
        self.reference_links_demo = self.demo;
    }

    pub fn preview_reference_thumbnails(&mut self) {
        self.reference_thumbnails_demo = self.demo;
    }

    pub fn preview_reference_outline(&mut self) {
        self.reference_outline_demo = self.demo;
    }

    pub(in crate::app) fn advance_reference_demo(&mut self) {
        if !self.demo {
            return;
        }
        if self.reference_region_preview
            && let Some(reference) = &mut self.reference
            && !reference.busy()
            && reference.pane.reader_preferences.is_some()
        {
            reference.set_zoom(ZoomMode::Custom, 4.0);
            self.reference_region_preview = false;
        }
        if self.advance_reference_text_demo() {
            return;
        }
        if self.advance_reference_bookmarks_demo() {
            return;
        }
        if self.reference_demo == 2
            && let Some(document) = self
                .password
                .as_ref()
                .and_then(|challenge| challenge.reference_document())
                .cloned()
        {
            match self.reference_password_demo {
                3 | 2 => {
                    let correct = self.reference_password_demo == 2;
                    self.reference_password_demo = if correct { 0 } else { 2 };
                    self.password = None;
                    self.retry_reference_password(
                        document,
                        if correct {
                            "fixture-password"
                        } else {
                            "synthetic incorrect credential"
                        }
                        .into(),
                    );
                }
                4 => {
                    self.password = None;
                    self.cancel_reference_password(&document.id);
                    assert_eq!(self.reader.active.as_ref().unwrap().0.id, "demo-0");
                    assert_eq!(self.reader.page, 0);
                    self.reference_demo = 7;
                }
                5 => {
                    let path = self.reader.active.as_ref().unwrap().0.path.clone();
                    self.import(vec![path]);
                    assert!(self.password.is_none());
                    assert!(
                        self.reference
                            .as_ref()
                            .is_some_and(|reference| !reference.busy())
                    );
                    assert_eq!(self.reader.active.as_ref().unwrap().0.id, "demo-0");
                    self.reference_demo = 7;
                }
                _ => {}
            }
        }
        match self.reference_demo {
            1 if self.reference.as_ref().is_some_and(|reference| {
                !reference.busy()
                    && reference
                        .pane
                        .viewport
                        .pages
                        .values()
                        .any(|slot| slot.image.is_some())
            }) =>
            {
                if let Some(document) = self
                    .documents
                    .iter()
                    .find(|document| {
                        self.reader
                            .active
                            .as_ref()
                            .is_some_and(|(primary, _)| primary.id != document.id)
                    })
                    .cloned()
                {
                    self.open_reference(document);
                    self.reference_demo = 2;
                }
            }
            2 if self.reference.as_ref().is_some_and(|reference| {
                !reference.busy()
                    && reference
                        .pane
                        .active
                        .as_ref()
                        .is_some_and(|(doc, _)| doc.id == "demo-1")
                    && reference
                        .pane
                        .viewport
                        .pages
                        .values()
                        .any(|slot| slot.image.is_some())
            }) =>
            {
                if self.capture.is_some() {
                    assert_eq!(self.reader.active.as_ref().unwrap().0.id, "demo-0");
                    assert_eq!(self.reader.page, 0);
                }
                self.reference_demo = 3;
                if self.reference_close_demo {
                    self.request_reference_close();
                    self.reference_demo = 4;
                } else if self.reference_swap_demo {
                    let reference = self.reference.as_mut().unwrap();
                    reference.set_zoom(ZoomMode::Custom, 1.2);
                    reference.swap_positions();
                    self.reference_demo = 8;
                } else if self.reference_links_demo {
                    self.reference.as_mut().unwrap().pane.links.preview =
                        links::LinkPreview::Activate;
                    self.reference_demo = 12;
                } else if self.reference_text_demo {
                    let reference = self.reference.as_mut().unwrap();
                    reference.text_preview = true;
                    reference.set_page(0);
                    self.reference_demo = 36;
                } else if self.reference_bookmarks_demo {
                    let reference = self.reference.as_mut().unwrap();
                    reference.bookmarks_preview = true;
                    reference.set_page(1);
                    self.reader.bookmarks.prepare(&self.runtime, "demo-0");
                    self.reference_demo = 20;
                } else if self.reference_thumbnails_demo {
                    self.reference.as_mut().unwrap().thumbnails_preview = true;
                    self.reference_demo = 17;
                } else if self.reference_outline_demo {
                    self.reference.as_mut().unwrap().outline_preview = true;
                    self.reference_demo = 14;
                }
            }
            8 if self
                .reference
                .as_ref()
                .is_some_and(|reference| reference.swapped && !reference.pane.loading) =>
            {
                let reference = self.reference.as_ref().unwrap();
                assert_eq!(reference.pane.active.as_ref().unwrap().0.id, "demo-1");
                assert_eq!(reference.pane.page, 0);
                assert!((reference.pane.viewport.scale - 1.2).abs() < 0.0001);
                assert_eq!(self.reader.active.as_ref().unwrap().0.id, "demo-0");
                assert_eq!(self.reader.page, 0);
                assert_eq!(self.reader.zoom, 1.0);
                assert_eq!(self.reference_native_closes, 0);
                self.reference_demo = 9;
                if self.reference_find_demo {
                    self.reference
                        .as_mut()
                        .unwrap()
                        .preview_reference_find(&self.runtime);
                    self.reference_demo = 10;
                }
            }
            10 if self.reference.as_ref().is_some_and(|reference| {
                reference.pane.page == 1 && !reference.pane.loading && reference.pane.find.ready()
            }) =>
            {
                let summary = self.reference.as_ref().unwrap().pane.find.capture_summary();
                assert_eq!(summary["matches"], 1);
                assert_eq!(summary["page"], 1);
                assert_eq!(self.reader.page, 0);
                assert!(self.reader.find.capture_summary().is_null());
                assert_eq!(self.reader.active.as_ref().unwrap().0.id, "demo-0");
                self.reference_demo = 11;
            }
            12 if self.reference.as_ref().is_some_and(|reference| {
                reference.pane.links.preview == links::LinkPreview::Complete
                    && !reference.pane.loading
                    && (reference.pane.page == 1 || reference.pane.links.confirmation_open())
            }) =>
            {
                assert_eq!(self.reader.page, 0);
                assert_eq!(self.reader.active.as_ref().unwrap().0.id, "demo-0");
                assert!(!self.reader.links.confirmation_open());
                assert_eq!(
                    self.reference
                        .as_ref()
                        .unwrap()
                        .pane
                        .active
                        .as_ref()
                        .unwrap()
                        .0
                        .id,
                    "demo-1"
                );
                self.reference_demo = 13;
            }
            14 if self
                .reference
                .as_ref()
                .is_some_and(|reference| reference.pane.outline.ready()) =>
            {
                let reference = self.reference.as_mut().unwrap();
                assert_eq!(reference.pane.outline.capture_summary()["rows"], 6);
                reference.pane.outline.toggle(2);
                assert_eq!(reference.pane.outline.capture_summary()["rows"], 5);
                reference.pane.outline.toggle(2);
                assert_eq!(reference.pane.outline.capture_summary()["rows"], 6);
                let destination = reference.pane.outline.destination(3).unwrap();
                assert_eq!(destination.page, 1);
                reference.pane.reader_destination = Some(("demo-1".into(), destination));
                self.reference_demo = 15;
            }
            15 if self
                .reference
                .as_ref()
                .is_some_and(|reference| reference.pane.page == 1 && !reference.pane.loading) =>
            {
                let reference = self.reference.as_ref().unwrap();
                assert!((reference.pane.viewport.scale - 1.5).abs() < 0.0001);
                assert_eq!(self.reader.page, 0);
                assert_eq!(self.reader.outline.capture_summary()["entries"], 0);
                self.reference_demo = 16;
            }
            17 if self
                .reference
                .as_ref()
                .is_some_and(|reference| reference.pane.thumbnails.ready()) =>
            {
                let reference = self.reference.as_mut().unwrap();
                assert!((1..=16).contains(&reference.pane.thumbnails.resident_count()));
                assert_eq!(self.reader.thumbnails.resident_count(), 0);
                reference.stop_navigator_requests();
                assert_eq!(reference.pane.thumbnails.resident_count(), 0);
                reference.set_page(1);
                self.reference_demo = 18;
            }
            18 if self.reference.as_ref().is_some_and(|reference| {
                reference.pane.page == 1
                    && !reference.pane.loading
                    && reference.pane.thumbnails.ready()
            }) =>
            {
                assert_eq!(self.reader.page, 0);
                assert_eq!(self.reader.thumbnails.resident_count(), 0);
                self.reference_demo = 19;
            }
            4 if self.reference.is_none() => {
                if self.capture.is_some() {
                    assert_eq!(self.reference_native_closes, 1);
                    assert_eq!(self.reader.active.as_ref().unwrap().0.id, "demo-0");
                    assert_eq!(self.reader.page, 0);
                }
                self.reference_demo = 5;
            }
            _ => {}
        }
    }
}
