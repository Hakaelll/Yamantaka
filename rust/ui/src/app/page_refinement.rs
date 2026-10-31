//! A settled visible crop overlays the retained whole-page preview.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub cap: u32,
    pub bytes: usize,
}

pub(super) fn target(
    key: &RenderKey,
    paper: egui::Rect,
    clip: egui::Rect,
    dpr: f32,
    limits: Limits,
) -> Option<RenderKey> {
    if limits.bytes < 4 || !paper.is_finite() || paper.width() <= 0.0 || paper.height() <= 0.0 {
        return None;
    }
    let visible = paper.intersect(clip);
    if visible.width() <= 0.0 || visible.height() <= 0.0 {
        return None;
    }
    let mut canvas = preview_dimensions(paper.size(), dpr, limits.cap, usize::MAX);
    for _ in 0..4 {
        let px = |position: f32, start: f32, length: f32, count: u32| {
            f64::from((position - start) / length).clamp(0.0, 1.0) * f64::from(count)
        };
        let x = (px(visible.left(), paper.left(), paper.width(), canvas[0]).floor() as u32)
            .saturating_sub(2);
        let y = (px(visible.top(), paper.top(), paper.height(), canvas[1]).floor() as u32)
            .saturating_sub(2);
        let right = (px(visible.right(), paper.left(), paper.width(), canvas[0]).ceil() as u32)
            .saturating_add(2)
            .min(canvas[0]);
        let bottom = (px(visible.bottom(), paper.top(), paper.height(), canvas[1]).ceil() as u32)
            .saturating_add(2)
            .min(canvas[1]);
        let pixels = u64::from(right.saturating_sub(x)) * u64::from(bottom.saturating_sub(y));
        if pixels == 0 {
            return None;
        }
        if pixels.saturating_mul(4) <= limits.bytes as u64 {
            if f64::from(canvas[0]) <= f64::from(key.physical_width) * 1.05
                && f64::from(canvas[1]) <= f64::from(key.physical_height) * 1.05
            {
                return None;
            }
            let mut target = key.clone();
            target.physical_width = canvas[0];
            target.physical_height = canvas[1];
            target.region = Some([x, y, right - x, bottom - y]);
            return Some(target);
        }
        let scale = ((limits.bytes as f64 / (pixels as f64 * 4.0)).sqrt() * 0.98).min(0.98);
        canvas = [
            (f64::from(canvas[0]) * scale).floor().max(1.0) as u32,
            (f64::from(canvas[1]) * scale).floor().max(1.0) as u32,
        ];
    }
    None
}

#[derive(Default)]
pub(super) struct State {
    target: Option<(RenderKey, Instant)>,
    requested: Option<RenderKey>,
    cancel: Cancellation,
    image: Option<PageImage>,
    image_key: Option<RenderKey>,
    blocked: bool,
}
impl State {
    pub fn paint(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        desired: Option<RenderKey>,
        paper: egui::Rect,
        rotation: u16,
        base_ready: bool,
    ) {
        if self.target.as_ref().map(|(key, _)| key) != desired.as_ref() {
            self.cancel.cancel();
            self.requested = None;
            self.blocked = false;
            self.target = desired.map(|key| (key, Instant::now()));
        }
        if let Some((key, since)) = &self.target {
            if base_ready
                && !self.blocked
                && self.image_key.as_ref() != Some(key)
                && self.requested.as_ref() != Some(key)
            {
                let due = *since + Duration::from_millis(150);
                if Instant::now() < due {
                    ui.ctx()
                        .request_repaint_after(due.saturating_duration_since(Instant::now()));
                } else {
                    self.cancel = Cancellation::default();
                    if runtime
                        .submit(Action::Render(
                            key.clone(),
                            yamantaka_domain::Priority::Visible,
                            self.cancel.clone(),
                        ))
                        .is_ok()
                    {
                        self.requested = Some(key.clone());
                    } else {
                        self.blocked = true;
                        ui.ctx().request_repaint();
                    }
                }
            }
        } else if base_ready {
            self.image = None;
            self.image_key = None;
        }
        if let (Some(image), Some(key)) = (&self.image, &self.image_key)
            && key.rotation == rotation
            && let Some([x, y, width, height]) = key.region
        {
            let min = paper.min
                + paper.size()
                    * egui::vec2(
                        x as f32 / key.physical_width as f32,
                        y as f32 / key.physical_height as f32,
                    );
            let size = paper.size()
                * egui::vec2(
                    width as f32 / key.physical_width as f32,
                    height as f32 / key.physical_height as f32,
                );
            let mut mesh = egui::Mesh::with_texture(image.id());
            mesh.add_rect_with_uv(
                egui::Rect::from_min_size(min, size),
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().add(egui::Shape::mesh(mesh));
        }
    }
    pub fn accepts(&self, page: &yamantaka_domain::RenderedPage, cancel: &Cancellation) -> bool {
        !cancel.is_cancelled()
            && self.cancel.same_request(cancel)
            && self.requested.as_ref() == Some(&page.key)
    }
    pub fn install(&mut self, page: &yamantaka_domain::RenderedPage, image: PageImage) -> bool {
        let replaced = self.image.is_some();
        self.image = Some(image);
        self.image_key = Some(page.key.clone());
        replaced
    }
    pub fn fail(&mut self, cancel: &Cancellation) -> bool {
        if self.requested.is_none() || !self.cancel.same_request(cancel) {
            return false;
        }
        self.blocked = true;
        self.requested = None;
        true
    }
    pub fn reset(&mut self) {
        self.cancel.cancel();
        self.requested = None;
        self.target = None;
        self.blocked = false;
    }
    pub fn blocked(&self) -> bool {
        self.blocked
    }
    pub fn settled(&self) -> bool {
        self.blocked
            || self
                .target
                .as_ref()
                .is_none_or(|(key, _)| self.image_key.as_ref() == Some(key))
    }
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({"settled":self.settled(),"blocked":self.blocked,"target":self.target.as_ref().map(|(key,_)|serde_json::json!({"canvas":[key.physical_width,key.physical_height],"region":key.region,"generation":key.generation})),"image":self.image_key.as_ref().map(|key|serde_json::json!({"canvas":[key.physical_width,key.physical_height],"region":key.region,"generation":key.generation}))})
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
