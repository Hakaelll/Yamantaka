use super::*;

impl YamantakaApp {
    pub(super) fn advance_cover_picker_source_change(&mut self, action: &str) {
        match self.cover_picker_demo.stage {
            0 => {
                if let Some(document) = self.documents.first().cloned() {
                    self.begin_cover_picker(document);
                    self.cover_picker_demo.stage = 1;
                }
            }
            1 if self
                .cover_picker
                .as_ref()
                .is_some_and(|picker| picker.loaded) =>
            {
                self.cover_picker.as_mut().unwrap().page = 1;
                self.preview_cover_picker();
                self.cover_picker_demo.stage = 2;
            }
            2 if self
                .cover_picker
                .as_ref()
                .is_some_and(|picker| picker.ready()) =>
            {
                let path = self.cover_picker.as_ref().unwrap().document.path.clone();
                let replacement =
                    crate::qa_ui_root().join("../../build/rust-fixtures/covers/cover-1.pdf");
                self.cover_picker_demo.source_worker = Some(std::thread::spawn(move || {
                    let modified = path.metadata()?.modified()?;
                    std::fs::copy(replacement, &path)?;
                    std::fs::OpenOptions::new()
                        .write(true)
                        .open(path)?
                        .set_times(std::fs::FileTimes::new().set_modified(modified))?;
                    Ok(())
                }));
                self.cover_picker_demo.stage = 5;
            }
            5 if self
                .cover_picker_demo
                .source_worker
                .as_ref()
                .is_some_and(std::thread::JoinHandle::is_finished) =>
            {
                self.cover_picker_demo
                    .source_worker
                    .take()
                    .unwrap()
                    .join()
                    .unwrap()
                    .unwrap();
                self.save_cover_picker();
                self.cover_picker_demo.stage = 3;
            }
            3 if self.cover_picker_demo.saved => {
                assert!(action == "stale-source" || self.cover_picker_demo.source_rejected);
                self.cover_picker_demo.stage = 4;
            }
            3 if self
                .cover_picker
                .as_ref()
                .is_some_and(|picker| picker.saving.is_none() && picker.error.is_some()) =>
            {
                assert_ne!(
                    action, "stale-source",
                    "The baseline no longer accepts a stale source"
                );
                assert!(matches!(
                    self.cover_picker.as_ref().unwrap().error,
                    Some(yamantaka_domain::Failure::CoverSourceChanged)
                ));
                self.cover_picker_demo.source_rejected = true;
                if action == "refreshed-source" {
                    self.preview_cover_picker();
                    self.cover_picker_demo.stage = 6;
                } else {
                    self.cover_picker_demo.stage = 4;
                }
            }
            6 if self
                .cover_picker
                .as_ref()
                .is_some_and(|picker| picker.ready()) =>
            {
                self.save_cover_picker();
                self.cover_picker_demo.stage = 3;
            }
            _ => {}
        }
    }
}
