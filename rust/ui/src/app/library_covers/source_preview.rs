use super::*;
use std::{fs::FileTimes, thread::JoinHandle, time::Instant};

pub(super) struct Preview {
    path: PathBuf,
    replacement: PathBuf,
    expect_stale: bool,
    old: Option<(egui::TextureId, u64, Cancellation)>,
    worker: Option<JoinHandle<std::io::Result<()>>>,
    changed: bool,
    started: Instant,
}
impl Drop for Preview {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
impl YamantakaApp {
    pub fn preview_cover_source_change(
        &mut self,
        path: PathBuf,
        replacement: PathBuf,
        expect_stale: bool,
    ) {
        if self.demo && self.capture.is_some() {
            self.library_covers.source_preview = Some(Preview {
                path,
                replacement,
                expect_stale,
                old: None,
                worker: None,
                changed: false,
                started: Instant::now(),
            });
        }
    }
    pub(in crate::app) fn update_cover_source_preview(&mut self, ctx: &egui::Context) {
        let Some(mut preview) = self.library_covers.source_preview.take() else {
            return;
        };
        if preview.old.is_none()
            && let Some(slot) = self.library_covers.slots.get("demo-0")
            && slot.finished
            && let Some(image) = &slot.image
        {
            assert_eq!(slot.source_page, Some(1));
            preview.old = Some((image.id(), slot.generation, slot.cancel.clone()));
            let path = preview.path.clone();
            let replacement = preview.replacement.clone();
            preview.worker = Some(std::thread::spawn(move || {
                let modified = path.metadata()?.modified()?;
                std::fs::copy(replacement, &path)?;
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(path)?
                    .set_times(FileTimes::new().set_modified(modified))?;
                Ok(())
            }));
            preview.started = Instant::now();
        }
        if preview.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            preview.worker.take().unwrap().join().unwrap().unwrap();
            preview.changed = true;
        }
        if preview.changed
            && let Some((old, generation, cancel)) = &preview.old
        {
            let replaced = self.library_covers.slots.get("demo-0").is_some_and(|slot| {
                slot.finished
                    && slot.source_page == Some(1)
                    && slot.image.as_ref().is_some_and(|image| image.id() != *old)
                    && slot.generation > *generation
                    && cancel.is_cancelled()
            });
            if replaced || preview.expect_stale && preview.started.elapsed().as_millis() > 1600 {
                assert_eq!(replaced, !preview.expect_stale);
                self.library_covers.source_verified = Some(serde_json::json!({
                    "fileReplaced":true,"originalModifiedTimeRestored":true,
                    "newImage":replaced,"staleImageObserved":!replaced,"manualPagePreserved":true,
                }));
                return;
            }
            assert!(
                preview.started.elapsed().as_secs() < 20,
                "Changed cover source was not refreshed"
            );
        }
        ctx.request_repaint_after(Duration::from_millis(50));
        self.library_covers.source_preview = Some(preview);
    }
}
