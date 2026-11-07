use super::*;
use yamantaka_application::annotations::{AnnotationMarksReply, AnnotationMarksRequest};
use yamantaka_domain::annotations::marks::AnnotationMarks;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eraser_hits_the_rotated_outline_without_hitting_its_interior() {
        let paper = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(200.0, 100.0));
        let points = [[0.2, 0.2], [0.8, 0.8]];
        let hit = |point| {
            drawing_hit(
                paper,
                PageRotation::Deg90,
                AnnotationKind::Rectangle,
                &points,
                point,
            )
        };
        assert!(hit(egui::pos2(50.0, 58.0)));
        assert!(!hit(paper.center()));
        assert!(drawing_hit(
            paper,
            PageRotation::Deg0,
            AnnotationKind::Arrow,
            &points,
            paper.center()
        ));
    }

    #[test]
    fn badges_group_within_one_target_height() {
        let anchors = [
            (20.0, "first"),
            (32.0, "second"),
            (50.0, "third"),
            (54.0, "fourth"),
        ];
        assert_eq!(badge_group_len(&anchors), 2);
        assert_eq!(badge_group_len(&anchors[2..]), 2);
    }
}

#[derive(Default)]
struct MarkSlot {
    payload: Option<AnnotationMarks>,
    pending: Option<Cancellation>,
    dirty: bool,
    error: Option<String>,
}
impl MarkSlot {
    fn cancel(&mut self) {
        if let Some(cancel) = self.pending.take() {
            cancel.cancel();
        }
    }
    fn ready(&self) -> bool {
        self.payload.is_some() && self.pending.is_none() && !self.dirty && self.error.is_none()
    }
    fn refresh(&mut self) {
        self.cancel();
        self.dirty = true;
        self.error = None;
    }
}
impl Drop for MarkSlot {
    fn drop(&mut self) {
        self.cancel();
    }
}
#[derive(Default)]
pub(in crate::app) struct AnnotationPaint {
    document: Option<String>,
    pages: HashMap<u32, MarkSlot>,
    pub(super) focused: Option<String>,
}
impl AnnotationPaint {
    pub(in crate::app) fn invalidate(&mut self) {
        self.pages.clear();
        self.focused = None;
    }
    fn retain(&mut self, document: &str, visible: &[u32]) {
        if self.document.as_deref() != Some(document) {
            self.invalidate();
            self.document = Some(document.into());
        }
        self.pages.retain(|page, _| visible.contains(page));
    }
    pub(in crate::app) fn owns(&self, reply: &AnnotationMarksReply) -> bool {
        self.document.as_deref() == Some(&reply.document)
            && !reply.cancel.is_cancelled()
            && self
                .pages
                .get(&reply.page)
                .and_then(|slot| slot.pending.as_ref())
                .is_some_and(|current| current.same_request(&reply.cancel))
    }
    pub(in crate::app) fn accept(&mut self, reply: AnnotationMarksReply) {
        if !self.owns(&reply) {
            return;
        }
        let slot = self.pages.get_mut(&reply.page).unwrap();
        slot.cancel();
        match reply.result {
            Ok(payload) if payload.page() == reply.page => {
                slot.payload = Some(payload);
                slot.dirty = false;
                slot.error = None;
            }
            Ok(_) => slot.error = Some("Annotation marks belong to another page".into()),
            Err(error) => {
                // A retained snapshot must not prevent recovery after allocation failure.
                if matches!(&error, Failure::MemoryBudget) {
                    slot.payload = None;
                }
                slot.error = Some(error.to_string());
            }
        }
    }
    pub(in crate::app) fn invalidate_document(&mut self, document: Option<&str>) {
        if document.is_none_or(|document| self.document.as_deref() == Some(document)) {
            for slot in self.pages.values_mut() {
                slot.refresh();
            }
        }
    }
    fn marks(&self, page: u32) -> Option<&AnnotationMarks> {
        self.pages.get(&page)?.payload.as_ref()
    }
    pub(in crate::app) fn error(&self) -> Option<&str> {
        self.pages.values().find_map(|slot| slot.error.as_deref())
    }
    pub(in crate::app) fn ready(&self) -> bool {
        !self.pages.is_empty() && self.pages.values().all(MarkSlot::ready)
    }
    pub(in crate::app) fn capture_summary(&self) -> serde_json::Value {
        let mut details: Vec<_> = self.pages.iter().filter_map(|(page, slot)|slot.payload.as_ref().map(|payload|serde_json::json!({
            "page":page, "kinds":payload.marks().iter().map(|mark|format!("{:?}",mark.kind())).collect::<Vec<_>>(),
            "colors":payload.marks().iter().map(|mark|format!("{:?}",mark.color())).collect::<Vec<_>>()
        }))).collect();
        details.sort_by_key(|detail| detail["page"].as_u64());
        serde_json::json!({"document":self.document,"residentPages":self.pages.len(),"readyPages":self.pages.values().filter(|slot|slot.ready()).count(),
            "marks":self.pages.values().filter_map(|slot|slot.payload.as_ref().map(|payload|payload.marks().len())).sum::<usize>(),
            "retainedPages":self.pages.values().filter(|slot|slot.payload.is_some() && !slot.ready()).count(),"details":details})
    }
    pub(in crate::app) fn request(
        &mut self,
        runtime: &Runtime,
        governor: &MemoryGovernor,
        document: &str,
        visible: &[u32],
    ) {
        self.retain(document, visible);
        let pending = self
            .pages
            .values()
            .filter(|slot| slot.pending.is_some())
            .count();
        let mut available = 2usize.saturating_sub(pending);
        for &page in visible {
            if available == 0 {
                break;
            }
            let slot = self.pages.entry(page).or_default();
            if slot.pending.is_some()
                || slot.error.is_some()
                || slot.payload.is_some() && !slot.dirty
            {
                continue;
            }
            available -= 1;
            let cancel = Cancellation::default();
            slot.pending = Some(cancel.clone());
            let request = AnnotationMarksRequest {
                document: document.into(),
                page,
                governor: governor.clone(),
                cancel: cancel.clone(),
            };
            if let Err(error) = runtime.submit(Action::AnnotationMarks(Box::new(request))) {
                self.accept(AnnotationMarksReply {
                    document: document.into(),
                    page,
                    result: Err(error),
                    cancel,
                });
            }
        }
    }
}

impl AnnotationPaint {
    pub(in crate::app) fn erase_target(
        &self,
        ui: &egui::Ui,
        page: u32,
        paper: egui::Rect,
        rotation: PageRotation,
        pointer: egui::Pos2,
        p: Palette,
    ) -> Option<&str> {
        let marks = self.marks(page)?;
        let mark = marks.marks().iter().rev().find(|mark| {
            if mark.has_note() {
                return false;
            }
            if let Some(drawing) = mark.drawing() {
                return drawing_hit(paper, rotation, mark.kind(), drawing.points(), pointer);
            }
            matches!(
                mark.kind(),
                AnnotationKind::Highlight
                    | AnnotationKind::Underline
                    | AnnotationKind::Strikeout
                    | AnnotationKind::PageMarker
            ) && mark.rects().iter().any(|rect| {
                reader_viewport::page_region(paper, rotation, rect.region())
                    .expand(3.0)
                    .contains(pointer)
            })
        })?;
        let painter = ui.painter().with_clip_rect(paper);
        if let Some(drawing) = mark.drawing() {
            let width = drawing_width(paper);
            paint_shape(
                &painter,
                paper,
                rotation,
                mark.kind(),
                drawing.points(),
                egui::Stroke::new(width + 4.0, p.accent),
            );
            paint_shape(
                &painter,
                paper,
                rotation,
                mark.kind(),
                drawing.points(),
                egui::Stroke::new(width, panel::mark_color(mark.color())),
            );
        } else {
            for rect in mark.rects() {
                let rect = reader_viewport::page_region(paper, rotation, rect.region());
                painter.rect_stroke(
                    rect.expand(1.0),
                    3,
                    egui::Stroke::new(1.5, p.accent),
                    egui::StrokeKind::Outside,
                );
            }
        }
        Some(mark.id())
    }

    pub(in crate::app) fn paint(
        &self,
        ui: &egui::Ui,
        page: u32,
        paper: egui::Rect,
        rotation: PageRotation,
    ) {
        let Some(records) = self.marks(page) else {
            return;
        };
        for mark in records.marks() {
            if let Some(drawing) = mark.drawing() {
                paint_drawing(
                    ui,
                    paper,
                    rotation,
                    mark.kind(),
                    drawing.points(),
                    mark.color(),
                );
            }
            let color = panel::mark_color(mark.color());
            for rect in mark.rects() {
                let region = reader_viewport::page_region(paper, rotation, rect.region());
                match mark.kind() {
                    AnnotationKind::Underline | AnnotationKind::Strikeout => {
                        let native = rect.region();
                        let y = if mark.kind() == AnnotationKind::Underline {
                            native.max()[1]
                        } else {
                            (native.min()[1] + native.max()[1]) * 0.5
                        };
                        let a = rotation.rotate_point([native.min()[0], y]);
                        let b = rotation.rotate_point([native.max()[0], y]);
                        let map = |point: [f64; 2]| {
                            paper.min
                                + egui::vec2(
                                    point[0] as f32 * paper.width(),
                                    point[1] as f32 * paper.height(),
                                )
                        };
                        ui.painter()
                            .line_segment([map(a), map(b)], egui::Stroke::new(1.5, color));
                    }
                    AnnotationKind::PageComment => {}
                    _ => {
                        ui.painter()
                            .rect_filled(region, 0, color.linear_multiply(0.2));
                    }
                }
                if self.focused.as_deref() == Some(mark.id()) {
                    ui.painter().rect_stroke(
                        region.expand(1.0),
                        2,
                        egui::Stroke::new(1.3, color),
                        egui::StrokeKind::Outside,
                    );
                }
            }
        }
    }

    pub(in crate::app) fn note_badges(
        &self,
        ui: &mut egui::Ui,
        page: u32,
        paper: egui::Rect,
        rotation: PageRotation,
        en: bool,
        p: Palette,
    ) -> Option<String> {
        let marks = self.marks(page)?;
        let visible = ui.clip_rect();
        let outside = paper.right() + 42.0 <= visible.right();
        let x = if outside {
            paper.right() + 6.0
        } else {
            paper.right().min(visible.right()) - 32.0
        };
        if x + 28.0 < visible.left() {
            return None;
        }
        let mut anchors = Vec::new();
        for mark in marks.marks().iter().filter(|mark| mark.has_note()) {
            let y = mark
                .rects()
                .first()
                .map(|rect| {
                    reader_viewport::page_region(paper, rotation, rect.region())
                        .center()
                        .y
                })
                .or_else(|| {
                    mark.drawing()
                        .and_then(|drawing| drawing.points().first())
                        .map(|point| draw_position(paper, rotation, *point).y)
                })
                .unwrap_or(paper.top() + 16.0);
            if y >= visible.top() + 12.0 && y <= visible.bottom() - 12.0 {
                anchors.push((y, mark.id()));
            }
        }
        anchors.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut action = None;
        let mut remaining = anchors.as_slice();
        while !remaining.is_empty() {
            let count = badge_group_len(remaining);
            let (group, rest) = remaining.split_at(count);
            remaining = rest;
            let label = if count == 1 {
                if en {
                    format!("Open note on page {}", page + 1)
                } else {
                    format!("Abrir nota de la página {}", page + 1)
                }
            } else if en {
                format!("{count} notes on page {}", page + 1)
            } else {
                format!("{count} notas de la página {}", page + 1)
            };
            let rect =
                egui::Rect::from_min_size(egui::pos2(x, group[0].0 - 12.0), egui::vec2(28.0, 24.0));
            let selected = group
                .iter()
                .any(|(_, id)| self.focused.as_deref() == Some(*id));
            let response = ui.interact(
                rect,
                ui.id().with(("study-note-badge", page, group[0].1)),
                egui::Sense::click(),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), &label)
            });
            theme::record_control_bounds(ui, &label, rect);
            let fill = if response.hovered() || selected {
                p.raised
            } else {
                p.surface
            };
            let outline = if response.has_focus() || selected {
                p.accent
            } else {
                p.line
            };
            ui.painter().rect(
                rect,
                5,
                fill,
                egui::Stroke::new(1.0, outline),
                egui::StrokeKind::Inside,
            );
            if count == 1 {
                theme::icon(
                    ui.painter(),
                    rect.center(),
                    theme::Icon::List,
                    if selected { p.accent } else { p.secondary },
                );
                if response.clicked() {
                    action = Some(group[0].1.to_owned());
                }
            } else {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    count.to_string(),
                    egui::FontId::proportional(12.0),
                    p.ink,
                );
                egui::Popup::menu(&response).show(|ui| {
                    egui::ScrollArea::vertical()
                        .max_height(160.0)
                        .show(ui, |ui| {
                            for (index, (_, id)) in group.iter().enumerate() {
                                let label = format!(
                                    "{} {} · {} {}",
                                    if en { "Note" } else { "Nota" },
                                    index + 1,
                                    if en { "page" } else { "página" },
                                    page + 1
                                );
                                if theme::navigation(
                                    ui,
                                    &label,
                                    self.focused.as_deref() == Some(*id),
                                    p,
                                )
                                .clicked()
                                {
                                    action = Some((*id).into());
                                    ui.close();
                                }
                            }
                        });
                });
            }
            response.on_hover_text(label);
        }
        action
    }
}

fn badge_group_len(anchors: &[(f32, &str)]) -> usize {
    anchors
        .iter()
        .take_while(|(y, _)| *y - anchors[0].0 < 24.0)
        .count()
}

fn draw_position(paper: egui::Rect, rotation: PageRotation, point: [f64; 2]) -> egui::Pos2 {
    let [x, y] = rotation.rotate_point(point);
    paper.min + egui::vec2(x as f32 * paper.width(), y as f32 * paper.height())
}

fn drawing_width(paper: egui::Rect) -> f32 {
    (paper.width().min(paper.height()) * 0.003).max(1.0)
}

fn arrow_head(a: egui::Pos2, b: egui::Pos2) -> [egui::Pos2; 2] {
    let turn = egui::emath::Rot2::from_angle(std::f32::consts::TAU / 10.0);
    let wing = (b - a) / 4.0;
    [b - turn * wing, b - turn.inverse() * wing]
}

fn drawing_hit(
    paper: egui::Rect,
    rotation: PageRotation,
    kind: AnnotationKind,
    points: &[[f64; 2]],
    pointer: egui::Pos2,
) -> bool {
    let [start, end, ..] = points else {
        return false;
    };
    let position = |point| draw_position(paper, rotation, point);
    let (a, b) = (position(*start), position(*end));
    let radius = 5.0 + drawing_width(paper) * 0.5;
    let near = |a: egui::Pos2, b: egui::Pos2| {
        let line = b - a;
        let offset = pointer - a;
        let length = line.length_sq();
        let fraction = if length > 0.0 {
            (offset.dot(line) / length).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (offset - fraction * line).length_sq() <= radius * radius
    };
    match kind {
        AnnotationKind::Ink => points
            .windows(2)
            .any(|pair| near(position(pair[0]), position(pair[1]))),
        AnnotationKind::Arrow => {
            near(a, b) || arrow_head(a, b).into_iter().any(|wing| near(b, wing))
        }
        AnnotationKind::Rectangle => {
            let rect = egui::Rect::from_two_pos(a, b);
            rect.expand(radius).contains(pointer) && !rect.shrink(radius).contains(pointer)
        }
        _ => false,
    }
}

pub(in crate::app) fn paint_drawing(
    ui: &egui::Ui,
    paper: egui::Rect,
    rotation: PageRotation,
    kind: AnnotationKind,
    points: &[[f64; 2]],
    color: AnnotationColor,
) {
    paint_shape(
        &ui.painter().with_clip_rect(paper),
        paper,
        rotation,
        kind,
        points,
        egui::Stroke::new(drawing_width(paper), panel::mark_color(color)),
    );
}

fn paint_shape(
    painter: &egui::Painter,
    paper: egui::Rect,
    rotation: PageRotation,
    kind: AnnotationKind,
    points: &[[f64; 2]],
    stroke: egui::Stroke,
) {
    let [start, end, ..] = points else { return };
    let position = |point| draw_position(paper, rotation, point);
    let a = position(*start);
    let b = position(*end);
    match kind {
        AnnotationKind::Ink => {
            painter.add(egui::Shape::line(
                points.iter().copied().map(position).collect(),
                stroke,
            ));
        }
        AnnotationKind::Arrow => {
            painter.line_segment([a, b], stroke);
            for wing in arrow_head(a, b) {
                painter.line_segment([b, wing], stroke);
            }
        }
        AnnotationKind::Rectangle => {
            painter.rect_stroke(
                egui::Rect::from_two_pos(a, b),
                0,
                stroke,
                egui::StrokeKind::Middle,
            );
        }
        _ => {}
    }
}
