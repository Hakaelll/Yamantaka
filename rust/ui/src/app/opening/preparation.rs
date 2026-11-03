use super::*;
use yamantaka_domain::reading_sessions::restoration::ReaderSessionRestoration;
use yamantaka_domain::{Failure, RenderedPage};

pub(super) enum Step<T> {
    Waiting,
    Submitted,
    Ready(T),
}
pub(super) struct Candidate {
    pub document: Box<Document>,
    pub info: PdfInfo,
    pub page: u32,
    pub preferences: Step<ReaderPreferences>,
    previews: Option<Vec<Preview>>,
    planned_pages: Vec<u32>,
    pub geometry: Option<(ReaderPreferences, ReadingDirection, Arc<ReaderLayout>)>,
}
impl Candidate {
    pub(super) fn cancel(&self) {
        for preview in self.previews.iter().flatten() {
            preview.cancel.cancel();
        }
    }
}
pub(super) struct Prepared {
    pub document: Box<Document>,
    pub info: PdfInfo,
    pub cancel: Cancellation,
    pub preferences: ReaderPreferences,
    pub context: Option<ReaderSessionRestoration>,
    pub previews: Vec<(RenderKey, PageImage)>,
}

impl YamantakaApp {
    pub(in crate::app) fn begin_open_preparation(
        &mut self,
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
        if info.pages.get(page as usize).is_none() {
            self.reject_open_preparation(Failure::Invalid("The PDF has no opening page".into()));
            return;
        }
        self.opening.pending.as_mut().unwrap().candidate = Some(Candidate {
            document,
            info,
            page,
            preferences: Step::Waiting,
            previews: None,
            planned_pages: Vec::new(),
            geometry: None,
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
            for preview in candidate.previews.iter_mut().flatten() {
                if matches!(preview.image, Step::Waiting) {
                    self.runtime.submit(Action::Render(
                        preview.key.clone(),
                        yamantaka_domain::Priority::Visible,
                        preview.cancel.clone(),
                    ))?;
                    preview.image = Step::Submitted;
                }
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
            && candidate.previews.as_ref().is_some_and(|previews| {
                previews
                    .iter()
                    .all(|preview| matches!(preview.image, Step::Ready(_)))
            })
        {
            let candidate = pending.candidate.take().unwrap();
            candidate.cancel();
            let Step::Ready(preferences) = candidate.preferences else {
                unreachable!()
            };
            let previews = candidate
                .previews
                .unwrap()
                .into_iter()
                .map(|preview| {
                    let Step::Ready(image) = preview.image else {
                        unreachable!()
                    };
                    (preview.key, image)
                })
                .collect();
            let prepared = Prepared {
                document: candidate.document,
                info: candidate.info,
                previews,
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
        let preview = self
            .opening
            .pending
            .as_mut()
            .and_then(|pending| pending.candidate.as_mut())
            .and_then(|candidate| candidate.previews.as_mut())
            .and_then(|previews| {
                previews
                    .iter_mut()
                    .find(|preview| preview.cancel.same_request(cancel))
            });
        let Some(preview) = preview else {
            return false;
        };
        if cancel.is_cancelled()
            || page.key != preview.key
            || !matches!(preview.image, Step::Submitted)
        {
            return true;
        }
        match render_updates::upload_image(ctx, &mut self.presenter, &mut self.managed_pages, page)
        {
            Ok(image) => preview.image = Step::Ready(image),
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
                    candidate
                        .previews
                        .iter()
                        .flatten()
                        .any(|preview| preview.cancel.same_request(token))
                        || pending.preferences.same_request(token)
                })
            })
        });
        if owns {
            self.reject_open_preparation(error.clone());
        }
        owns
    }
    pub(super) fn reject_open_preparation(&mut self, error: Failure) {
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

struct Preview {
    key: RenderKey,
    cancel: Cancellation,
    image: Step<PageImage>,
}
fn plan(
    candidate: &Candidate,
    pages: Vec<u32>,
    cap: u32,
    bytes: usize,
) -> yamantaka_domain::Result<Vec<Preview>> {
    let per_page = bytes / pages.len();
    if per_page < 4 {
        return Err(Failure::MemoryBudget);
    }
    pages
        .into_iter()
        .map(|page| {
            let size = candidate
                .info
                .pages
                .get(page as usize)
                .ok_or_else(|| Failure::Invalid("The opening page is outside the PDF".into()))?
                .size();
            let dimensions = preview_dimensions(egui::vec2(size[0], size[1]), 1.0, cap, per_page);
            Ok(Preview {
                key: RenderKey {
                    document_id: candidate.document.id.clone(),
                    page,
                    region: None,
                    physical_width: dimensions[0],
                    physical_height: dimensions[1],
                    rotation: 0,
                    generation: 0,
                },
                cancel: Cancellation::default(),
                image: Step::Waiting,
            })
        })
        .collect()
}

impl Candidate {
    pub(super) fn plan_pages(
        &mut self,
        pages: Vec<u32>,
        cap: u32,
        bytes: usize,
    ) -> yamantaka_domain::Result<()> {
        if self.previews.is_none() || self.planned_pages != pages {
            self.cancel();
            self.previews = Some(plan(self, pages.clone(), cap, bytes)?);
            self.planned_pages = pages;
        }
        Ok(())
    }
    pub(super) fn inject_failure(&mut self) {
        if let Some(preview) = self
            .previews
            .as_mut()
            .and_then(|previews| previews.last_mut())
        {
            preview.key.page = self.info.pages.len() as u32;
        }
    }
}
