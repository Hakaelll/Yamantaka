use super::*;
use yamantaka_domain::{Failure, RenderedPage};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum Preview {
    #[default]
    Off,
    FailOnce,
    Retry,
    PressureOnce,
    CancelAfterSubmit,
    Closing,
    Complete,
}
pub(super) struct OpenRequest {
    pub document: Document,
    pub context: Option<yamantaka_domain::workspaces::WorkspaceComparison>,
}

pub(super) struct Candidate {
    pub pane: reader_pane::ReaderPane,
    pub open: Cancellation,
    pub key: RenderKey,
    pub render: Cancellation,
    pub progress: Progress,
}
pub(super) enum Progress {
    Waiting(Instant),
    Submitted,
}
impl Candidate {
    pub(super) fn new(
        mut pane: reader_pane::ReaderPane,
        open: Cancellation,
        cap: u32,
        bytes: usize,
        direction: ReadingDirection,
        context: Option<&yamantaka_domain::workspaces::WorkspaceComparison>,
    ) -> yamantaka_domain::Result<Self> {
        if let Some(saved) = context {
            saved.validate()?;
            if pane.active.as_ref().is_none_or(|(document, info)| {
                document.id != saved.document || saved.page as usize >= info.pages.len()
            }) {
                return Err(Failure::Invalid(
                    "The saved reference page is outside the opened PDF".into(),
                ));
            }
            pane.reader_preferences = Some(saved.preferences()?);
            pane.zoom = saved.zoom as f32;
            if let Some(navigation) = &mut pane.navigation {
                navigation.update_anchor(ReaderPosition::at(saved.page, saved.point)?);
            }
            pane.navigation_changed();
            pane.rebuild_layout(direction)?;
            pane.restoration.scroll = saved.scroll_offset;
        }
        let (document, info) = pane
            .active
            .as_ref()
            .ok_or_else(|| Failure::Invalid("The reference has no PDF".into()))?;
        let size = info
            .pages
            .get(pane.page as usize)
            .ok_or_else(|| Failure::Invalid("The reference page is outside the PDF".into()))?
            .size();
        let dimensions = preview_dimensions(egui::vec2(size[0], size[1]), 1.0, cap, bytes);
        let key = RenderKey {
            document_id: document.id.clone(),
            page: pane.page,
            region: None,
            physical_width: dimensions[0],
            physical_height: dimensions[1],
            rotation: 0,
            generation: pane.viewport.generation,
        };
        Ok(Self {
            pane,
            open,
            key,
            render: Cancellation::default(),
            progress: Progress::Waiting(Instant::now()),
        })
    }
    pub(super) fn cancel(&self) {
        self.render.cancel();
        self.open.cancel();
    }
}

#[derive(Default)]
pub(super) struct Probe {
    pub adopted_without_image: bool,
    pub first_paint_seen: bool,
    pub first_paint_empty: bool,
    pub empty_paints: u64,
    pub failures: u32,
    pub retained_on_failure: bool,
    pub queue_retries: u32,
}
impl Probe {
    pub(super) fn adopted(&mut self, pane: &reader_pane::ReaderPane) {
        self.adopted_without_image = pane
            .viewport
            .pages
            .get(&pane.page)
            .is_none_or(|slot| slot.image.is_none());
        self.first_paint_seen = false;
        self.first_paint_empty = false;
        self.empty_paints = 0;
    }
    pub(super) fn observe(&mut self, surface: &reader_surface::ReaderSurface) {
        if !self.first_paint_seen && surface.paint_count > 0 {
            self.first_paint_seen = true;
            self.first_paint_empty = surface.empty_paints > 0;
        }
        self.empty_paints = surface.empty_paints;
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"adoptedWithoutImage":self.adopted_without_image,"firstPaintSeen":self.first_paint_seen,"firstPaintEmpty":self.first_paint_empty,"emptyPaints":self.empty_paints,"failures":self.failures,"retainedOnFailure":self.retained_on_failure,"queueRetries":self.queue_retries})
    }
}

impl ReferenceView {
    pub(super) fn reject_preparation(&mut self, primary: &str, error: &Failure) {
        let Admission::Preparing(candidate) =
            std::mem::replace(&mut self.admission, Admission::Ready)
        else {
            unreachable!()
        };
        candidate.cancel();
        if !matches!(error, Failure::Cancelled) {
            self.error = Some(error.to_string());
            if let Some(probe) = &mut self.opening_probe {
                probe.failures += 1;
                probe.retained_on_failure = self
                    .pane
                    .viewport
                    .pages
                    .values()
                    .any(|slot| slot.image.is_some());
            }
        }
        self.admission = Admission::Confirming {
            document: candidate.key.document_id,
            cancel: candidate.open,
            visible: vec![primary.into()],
            submitted: false,
            blocked: false,
        };
    }
}

impl YamantakaApp {
    pub(in crate::app) fn pump_reference_preparation(&mut self, ctx: &egui::Context) {
        let Some(reference) = &mut self.reference else {
            return;
        };
        let Admission::Preparing(candidate) = &mut reference.admission else {
            return;
        };
        let Some((primary, _)) = &self.reader.active else {
            return;
        };
        if candidate.open.is_cancelled() || reference.close_requested {
            reference.reject_preparation(&primary.id, &Failure::Cancelled);
            return;
        }
        if let Progress::Waiting(at) = candidate.progress {
            if let Some(delay) = at.checked_duration_since(Instant::now()) {
                ctx.request_repaint_after(delay);
                return;
            }
            match self.runtime.submit(Action::Render(
                candidate.key.clone(),
                yamantaka_domain::Priority::Visible,
                candidate.render.clone(),
            )) {
                Ok(()) => candidate.progress = Progress::Submitted,
                Err(Failure::QueueFull) => {
                    candidate.progress =
                        Progress::Waiting(Instant::now() + Duration::from_millis(100));
                    ctx.request_repaint_after(Duration::from_millis(100));
                }
                Err(error) => {
                    reference.reject_preparation(&primary.id, &error);
                    return;
                }
            }
        }
        if matches!(candidate.progress, Progress::Submitted)
            && self.reference_prepare_preview == Preview::PressureOnce
        {
            let token = candidate.render.clone();
            self.reference_prepare_preview = Preview::Off;
            self.fail_reference_preparation(ctx, &Failure::QueueFull, Some(&token));
        } else if matches!(candidate.progress, Progress::Submitted)
            && self.reference_prepare_preview == Preview::CancelAfterSubmit
        {
            self.reference_prepare_preview = Preview::Closing;
            self.request_reference_close();
        }
    }
    pub(in crate::app) fn fail_reference_preparation(
        &mut self,
        ctx: &egui::Context,
        error: &Failure,
        token: Option<&Cancellation>,
    ) -> bool {
        let Some(reference) = &mut self.reference else {
            return false;
        };
        let owns = matches!(&reference.admission, Admission::Preparing(candidate) if token.is_some_and(|token|candidate.render.same_request(token)));
        if owns && let Some((primary, _)) = &self.reader.active {
            if matches!(error, Failure::QueueFull)
                && !reference.close_requested
                && let Admission::Preparing(candidate) = &mut reference.admission
                && !candidate.open.is_cancelled()
            {
                candidate.render.cancel();
                candidate.render = Cancellation::default();
                candidate.progress = Progress::Waiting(Instant::now() + Duration::from_millis(100));
                ctx.request_repaint_after(Duration::from_millis(100));
                if let Some(probe) = &mut reference.opening_probe {
                    probe.queue_retries += 1;
                }
                return true;
            }
            reference.reject_preparation(&primary.id, error);
        }
        owns
    }
    pub(in crate::app) fn accept_reference_preparation_image(
        &mut self,
        ctx: &egui::Context,
        page: &RenderedPage,
        token: &Cancellation,
    ) -> bool {
        let Some(reference) = &mut self.reference else {
            return false;
        };
        let Admission::Preparing(candidate) = &reference.admission else {
            return false;
        };
        if !candidate.render.same_request(token) {
            return false;
        }
        if token.is_cancelled()
            || candidate.open.is_cancelled()
            || reference.close_requested
            || !matches!(candidate.progress, Progress::Submitted)
            || candidate.key != page.key
        {
            return true;
        }
        let Some((primary, _)) = &self.reader.active else {
            return true;
        };
        let image = match render_updates::upload_image(
            ctx,
            &mut self.presenter,
            &mut self.managed_pages,
            page,
        ) {
            Ok(image) => image,
            Err(error) => {
                reference.reject_preparation(&primary.id, &error);
                return true;
            }
        };
        if token.is_cancelled()
            || matches!(&reference.admission,Admission::Preparing(candidate) if candidate.open.is_cancelled())
        {
            reference.reject_preparation(&primary.id, &Failure::Cancelled);
            return true;
        }
        let Admission::Preparing(candidate) =
            std::mem::replace(&mut reference.admission, Admission::Ready)
        else {
            unreachable!()
        };
        let mut candidate = *candidate;
        candidate.render.cancel();
        candidate.pane.viewport.pages.insert(
            page.key.page,
            ReaderPage {
                requested: None,
                cancel: Cancellation::default(),
                image: Some(image),
                image_key: Some(page.key.clone()),
                plan: Default::default(),
                refinement: Default::default(),
            },
        );
        if let Some(probe) = &mut reference.opening_probe {
            probe.adopted(&candidate.pane);
        }
        reference.pane = candidate.pane;
        reference.retry_open = None;
        reference.follow_page = false;
        reference.follow_zoom = false;
        reference.follow_position = false;
        reference.followed_position = None;
        reference.pending_primary_page = None;
        reference.pending_primary_zoom = false;
        reference.differences.clear();
        reference.difference_target = None;
        reference.admission = Admission::Confirming {
            document: page.key.document_id.clone(),
            cancel: candidate.open,
            visible: vec![primary.id.clone(), page.key.document_id.clone()],
            submitted: false,
            blocked: false,
        };
        let document = reference.pane.active.as_ref().unwrap().0.clone();
        self.record_reader_health(
            &document,
            yamantaka_domain::quarantine::ReaderHealth::Opened,
        );
        true
    }
}
