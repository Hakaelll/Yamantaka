use super::*;
impl YamantakaApp {
    pub fn preview_quarantine(&mut self, mode: u8) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        self.quarantine.preview = 1;
        self.quarantine.preview_mode = mode;
        self.show_quarantine();
    }
    pub(in crate::app) fn advance_quarantine_preview(&mut self) {
        if !self.demo
            || self.capture.is_none()
            || matches!(self.quarantine.preview, 0 | 90)
            || self.quarantine.pending.is_some()
            || self.quarantine.health_pending > 0
        {
            return;
        }
        let mode = self.quarantine.preview_mode;
        match self.quarantine.preview {
            1 if self.quarantine.page.is_some() => {
                let page = self.quarantine.page.as_ref().unwrap();
                assert_eq!(page.total, 2);
                let entry = page
                    .entries
                    .iter()
                    .find(|entry| entry.document.id == "demo-0")
                    .unwrap();
                match mode {
                    1 => {
                        self.quarantine.release = Some(entry.revision.clone());
                        self.quarantine.preview = 90;
                    }
                    2 => {
                        let revision = entry.revision.clone();
                        self.query_quarantine(Operation::Release(revision), 0);
                        self.quarantine.preview = 2;
                    }
                    3 => {
                        let document = entry.document.clone();
                        self.retry_quarantined(document);
                        self.quarantine.preview = 3;
                    }
                    4 => {
                        let cancel = self.open_intent.lookup("demo-0".into(), None);
                        self.send(Action::GetRestorableDocument("demo-0".into(), cancel));
                        self.quarantine.preview = 4;
                    }
                    5 => {
                        let cancel = self.open_intent.lookup("demo-2".into(), None);
                        self.send(Action::GetDocument("demo-2".into(), cancel));
                        self.quarantine.close();
                        self.quarantine.preview = 5;
                    }
                    _ => self.quarantine.preview = 90,
                }
            }
            2 if self
                .quarantine
                .page
                .as_ref()
                .is_some_and(|page| page.total == 1) =>
            {
                self.quarantine.preview = 90
            }
            3 if self
                .reader
                .active
                .as_ref()
                .is_some_and(|(document, _)| document.id == "demo-0")
                && !self.opening.busy() =>
            {
                self.show_quarantine();
                self.quarantine.preview = 2;
            }
            4 if self.error.is_some() => {
                assert!(self.reader.active.is_none());
                self.error = None;
                self.quarantine.preview = 90;
            }
            5 if self.error.is_some() && !self.opening.busy() => {
                assert!(self.reader.active.is_none());
                self.error = None;
                self.show_quarantine();
                self.quarantine.preview = 6;
            }
            6 if self
                .quarantine
                .page
                .as_ref()
                .is_some_and(|page| page.total == 3) =>
            {
                assert!(
                    self.quarantine
                        .page
                        .as_ref()
                        .unwrap()
                        .entries
                        .iter()
                        .any(|entry| entry.document.id == "demo-2"
                            && entry.reason == QuarantineReason::PdfOpenFailed)
                );
                assert_eq!(
                    std::fs::read(self.store.join("quarantine-broken.pdf")).unwrap(),
                    b"This is not a valid PDF."
                );
                self.quarantine.preview = 90;
            }
            _ => {}
        }
    }
}
