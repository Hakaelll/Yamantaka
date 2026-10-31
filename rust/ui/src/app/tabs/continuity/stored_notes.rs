use super::*;
use yamantaka_application::annotation_draft_recovery::{Reply, Request};
use yamantaka_domain::annotations::stored_note_draft::StoredNoteDraft;

pub(super) struct PendingRestore {
    draft: StoredNoteDraft,
    cancel: Cancellation,
}

impl YamantakaApp {
    pub(in crate::app) fn capture_stored_note_drafts(&self) -> Vec<StoredNoteDraft> {
        let mut drafts = self.tabs.continuity.retained_edits.clone();
        for draft in [
            self.reader.annotations.stored_note_draft(false),
            self.reference
                .as_ref()
                .and_then(|view| view.pane.annotations.stored_note_draft(true)),
        ]
        .into_iter()
        .flatten()
        {
            drafts.retain(|old| !old.same_editor(&draft));
            drafts.push(draft);
        }
        drafts
    }
    pub(in crate::app) fn forget_stored_note_documents(&mut self, ids: &[String]) {
        self.tabs
            .continuity
            .retained_edits
            .retain(|draft| !ids.contains(&draft.document));
        if self
            .tabs
            .continuity
            .stored_restore
            .as_ref()
            .is_some_and(|pending| ids.contains(&pending.draft.document))
            && let Some(pending) = self.tabs.continuity.stored_restore.take()
        {
            pending.cancel.cancel();
        }
    }
    fn stored_draft_pane(&self, draft: &StoredNoteDraft) -> Option<&reader_pane::ReaderPane> {
        let pane = if draft.reference {
            &self.reference.as_ref().filter(|view| !view.busy())?.pane
        } else {
            &self.reader
        };
        (!pane.loading
            && !pane.annotations.editing()
            && !pane.annotations.writing()
            && pane
                .active
                .as_ref()
                .is_some_and(|(doc, _)| doc.id == draft.document))
        .then_some(pane)
    }
    pub(in crate::app) fn stored_note_recovery_ready(&self) -> bool {
        self.tabs.continuity.stored_restore.is_none()
            && !self
                .tabs
                .continuity
                .retained_edits
                .iter()
                .any(|draft| self.stored_draft_pane(draft).is_some())
    }
    pub(in crate::app) fn restore_stored_note_drafts(&mut self) {
        if self.tabs.continuity.stored_restore.is_some() {
            return;
        }
        let Some(draft) = self
            .tabs
            .continuity
            .retained_edits
            .iter()
            .find(|draft| self.stored_draft_pane(draft).is_some())
            .cloned()
        else {
            return;
        };
        let cancel = Cancellation::default();
        let request = Request {
            draft: draft.clone(),
            governor: self.governor.clone(),
            cancel: cancel.clone(),
        };
        match self
            .runtime
            .submit(Action::RecoverAnnotation(Box::new(request)))
        {
            Ok(()) => self.tabs.continuity.stored_restore = Some(PendingRestore { draft, cancel }),
            Err(Failure::QueueFull) => {}
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_stored_note_recovery(&mut self, reply: Reply) {
        if !self
            .tabs
            .continuity
            .stored_restore
            .as_ref()
            .is_some_and(|pending| {
                pending.cancel.same_request(&reply.cancel)
                    && pending.draft.same_editor(&reply.draft)
            })
        {
            return;
        }
        self.tabs.continuity.stored_restore = None;
        if reply.cancel.is_cancelled()
            || self
                .library_selection
                .removing_document(&reply.draft.document)
        {
            return;
        }
        let Some(pane) = self.stored_draft_pane(&reply.draft) else {
            return;
        };
        let valid_source = pane.active.as_ref().is_some_and(|(doc, info)| {
            !doc.fingerprint.is_empty()
                && doc.fingerprint == reply.draft.fingerprint
                && (reply.draft.page as usize) < info.pages.len()
        });
        let record = reply.result.ok().filter(|_| valid_source);
        self.tabs
            .continuity
            .retained_edits
            .retain(|draft| !draft.same_editor(&reply.draft));
        let reference = reply.draft.reference;
        let pane = if reference {
            &mut self.reference.as_mut().unwrap().pane
        } else {
            &mut self.reader
        };
        pane.annotations.install_stored_draft(reply.draft, record);
        pane.panel = true;
        pane.reader_panel_tab = outline::ReaderPanelTab::Annotations;
        if reference {
            self.reference.as_mut().unwrap().navigator_requested = true;
        }
    }
    pub(in crate::app) fn retry_stored_note(&mut self, reference: bool) {
        let draft = if reference {
            self.reference
                .as_ref()
                .and_then(|view| view.pane.annotations.retry_draft())
        } else {
            self.reader.annotations.retry_draft()
        };
        let Some(draft) = draft else {
            return;
        };
        let mut drafts = self.tabs.continuity.retained_edits.clone();
        drafts.retain(|old| !old.same_editor(&draft));
        drafts.push(draft);
        match ReaderContinuity::default().with_annotation_edits(drafts) {
            Ok(snapshot) => {
                self.tabs.continuity.retained_edits = snapshot.annotation_edits().to_vec();
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
    pub(in crate::app) fn stored_note_recovery_summary(&self) -> serde_json::Value {
        serde_json::json!({"retained":self.tabs.continuity.retained_edits.len(),"pending":self.tabs.continuity.stored_restore.is_some(),"primary":self.reader.annotations.stored_note_draft(false),"reference":self.reference.as_ref().and_then(|view|view.pane.annotations.stored_note_draft(true)),"primaryConflict":self.reader.annotations.recovery_conflict(),"referenceConflict":self.reference.as_ref().is_some_and(|view|view.pane.annotations.recovery_conflict())})
    }
}
