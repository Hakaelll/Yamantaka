use super::*;
use yamantaka_application::cover_regeneration::RegenerationRequest;

impl YamantakaApp {
    pub(in crate::app) fn cover_batch_review_open(&self) -> bool {
        self.library_selection.cover_review.is_some()
    }
    pub(in crate::app) fn open_cover_batch_review(&mut self) {
        if !self.library_selection.writing() && !self.library_selection.model.ids().is_empty() {
            self.library_selection.cover_review = Some(CoverReview {
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
    pub(in crate::app) fn cover_batch_review_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut review) = self.library_selection.cover_review.take() else {
            return;
        };
        let p = Palette::new(self.dark);
        let compact = ctx.content_rect().height() < 420.0;
        let mut approved = false;
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::new("cover-batch-review"))
            .frame(egui::Frame::new().fill(p.surface).stroke(egui::Stroke::new(1.0,p.line))
                .corner_radius(12).inner_margin(if compact {12} else {24}))
            .show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width()-56.0).clamp(160.0,420.0));
                ui.spacing_mut().item_spacing.y = if compact {8.0} else {16.0};
                theme::display(ui,self.tr("Portadas automáticas","Automatic covers"),if compact {20.0} else {24.0});
                let count = format!("{} {}",review.ids.len(),if review.ids.len()==1 {
                    self.tr("libro seleccionado.","selected book.")
                } else {self.tr("libros seleccionados.","selected books.")});
                ui.label(format!("{count} {}", self.tr(
                    "Sustituye sus portadas manuales. Los PDF no cambian.",
                    "Replaces their manual covers. PDF files stay unchanged.")))
                    .on_hover_text(self.tr("Cualquier elección posterior se conserva.","Later choices are kept."));
                if !compact {
                    ui.label(RichText::new(self.tr(
                        "Las elecciones posteriores se conservan. Cada portada sigue visible hasta tener su sustitución. Puedes cancelar los libros pendientes.",
                        "Later choices are kept. Each cover stays visible until its replacement is ready. You can cancel the remaining books."
                    )).color(p.secondary));
                }
                ui.horizontal_wrapped(|ui| {
                    let cancel = theme::navigation(ui,self.tr("Cancelar","Cancel"),false,p);
                    if review.focus_cancel {cancel.request_focus();review.focus_cancel=false;}
                    close = cancel.clicked();
                    approved = theme::control(ui,self.tr("Regenerar","Regenerate"),None,true,false,p).clicked();
                });
            });
        close |= modal.should_close();
        if approved {
            self.submit_cover_batch(review.ids);
        } else if !close {
            self.library_selection.cover_review = Some(review);
        }
    }
}
