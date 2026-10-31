//! Optional CPU previews share the native cache; visible pages own GPU presentation.
use super::*;
use yamantaka_domain::reader::ReaderPosition;
use yamantaka_domain::reader_layout::{ReaderFrame, ReaderLayout};

pub(super) struct View<'a> {
    pub document: &'a str,
    pub layout: &'a ReaderLayout,
    pub frame: &'a ReaderFrame,
    pub preferences: &'a ReaderPreferences,
    pub dpr: f32,
    pub cap: u32,
    pub bytes: usize,
}
impl View<'_> {
    fn keys(&self, generation: u64) -> Vec<RenderKey> {
        let Some(first) = self.frame.pages.iter().map(|page| page.page).min() else {
            return Vec::new();
        };
        let last = self.frame.pages.iter().map(|page| page.page).max().unwrap();
        let mut keys = Vec::with_capacity(2);
        for page in [
            last.checked_add(1)
                .filter(|page| *page < self.layout.page_count()),
            first.checked_sub(1),
        ]
        .into_iter()
        .flatten()
        {
            let anchor = ReaderPosition::new(page, 0.0).expect("A page-top anchor is normalized");
            let Ok(scroll) = self.layout.scroll_for_anchor(
                self.frame.viewport,
                anchor,
                self.preferences.zoom_mode,
                self.frame.zoom,
            ) else {
                continue;
            };
            let Ok(frame) = self.layout.frame(
                self.frame.viewport,
                anchor,
                self.preferences.zoom_mode,
                self.frame.zoom,
                scroll,
                0.0,
            ) else {
                continue;
            };
            for future in &frame.pages {
                if self
                    .frame
                    .pages
                    .iter()
                    .any(|visible| visible.page == future.page)
                    || keys.iter().any(|key: &RenderKey| key.page == future.page)
                {
                    continue;
                }
                let physical = preview_dimensions(
                    egui::vec2(future.rect.width as f32, future.rect.height as f32),
                    self.dpr,
                    self.cap,
                    (self.bytes / frame.pages.len().max(1)).max(4),
                );
                keys.push(render_plan::preview_key(&RenderKey {
                    document_id: self.document.into(),
                    page: future.page,
                    region: None,
                    physical_width: physical[0],
                    physical_height: physical[1],
                    rotation: self.layout.rotation().degrees(),
                    generation,
                }));
                if keys.len() == 2 {
                    return keys;
                }
            }
        }
        keys
    }
}
enum Phase {
    Planned,
    Pending,
    Ready,
    Skipped,
}
struct Slot {
    key: RenderKey,
    cancel: Cancellation,
    phase: Phase,
}
#[derive(Default)]
pub(super) struct State {
    enabled: bool,
    slots: Vec<Slot>,
    since: Option<Instant>,
    was_ready: bool,
}
impl State {
    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled != enabled {
            self.clear();
            self.enabled = enabled;
        }
    }
    pub fn clear(&mut self) {
        for slot in &self.slots {
            slot.cancel.cancel();
        }
        self.slots.clear();
        self.since = None;
        self.was_ready = false;
    }
    pub fn plan(
        &mut self,
        ctx: &egui::Context,
        runtime: &Runtime,
        view: View<'_>,
        generation: u64,
        visible_ready: bool,
    ) {
        if !self.enabled {
            return;
        }
        let keys = view.keys(generation);
        if !self.slots.iter().map(|slot| &slot.key).eq(keys.iter()) {
            self.clear();
            self.slots = keys
                .into_iter()
                .map(|key| Slot {
                    key,
                    cancel: Cancellation::default(),
                    phase: Phase::Planned,
                })
                .collect();
            self.since = Some(Instant::now());
        }
        if self.slots.is_empty() {
            return;
        }
        if !visible_ready {
            self.was_ready = false;
            return;
        }
        if !self.was_ready {
            self.since = Some(Instant::now());
            self.was_ready = true;
        }
        let due = self.since.unwrap() + Duration::from_millis(200);
        if Instant::now() < due {
            ctx.request_repaint_after(due.saturating_duration_since(Instant::now()));
            return;
        }
        for slot in &mut self.slots {
            if matches!(slot.phase, Phase::Planned) {
                slot.phase = if runtime
                    .submit(Action::Prefetch(slot.key.clone(), slot.cancel.clone()))
                    .is_ok()
                {
                    Phase::Pending
                } else {
                    Phase::Skipped
                };
            }
        }
    }
    pub fn accept(
        &mut self,
        key: &RenderKey,
        cancel: &Cancellation,
        result: &yamantaka_domain::Result<()>,
    ) -> bool {
        let Some(slot) = self
            .slots
            .iter_mut()
            .find(|slot| slot.key == *key && slot.cancel.same_request(cancel))
        else {
            return false;
        };
        if cancel.is_cancelled() {
            return false;
        }
        slot.phase = if result.is_ok() {
            Phase::Ready
        } else {
            Phase::Skipped
        };
        true
    }
    pub fn settled(&self) -> bool {
        self.slots
            .iter()
            .all(|slot| matches!(slot.phase, Phase::Ready | Phase::Skipped))
    }
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({"enabled":self.enabled,"settled":self.settled(),"pages":self.slots.iter().map(|slot|serde_json::json!({"page":slot.key.page,"generation":slot.key.generation,"pixels":[slot.key.physical_width,slot.key.physical_height],"state":match slot.phase {Phase::Planned=>"planned",Phase::Pending=>"pending",Phase::Ready=>"ready",Phase::Skipped=>"skipped"}})).collect::<Vec<_>>()})
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.clear();
    }
}
