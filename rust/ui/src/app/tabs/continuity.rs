use super::*;
use yamantaka_application::reader_continuity::{
    ContinuityChange, ContinuityReply, ContinuityRequest,
};
use yamantaka_domain::{Failure, Result};

#[derive(Default)]
pub(in crate::app) struct ContinuityState {
    pub(in crate::app) overridden: bool,
    loading: Option<Cancellation>,
    loaded: bool,
    had_snapshot: bool,
    paused: bool,
    revision: u64,
    saved: DocumentTabs,
    pending: Option<(u64, DocumentTabs, Cancellation)>,
    due: Option<Instant>,
}
impl YamantakaApp {
    pub(in crate::app) fn enable_continuity_capture(&mut self) {
        if self.demo && self.capture.is_some() {
            self.tabs.continuity.loaded = true;
        }
    }
    pub(in crate::app) fn continuity_settled(&self) -> bool {
        let state = &self.tabs.continuity;
        state.loaded && !state.paused && state.pending.is_none() && state.saved == self.tabs.model
    }
    pub(in crate::app) fn continuity_available_for_workspace(&self) -> Option<bool> {
        let state = &self.tabs.continuity;
        (state.loaded && !state.paused && !state.overridden).then_some(state.had_snapshot)
    }
    pub(in crate::app) fn continuity_loading(&self) -> bool {
        self.tabs.continuity.loading.is_some()
    }

    pub(in crate::app) fn begin_reader_continuity(&mut self) {
        if self.demo || self.startup_mode.is_safe() {
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
        if (!restored.tabs().is_empty() || !restored.closed().is_empty())
            && (self.reader.active.is_some()
                || self.opening.busy()
                || !self.tabs.model.tabs().is_empty())
        {
            self.error = Some(self.tr("La lectura actual tiene prioridad. La sesión anterior se conserva para su recuperación.", "The current reading takes priority. The previous session is preserved for recovery.").into());
            return;
        }
        let restore = self
            .preferences
            .get("restoreLastSession")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        if restore {
            self.tabs.model = restored;
        } else if let Err(error) =
            self.tabs
                .model
                .restore_continuity(vec![], None, restored.closed().to_vec())
        {
            self.error = Some(error.to_string());
            return;
        }
        self.tabs.continuity.saved = self.tabs.model.clone();
        self.tabs.continuity.loaded = true;
        if restore
            && !self.tabs.continuity.overridden
            && let Some(id) = self.tabs.model.active().map(|tab| tab.id().to_owned())
        {
            self.select_reader_tab(&id);
        }
    }
    pub(in crate::app) fn pump_reader_continuity(&mut self, ctx: &egui::Context) {
        if self.startup_mode.is_safe() {
            return;
        }
        if self.library_selection.removing()
            || !self.tabs.continuity.loaded
            || self.tabs.continuity.paused
        {
            return;
        }
        self.sync_tab_context();
        let state = &mut self.tabs.continuity;
        if state.pending.is_some() {
            return;
        }
        if self.tabs.model == state.saved {
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
        let snapshot = self.tabs.model.clone();
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
            .persist_reader_continuity(revision, self.tabs.model.clone())
    }
}
