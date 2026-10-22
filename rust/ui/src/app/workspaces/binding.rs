use super::*;
use yamantaka_domain::{
    reader_tabs::{DocumentTabs, QueueEntry},
    workspaces::WorkspaceView,
};

#[derive(Clone, PartialEq)]
pub(super) struct QueueContext {
    entries: Vec<(String, QueueEntry)>,
    active: Option<String>,
}
impl QueueContext {
    pub(super) fn capture(tabs: &DocumentTabs) -> Self {
        Self {
            entries: tabs
                .tabs()
                .iter()
                .map(|tab| (tab.document().into(), tab.queue().clone()))
                .collect(),
            active: tabs.active().map(|tab| tab.document().into()),
        }
    }
}
pub(super) struct WorkspaceBinding {
    pub(super) workspace: Workspace,
    pub(super) queue: QueueContext,
    pub(super) focus: bool,
    pub(super) paused: bool,
}
pub(super) struct PendingContext {
    pub(super) workspace: Workspace,
    pub(super) view: WorkspaceView,
    pub(super) target: Option<String>,
}
impl YamantakaApp {
    fn workspace_dirty(&self) -> bool {
        self.workspaces.binding.as_ref().is_some_and(|binding| {
            !binding.paused
                && (binding.queue != QueueContext::capture(&self.tabs.model)
                    || binding.focus != self.reader_focus)
        })
    }
    pub(in crate::app) fn guard_workspace_close(&mut self, ctx: &egui::Context) {
        if ctx.input(|input| input.viewport().close_requested())
            && (self.workspaces.pending_change.is_some() || self.workspace_dirty())
        {
            self.workspaces.close_after = true;
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
    }
    pub(in crate::app) fn pump_workspace_context(&mut self, ctx: &egui::Context) {
        if self.workspaces.pending_open.is_some()
            && !self.tabs.busy()
            && !self.opening.busy()
            && !self.reader.reader_preferences_loading
        {
            let pending = self.workspaces.pending_open.take().unwrap();
            if self.tabs.model.active().map(|tab| tab.id()) == pending.target.as_deref() {
                if let Err(error) = self.apply_workspace_view(&pending.view) {
                    self.error = Some(error.to_string());
                    return;
                }
                self.workspaces.binding = Some(WorkspaceBinding {
                    workspace: pending.workspace,
                    queue: QueueContext::capture(&self.tabs.model),
                    focus: pending.view.focus,
                    paused: false,
                });
            }
        }
        if let Some(binding) = &self.workspaces.binding
            && let Err(error) = self
                .tabs
                .model
                .set_workspace_reference(Some((&binding.workspace).into()))
        {
            self.error = Some(error.to_string());
        }
        if self.workspaces.close_after
            && self
                .workspaces
                .binding
                .as_ref()
                .is_some_and(|binding| binding.paused)
        {
            self.workspaces.close_after = false;
            return;
        }
        if self.workspaces.pending.is_some()
            || self.workspaces.pending_open.is_some()
            || self.tabs.busy()
            || self.opening.busy()
            || self.reader.reader_preferences_loading
            || self.library_selection.removing()
        {
            if self.workspaces.close_after {
                ctx.request_repaint_after(Duration::from_millis(30));
            }
            return;
        }
        if self.workspace_dirty() {
            let binding = self.workspaces.binding.as_ref().unwrap();
            let expected = (&binding.workspace).into();
            let change = if binding.queue != QueueContext::capture(&self.tabs.model) {
                WorkspaceChange::Replace {
                    expected,
                    tabs: Box::new(self.tabs.model.clone()),
                    view: None,
                }
            } else {
                WorkspaceChange::Focus {
                    expected,
                    enabled: self.reader_focus,
                }
            };
            self.workspace_change(change);
            ctx.request_repaint();
        } else if self.workspaces.close_after {
            self.workspaces.close_after = false;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
    pub(super) fn accept_workspace_change(
        &mut self,
        changed: yamantaka_domain::workspaces::WorkspaceChanged,
        change: Option<WorkspaceChange>,
    ) {
        use yamantaka_domain::workspaces::WorkspaceChanged;
        match changed {
            WorkspaceChanged::Deleted(undo) => {
                if self
                    .tabs
                    .model
                    .workspace_reference()
                    .is_some_and(|reference| reference.id == undo.id)
                {
                    let _ = self.tabs.model.set_workspace_reference(None);
                }
                if self
                    .workspaces
                    .binding
                    .as_ref()
                    .is_some_and(|binding| binding.workspace.id == undo.id)
                {
                    self.workspaces.binding = None;
                }
                self.workspaces.undo = Some(undo);
            }
            WorkspaceChanged::Saved(workspace) => {
                if matches!(change, Some(WorkspaceChange::UndoDelete { .. })) {
                    self.workspaces.undo = None;
                }
                if let Some(WorkspaceChange::Create { tabs, view, .. }) = &change
                    && QueueContext::capture(tabs) == QueueContext::capture(&self.tabs.model)
                {
                    self.workspaces.binding = Some(WorkspaceBinding {
                        workspace,
                        queue: QueueContext::capture(tabs),
                        focus: view.focus,
                        paused: false,
                    });
                    return;
                }
                if let Some(binding) = &mut self.workspaces.binding
                    && binding.workspace.id == workspace.id
                {
                    binding.workspace = workspace;
                    match change {
                        Some(WorkspaceChange::Replace { tabs, view, .. }) => {
                            binding.paused = false;
                            binding.queue = QueueContext::capture(&tabs);
                            if let Some(view) = view {
                                binding.focus = view.focus;
                            }
                        }
                        Some(WorkspaceChange::Focus { enabled, .. }) => {
                            binding.focus = enabled;
                            binding.paused = false;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
