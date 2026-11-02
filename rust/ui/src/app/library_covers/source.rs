use super::*;
use yamantaka_windows::source_monitor::{FileState, Monitor, Target};

impl YamantakaApp {
    pub(super) fn observe_library_sources(&mut self, ctx: &egui::Context) {
        if !self.library_covers.monitor_attempted && !self.library_covers.slots.is_empty() {
            self.library_covers.monitor_attempted = true;
            let notify = ctx.clone();
            match Monitor::start(move || notify.request_repaint()) {
                Ok(monitor) => {
                    self.library_covers.monitor = Some(monitor);
                    self.library_covers.monitor_dirty = true;
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    for slot in self.library_covers.slots.values_mut() {
                        slot.finished = true;
                        slot.refresh_requested = false;
                    }
                }
            }
        }
        let Some(monitor) = &self.library_covers.monitor else {
            return;
        };
        if self.library_covers.monitor_dirty {
            let mut targets: Vec<_> = self
                .library_covers
                .slots
                .iter()
                .map(|(id, slot)| Target {
                    document: id.clone(),
                    path: slot.path.clone(),
                    generation: slot.monitor_generation,
                })
                .collect();
            targets.sort_by(|left, right| left.document.cmp(&right.document));
            if let Err(error) = monitor.set_targets(targets) {
                self.error = Some(error.to_string());
                return;
            }
            self.library_covers.monitor_dirty = false;
        }
        for observation in monitor.take().unwrap_or_default() {
            let Some(slot) = self
                .library_covers
                .slots
                .get_mut(&observation.target.document)
            else {
                continue;
            };
            if slot.monitor_generation != observation.target.generation
                || slot.path != observation.target.path
                || slot.observed.as_ref() == Some(&observation.state)
            {
                continue;
            }
            let changed = slot.observed.is_some();
            slot.observed = Some(observation.state);
            slot.cancel.cancel();
            slot.deferred = None;
            slot.finished = true;
            slot.validation_pending = true;
            slot.refresh_requested = matches!(slot.observed, Some(FileState::Present(_)));
            if changed || !slot.refresh_requested || slot.image.is_some() {
                slot.image = None;
                slot.source_page = None;
                ctx.data_mut(|data| {
                    data.remove::<book::PdfCover>(book::cover_id(&observation.target.document))
                });
            }
            if changed {
                self.library_covers.source_invalidations += 1;
            }
            self.runtime.wake();
            ctx.request_repaint();
        }
    }
}
