use super::*;
use yamantaka_domain::reading_sessions::restoration::ReaderSessionRestoration;
use yamantaka_domain::{Failure, RenderedPage};

enum Step<T> {
    Waiting,
    Submitted,
    Ready(T),
}
pub(super) struct Candidate {
    document: Box<Document>,
    info: PdfInfo,
    key: RenderKey,
    render: Cancellation,
    preferences: Step<ReaderPreferences>,
    image: Step<PageImage>,
}
impl Candidate {
    pub(super) fn cancel(&self) {
        self.render.cancel();
    }
}
pub(super) struct Prepared {
    pub document: Box<Document>,
    pub info: PdfInfo,
    pub cancel: Cancellation,
    pub preferences: ReaderPreferences,
    pub context: Option<ReaderSessionRestoration>,
    pub key: RenderKey,
    pub image: PageImage,
}

impl YamantakaApp {
    pub(in crate::app) fn begin_open_preparation(
        &mut self,
        ctx: &egui::Context,
        document: Box<Document>,
        info: PdfInfo,
        cancel: Cancellation,
    ) {
        if cancel.is_cancelled() || !self.opening.matches(&document.id, &cancel) {
            cancel.cancel();
            self.open_confirmation = Some((document.id.clone(), cancel));
            return;
        }
        let page = self
            .open_intent
            .pending_page(&document.id)
            .unwrap_or(document.last_page_index)
            .min(info.pages.len().saturating_sub(1) as u32);
        let Some(size) = info.pages.get(page as usize).map(|page| page.size()) else {
            self.reject_open_preparation(Failure::Invalid("The PDF has no opening page".into()));
            return;
        };
        let cap = (ctx.input(|input| input.max_texture_side) as u32).min(768);
        let dimensions = preview_dimensions(
            egui::vec2(size[0], size[1]),
            1.0,
            cap,
            (self.governor.limits().pages / 6).clamp(4, 2 * 1024 * 1024),
        );
        let mut key = RenderKey {
            document_id: document.id.clone(),
            page,
            region: None,
            physical_width: dimensions[0],
            physical_height: dimensions[1],
            rotation: 0,
            generation: 0,
        };
        if self.opening.first_raster_probe.inject_failure(&document.id) {
            key.page = info.pages.len() as u32;
        }
        self.opening.pending.as_mut().unwrap().candidate = Some(Candidate {
            document,
            info,
            key,
            render: Cancellation::default(),
            preferences: Step::Waiting,
            image: Step::Waiting,
        });
    }

    pub(in crate::app) fn pump_open_preparation(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.opening.pending.as_mut() else {
            return;
        };
        let Some(candidate) = pending.candidate.as_mut() else {
            return;
        };
        if pending.open.is_cancelled() {
            self.reject_open_preparation(Failure::Cancelled);
            return;
        }
        let result = (|| {
            if matches!(candidate.preferences, Step::Waiting) {
                self.runtime.submit(Action::LoadReaderPreferences(
                    pending.document.clone(),
                    pending.preferences.clone(),
                ))?;
                candidate.preferences = Step::Submitted;
            }
            if matches!(candidate.image, Step::Waiting) {
                self.runtime.submit(Action::Render(
                    candidate.key.clone(),
                    yamantaka_domain::Priority::Visible,
                    candidate.render.clone(),
                ))?;
                candidate.image = Step::Submitted;
            }
            yamantaka_domain::Result::Ok(())
        })();
        if let Err(error) = result {
            if matches!(error, Failure::QueueFull) {
                ctx.request_repaint_after(Duration::from_millis(100));
            } else {
                self.reject_open_preparation(error);
            }
            return;
        }
        if matches!(candidate.preferences, Step::Ready(_))
            && matches!(candidate.image, Step::Ready(_))
        {
            let candidate = pending.candidate.take().unwrap();
            candidate.render.cancel();
            let (Step::Ready(preferences), Step::Ready(image)) =
                (candidate.preferences, candidate.image)
            else {
                unreachable!()
            };
            let prepared = Prepared {
                document: candidate.document,
                info: candidate.info,
                key: candidate.key,
                image,
                preferences,
                context: pending.context.take(),
                cancel: pending.open.clone(),
            };
            self.adopt_prepared_open(prepared);
        }
    }

    pub(in crate::app) fn accept_open_preparation_preferences(
        &mut self,
        id: &str,
        preferences: Option<ReaderPreferences>,
        cancel: &Cancellation,
    ) -> bool {
        let Some(pending) =
            self.opening.pending.as_mut().filter(|pending| {
                pending.document == id && pending.preferences.same_request(cancel)
            })
        else {
            return false;
        };
        let Some(candidate) = pending.candidate.as_mut() else {
            return false;
        };
        if !cancel.is_cancelled() && matches!(candidate.preferences, Step::Submitted) {
            candidate.preferences =
                Step::Ready(preferences.unwrap_or_else(|| ReaderPreferences::new(id)));
        }
        true
    }

    pub(in crate::app) fn accept_open_preparation_image(
        &mut self,
        ctx: &egui::Context,
        page: &RenderedPage,
        cancel: &Cancellation,
    ) -> bool {
        let Some(candidate) = self
            .opening
            .pending
            .as_ref()
            .and_then(|pending| pending.candidate.as_ref())
            .filter(|candidate| candidate.render.same_request(cancel))
        else {
            return false;
        };
        if cancel.is_cancelled()
            || page.key != candidate.key
            || !matches!(candidate.image, Step::Submitted)
        {
            return true;
        }
        match render_updates::upload_image(ctx, &mut self.presenter, &mut self.managed_pages, page)
        {
            Ok(image) => {
                self.opening
                    .pending
                    .as_mut()
                    .unwrap()
                    .candidate
                    .as_mut()
                    .unwrap()
                    .image = Step::Ready(image)
            }
            Err(error) => self.reject_open_preparation(error),
        }
        true
    }

    pub(in crate::app) fn fail_open_preparation(
        &mut self,
        error: &Failure,
        cancel: Option<&Cancellation>,
    ) -> bool {
        let owns = cancel.is_some_and(|token| {
            self.opening.pending.as_ref().is_some_and(|pending| {
                pending.candidate.as_ref().is_some_and(|candidate| {
                    candidate.render.same_request(token) || pending.preferences.same_request(token)
                })
            })
        });
        if owns {
            self.reject_open_preparation(error.clone());
        }
        owns
    }
    fn reject_open_preparation(&mut self, error: Failure) {
        let Some(pending) = self.opening.pending.as_ref() else {
            return;
        };
        let id = pending.document.clone();
        let cancel = pending.open.clone();
        self.tabs.fail(&id, &cancel);
        self.opening.fail(&id, &cancel, false);
        cancel.cancel();
        self.open_confirmation = Some((id, cancel));
        self.reader.loading = false;
        if !matches!(error, Failure::Cancelled) {
            self.error = Some(error.to_string());
        }
    }
}
