use super::*;

impl YamantakaApp {
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
