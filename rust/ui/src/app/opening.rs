use super::*;
mod adoption;
mod layout;
mod preparation;
mod preview;

struct PendingOpening {
    document: String,
    open: Cancellation,
    preferences: Cancellation,
    password: bool,
    candidate: Option<preparation::Candidate>,
    context: Option<yamantaka_domain::reading_sessions::restoration::ReaderSessionRestoration>,
}
#[derive(Default)]
pub(super) struct ReaderOpening {
    pending: Option<PendingOpening>,
    preview: ReplacementPreview,
    first_raster_probe: preview::Probe,
}
#[derive(Default, Debug, PartialEq, Eq)]
enum ReplacementPreview {
    #[default]
    Off,
    Configure,
    Settling,
    Requested,
    Complete,
}
impl ReaderOpening {
    pub(super) fn observe_frame(
        &mut self,
        document: &str,
        frame: &yamantaka_domain::reader_layout::ReaderFrame,
        surface: &reader_surface::ReaderSurface,
    ) {
        self.first_raster_probe
            .observe_frame(document, frame, surface);
    }
    pub(super) fn ready(&self) -> bool {
        !self.busy()
            && self.first_raster_probe.ready()
            && matches!(
                self.preview,
                ReplacementPreview::Off | ReplacementPreview::Complete
            )
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"pending":self.pending.as_ref().map(|pending|pending.document.as_str()),"password":self.pending.as_ref().is_some_and(|pending|pending.password),"replacementPreview":format!("{:?}",self.preview),"firstRaster":self.first_raster_probe.summary()})
    }
    pub(super) fn begin(&mut self, document: String) -> Cancellation {
        let open = Cancellation::default();
        self.bind(document, open.clone());
        open
    }
    pub(super) fn bind(&mut self, document: String, open: Cancellation) {
        self.cancel();
        self.pending = Some(PendingOpening {
            document,
            open,
            preferences: Cancellation::default(),
            password: false,
            candidate: None,
            context: None,
        });
    }
    pub(super) fn matches(&self, document: &str, open: &Cancellation) -> bool {
        self.pending
            .as_ref()
            .is_some_and(|pending| pending.document == document && pending.open.same_request(open))
    }
    pub(super) fn busy(&self) -> bool {
        self.pending
            .as_ref()
            .is_some_and(|pending| !pending.password)
    }
    pub(super) fn token(&self) -> Option<Cancellation> {
        self.pending.as_ref().map(|pending| pending.open.clone())
    }
    pub(super) fn arm_context(
        &mut self,
        context: yamantaka_domain::reading_sessions::restoration::ReaderSessionRestoration,
    ) {
        if let Some(pending) = &mut self.pending {
            pending.context = Some(context);
        }
    }
    pub(super) fn accept(&mut self, document: &str, open: &Cancellation) -> Option<Cancellation> {
        if open.is_cancelled() || !self.matches(document, open) {
            return None;
        }
        self.pending.take().map(|pending| {
            if let Some(candidate) = pending.candidate {
                candidate.cancel();
            }
            pending.preferences
        })
    }
    pub(super) fn fail(&mut self, document: &str, open: &Cancellation, password: bool) -> bool {
        if !self.matches(document, open) {
            return false;
        }
        if password && !open.is_cancelled() {
            self.pending.as_mut().unwrap().password = true;
        } else {
            self.cancel();
        }
        true
    }
    pub(super) fn retry(&mut self, document: String) -> Cancellation {
        if let Some(pending) = self.pending.as_mut()
            && pending.document == document
        {
            pending.open.cancel();
            pending.open = Cancellation::default();
            pending.password = false;
            pending.open.clone()
        } else {
            self.begin(document)
        }
    }
    pub(super) fn cancel(&mut self) {
        if let Some(pending) = self.pending.take() {
            pending.open.cancel();
            pending.preferences.cancel();
            if let Some(candidate) = pending.candidate {
                candidate.cancel();
            }
        }
    }
}
impl Drop for ReaderOpening {
    fn drop(&mut self) {
        self.cancel();
    }
}

impl YamantakaApp {
    pub fn preview_failed_replacement(&mut self) {
        if self.demo && self.capture.is_some() {
            self.preview_reader = true;
            self.opening.preview = ReplacementPreview::Configure;
        }
    }
    pub(super) fn advance_replacement_preview(&mut self) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        match self.opening.preview {
            ReplacementPreview::Configure
                if self.reader.active.is_some()
                    && self.reader.reader_preferences.is_some()
                    && !self.opening.busy() =>
            {
                let mut preferences = self.reader.reader_preferences.clone().unwrap();
                preferences.zoom_mode = ZoomMode::Custom;
                preferences.zoom_level = 1.75;
                preferences.page_layout = yamantaka_domain::PageLayout::Continuous;
                preferences.reading_theme = ReaderTheme::Sepia;
                preferences.scroll_offset = 1500.0;
                self.apply_reader_preferences(preferences);
                self.navigate(1);
                self.reader.restoration.scroll = Some(1500.0);
                self.opening.preview = ReplacementPreview::Settling;
            }
            ReplacementPreview::Settling
                if !self.reader.loading
                    && self.reader.restoration.ready()
                    && self
                        .reader
                        .viewport
                        .pages
                        .values()
                        .any(|slot| slot.image.is_some()) =>
            {
                if let Some(mut candidate) = self.documents.get(1).cloned() {
                    candidate.path =
                        std::path::PathBuf::from("synthetic-unavailable-replacement.pdf");
                    if self.open(candidate) {
                        self.opening.preview = ReplacementPreview::Requested;
                    }
                }
            }
            ReplacementPreview::Requested
                if self.opening.pending.is_none() && self.error.is_some() =>
            {
                self.opening.preview = ReplacementPreview::Complete;
            }
            _ => {}
        }
    }
    pub(super) fn replace_reader_state(&mut self, preferences: Cancellation) {
        self.reference = None;
        self.reader.prepare_replacement(preferences);
        self.dismiss_password_challenge();
        self.password_attempted = false;
    }
    pub(super) fn cancel_reader_opening(&mut self) {
        self.tabs.cancel();
        self.opening.cancel();
        self.reader.restoration.clear();
        self.open_intent.leave_destination();
        self.dismiss_password_challenge();
        self.password_attempted = false;
        self.reader.loading = false;
    }
}
