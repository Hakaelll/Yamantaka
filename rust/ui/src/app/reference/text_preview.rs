use super::*;

impl YamantakaApp {
    pub fn preview_reference_text(&mut self) {
        self.reference_text_demo = self.demo;
    }

    pub(super) fn advance_reference_text_demo(&mut self) -> bool {
        if !(36..=40).contains(&self.reference_demo) {
            return false;
        }
        let ready = self
            .reference
            .as_ref()
            .is_some_and(|reference| reference.pane.text.ready() && !reference.pane.loading);
        match self.reference_demo {
            36 if ready => {
                let reference = self.reference.as_mut().unwrap();
                let page = reference.pane.text.payload().unwrap();
                assert_eq!(page.page(), 0);
                if page.text().trim().is_empty() {
                    assert!(page.characters().is_empty());
                    assert_eq!(self.reader.page, 0);
                    self.reference_demo = 40;
                } else {
                    assert!(page.text().contains("Yamantaka Rust"));
                    reference.pane.text.request(&self.runtime, "demo-1", 1);
                    reference.pane.text.request(&self.runtime, "demo-1", 0);
                    reference.set_page(1);
                    self.reference_demo = 37;
                }
            }
            37 if ready => {
                let reference = self.reference.as_mut().unwrap();
                let page = reference.pane.text.payload().unwrap();
                assert_eq!(page.page(), 1);
                assert!(page.text().contains("Mixed landscape page"));
                reference.stop_navigator_requests();
                assert!(reference.pane.text.payload().is_none());
                assert!(!reference.pane.text.pending());
                self.reference_demo = 38;
            }
            38 if ready => {
                let reference = self.reference.as_ref().unwrap();
                let page = reference.pane.text.payload().unwrap();
                assert_eq!(reference.pane.page, 1);
                assert_eq!(page.page(), 1);
                assert!(page.text().contains("Mixed landscape page"));
                assert_eq!(self.reader.page, 0);
                if self.reader.panel {
                    let primary = self.reader.text.payload().unwrap();
                    assert_eq!(primary.page(), 0);
                    assert!(primary.text().contains("Yamantaka Rust"));
                }
                self.reference_demo = 39;
            }
            _ => {}
        }
        true
    }
}
