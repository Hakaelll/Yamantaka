use super::*;
use yamantaka_domain::{annotations::AnnotationColor, pdf_geometry::NormalizedPageRect};

type Anchor = (egui::Id, egui::LayerId, egui::Rect);

#[derive(Default)]
pub(super) struct Passive {
    pub enabled: bool,
    pub completed: bool,
    pub targets: BTreeMap<u32, Vec<NormalizedPageRect>>,
    anchor: Option<Anchor>,
    layers: Vec<egui::LayerId>,
}

impl SelectionState {
    pub(in crate::app) fn request_passive(
        &mut self,
        runtime: &Runtime,
        document: &str,
        visible: &[u32],
        enabled: bool,
    ) {
        self.passive.anchor = None;
        self.retain(document, visible);
        self.passive.enabled = enabled;
        if self.enabled {
            self.request(runtime, document, visible);
        } else if enabled {
            for page in self.next_pages(visible) {
                self.pages
                    .entry(page)
                    .or_default()
                    .request(runtime, document, page);
            }
        }
    }

    pub(in crate::app) fn has_passage(&self) -> bool {
        self.range().is_some_and(|range| !range.is_empty())
    }

    pub(in crate::app) fn dismiss_passage(&mut self) {
        self.gesture = SelectionGesture::Empty;
        self.passive.completed = false;
        self.passive.anchor = None;
    }

    pub(in crate::app) fn cancel_pointer_gesture(&mut self) -> bool {
        if matches!(self.gesture, SelectionGesture::Dragging(..)) {
            self.dismiss_passage();
            true
        } else {
            false
        }
    }

    pub(in crate::app) fn take_completed_pointer(&mut self) -> bool {
        std::mem::take(&mut self.passive.completed)
    }

    pub(in crate::app) fn context_layers(&self) -> impl Iterator<Item = egui::LayerId> + '_ {
        self.passive.layers.iter().copied()
    }

    pub(in crate::app) fn context_actions(
        &mut self,
        ui: &egui::Ui,
        english: bool,
        color: AnnotationColor,
        p: Palette,
    ) -> Option<SelectionAction> {
        self.passive.layers.clear();
        let (anchor_id, layer, anchor) = self.passive.anchor?;
        if !self.has_passage() || !matches!(self.gesture, SelectionGesture::Selected(..)) {
            return None;
        }
        let id = anchor_id.with("selection-study-actions");
        let tr = |es, en| if english { en } else { es };
        let frame = egui::Frame::popup(ui.style())
            .fill(p.surface)
            .stroke(egui::Stroke::new(1.0, p.line));
        let mut action = None;
        let focus = self.focused_page(ui.ctx()).is_some()
            && ui
                .ctx()
                .input_mut(|input| input.consume_key(egui::Modifiers::SHIFT, egui::Key::F10));
        egui::Popup::new(id, ui.ctx().clone(), anchor, layer)
            .open(true)
            .frame(frame)
            .gap(6.0)
            .show(|ui| {
                self.passive.layers.push(ui.layer_id());
                let width = ui.ctx().content_rect().width();
                ui.set_max_width((width - 20.0).max(140.0));
                ui.horizontal_wrapped(|ui| {
                    for (label, candidate) in [
                        (tr("Resaltar", "Highlight"), SelectionAction::Highlight),
                        (tr("Subrayar", "Underline"), SelectionAction::Underline),
                        (tr("Añadir nota", "Add note"), SelectionAction::Annotate),
                        (tr("Copiar", "Copy"), SelectionAction::Copy),
                    ] {
                        let response = theme::navigation(ui, label, false, p);
                        if focus && matches!(candidate, SelectionAction::Highlight) {
                            response.request_focus();
                        }
                        if response.clicked() {
                            action = Some(candidate);
                        }
                    }
                    let mut chosen = color;
                    let color_label = annotations::panel::color_label(color, english);
                    let button = theme::icon_button(
                        ui,
                        &format!("Color · {color_label}"),
                        theme::Icon::Down,
                        p,
                    );
                    ui.painter().circle_filled(
                        button.rect.center() - egui::vec2(8.0, 0.0),
                        4.0,
                        annotations::panel::mark_color(color),
                    );
                    egui::Popup::menu(&button).show(|ui| {
                        self.passive.layers.push(ui.layer_id());
                        for option in [
                            AnnotationColor::Yellow,
                            AnnotationColor::Green,
                            AnnotationColor::Blue,
                            AnnotationColor::Rose,
                            AnnotationColor::Violet,
                        ] {
                            let label = annotations::panel::color_label(option, english);
                            if theme::navigation(ui, label, color == option, p).clicked() {
                                chosen = option;
                                ui.close();
                            }
                        }
                    });
                    if chosen != color {
                        action = Some(SelectionAction::Color(chosen));
                    }
                });
            });
        action
    }
}

impl Passive {
    pub(super) fn anchor(&mut self, ui: &egui::Ui, id: egui::Id, bounds: egui::Rect) {
        let bounds = bounds.intersect(ui.clip_rect());
        if bounds.is_positive() {
            self.anchor = Some((id, ui.layer_id(), bounds));
        }
    }

    pub(super) fn select(
        &mut self,
        ui: &egui::Ui,
        payload: &PdfPageText,
        page: &SelectionPage<'_>,
        id: egui::Id,
        label: &str,
        gesture: &mut SelectionGesture,
    ) {
        let paper = page.paper.intersect(ui.clip_rect());
        if !paper.is_positive() {
            return;
        }
        let focus = ui.interact(paper, id, egui::Sense::focusable_noninteractive());
        focus.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Image, ui.is_enabled(), label)
        });
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(
                id,
                egui::EventFilter {
                    horizontal_arrows: true,
                    escape: true,
                    ..Default::default()
                },
            )
        });
        let targets = self.targets.entry(page.page).or_insert_with(|| {
            glyph_targets(payload.characters().iter().filter_map(|char| char.region))
        });
        for (index, bounds) in targets.iter().enumerate() {
            let bounds =
                reader_viewport::page_region(page.paper, page.rotation, *bounds).intersect(paper);
            if !bounds.is_positive() {
                continue;
            }
            let response = ui.interact(bounds, id.with(("glyphs", index)), egui::Sense::drag());
            if response.drag_started_by(egui::PointerButton::Primary)
                && let Some(origin) = ui.input(|input| input.pointer.press_origin())
                && let Some(index) = caret_at(payload, page.paper, page.rotation, origin)
                && let Ok(position) = SelectionPosition::new(page.page, index)
            {
                focus.request_focus();
                *gesture = SelectionGesture::Dragging(position, position);
            }
            response.on_hover_cursor(egui::CursorIcon::Text);
        }
        if matches!(*gesture, SelectionGesture::Selected(..))
            && ui.input(|input| input.pointer.primary_pressed())
            && ui.rect_contains_pointer(paper)
        {
            *gesture = SelectionGesture::Empty;
        }
    }
}

fn glyph_targets(regions: impl IntoIterator<Item = NormalizedPageRect>) -> Vec<NormalizedPageRect> {
    let mut lines: Vec<NormalizedPageRect> = Vec::new();
    for region in regions {
        if let Some(last) = lines.last_mut() {
            let [x, y] = region.min();
            let [right, bottom] = region.max();
            let [previous_x, previous_y] = last.min();
            let [previous_right, previous_bottom] = last.max();
            let height = (bottom - y).min(previous_bottom - previous_y);
            if (bottom.min(previous_bottom) - y.max(previous_y)) >= height * 0.65
                && x <= previous_right + height * 0.45
                && previous_x <= right + height * 0.45
            {
                *last = NormalizedPageRect::new(
                    [x.min(previous_x), y.min(previous_y)],
                    [right.max(previous_right), bottom.max(previous_bottom)],
                )
                .unwrap();
                continue;
            }
        }
        lines.push(region);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_targets_keep_separate_lines_and_columns() {
        let rect = |x, y| NormalizedPageRect::new([x, y], [x + 0.02, y + 0.03]).unwrap();
        let targets = glyph_targets([
            rect(0.10, 0.10),
            rect(0.12, 0.10),
            rect(0.60, 0.10),
            rect(0.10, 0.16),
            rect(0.12, 0.16),
        ]);
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0].min(), [0.10, 0.10]);
        assert_eq!(targets[0].max(), rect(0.12, 0.10).max());
        assert_eq!(targets[1], rect(0.60, 0.10));
        assert_eq!(targets[2].min(), [0.10, 0.16]);
    }
}
