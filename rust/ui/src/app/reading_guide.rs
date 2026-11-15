use super::*;

pub(super) struct Guide {
    active: bool,
    height: f32,
    position: f32,
}

impl Default for Guide {
    fn default() -> Self {
        Self {
            active: false,
            height: 44.0,
            position: 0.35,
        }
    }
}

impl YamantakaApp {
    pub(crate) fn preview_reading_guide(&mut self) {
        if self.demo && self.capture.is_some() {
            self.reader.guide.active = true;
        }
    }
}

impl Guide {
    pub(super) fn controls(&mut self, ui: &mut egui::Ui, english: bool, p: Palette) {
        let label = if english {
            "Reading guide"
        } else {
            "Guía de lectura"
        };
        if theme::navigation(ui, label, self.active, p).clicked() {
            self.active = !self.active;
        }
        if !self.active {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            for (height, es, en) in [
                (26.0, "Estrecha", "Narrow"),
                (44.0, "Media", "Medium"),
                (68.0, "Ancha", "Wide"),
            ] {
                if theme::navigation(ui, if english { en } else { es }, self.height == height, p)
                    .clicked()
                {
                    self.height = height;
                }
            }
        });
        ui.add(
            egui::Slider::new(&mut self.position, 0.0..=1.0)
                .show_value(false)
                .text(if english {
                    "Guide position"
                } else {
                    "Posición de la guía"
                }),
        );
    }

    pub(super) fn paint(
        &mut self,
        ui: &mut egui::Ui,
        page: u32,
        paper: egui::Rect,
        english: bool,
        p: Palette,
    ) {
        if !self.active || paper.height() < 1.0 {
            return;
        }
        let half = self.height.min(paper.height()) / 2.0;
        let range = (paper.height() - 2.0 * half).max(0.0);
        let center = paper.top() + half + range * self.position;
        let handle = egui::Rect::from_center_size(
            egui::pos2(paper.right() - 10.0, center),
            egui::vec2(20.0, 26.0),
        );
        let response = ui.interact(
            handle,
            ui.id().with(("reading-guide", page)),
            egui::Sense::click_and_drag(),
        );
        let label = if english {
            "Move reading guide"
        } else {
            "Mover guía de lectura"
        };
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Slider, true, label));
        theme::record_control_bounds(ui, label, response.rect);
        let delta = if response.has_focus() {
            ui.input_mut(|input| {
                let down = input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown);
                let up = input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp);
                if input.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                    self.active = false;
                }
                (i32::from(down) - i32::from(up)) as f32 * 16.0
            })
        } else {
            0.0
        };
        if range > 0.0 {
            if response.dragged() {
                self.position += response.drag_delta().y / range;
            }
            self.position = (self.position + delta / range).clamp(0.0, 1.0);
        }
        let center = paper.top() + half + range * self.position;
        let painter = ui.painter().with_clip_rect(paper.intersect(ui.clip_rect()));
        for [top, bottom] in [
            [paper.top(), center - half],
            [center + half, paper.bottom()],
        ] {
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(paper.left(), top),
                    egui::pos2(paper.right(), bottom),
                ),
                0,
                Color32::from_black_alpha(29),
            );
        }
        painter.rect_filled(
            handle.translate(egui::vec2(0.0, center - handle.center().y)),
            7,
            p.surface,
        );
        theme::icon(
            &painter,
            egui::pos2(handle.center().x, center),
            theme::Icon::List,
            if response.has_focus() {
                p.accent
            } else {
                p.secondary
            },
        );
        response.on_hover_text(label);
    }
}
