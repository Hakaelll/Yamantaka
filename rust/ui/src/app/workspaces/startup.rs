use super::*;
use binding::QueueContext;
#[derive(Default, PartialEq, Eq)]
pub(super) enum Startup {
    #[default]
    Waiting,
    Submitted,
    Finished,
}
impl YamantakaApp {
    pub(super) fn cancel_workspace_startup(&mut self) {
        if self.workspaces.startup == Startup::Submitted
            && let Some(cancel) = self.workspaces.pending.take()
        {
            cancel.cancel();
        }
        self.workspaces.startup = Startup::Finished;
    }
    pub(in crate::app) fn pump_workspace_startup(&mut self, ctx: &egui::Context) {
        if self.workspaces.startup != Startup::Waiting {
            return;
        }
        if self.demo {
            self.workspaces.startup = Startup::Finished;
            return;
        }
        if self.continuity_loading() {
            return;
        }
        let Some(had_snapshot) = self.continuity_available_for_workspace() else {
            self.workspaces.startup = Startup::Finished;
            return;
        };
        if !self
            .preferences
            .get("restoreLastSession")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
        {
            self.workspaces.startup = Startup::Finished;
            return;
        }
        if self.workspaces.pending.is_some()
            || self.tabs.busy()
            || self.opening.busy()
            || self.reader.reader_preferences_loading
        {
            return;
        }
        let reference = self.tabs.model.workspace_reference().cloned();
        if reference.is_none() && (had_snapshot || !self.tabs.model.tabs().is_empty()) {
            self.workspaces.startup = Startup::Finished;
            return;
        }
        self.workspaces.startup_view = Some(self.capture_workspace_view());
        self.workspace_request(WorkspaceCommand::Startup(reference));
        if self.workspaces.pending.is_some() {
            self.workspaces.startup = Startup::Submitted;
        } else {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    pub(super) fn accept_workspace_startup(
        &mut self,
        saved: Option<Box<yamantaka_domain::workspaces::WorkspaceReadings>>,
    ) {
        if self.workspaces.startup != Startup::Submitted {
            return;
        }
        self.workspaces.startup = Startup::Finished;
        let Some(saved) = saved else {
            return;
        };
        let unchanged = self
            .workspaces
            .startup_view
            .take()
            .is_some_and(|view| view == self.capture_workspace_view());
        if self.continuity_available_for_workspace().is_none()
            || self.tabs.busy()
            || self.opening.busy()
            || self.reader.reader_preferences_loading
        {
            return;
        }
        if let Some(reference) = self.tabs.model.workspace_reference() {
            if *reference != WorkspaceRevision::from(&saved.workspace)
                || QueueContext::capture(&self.tabs.model) != QueueContext::capture(&saved.tabs)
            {
                let _ = self.tabs.model.set_workspace_reference(None);
                self.error=Some(self.tr("La cola guardada ha cambiado. Tus lecturas se conservan sin vincular el espacio.","The saved queue changed. Your readings are preserved without linking the workspace.").into());
                return;
            }
            if unchanged && let Err(error) = self.apply_workspace_view(&saved.view) {
                self.error = Some(error.to_string());
                return;
            }
            for (id, title) in saved.captions {
                self.tabs.captions.insert(id, title);
            }
            self.workspaces.binding = Some(WorkspaceBinding {
                workspace: saved.workspace,
                queue: QueueContext::capture(&self.tabs.model),
                focus: saved.view.focus,
                paused: false,
                comparison: saved
                    .view
                    .parsed_query()
                    .ok()
                    .and_then(|query| query.comparison),
            });
            if unchanged {
                self.schedule_workspace_comparison(&saved.view, saved.reference_document);
            } else {
                let comparison = self.capture_workspace_comparison();
                self.workspaces.binding.as_mut().unwrap().comparison = comparison;
            }
        } else if unchanged && self.tabs.model.tabs().is_empty() && self.reader.active.is_none() {
            self.open_saved_workspace(*saved);
        }
    }
    pub(in crate::app) fn workspace_capture_summary(&self) -> serde_json::Value {
        serde_json::json!({"startupFinished":self.workspaces.startup==Startup::Finished,"bindingName":self.workspaces.binding.as_ref().map(|binding|&binding.workspace.name),"bindingId":self.workspaces.binding.as_ref().map(|binding|&binding.workspace.id),"paused":self.workspaces.binding.as_ref().is_some_and(|binding|binding.paused),"reference":self.tabs.model.workspace_reference().map(|reference|&reference.id),"focus":self.reader_focus,"undoAvailable":self.workspaces.undo.is_some(),"queue":self.tabs.model.tabs().iter().map(|tab|serde_json::json!({"document":tab.document(),"title":self.tabs.captions.get(tab.document()),"status":tab.queue().status(),"nextAction":tab.queue().next_action()})).collect::<Vec<_>>()})
    }
}
