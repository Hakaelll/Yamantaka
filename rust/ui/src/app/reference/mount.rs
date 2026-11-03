use super::*;
use preparation::{Candidate, Progress};
use yamantaka_domain::{Failure, RenderedPage, reader_layout::ReaderFrame};

struct Page {
    key: RenderKey,
    cancel: Cancellation,
    progress: Progress,
    image: Option<PageImage>,
}
pub(super) struct PrimaryStage {
    source: Cancellation,
    pages: std::collections::BTreeMap<u32, Page>,
}
impl PrimaryStage {
    pub(super) fn new(source: Cancellation) -> Self {
        Self {
            source,
            pages: Default::default(),
        }
    }
}
impl Drop for Page {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
pub(in crate::app) struct Mount {
    view: ReferenceView,
    source: Cancellation,
    pages: std::collections::BTreeMap<u32, Page>,
}
impl Mount {
    pub(super) fn new(
        source: &reader_pane::ReaderPane,
        direction: ReadingDirection,
        ctx: &egui::Context,
        bytes: usize,
        capture: bool,
    ) -> yamantaka_domain::Result<Self> {
        let mut view = ReferenceView::new(source, direction)?;
        let pane = std::mem::take(&mut view.pane);
        let candidate = Candidate::new(
            pane,
            Cancellation::default(),
            (ctx.input(|input| input.max_texture_side) as u32).min(768),
            (bytes / 12).clamp(4, 1024 * 1024),
            direction,
            None,
        )?;
        view.admission = Admission::Preparing(Box::new(candidate));
        if capture {
            view.opening_probe = Some(Default::default());
        }
        Ok(Self {
            view,
            source: source.reader_preferences_cancel.clone(),
            pages: Default::default(),
        })
    }
    pub(super) fn inject_failure(&mut self) {
        if let Admission::Preparing(candidate) = &mut self.view.admission {
            candidate.key.page = candidate.pane.active.as_ref().unwrap().1.pages.len() as u32;
        }
    }
    fn matches(&self, reader: &reader_pane::ReaderPane) -> bool {
        if self.source.is_cancelled()
            || !self.source.same_request(&reader.reader_preferences_cancel)
        {
            return false;
        }
        let document = match &self.view.admission {
            Admission::Preparing(candidate) => Some(candidate.key.document_id.as_str()),
            Admission::Ready => self
                .view
                .pane
                .active
                .as_ref()
                .map(|(document, _)| document.id.as_str()),
            _ => None,
        };
        reader
            .active
            .as_ref()
            .is_some_and(|(active, _)| Some(active.id.as_str()) == document)
    }
    fn owns(&self, token: &Cancellation) -> bool {
        matches!(&self.view.admission,Admission::Preparing(candidate) if candidate.render.same_request(token))
            || self
                .pages
                .values()
                .any(|page| page.cancel.same_request(token))
    }
    fn retry(&mut self, ctx: &egui::Context, token: &Cancellation) {
        if let Admission::Preparing(candidate) = &mut self.view.admission
            && candidate.render.same_request(token)
        {
            candidate.render.cancel();
            candidate.render = Cancellation::default();
            candidate.progress = Progress::Waiting(Instant::now() + Duration::from_millis(100));
        } else if let Some(page) = self
            .pages
            .values_mut()
            .find(|page| page.cancel.same_request(token))
        {
            page.cancel.cancel();
            page.cancel = Cancellation::default();
            page.progress = Progress::Waiting(Instant::now() + Duration::from_millis(100));
        }
        if let Some(probe) = &mut self.view.opening_probe {
            probe.queue_retries += 1;
        }
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn frame_summary(frame: &ReaderFrame) -> serde_json::Value {
    serde_json::json!({"viewport":frame.viewport,"scroll":frame.scroll,"zoom":frame.zoom,"visiblePages":frame.pages.iter().map(|page|page.page).collect::<Vec<_>>()})
}
impl ReferenceView {
    pub(in crate::app) fn observe_primary_mount(
        &mut self,
        frame: &ReaderFrame,
        surface: &reader_surface::ReaderSurface,
    ) {
        let Some(probe) = &mut self.opening_probe else {
            return;
        };
        let Some(plan) = &probe.primary_plan else {
            return;
        };
        if probe.primary_first.is_some() {
            return;
        }
        let mut observed = frame_summary(frame);
        for key in ["viewport", "scroll", "zoom", "visiblePages"] {
            assert_eq!(
                plan[key], observed[key],
                "Initial comparison geometry differs at {key}"
            );
        }
        observed["missingPages"] = serde_json::json!(
            frame
                .pages
                .iter()
                .filter(|page| surface
                    .pages
                    .get(&page.page)
                    .is_none_or(|slot| slot.image.is_none()))
                .map(|page| page.page)
                .collect::<Vec<_>>()
        );
        assert_eq!(observed["missingPages"], serde_json::json!([]));
        probe.primary_first = Some(observed);
    }
}

impl YamantakaApp {
    pub(in crate::app) fn prepare_saved_primary(&mut self, ui: &mut egui::Ui) {
        let Some(reference) = &mut self.reference else {
            return;
        };
        if reference.busy() || reference.close_requested {
            return;
        }
        let Some(stage) = &mut reference.primary_stage else {
            return;
        };
        if stage.source.is_cancelled()
            || !stage
                .source
                .same_request(&self.reader.reader_preferences_cancel)
        {
            self.close_reference_preserving_continuity();
            return;
        }
        if self.reader.reader_destination.is_some() {
            return;
        }
        let geometry = comparison_rects(
            ui.available_rect_before_wrap(),
            reference.layout,
            reference.ratio,
            reference.swapped,
        );
        let frame = match plan_primary(
            &self.reader,
            &mut stage.pages,
            geometry.primary,
            &self.runtime,
            ui,
            self.governor.limits().pages,
        ) {
            Ok(frame) => frame,
            Err(error) => {
                self.error = Some(error.to_string());
                self.close_reference_preserving_continuity();
                return;
            }
        };
        if stage.pages.values().any(|page| page.image.is_none()) {
            return;
        }
        if let Some(probe) = &mut reference.opening_probe {
            probe.primary_plan = Some(frame_summary(&frame));
            probe.primary_staged = stage.pages.keys().copied().collect();
        }
        for (page, slot) in &mut stage.pages {
            let mut key = slot.key.clone();
            key.generation = self.reader.viewport.generation;
            if let Some(previous) = self.reader.viewport.pages.insert(
                *page,
                ReaderPage {
                    requested: None,
                    cancel: Cancellation::default(),
                    image: slot.image.take(),
                    image_key: Some(key),
                    plan: Default::default(),
                    refinement: Default::default(),
                },
            ) {
                previous.cancel.cancel();
            }
        }
        self.reader.restoration.scroll = Some(frame.scroll[1]);
        self.reader.viewport.scroll_anchor = true;
        reference.primary_stage = None;
        reference.deferred = false;
    }
    pub(in crate::app) fn accept_saved_primary_image(
        &mut self,
        ctx: &egui::Context,
        page: &RenderedPage,
        token: &Cancellation,
    ) -> bool {
        let Some(stage) = self
            .reference
            .as_mut()
            .and_then(|reference| reference.primary_stage.as_mut())
        else {
            return false;
        };
        let Some(slot) = stage
            .pages
            .get_mut(&page.key.page)
            .filter(|slot| slot.cancel.same_request(token))
        else {
            return false;
        };
        if token.is_cancelled()
            || stage.source.is_cancelled()
            || slot.key != page.key
            || !matches!(slot.progress, Progress::Submitted)
        {
            return true;
        }
        match render_updates::upload_image(ctx, &mut self.presenter, &mut self.managed_pages, page)
        {
            Ok(image) => slot.image = Some(image),
            Err(error) => {
                self.error = Some(error.to_string());
                self.close_reference_preserving_continuity();
            }
        }
        true
    }
    pub(in crate::app) fn fail_saved_primary(
        &mut self,
        ctx: &egui::Context,
        error: &Failure,
        token: Option<&Cancellation>,
    ) -> bool {
        let Some(stage) = self
            .reference
            .as_mut()
            .and_then(|reference| reference.primary_stage.as_mut())
        else {
            return false;
        };
        let Some(slot) = stage
            .pages
            .values_mut()
            .find(|slot| token.is_some_and(|token| slot.cancel.same_request(token)))
        else {
            return false;
        };
        if matches!(error, Failure::QueueFull) && !stage.source.is_cancelled() {
            slot.cancel.cancel();
            slot.cancel = Cancellation::default();
            slot.progress = Progress::Waiting(Instant::now() + Duration::from_millis(100));
            ctx.request_repaint_after(Duration::from_millis(100));
        } else {
            if !matches!(error, Failure::Cancelled) {
                self.error = Some(error.to_string());
            }
            self.close_reference_preserving_continuity();
        }
        true
    }
    pub fn preview_reference_mount(&mut self, cancel: bool, pressure: bool) {
        if self.demo && self.capture.is_some() {
            self.reference_mount_preview = if pressure {
                PreparationPreview::PressureOnce
            } else if cancel {
                PreparationPreview::CancelAfterSubmit
            } else {
                PreparationPreview::FailOnce
            };
        }
    }
    pub(in crate::app) fn reference_mount_summary(&self) -> serde_json::Value {
        serde_json::json!({"pending":self.reference_mount.is_some(),"visible":self.reference.is_some(),"mainImage":self.reader.viewport.pages.values().any(|slot|slot.image.is_some()),"sourceCancelled":self.reader.reader_preferences_cancel.is_cancelled(),"error":self.error.is_some(),"cancelled":self.reference_mount_preview==PreparationPreview::Complete})
    }
    pub(in crate::app) fn prepare_reference_mount(&mut self, ui: &mut egui::Ui) {
        let Some(mut mount) = self.reference_mount.take() else {
            return;
        };
        if !mount.matches(&self.reader)
            || self.reference.is_some()
            || self.opening.busy()
            || !matches!(self.surface, Surface::Reader)
        {
            return;
        }
        if self.reader.reader_destination.is_some() {
            self.reference_mount = Some(mount);
            return;
        }
        let result = (|| {
            let geometry = comparison_rects(
                ui.available_rect_before_wrap(),
                mount.view.layout,
                mount.view.ratio,
                mount.view.swapped,
            );
            let frame = plan_primary(
                &self.reader,
                &mut mount.pages,
                geometry.primary,
                &self.runtime,
                ui,
                self.governor.limits().pages,
            )?;
            if let Admission::Preparing(candidate) = &mut mount.view.admission {
                preparation::submit(
                    &self.runtime,
                    ui.ctx(),
                    &candidate.key,
                    &candidate.render,
                    &mut candidate.progress,
                )?;
                if matches!(candidate.progress, Progress::Submitted)
                    && self.reference_mount_preview == PreparationPreview::PressureOnce
                {
                    let token = candidate.render.clone();
                    self.reference_mount_preview = PreparationPreview::Off;
                    mount.retry(ui.ctx(), &token);
                }
            }
            yamantaka_domain::Result::Ok(frame)
        })();
        let frame = match result {
            Ok(frame) => frame,
            Err(error) => {
                self.error = Some(error.to_string());
                return;
            }
        };
        if self.reference_mount_preview == PreparationPreview::CancelAfterSubmit
            && matches!(&mount.view.admission,Admission::Preparing(candidate) if matches!(candidate.progress,Progress::Submitted))
        {
            self.reference_mount_preview = PreparationPreview::Complete;
            return;
        }
        let ready = matches!(mount.view.admission, Admission::Ready)
            && mount.pages.values().all(|page| page.image.is_some());
        if !ready {
            self.reference_mount = Some(mount);
            return;
        }
        if !mount.matches(&self.reader) {
            return;
        }
        for (page, slot) in &mut mount.pages {
            let mut key = slot.key.clone();
            key.generation = self.reader.viewport.generation;
            if let Some(previous) = self.reader.viewport.pages.insert(
                *page,
                ReaderPage {
                    requested: None,
                    cancel: Cancellation::default(),
                    image: slot.image.take(),
                    image_key: Some(key),
                    plan: Default::default(),
                    refinement: Default::default(),
                },
            ) {
                previous.cancel.cancel();
            }
        }
        if let Some(probe) = &mut mount.view.opening_probe {
            probe.primary_plan = Some(frame_summary(&frame));
            probe.primary_staged = mount.pages.keys().copied().collect();
        }
        self.reader.restoration.scroll = Some(frame.scroll[1]);
        self.reader.viewport.scroll_anchor = true;
        self.discard_retained_comparison();
        self.reference = Some(mount.view);
    }
    pub(in crate::app) fn fail_reference_mount(
        &mut self,
        ctx: &egui::Context,
        error: &Failure,
        token: Option<&Cancellation>,
    ) -> bool {
        let Some(mount) = &mut self.reference_mount else {
            return false;
        };
        let Some(token) = token.filter(|token| mount.owns(token)) else {
            return false;
        };
        if matches!(error, Failure::QueueFull) && mount.matches(&self.reader) {
            mount.retry(ctx, token);
        } else {
            self.reference_mount = None;
            if !matches!(error, Failure::Cancelled) {
                self.error = Some(error.to_string());
            }
        }
        true
    }
    pub(in crate::app) fn accept_reference_mount_image(
        &mut self,
        ctx: &egui::Context,
        page: &RenderedPage,
        token: &Cancellation,
    ) -> bool {
        let Some(mount) = &self.reference_mount else {
            return false;
        };
        if !mount.owns(token) {
            return false;
        }
        if token.is_cancelled() || !mount.matches(&self.reader) {
            self.reference_mount = None;
            return true;
        }
        let reference = matches!(&mount.view.admission,Admission::Preparing(candidate) if candidate.render.same_request(token) && candidate.key==page.key && matches!(candidate.progress,Progress::Submitted));
        let primary = mount.pages.get(&page.key.page).is_some_and(|slot| {
            slot.cancel.same_request(token)
                && slot.key == page.key
                && matches!(slot.progress, Progress::Submitted)
        });
        if !reference && !primary {
            return true;
        }
        let image = match render_updates::upload_image(
            ctx,
            &mut self.presenter,
            &mut self.managed_pages,
            page,
        ) {
            Ok(image) => image,
            Err(error) => {
                self.reference_mount = None;
                self.error = Some(error.to_string());
                return true;
            }
        };
        let mount = self.reference_mount.as_mut().unwrap();
        if token.is_cancelled() || !mount.matches(&self.reader) {
            self.reference_mount = None;
            return true;
        }
        if primary {
            mount.pages.get_mut(&page.key.page).unwrap().image = Some(image);
        } else {
            let Admission::Preparing(candidate) =
                std::mem::replace(&mut mount.view.admission, Admission::Ready)
            else {
                unreachable!()
            };
            let (pane, open) = candidate.ready(image);
            open.cancel();
            if let Some(probe) = &mut mount.view.opening_probe {
                probe.adopted(&pane);
            }
            mount.view.pane = pane;
        }
        true
    }
}

fn plan_primary(
    reader: &reader_pane::ReaderPane,
    pages: &mut std::collections::BTreeMap<u32, Page>,
    rect: egui::Rect,
    runtime: &Runtime,
    ui: &mut egui::Ui,
    bytes: usize,
) -> yamantaka_domain::Result<ReaderFrame> {
    let layout = reader
        .viewport
        .layout
        .as_ref()
        .ok_or_else(|| Failure::Invalid("The primary reader layout is unavailable".into()))?;
    let preferences = reader
        .reader_preferences
        .as_ref()
        .ok_or_else(|| Failure::Invalid("The primary reader preferences are unavailable".into()))?;
    let navigation = reader
        .navigation
        .as_ref()
        .ok_or_else(|| Failure::Invalid("The primary reader position is unavailable".into()))?;
    let (document, info) = reader.active.as_ref().unwrap();
    let size = rect.size();
    let viewport = [f64::from(size.x.max(1.0)), f64::from(size.y.max(1.0))];
    let anchor = navigation.position();
    let zoom = f64::from(reader.zoom);
    let [x, y] = layout.scroll_for_anchor(viewport, anchor, preferences.zoom_mode, zoom)?;
    let y = reader.restoration.scroll.unwrap_or(y);
    let frame = layout.frame(
        viewport,
        anchor,
        preferences.zoom_mode,
        zoom,
        [f64::from(x as f32), f64::from(y as f32)],
        0.0,
    )?;
    let mut missing: Vec<_> = frame
        .pages
        .iter()
        .filter(|page| {
            reader
                .viewport
                .pages
                .get(&page.page)
                .is_none_or(|slot| slot.image.is_none())
        })
        .map(|page| page.page)
        .collect();
    missing.sort_unstable();
    missing.dedup();
    if pages.keys().copied().ne(missing.iter().copied()) {
        pages.clear();
        let budget = (bytes / 12).clamp(4, 1024 * 1024) / missing.len().max(1);
        if budget < 4 {
            return Err(Failure::MemoryBudget);
        }
        let cap = (ui.input(|input| input.max_texture_side) as u32).min(768);
        for page in missing {
            let size = info
                .pages
                .get(page as usize)
                .ok_or_else(|| Failure::Invalid("A comparison page is outside the PDF".into()))?
                .size();
            let pixels = preview_dimensions(egui::vec2(size[0], size[1]), 1.0, cap, budget);
            pages.insert(
                page,
                Page {
                    key: RenderKey {
                        document_id: document.id.clone(),
                        page,
                        region: None,
                        physical_width: pixels[0],
                        physical_height: pixels[1],
                        rotation: 0,
                        generation: 0,
                    },
                    cancel: Cancellation::default(),
                    progress: Progress::Waiting(Instant::now()),
                    image: None,
                },
            );
        }
    }
    for page in pages.values_mut().filter(|page| page.image.is_none()) {
        preparation::submit(
            runtime,
            ui.ctx(),
            &page.key,
            &page.cancel,
            &mut page.progress,
        )?;
    }
    Ok(frame)
}
