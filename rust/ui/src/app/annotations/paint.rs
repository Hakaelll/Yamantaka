use super::*;
use yamantaka_application::annotations::{AnnotationMarksReply, AnnotationMarksRequest};
use yamantaka_domain::annotations::marks::AnnotationMarks;

enum Phase {
    Pending(Cancellation),
    Ready(AnnotationMarks),
    Failed(String),
}
impl Drop for Phase {
    fn drop(&mut self) {
        if let Self::Pending(cancel) = self {
            cancel.cancel();
        }
    }
}
#[derive(Default)]
pub(in crate::app) struct AnnotationPaint {
    document: Option<String>,
    pages: HashMap<u32, Phase>,
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
    pub(in crate::app) fn accept(&mut self, reply: AnnotationMarksReply) {
        if self.document.as_deref() != Some(&reply.document) || reply.cancel.is_cancelled() {
            return;
        }
        let Some(Phase::Pending(current)) = self.pages.get(&reply.page) else {
            return;
        };
        if !current.same_request(&reply.cancel) {
            return;
        }
        let phase = match reply.result {
            Ok(payload) if payload.page() == reply.page => Phase::Ready(payload),
            Ok(_) => Phase::Failed("Annotation marks belong to another page".into()),
            Err(error) => Phase::Failed(error.to_string()),
        };
        self.pages.insert(reply.page, phase);
    }
    pub(super) fn marks(&self, page: u32) -> Option<&AnnotationMarks> {
        match self.pages.get(&page) {
            Some(Phase::Ready(payload)) => Some(payload),
            _ => None,
        }
    }
    pub(super) fn error(&self) -> Option<&str> {
        self.pages.values().find_map(|phase| match phase {
            Phase::Failed(error) => Some(error.as_str()),
            _ => None,
        })
    }
    pub(in crate::app) fn ready(&self) -> bool {
        !self.pages.is_empty()
            && self
                .pages
                .values()
                .all(|phase| matches!(phase, Phase::Ready(_)))
    }
    pub(in crate::app) fn capture_summary(&self) -> serde_json::Value {
        serde_json::json!({"residentPages":self.pages.len(),"readyPages":self.pages.values().filter(|phase|matches!(phase,Phase::Ready(_))).count(),"marks":self.pages.values().filter_map(|phase|match phase {Phase::Ready(payload)=>Some(payload.marks().len()),_=>None}).sum::<usize>()})
    }
}
impl YamantakaApp {
    pub(in crate::app) fn request_annotation_marks(&mut self, document: &str, visible: &[u32]) {
        self.reader.annotation_paint.retain(document, visible);
        let pending = self
            .reader
            .annotation_paint
            .pages
            .values()
            .filter(|phase| matches!(phase, Phase::Pending(_)))
            .count();
        let mut available = 2usize.saturating_sub(pending);
        for &page in visible {
            if available == 0 {
                break;
            }
            if self.reader.annotation_paint.pages.contains_key(&page) {
                continue;
            }
            available -= 1;
            let cancel = Cancellation::default();
            self.reader
                .annotation_paint
                .pages
                .insert(page, Phase::Pending(cancel.clone()));
            let request = AnnotationMarksRequest {
                document: document.into(),
                page,
                governor: self.governor.clone(),
                cancel: cancel.clone(),
            };
            if let Err(error) = self
                .runtime
                .submit(Action::AnnotationMarks(Box::new(request)))
            {
                self.reader.annotation_paint.accept(AnnotationMarksReply {
                    document: document.into(),
                    page,
                    result: Err(error),
                    cancel,
                });
            }
        }
    }
}
