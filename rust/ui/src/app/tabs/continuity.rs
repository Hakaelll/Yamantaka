use super::*;
use yamantaka_application::reader_continuity::{
    ContinuityChange, ContinuityReply, ContinuityRequest,
};
use yamantaka_domain::{
    Failure, Result, reader_continuity::ReaderContinuity, workspaces::WorkspaceComparison,
};

#[derive(Default)]
pub(in crate::app) struct ContinuityState {
    pub(in crate::app) overridden: bool,
    loading: Option<Cancellation>,
    loaded: bool,
    had_snapshot: bool,
    paused: bool,
    revision: u64,
    saved: ReaderContinuity,
    retained_comparison: Option<WorkspaceComparison>,
    restoring_comparison: bool,
    retained_page_notes: Vec<yamantaka_domain::annotations::page_note_draft::PageNoteDraft>,
    pending: Option<(u64, ReaderContinuity, Cancellation)>,
    due: Option<Instant>,
}
impl YamantakaApp {
    pub(in crate::app) fn continuity_summary(&self) -> serde_json::Value {
        serde_json::json!({"loaded":self.tabs.continuity.loaded,"restoring":self.tabs.continuity.restoring_comparison,"retainedComparison":self.tabs.continuity.retained_comparison,"comparison":self.capture_comparison()})
    }
    pub(in crate::app) fn forget_continuity_documents(&mut self, ids: &[String]) {
        self.tabs
            .continuity
            .retained_page_notes
            .retain(|draft| !ids.contains(&draft.document));
        if self
            .tabs
            .continuity
            .retained_comparison
            .as_ref()
            .is_some_and(|saved| ids.contains(&saved.primary) || ids.contains(&saved.document))
        {
            self.discard_retained_comparison();
        }
    }

    fn capture_reader_continuity(&self) -> Result<ReaderContinuity> {
        let saved = self
            .tabs
            .continuity
            .retained_comparison
            .as_ref()
            .filter(|saved| {
                self.tabs
                    .model
                    .active()
                    .is_some_and(|tab| tab.document() == saved.primary)
            })
            .cloned();
        let mut drafts = self.tabs.continuity.retained_page_notes.clone();
        for draft in [
            self.reader.annotations.page_note_draft(false),
            self.reference
                .as_ref()
                .and_then(|view| view.pane.annotations.page_note_draft(true)),
        ]
        .into_iter()
        .flatten()
        {
            drafts.retain(|old| old.document != draft.document || old.reference != draft.reference);
            drafts.push(draft);
        }
        ReaderContinuity::new(
            self.tabs.model.clone(),
            saved.or_else(|| self.capture_comparison()),
        )?
        .with_page_notes(drafts)
    }
    fn restore_page_note_drafts(&mut self) {
        let mut remaining = Vec::new();
        for draft in std::mem::take(&mut self.tabs.continuity.retained_page_notes) {
            let pane = if draft.reference {
                self.reference
                    .as_mut()
                    .filter(|view| !view.busy())
                    .map(|view| &mut view.pane)
            } else {
                Some(&mut self.reader)
            };
            let result = pane.map_or(Ok(false), |pane| pane.restore_page_note(&draft));
            match result {
                Ok(true) => {
                    if draft.reference
                        && let Some(view) = &mut self.reference
                    {
                        view.navigator_requested = true;
                    }
                }
                Ok(false) => remaining.push(draft),
                Err(error) => {
                    if self.error.is_none() {
                        self.error = Some(error.to_string());
                    }
                    remaining.push(draft);
                }
            }
        }
        self.tabs.continuity.retained_page_notes = remaining;
    }
    pub(in crate::app) fn page_note_recovery_summary(&self) -> serde_json::Value {
        serde_json::json!({"retained":self.tabs.continuity.retained_page_notes.len(),"primary":self.reader.annotations.page_note_draft(false),"reference":self.reference.as_ref().and_then(|view|view.pane.annotations.page_note_draft(true))})
    }
    pub(in crate::app) fn discard_retained_comparison(&mut self) {
        self.tabs.continuity.retained_comparison = None;
        self.tabs.continuity.restoring_comparison = false;
        self.cancel_continuity_comparison_restore();
    }
    pub(in crate::app) fn finish_continuity_comparison(&mut self, success: bool) {
        self.tabs.continuity.restoring_comparison = false;
        if success {
            self.tabs.continuity.retained_comparison = None;
        }
    }

    pub(in crate::app) fn enable_continuity_capture(&mut self) {
        if self.demo && self.capture.is_some() {
            self.tabs.continuity.loaded = true;
        }
    }
    pub(in crate::app) fn continuity_settled(&self) -> bool {
        let state = &self.tabs.continuity;
        state.loaded
            && !state.paused
            && state.pending.is_none()
            && self
                .capture_reader_continuity()
                .is_ok_and(|snapshot| snapshot == state.saved)
    }
    pub(in crate::app) fn continuity_available_for_workspace(&self) -> Option<bool> {
        let state = &self.tabs.continuity;
        (state.loaded && !state.paused && !state.overridden && !state.restoring_comparison)
            .then_some(state.had_snapshot)
    }
    pub(in crate::app) fn continuity_loading(&self) -> bool {
        self.tabs.continuity.loading.is_some() || self.tabs.continuity.restoring_comparison
    }

    pub(in crate::app) fn begin_reader_continuity(&mut self) {
        if (self.demo && !crate::graphics_recovery::continuity_resume())
            || self.startup_mode.is_safe()
        {
            return;
        }
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::ReaderContinuity(Box::new(ContinuityRequest {
                change: ContinuityChange::Load,
                cancel: cancel.clone(),
            }))) {
            Ok(()) => self.tabs.continuity.loading = Some(cancel),
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn accept_reader_continuity(&mut self, reply: ContinuityReply) {
        let state = &mut self.tabs.continuity;
        if let Some(revision) = reply.revision {
            if !state.pending.as_ref().is_some_and(|(expected, _, token)| {
                *expected == revision && token.same_request(&reply.cancel)
            }) {
                return;
            }
            let (_, snapshot, _) = state.pending.take().unwrap();
            match reply.result {
                Ok(_) => state.saved = snapshot,
                Err(error) => {
                    state.paused = true;
                    self.error = Some(error.to_string());
                }
            }
            return;
        }
        if !state
            .loading
            .as_ref()
            .is_some_and(|token| token.same_request(&reply.cancel))
        {
            return;
        }
        state.loading = None;
        let restored = match reply.result {
            Ok(restored) => {
                state.had_snapshot = restored.is_some();
                restored.unwrap_or_default()
            }
            Err(error) => {
                self.error = Some(error.to_string());
                return;
            }
        };
        if (!restored.tabs().tabs().is_empty() || !restored.tabs().closed().is_empty())
            && (self.reader.active.is_some()
                || self.opening.busy()
                || !self.tabs.model.tabs().is_empty())
        {
            self.error = Some(self.tr("La lectura actual tiene prioridad. La sesión anterior se conserva para su recuperación.", "The current reading takes priority. The previous session is preserved for recovery.").into());
            return;
        }
        self.tabs.continuity.retained_page_notes = restored.page_notes().to_vec();
        let restore = crate::graphics_recovery::runtime_fallback()
            || self
                .preferences
                .get("restoreLastSession")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
        if restore {
            self.tabs.model = restored.tabs().clone();
            self.tabs.continuity.retained_comparison = restored.comparison().cloned();
        } else if let Err(error) =
            self.tabs
                .model
                .restore_continuity(vec![], None, restored.tabs().closed().to_vec())
        {
            self.error = Some(error.to_string());
            return;
        }
        self.tabs.continuity.saved = if restore {
            restored
        } else {
            self.tabs.model.clone().into()
        };
        self.tabs.continuity.loaded = true;
        if restore
            && !self.tabs.continuity.overridden
            && let Some(id) = self.tabs.model.active().map(|tab| tab.id().to_owned())
        {
            self.restore_reader_tab(&id);
            if let Some(saved) = self.tabs.continuity.retained_comparison.clone() {
                self.tabs.continuity.restoring_comparison = true;
                self.schedule_continuity_comparison(saved, reply.reference_document);
            }
        }
    }
    pub(in crate::app) fn pump_reader_continuity(&mut self, ctx: &egui::Context) {
        if self.startup_mode.is_safe() {
            return;
        }
        if self.library_selection.removing()
            || !self.tabs.continuity.loaded
            || self.tabs.continuity.paused
            || self.tabs.continuity.restoring_comparison
        {
            return;
        }
        self.restore_page_note_drafts();
        self.sync_tab_context();
        if self
            .tabs
            .continuity
            .retained_comparison
            .as_ref()
            .is_some_and(|saved| {
                self.tabs
                    .model
                    .active()
                    .is_none_or(|tab| tab.document() != saved.primary)
            })
        {
            self.tabs.continuity.retained_comparison = None;
        }
        let snapshot = match self.capture_reader_continuity() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.error = Some(error.to_string());
                self.tabs.continuity.paused = true;
                return;
            }
        };
        let state = &mut self.tabs.continuity;
        if state.pending.is_some() {
            return;
        }
        if snapshot == state.saved {
            state.due = None;
            return;
        }
        let now = Instant::now();
        let due = *state.due.get_or_insert(now + Duration::from_millis(500));
        if now < due {
            ctx.request_repaint_after(due - now);
            return;
        }
        let Some(revision) = state.revision.checked_add(1) else {
            state.paused = true;
            self.error = Some("Reader continuity revision exhausted".into());
            return;
        };
        let cancel = Cancellation::default();
        let request = ContinuityRequest {
            change: ContinuityChange::Save {
                revision,
                tabs: Box::new(snapshot.clone()),
            },
            cancel: cancel.clone(),
        };
        match self
            .runtime
            .submit(Action::ReaderContinuity(Box::new(request)))
        {
            Ok(()) => {
                state.revision = revision;
                state.pending = Some((revision, snapshot, cancel));
                state.due = None;
            }
            Err(Failure::QueueFull) => {
                state.due = Some(now + Duration::from_millis(100));
                ctx.request_repaint_after(Duration::from_millis(100));
            }
            Err(error) => {
                state.paused = true;
                self.error = Some(error.to_string());
            }
        }
    }
    pub(in crate::app) fn persist_reader_continuity_on_close(&mut self) -> Result<()> {
        if self.startup_mode.is_safe() {
            return Ok(());
        }
        if !self.tabs.continuity.loaded {
            return Ok(());
        }
        self.sync_tab_context();
        let revision = self
            .tabs
            .continuity
            .revision
            .checked_add(1)
            .ok_or_else(|| Failure::Invalid("Reader continuity revision exhausted".into()))?;
        self.runtime
            .persist_reader_continuity(revision, self.capture_reader_continuity()?)
    }
}
