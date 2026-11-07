use super::*;
use yamantaka_application::reader_close::{ReaderCloseReply, ReaderCloseRequest};

pub(in crate::app) enum Admission {
    Ready,
    Opening {
        document: String,
        cancel: Cancellation,
    },
    Password {
        document: String,
    },
    Preparing(Box<preparation::Candidate>),
    Confirming {
        document: String,
        cancel: Cancellation,
        visible: Vec<String>,
        submitted: bool,
        blocked: bool,
    },
    Closing {
        document: String,
        cancel: Cancellation,
    },
}

impl ReferenceView {
    pub(in crate::app) fn busy(&self) -> bool {
        !matches!(self.admission, Admission::Ready)
    }
}

impl YamantakaApp {
    pub(in crate::app) fn reference_blocks_switch(&mut self) -> bool {
        let blocked = self.reference.as_ref().is_some_and(|reference| {
            reference.busy()
                || reference.pane.active.as_ref().map(|(doc, _)| &doc.id)
                    != self.reader.active.as_ref().map(|(doc, _)| &doc.id)
        });
        if blocked {
            self.error = Some(
                self.tr(
                    "Cierra la referencia antes de cambiar de lectura.",
                    "Close the reference before switching readings.",
                )
                .into(),
            );
        }
        blocked
    }

    pub(in crate::app) fn open_reference(&mut self, document: Document) {
        let context = self
            .reference
            .as_ref()
            .and_then(|reference| reference.retry_open.as_ref())
            .filter(|request| request.document.id == document.id)
            .and_then(|request| request.context.clone());
        self.open_reference_with_context(document, context);
    }
    pub(in crate::app) fn open_reference_with_context(
        &mut self,
        document: Document,
        context: Option<yamantaka_domain::workspaces::WorkspaceComparison>,
    ) {
        if self.opening.busy()
            || self.password.is_some()
            || self.tabs.busy()
            || self.open_confirmation.is_some()
            || self.reader.reader_preferences_loading
        {
            return;
        }
        let Some(reference) = &mut self.reference else {
            return;
        };
        if let Some(saved) = &context {
            if let Err(error) = saved.validate() {
                reference.error = Some(error.to_string());
                return;
            }
            if saved.document != document.id
                || self
                    .reader
                    .active
                    .as_ref()
                    .is_none_or(|(primary, _)| primary.id != saved.primary)
            {
                reference.error = Some("The saved comparison belongs to another reading".into());
                return;
            }
        }
        if reference.pane.annotations.editing() && !reference.pane.annotations.writing() {
            self.error = Some(
                self.tr(
                    "Guarda o cancela la nota antes de cambiar de referencia.",
                    "Save or cancel the note before changing the reference.",
                )
                .into(),
            );
            return;
        }
        if reference.busy()
            || reference.pane.active.as_ref().is_none_or(|(doc, _)| {
                self.reader
                    .active
                    .as_ref()
                    .is_none_or(|(primary, _)| primary.id != doc.id)
            })
        {
            return;
        }
        if reference
            .pane
            .active
            .as_ref()
            .is_some_and(|(doc, _)| doc.id == document.id)
        {
            return;
        }
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::Open(document.clone(), None, cancel.clone()))
        {
            Ok(()) => {
                reference.error = None;
                reference.password_attempted = false;
                reference.retry_open = Some(preparation::OpenRequest {
                    document: document.clone(),
                    context,
                });
                reference.admission = Admission::Opening {
                    document: document.id,
                    cancel,
                };
            }
            Err(error) => reference.error = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn accept_reference_opened(
        &mut self,
        ctx: &egui::Context,
        document: &Document,
        info: &PdfInfo,
        token: &Cancellation,
    ) -> bool {
        let Some(reference) = &mut self.reference else {
            return false;
        };
        if !matches!(&reference.admission, Admission::Opening {document: id, cancel} if id == &document.id && cancel.same_request(token))
        {
            return false;
        }
        let Some((primary, _)) = &self.reader.active else {
            return false;
        };
        let visible = vec![primary.id.clone()];
        if !token.is_cancelled() {
            let count = u32::try_from(info.pages.len()).unwrap_or(0);
            let navigation =
                ReaderNavigation::new(count, document.last_page_index.min(count.saturating_sub(1)));
            match navigation {
                Ok(navigation) => {
                    let mut pane = reader_pane::ReaderPane::default();
                    pane.visual_profile = self.reader.visual_profile;
                    pane.page = navigation.position().page();
                    pane.reader_panel_tab = reference.pane.reader_panel_tab;
                    pane.navigation = Some(navigation);
                    pane.active = Some((document.clone(), Arc::new(info.clone())));
                    let mut preferences = ReaderPreferences::new(&document.id);
                    preferences.zoom_mode = ZoomMode::FitPage;
                    preferences.page_layout = PageLayout::Single;
                    preferences.view_mode = ReaderMode::Document;
                    if let Some(source) = &self.reader.reader_preferences {
                        preferences.reading_theme = source.reading_theme;
                    }
                    pane.reader_preferences = Some(preferences);
                    let direction = if self
                        .preferences
                        .get("readingDirection")
                        .and_then(serde_json::Value::as_str)
                        == Some("rightToLeft")
                    {
                        ReadingDirection::Rtl
                    } else {
                        ReadingDirection::Ltr
                    };
                    match pane.rebuild_layout(direction) {
                        Ok(()) => {
                            if self.capture.is_some() {
                                reference.opening_probe.get_or_insert_with(Default::default);
                            }
                            match preparation::Candidate::new(
                                pane,
                                token.clone(),
                                (ctx.input(|input| input.max_texture_side) as u32).min(768),
                                (self.governor.limits().pages / 12).clamp(4, 2 * 1024 * 1024),
                                direction,
                                reference
                                    .retry_open
                                    .as_ref()
                                    .and_then(|request| request.context.as_ref()),
                            ) {
                                Ok(mut candidate) => {
                                    if self.demo
                                        && self.capture.is_some()
                                        && self.reference_prepare_preview
                                            == preparation::Preview::FailOnce
                                    {
                                        candidate.key.page = info.pages.len() as u32;
                                        self.reference_prepare_preview =
                                            preparation::Preview::Retry;
                                    }
                                    reference.admission = Admission::Preparing(Box::new(candidate));
                                    return true;
                                }
                                Err(error) => {
                                    reference.error = Some(error.to_string());
                                    token.cancel();
                                }
                            }
                        }
                        Err(error) => {
                            reference.error = Some(error.to_string());
                            token.cancel();
                        }
                    }
                }
                Err(error) => {
                    reference.error = Some(error.to_string());
                    token.cancel();
                }
            }
        }
        reference.admission = Admission::Confirming {
            document: document.id.clone(),
            cancel: token.clone(),
            visible,
            submitted: false,
            blocked: false,
        };
        true
    }

    pub(in crate::app) fn accept_reference_failure(
        &mut self,
        document: &Document,
        token: &Cancellation,
        error: &yamantaka_domain::Failure,
    ) -> bool {
        let Some(reference) = &mut self.reference else {
            return false;
        };
        if !matches!(&reference.admission, Admission::Opening {document: id, cancel} if id == &document.id && cancel.same_request(token))
        {
            return false;
        }
        reference.admission = Admission::Ready;
        if token.is_cancelled() || reference.close_requested {
            return true;
        }
        if matches!(error, yamantaka_domain::Failure::PasswordRequired) {
            reference.admission = Admission::Password {
                document: document.id.clone(),
            };
            self.password = Some(crate::app::password::PasswordChallenge::reference(
                document.clone(),
                reference.password_attempted,
            ));
            return true;
        }
        if !matches!(error, yamantaka_domain::Failure::Cancelled) {
            reference.error = Some(error.to_string());
        }
        true
    }

    pub(in crate::app) fn accept_reference_confirmation(
        &mut self,
        document: &str,
        token: &Cancellation,
        result: &yamantaka_domain::Result<Option<Vec<String>>>,
    ) -> bool {
        let Some(reference) = &mut self.reference else {
            return false;
        };
        let Admission::Confirming {
            document: id,
            cancel,
            visible,
            submitted,
            blocked,
        } = &mut reference.admission
        else {
            return false;
        };
        if id != document || !cancel.same_request(token) {
            return false;
        }
        match result {
            Ok(Some(ids)) if token.is_cancelled() || ids == visible => {
                reference.admission = Admission::Ready
            }
            Err(yamantaka_domain::Failure::QueueFull) => *submitted = false,
            Err(error) => {
                *blocked = true;
                reference.error = Some(error.to_string());
            }
            _ => {
                *blocked = true;
                reference.error =
                    Some("Reference opening was superseded; retry confirmation".into());
            }
        }
        true
    }

    pub(in crate::app) fn request_reference_close(&mut self) -> bool {
        let closed = self.close_reference_preserving_continuity();
        if self
            .reference
            .as_ref()
            .is_none_or(|reference| reference.close_requested)
        {
            self.discard_retained_comparison();
        }
        closed
    }
    pub(in crate::app) fn close_reference_preserving_continuity(&mut self) -> bool {
        self.reference_mount = None;
        let Some(reference) = &mut self.reference else {
            return true;
        };
        if reference.pane.study.has_unsaved() {
            self.reading_focus.target = shortcuts::ReadingTarget::Reference;
            return false;
        }
        if reference.pane.annotations.editing() && !reference.pane.annotations.writing() {
            reference.pane.reader_panel_tab = outline::ReaderPanelTab::Annotations;
            reference.navigator_requested = true;
            self.error = Some(
                self.tr(
                    "Guarda o cancela la nota antes de cerrar la referencia.",
                    "Save or cancel the note before closing the reference.",
                )
                .into(),
            );
            return false;
        }
        if let Some(write) = reference.pane.annotations.retire_write() {
            self.retired_annotation_writes.push(write);
        }
        reference.close_requested = true;
        reference.pane.find.stop();
        reference.pane.links.clear();
        reference.pane.outline.cancel();
        reference.pane.thumbnails.clear();
        reference.pane.text.clear();
        reference.pane.selection.clear();
        reference.pane.annotation_paint.invalidate();
        reference.pane.bookmarks.cancel_query();
        if matches!(reference.admission, Admission::Password { .. }) {
            reference.admission = Admission::Ready;
            self.password = None;
        }
        if let Admission::Opening { cancel, .. } = &reference.admission {
            cancel.cancel();
            self.runtime.wake();
        }
        if reference.busy() {
            return false;
        }
        let same = reference.pane.active.as_ref().map(|(doc, _)| &doc.id)
            == self.reader.active.as_ref().map(|(doc, _)| &doc.id);
        if same {
            self.reference = None;
            self.reader.viewport.scroll_anchor = true;
            return true;
        }
        let document = reference.pane.active.as_ref().unwrap().0.id.clone();
        let cancel = Cancellation::default();
        match self.runtime.submit(Action::Close(ReaderCloseRequest {
            document: document.clone(),
            cancel: cancel.clone(),
        })) {
            Ok(()) => {
                reference.pane.viewport.retry = true;
                reference.admission = Admission::Closing { document, cancel };
            }
            Err(error) => {
                reference.error = Some(error.to_string());
                reference.close_requested = false;
                self.reference_return = None;
            }
        }
        false
    }

    pub(in crate::app) fn accept_reference_close(&mut self, reply: &ReaderCloseReply) -> bool {
        let Some(reference) = &mut self.reference else {
            return false;
        };
        if !matches!(&reference.admission, Admission::Closing {document, cancel} if document == &reply.document && cancel.same_request(&reply.cancel))
        {
            return false;
        }
        match &reply.result {
            Ok(()) => {
                self.reference_native_closes += 1;
                self.reference = None;
                self.reader.viewport.scroll_anchor = true;
            }
            Err(error) => {
                reference.admission = Admission::Ready;
                reference.close_requested = false;
                reference.pane.viewport.reset_requests();
                reference.error = Some(error.to_string());
                self.reference_return = None;
            }
        }
        true
    }

    pub(in crate::app) fn pump_reference(&mut self, ctx: &egui::Context) {
        self.pump_reference_preparation(ctx);
        if let Some(reference) = &mut self.reference
            && let Admission::Confirming {
                document,
                cancel,
                visible,
                submitted,
                blocked,
            } = &mut reference.admission
            && !*submitted
            && !*blocked
        {
            match self.runtime.submit(Action::ConfirmOpen {
                document: document.clone(),
                visible: visible.clone(),
                cancel: cancel.clone(),
            }) {
                Ok(()) => *submitted = true,
                Err(yamantaka_domain::Failure::QueueFull) => {
                    ctx.request_repaint_after(Duration::from_millis(100))
                }
                Err(error) => {
                    *blocked = true;
                    reference.error = Some(error.to_string());
                }
            }
        }
        if self
            .reference
            .as_ref()
            .is_some_and(|reference| reference.close_requested && !reference.busy())
        {
            self.close_reference_preserving_continuity();
        }
        if self.reference.is_none()
            && let Some(surface) = self.reference_return.take()
        {
            self.show_surface(surface);
        }
    }
}
