use super::*;
use yamantaka_application::cover_regeneration::RegenerationRequest;

impl YamantakaApp {
    pub(in crate::app) fn selection_review_open(&self) -> bool {
        self.library_selection.review.is_some()
    }
    pub(in crate::app) fn open_cover_batch_review(&mut self) {
        if !self.library_selection.writing() && !self.library_selection.model.ids().is_empty() {
            self.library_selection.review = Some(SelectionReview {
                action: ReviewAction::Covers,
                ids: self.library_selection.model.ids().to_vec(),
                focus_cancel: true,
            });
        }
    }
    pub(in crate::app) fn submit_cover_batch(&mut self, ids: Vec<String>) -> bool {
        if self.library_selection.writing() || ids.is_empty() {
            return false;
        }
        self.library_covers.pause(&ids);
        self.runtime.wake();
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::RegenerateCovers(Box::new(RegenerationRequest {
                ids: ids.clone(),
                cancel: cancel.clone(),
            }))) {
            Ok(()) => {
                self.library_selection.pending = Some(PendingBulk {
                    cancel,
                    action: PendingAction::Covers {
                        ids,
                        refreshed: Default::default(),
                    },
                });
                self.library_selection.notice = None;
                true
            }
            Err(error) => {
                self.library_covers.resume(&ids);
                self.library_selection.notice = Some(error.to_string());
                false
            }
        }
    }
    pub(in crate::app) fn update_cover_batch_progress(&mut self) {
        let Some(pending) = &mut self.library_selection.pending else {
            return;
        };
        let PendingAction::Covers { refreshed, .. } = &mut pending.action else {
            return;
        };
        let Some(progress) = self
            .runtime
            .cover_regeneration_progress()
            .filter(|progress| progress.cancel.same_request(&pending.cancel))
        else {
            return;
        };
        let ready: Vec<_> = progress
            .report
            .succeeded
            .into_iter()
            .filter(|id| refreshed.insert(id.clone()))
            .collect();
        self.library_covers.resume(&ready);
    }
    pub(super) fn cover_batch_progress(&self) -> Option<String> {
        let pending = self.library_selection.pending.as_ref()?;
        let PendingAction::Covers { ids, .. } = &pending.action else {
            return None;
        };
        let progress = self
            .runtime
            .cover_regeneration_progress()
            .filter(|progress| progress.cancel.same_request(&pending.cancel));
        let done = progress.as_ref().map_or(0, |progress| {
            progress.report.succeeded.len() + progress.report.failed.len()
        });
        Some(format!(
            "{} {done}/{}",
            self.tr("Portadas", "Covers"),
            ids.len()
        ))
    }
}
