pub(super) mod preview;
use super::*;
use yamantaka_application::{library_bulk::BulkReport, library_removal::RemovalRequest};

impl YamantakaApp {
    fn removal_available(&mut self, ids: &[String]) -> bool {
        if self.tabs.busy()
            || self.opening.busy()
            || self.global_notes.writing()
            || self.annotations.writing()
            || self.annotations.editing()
            || self.active.as_ref().is_some_and(|(document, _)| {
                ids.contains(&document.id) && self.reader_preferences_loading
            })
        {
            self.library_selection.notice = Some(
                self.tr(
                    "Termina la edición o el cambio de lectura antes de retirar libros.",
                    "Finish editing or switching readings before removing books.",
                )
                .into(),
            );
            return false;
        }
        true
    }
    pub(in crate::app) fn open_library_removal_review(&mut self) {
        self.review_removal_ids(self.library_selection.model.ids().to_vec());
    }
    pub(in crate::app) fn review_removal_ids(&mut self, ids: Vec<String>) {
        if !self.library_selection.writing() && !ids.is_empty() && self.removal_available(&ids) {
            if self.library_selection.model.ids() != ids.as_slice() {
                self.library_selection.model.all(&ids);
            }
            self.library_selection.review = Some(SelectionReview {
                action: ReviewAction::Remove,
                ids,
                focus_cancel: true,
            });
        }
    }
    pub(in crate::app) fn prepare_library_removal(&mut self, ids: Vec<String>) -> bool {
        if self.library_selection.writing() || ids.is_empty() || !self.removal_available(&ids) {
            return false;
        }
        self.sync_tab_context();
        self.cache_reading_context();
        if self
            .sessions
            .lifecycle
            .current_document()
            .is_some_and(|id| ids.iter().any(|selected| selected == id))
            && let Err(error) = self.finish_reading_session()
        {
            self.library_selection.notice = Some(error.to_string());
            return false;
        }
        self.library_selection.pending = Some(PendingBulk {
            cancel: Cancellation::default(),
            action: PendingAction::Removal {
                ids,
                phase: RemovalPhase::Preparing,
            },
        });
        self.library_selection.notice = None;
        true
    }
    pub(in crate::app) fn pump_library_removal(&mut self, ctx: &egui::Context) {
        let Some(pending) = &self.library_selection.pending else {
            return;
        };
        let PendingAction::Removal { ids, phase } = &pending.action else {
            return;
        };
        if matches!(phase, RemovalPhase::Preparing) {
            let ready = if pending.cancel.is_cancelled() {
                Err(yamantaka_domain::Failure::Cancelled)
            } else {
                self.sessions.lifecycle.settled_for(ids)
            };
            if matches!(ready, Ok(false)) {
                return;
            }
            let pending = self.library_selection.pending.take().unwrap();
            let PendingAction::Removal { ids, .. } = &pending.action else {
                unreachable!()
            };
            if let Err(error) = ready {
                self.library_selection.pending = Some(pending);
                self.accept_library_bulk(
                    ctx,
                    BulkReply {
                        cancel: self
                            .library_selection
                            .pending
                            .as_ref()
                            .unwrap()
                            .cancel
                            .clone(),
                        result: Err(error),
                    },
                );
                return;
            }
            self.library_covers.pause(ids);
            self.runtime.wake();
            if self
                .active
                .as_ref()
                .is_some_and(|(document, _)| ids.contains(&document.id))
            {
                self.cancel_reader_opening();
                self.replace_reader_state(Cancellation::default());
                self.reader_preferences_loading = false;
                self.active = None;
                self.navigation = None;
                self.show_surface(Surface::Library);
            }
            let result = self
                .runtime
                .submit(Action::RemoveFromLibrary(Box::new(RemovalRequest {
                    ids: ids.clone(),
                    cancel: pending.cancel.clone(),
                })));
            let cancel = pending.cancel.clone();
            let ids = ids.clone();
            self.library_selection.pending = Some(PendingBulk {
                cancel: cancel.clone(),
                action: PendingAction::Removal {
                    ids,
                    phase: RemovalPhase::Running,
                },
            });
            if let Err(error) = result {
                self.accept_library_bulk(
                    ctx,
                    BulkReply {
                        cancel,
                        result: Err(error),
                    },
                );
            }
        } else if let Some(progress) = self
            .runtime
            .library_removal_progress()
            .filter(|progress| progress.cancel.same_request(&pending.cancel))
        {
            self.reconcile_removal(ctx, &progress.report);
        }
    }
    pub(in crate::app) fn reconcile_removal(&mut self, ctx: &egui::Context, report: &BulkReport) {
        self.tabs.model.retire_documents(&report.retired);
        self.tabs.model.forget_documents(&report.succeeded);
        let removed: Vec<_> = self
            .documents
            .iter()
            .filter(|document| report.succeeded.contains(&document.id))
            .map(|document| document.id.clone())
            .collect();
        let clear_last = self
            .preferences
            .get("lastDocumentPath")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|path| {
                self.documents.iter().any(|document| {
                    report.succeeded.contains(&document.id)
                        && document.path.as_path() == std::path::Path::new(path)
                })
            });
        if clear_last {
            self.preferences
                .rust
                .insert("lastDocumentPath".into(), serde_json::json!(""));
            if !self.demo
                && let Err(error) = self.preferences.save(&self.store.join("preferences.json"))
            {
                self.error = Some(error.to_string());
            }
        }
        self.documents
            .retain(|document| !report.succeeded.contains(&document.id));
        if self
            .selected
            .as_ref()
            .is_some_and(|id| report.succeeded.contains(id))
        {
            self.selected = None;
        }
        for id in &report.succeeded {
            self.invalidate_library_cover(ctx, id);
        }
        if !removed.is_empty() {
            self.search.invalidate_removed(&removed);
        }
    }
    pub(in crate::app) fn removal_views_changed(&mut self) {
        self.bulk_tags_changed();
        self.bulk_collections_changed();
        self.global_notes.leave();
        self.history.leave();
        if matches!(self.surface, Surface::Notes) {
            self.show_global_notes();
        }
        if matches!(self.surface, Surface::History) {
            self.show_history();
        }
        if self.search.enabled {
            self.send(Action::IndexStatistics);
        }
    }
    pub(super) fn removal_progress_label(&self) -> Option<String> {
        let pending = self.library_selection.pending.as_ref()?;
        let PendingAction::Removal { ids, phase } = &pending.action else {
            return None;
        };
        if matches!(phase, RemovalPhase::Preparing) {
            return Some(self.tr("Guardando lectura…", "Saving reading…").into());
        }
        let done = self
            .runtime
            .library_removal_progress()
            .filter(|progress| progress.cancel.same_request(&pending.cancel))
            .map_or(0, |progress| {
                progress.report.succeeded.len() + progress.report.failed.len()
            });
        Some(format!(
            "{} {done}/{}",
            self.tr("Retirando", "Removing"),
            ids.len()
        ))
    }
}
