use super::*;
use yamantaka_application::selection_draft_recovery::{Reply, Request};
use yamantaka_domain::annotations::selection_note_draft::SelectionNoteDraft;

pub(super) struct PendingRestore {
    draft: SelectionNoteDraft,
    cancel: Cancellation,
}

impl YamantakaApp {
    pub(in crate::app) fn retry_annotation_draft(&mut self, reference: bool) {
        let selection = if reference {
            self.reference
                .as_ref()
                .is_some_and(|v| v.pane.annotations.selection_recovery_conflict())
        } else {
            self.reader.annotations.selection_recovery_conflict()
        };
        if selection {
            self.retry_selection_note(reference);
        } else {
            self.retry_stored_note(reference);
        }
    }
    pub(in crate::app) fn capture_selection_note_drafts(&self) -> Vec<SelectionNoteDraft> {
        let mut drafts = self.tabs.continuity.retained_selections.clone();
        for draft in [
            self.reader.annotations.selection_note_draft(false),
            self.reference
                .as_ref()
                .and_then(|view| view.pane.annotations.selection_note_draft(true)),
        ]
        .into_iter()
        .flatten()
        {
            drafts.retain(|old| !old.same_editor(&draft));
            drafts.push(draft);
        }
        drafts
    }
    pub(in crate::app) fn forget_selection_note_documents(&mut self, ids: &[String]) {
        self.tabs
            .continuity
            .retained_selections
            .retain(|draft| !ids.contains(&draft.document));
        if self
            .tabs
            .continuity
            .selection_restore
            .as_ref()
            .is_some_and(|pending| ids.contains(&pending.draft.document))
            && let Some(pending) = self.tabs.continuity.selection_restore.take()
        {
            pending.cancel.cancel();
        }
    }
    fn selection_draft_pane(&self, draft: &SelectionNoteDraft) -> Option<&reader_pane::ReaderPane> {
        let pane = if draft.reference {
            &self.reference.as_ref().filter(|view| !view.busy())?.pane
        } else {
            &self.reader
        };
        (!pane.loading
            && pane
                .viewport
                .pages
                .values()
                .any(|slot| slot.image.is_some())
            && !pane.annotations.editing()
            && !pane.annotations.writing()
            && pane
                .active
                .as_ref()
                .is_some_and(|(doc, _)| doc.id == draft.document))
        .then_some(pane)
    }
    pub(in crate::app) fn selection_note_recovery_ready(&self) -> bool {
        self.tabs.continuity.selection_restore.is_none()
            && !self
                .tabs
                .continuity
                .retained_selections
                .iter()
                .any(|draft| self.selection_draft_pane(draft).is_some())
    }
    pub(in crate::app) fn restore_selection_note_drafts(&mut self) {
        if self.tabs.continuity.selection_restore.is_some() {
            return;
        }
        let Some(draft) = self
            .tabs
            .continuity
            .retained_selections
            .iter()
            .find(|draft| self.selection_draft_pane(draft).is_some())
            .cloned()
        else {
            return;
        };
        let cancel = Cancellation::default();
        let source_matches = self.selection_draft_pane(&draft).is_some_and(|pane| {
            pane.active.as_ref().is_some_and(|(doc, info)| {
                !doc.fingerprint.is_empty()
                    && doc.fingerprint == draft.fingerprint
                    && (draft.end[0] as usize) < info.pages.len()
            })
        });
        if !source_matches {
            self.tabs.continuity.selection_restore = Some(PendingRestore {
                draft: draft.clone(),
                cancel: cancel.clone(),
            });
            self.accept_selection_note_recovery(Reply {
                draft,
                cancel,
                result: Err(Failure::Invalid(
                    "Recovered selection source changed".into(),
                )),
            });
            return;
        }
        let request = Request {
            draft: draft.clone(),
            governor: self.governor.clone(),
            cancel: cancel.clone(),
        };
        match self
            .runtime
            .submit(Action::RecoverSelection(Box::new(request)))
        {
            Ok(()) => {
                self.tabs.continuity.selection_restore = Some(PendingRestore { draft, cancel })
            }
            Err(Failure::QueueFull) => {}
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_selection_note_recovery(&mut self, reply: Reply) {
        if !self
            .tabs
            .continuity
            .selection_restore
            .as_ref()
            .is_some_and(|pending| {
                pending.cancel.same_request(&reply.cancel)
                    && pending.draft.same_editor(&reply.draft)
            })
        {
            return;
        }
        self.tabs.continuity.selection_restore = None;
        if reply.cancel.is_cancelled()
            || self
                .library_selection
                .removing_document(&reply.draft.document)
        {
            return;
        }
        let Some(pane) = self.selection_draft_pane(&reply.draft) else {
            return;
        };
        let valid_source = pane.active.as_ref().is_some_and(|(doc, info)| {
            !doc.fingerprint.is_empty()
                && doc.fingerprint == reply.draft.fingerprint
                && (reply.draft.end[0] as usize) < info.pages.len()
        });
        let snapshot = reply.result.ok().filter(|_| valid_source);
        self.tabs
            .continuity
            .retained_selections
            .retain(|draft| !draft.same_editor(&reply.draft));
        let reference = reply.draft.reference;
        let pane = if reference {
            &mut self.reference.as_mut().unwrap().pane
        } else {
            &mut self.reader
        };
        pane.annotations
            .install_selection_draft(reply.draft, snapshot);
        pane.panel = true;
        pane.reader_panel_tab = outline::ReaderPanelTab::Annotations;
        if reference {
            self.reference.as_mut().unwrap().navigator_requested = true;
        }
    }
    pub(in crate::app) fn retry_selection_note(&mut self, reference: bool) {
        let draft = if reference {
            self.reference
                .as_ref()
                .and_then(|view| view.pane.annotations.selection_note_draft(true))
        } else {
            self.reader.annotations.selection_note_draft(false)
        };
        let Some(draft) = draft else {
            return;
        };
        let mut drafts = self.tabs.continuity.retained_selections.clone();
        drafts.retain(|old| !old.same_editor(&draft));
        drafts.push(draft);
        match ReaderContinuity::default().with_selection_notes(drafts) {
            Ok(snapshot) => {
                self.tabs.continuity.retained_selections = snapshot.selection_notes().to_vec();
                if reference {
                    self.reference
                        .as_mut()
                        .unwrap()
                        .pane
                        .annotations
                        .clear_recovered_draft();
                } else {
                    self.reader.annotations.clear_recovered_draft();
                }
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn selection_note_recovery_summary(&self) -> serde_json::Value {
        serde_json::json!({"primaryGeometryPages":self.reader.annotations.selection_geometry_pages(),"referenceGeometryPages":self.reference.as_ref().map_or(0, |view| view.pane.annotations.selection_geometry_pages()),"retained":self.tabs.continuity.retained_selections.len(),"pending":self.tabs.continuity.selection_restore.is_some(),"primary":self.reader.annotations.selection_note_draft(false),"reference":self.reference.as_ref().and_then(|view|view.pane.annotations.selection_note_draft(true)),"primaryConflict":self.reader.annotations.selection_recovery_conflict(),"referenceConflict":self.reference.as_ref().is_some_and(|view|view.pane.annotations.selection_recovery_conflict())})
    }
}
