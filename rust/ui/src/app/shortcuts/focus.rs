use super::*;
use crate::theme::{self, Palette};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(in crate::app) enum ReadingTarget {
    #[default]
    Primary,
    Reference,
}

#[derive(Default)]
pub(in crate::app) struct ReadingFocus {
    pub target: ReadingTarget,
    last_widget: Option<egui::Id>,
    canvas_ids: [Option<egui::Id>; 2],
    reference_rect: Option<egui::Rect>,
    root_layer: Option<egui::LayerId>,
    pending: bool,
}

impl ReadingFocus {
    pub fn reference(&self) -> bool {
        self.target == ReadingTarget::Reference
    }

    pub fn observe(
        &mut self,
        ctx: &egui::Context,
        blocked: bool,
        reference_layers: Option<&HashSet<egui::LayerId>>,
    ) {
        let Some(layers) = reference_layers else {
            self.target = ReadingTarget::Primary;
            self.reference_rect = None;
            self.pending = false;
            return;
        };
        if blocked {
            return;
        }
        let focused = ctx.memory(|memory| memory.focused());
        if focused != self.last_widget
            && let Some(response) = focused.and_then(|id| ctx.read_response(id))
        {
            self.target = self.owner(response.layer_id, response.rect.center(), layers);
        }
        self.last_widget = focused;
        if let Some(position) = ctx.input(|input| {
            input
                .pointer
                .any_pressed()
                .then(|| input.pointer.interact_pos())
                .flatten()
        }) && let Some(layer) = ctx.layer_id_at(position)
        {
            self.target = self.owner(layer, position, layers);
        }
    }

    fn owner(
        &self,
        layer: egui::LayerId,
        position: egui::Pos2,
        layers: &HashSet<egui::LayerId>,
    ) -> ReadingTarget {
        if layers.contains(&layer)
            || Some(layer) == self.root_layer
                && self
                    .reference_rect
                    .is_some_and(|rect| rect.contains(position))
        {
            ReadingTarget::Reference
        } else {
            ReadingTarget::Primary
        }
    }

    pub fn cancel_canvas_request(&mut self) {
        self.pending = false;
    }

    pub fn switch(&mut self, ctx: &egui::Context) {
        self.resume(
            ctx,
            if self.reference() {
                ReadingTarget::Primary
            } else {
                ReadingTarget::Reference
            },
        );
    }
    pub(in crate::app) fn resume(&mut self, ctx: &egui::Context, target: ReadingTarget) {
        self.target = target;
        if let Some(id) = ctx.memory(|memory| memory.focused()) {
            ctx.memory_mut(|memory| memory.surrender_focus(id));
        }
        self.pending = true;
    }

    pub fn canvas_rect(&self, ctx: &egui::Context, target: ReadingTarget) -> Option<egui::Rect> {
        self.canvas_ids[usize::from(target == ReadingTarget::Reference)]
            .and_then(|id| ctx.read_response(id))
            .map(|response| response.rect)
    }

    pub fn canvas_focused(&self, ctx: &egui::Context) -> bool {
        ctx.memory(|memory| memory.focused())
            .is_some_and(|id| self.canvas_ids.contains(&Some(id)))
    }

    pub fn register(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        target: ReadingTarget,
        label: &str,
        p: Palette,
    ) {
        let index = usize::from(target == ReadingTarget::Reference);
        let id = ui.id().with(("reading-focus", target));
        self.canvas_ids[index] = Some(id);
        self.root_layer = Some(ui.layer_id());
        if target == ReadingTarget::Reference {
            self.reference_rect = Some(rect);
        }
        let response = ui.interact(rect, id, egui::Sense::focusable_noninteractive());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Panel, ui.is_enabled(), label)
        });
        theme::record_control_bounds(ui, label, rect);
        if self.pending && self.target == target {
            response.request_focus();
            self.pending = false;
        }
        if self.target == target {
            ui.painter().rect_stroke(
                rect.shrink(1.0),
                2,
                egui::Stroke::new(1.5, p.accent),
                egui::StrokeKind::Inside,
            );
        }
    }
}
