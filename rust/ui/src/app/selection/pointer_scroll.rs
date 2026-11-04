use super::*;
use yamantaka_domain::reader_layout::ReaderFrame;

impl SelectionState {
    pub(in crate::app) fn drag_viewport(
        &mut self,
        ui: &egui::Ui,
        frame: &ReaderFrame,
        origin: egui::Pos2,
        rotation: PageRotation,
    ) {
        let SelectionGesture::Dragging(anchor, previous) = self.gesture else {
            return;
        };
        let (down, pointer, dt) = ui.input(|input| {
            (
                input.pointer.primary_down(),
                input.pointer.interact_pos(),
                input.stable_dt.min(0.05),
            )
        });
        // Releasing outside a page must finish the gesture too.
        if !down {
            self.gesture = SelectionGesture::Selected(anchor, previous);
            return;
        }
        if !self.enabled || !ui.is_enabled() {
            return;
        }
        let Some(pointer) = pointer else { return };
        let viewport = ui.clip_rect();
        if !viewport.expand(32.0).contains(pointer) {
            return;
        }
        let point = viewport.clamp(pointer);
        let page_rect = |page: &yamantaka_domain::reader_layout::VisiblePage| {
            egui::Rect::from_min_size(
                origin + egui::vec2(page.rect.left as f32, page.rect.top as f32),
                egui::vec2(page.rect.width as f32, page.rect.height as f32),
            )
        };
        let Some(page) = frame.pages.iter().min_by(|a, b| {
            page_rect(a)
                .distance_to_pos(point)
                .total_cmp(&page_rect(b).distance_to_pos(point))
        }) else {
            return;
        };
        // Wait for the nearest visible page's governed text instead of skipping it.
        let Some(payload) = self
            .pages
            .get(&page.page)
            .and_then(text::TextState::payload)
        else {
            return;
        };
        let character = caret_at(payload, page_rect(page), rotation, point).unwrap_or(0);
        let Ok(extent) = SelectionPosition::new(page.page, character) else {
            return;
        };
        self.gesture = SelectionGesture::Dragging(anchor, extent);
        if extent != previous {
            ui.ctx().request_repaint();
        }

        let speed = |point: f32, min: f32, max: f32| {
            let edge = 28.0_f32.min((max - min) * 0.25);
            if edge <= 0.0 {
                return 0.0;
            }
            let proximity = if point < min + edge {
                (point - min - edge) / edge
            } else if point > max - edge {
                (point - max + edge) / edge
            } else {
                0.0
            };
            proximity.clamp(-1.0, 1.0) * 720.0
        };
        let velocity = [
            speed(pointer.x, viewport.left(), viewport.right()),
            speed(pointer.y, viewport.top(), viewport.bottom()),
        ];
        let mut delta = egui::Vec2::ZERO;
        for axis in 0..2 {
            let limit = (frame.content_size[axis] - frame.viewport[axis]).max(0.0);
            let next = (frame.scroll[axis] + f64::from(velocity[axis] * dt)).clamp(0.0, limit);
            delta[axis] = (frame.scroll[axis] - next) as f32;
        }
        if delta != egui::Vec2::ZERO {
            // Direct manipulation has no easing or inertia, including reduced-motion mode.
            ui.scroll_with_delta_animation(delta, egui::style::ScrollAnimation::none());
            ui.ctx().request_repaint();
        }
    }
}
