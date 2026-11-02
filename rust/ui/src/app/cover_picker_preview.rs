use super::*;
mod source_change;

#[derive(Default)]
pub(super) struct CoverPickerDemo {
    pub(super) action: Option<&'static str>,
    stage: u8,
    pub(super) saved: bool,
    source_worker: Option<std::thread::JoinHandle<std::io::Result<()>>>,
    source_rejected: bool,
}
impl Drop for CoverPickerDemo {
    fn drop(&mut self) {
        if let Some(worker) = self.source_worker.take() {
            let _ = worker.join();
        }
    }
}
impl YamantakaApp {
    pub fn preview_cover_editor(&mut self, action: &'static str) {
        if self.demo && self.capture.is_some() {
            self.cover_picker_demo.action = Some(action);
        }
    }
    pub(super) fn advance_cover_picker_preview(&mut self) {
        let Some(action) = self.cover_picker_demo.action else {
            return;
        };
        if action.ends_with("-source") {
            self.advance_cover_picker_source_change(action);
            return;
        }
        match self.cover_picker_demo.stage {
            0 => {
                if let Some(document) = self.documents.first().cloned() {
                    self.begin_cover_picker(document);
                    self.cover_picker_demo.stage = 1;
                }
            }
            1 => {
                if self
                    .cover_picker
                    .as_ref()
                    .is_some_and(|picker| picker.loaded)
                {
                    if let Some(picker) = &mut self.cover_picker {
                        picker.page = picker.document.page_count.saturating_sub(1).min(1);
                    }
                    self.preview_cover_picker();
                    self.cover_picker_demo.stage = 2;
                }
            }
            2 => {
                if self
                    .cover_picker
                    .as_ref()
                    .is_some_and(|picker| picker.ready())
                {
                    match action {
                        "restore" | "failed-restore" => {
                            self.restore_cover_picker();
                            self.cover_picker_demo.stage = 3;
                        }
                        "save" | "failed-save" => {
                            self.save_cover_picker();
                            self.cover_picker_demo.stage = 3;
                        }
                        "cancel" => {
                            self.cover_picker = None;
                            self.runtime.wake();
                            self.cover_picker_demo.stage = 4;
                        }
                        _ => self.cover_picker_demo.stage = 4,
                    }
                }
            }
            3 if self.cover_picker_demo.saved
                || self
                    .cover_picker
                    .as_ref()
                    .is_some_and(|picker| picker.save_failed || picker.restore_failed) =>
            {
                self.cover_picker_demo.stage = 4;
            }
            _ => {}
        }
    }
    pub(super) fn cover_picker_capture_ready(&self) -> bool {
        self.cover_picker_demo.action.is_none() || self.cover_picker_demo.stage == 4
    }
    pub(super) fn cover_picker_demo_summary(&self) -> serde_json::Value {
        serde_json::json!({"action":self.cover_picker_demo.action,"complete":self.cover_picker_demo.stage==4,"saved":self.cover_picker_demo.saved,"sourceRejected":self.cover_picker_demo.source_rejected})
    }
}
