use super::*;
use yamantaka_domain::memory::GlobalMemoryLimits;

pub(super) struct CoverBatchPreview {
    stage: u8,
    review: bool,
    pressure: bool,
    targets: Vec<String>,
    old: Vec<egui::TextureId>,
    limits: Option<GlobalMemoryLimits>,
}
impl YamantakaApp {
    pub fn preview_cover_batch(&mut self, mode: &str) {
        if self.demo && self.capture.is_some() {
            self.library_covers.preview = Some(CoverBatchPreview {
                stage: 0,
                review: mode == "review",
                pressure: mode == "pressure",
                targets: ["demo-0", "demo-1", "demo-2"].map(str::to_owned).to_vec(),
                old: Vec::new(),
                limits: None,
            });
        }
    }
    pub(in crate::app) fn update_cover_batch_preview(&mut self, ctx: &egui::Context) {
        let Some(mut preview) = self.library_covers.preview.take() else {
            return;
        };
        let ready = preview.targets.iter().all(|id| {
            self.library_covers.slots.get(id).is_some_and(|slot| {
                slot.finished && !slot.refresh_requested && slot.image.is_some()
            })
        });
        match preview.stage {
            0 if self.library_loaded && ready => {
                for id in &preview.targets {
                    let slot = &self.library_covers.slots[id];
                    assert_eq!(slot.source_page, Some(1));
                    preview.old.push(slot.image.as_ref().unwrap().id());
                }
                self.library_selection.model.all(&preview.targets);
                if preview.review {
                    self.open_cover_batch_review();
                    self.library_covers.verified = Some(
                        serde_json::json!({"review":true,"targets":preview.targets,"oldImagesKept":true}),
                    );
                    return;
                }
                if preview.pressure {
                    let mut limits = self.governor.limits();
                    preview.limits = Some(limits);
                    limits.gpu = Some(self.governor.usage().gpu);
                    self.governor.set_limits(limits);
                }
                assert!(self.submit_cover_batch(preview.targets.clone()));
                preview.stage = 1;
            }
            1 if preview.pressure
                && !self.library_selection.writing()
                && preview.targets.iter().all(|id| {
                    self.library_covers
                        .slots
                        .get(id)
                        .is_some_and(|slot| slot.deferred.is_some())
                }) =>
            {
                for (id, old) in preview.targets.iter().zip(&preview.old) {
                    assert_eq!(
                        self.library_covers.slots[id].image.as_ref().unwrap().id(),
                        *old
                    );
                    assert_eq!(self.library_covers.slots[id].source_page, Some(1));
                }
                self.governor.set_limits(preview.limits.take().unwrap());
                preview.stage = 2;
                ctx.request_repaint();
            }
            1 | 2
                if !self.library_selection.writing()
                    && ready
                    && (!preview.pressure || preview.stage == 2) =>
            {
                for (id, old) in preview.targets.iter().zip(&preview.old) {
                    let slot = &self.library_covers.slots[id];
                    assert_ne!(slot.image.as_ref().unwrap().id(), *old);
                    assert_eq!(slot.source_page, Some(0));
                }
                assert!(self.library_selection.model.ids().is_empty());
                assert!(self.library_covers.paused.is_empty());
                self.library_covers.verified = Some(
                    serde_json::json!({"review":false,"targets":preview.targets,"manualPageBefore":1,"automaticPageAfter":0,"newImages":true,"pressureRecovery":preview.pressure}),
                );
                return;
            }
            _ => {}
        }
        self.library_covers.preview = Some(preview);
    }
}
