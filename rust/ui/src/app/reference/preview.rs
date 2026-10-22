use super::*;

impl YamantakaApp {
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
                reference.set_zoom(ZoomMode::Custom, 0.8);
                self.reference_sync_demo = 3;
            }
            3 if (reference.pane.viewport.scale - 0.8).abs() < 0.0001
                && !reference.pane.loading =>
            {
                assert!(!reference.follow_zoom);
                assert!((self.reader.viewport.scale - 1.6).abs() < 0.0001);
                assert_eq!(self.reader.page, 1);
                self.reference_sync_demo = 4;
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
    pub fn preview_reference_book(&mut self, close: bool) {
        if self.demo {
            self.reference_demo = 1;
            self.reference_close_demo = close;
        }
    }

    pub(in crate::app) fn advance_reference_demo(&mut self) {
        if !self.demo {
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
                }
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
