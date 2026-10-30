use super::*;
use yamantaka_application::annotations::{AnnotationMarksReply, AnnotationMarksRequest};
use yamantaka_domain::annotations::marks::AnnotationMarks;

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
}
impl AnnotationPaint {
    pub(in crate::app) fn invalidate(&mut self) {
        self.pages.clear();
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
            }
        }
    }
}
