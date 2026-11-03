use super::*;
use yamantaka_application::workspaces::{
    WorkspaceCommand, WorkspaceOutcome, WorkspaceReply, WorkspaceRequest,
};
use yamantaka_domain::workspaces::{Workspace, WorkspaceChange, WorkspacePage, WorkspaceRevision};
mod binding;
mod comparison;
mod context;
mod opening;
mod preview;
mod startup;
mod view;
use binding::{PendingContext, WorkspaceBinding};

#[derive(Default)]
pub(super) struct WorkspaceState {
    pub(super) open: bool,
    entries: Vec<Workspace>,
    total: u64,
    offset: usize,
    pending: Option<Cancellation>,
    loading: bool,
    pending_change: Option<WorkspaceChange>,
    pending_open: Option<PendingContext>,
    comparison_restore: Option<comparison::RestoreComparison>,
    comparison_due: Option<(
        Option<yamantaka_domain::workspaces::WorkspaceComparison>,
        Instant,
    )>,
    binding: Option<WorkspaceBinding>,
    pub(super) close_after: bool,
    draft: String,
    edit: Option<(Workspace, String, bool)>,
    confirm: Option<(Workspace, bool)>,
    error: Option<String>,
    notice: Option<String>,
    focus_close: bool,
    focus_cancel: bool,
    preview: u8,
    continuous_preview: bool,
    synchronized_preview: bool,
    startup: startup::Startup,
    startup_view: Option<yamantaka_domain::workspaces::WorkspaceView>,
    undo: Option<yamantaka_domain::workspaces::WorkspaceUndo>,
}
impl WorkspaceState {
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none()
            && self.pending_open.is_none()
            && self.comparison_restore.is_none()
            && self.preview == 0
            && self.startup == startup::Startup::Finished
    }
    pub(super) fn cancel(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
    }
}
impl YamantakaApp {
    pub(super) fn show_workspaces(&mut self) {
        self.cancel_workspace_startup();
        self.sync_tab_context();
        self.workspaces.open = true;
        self.workspaces.focus_close = true;
        self.workspaces.offset = 0;
        self.workspace_request(WorkspaceCommand::List { offset: 0 });
    }
    fn workspace_request(&mut self, command: WorkspaceCommand) {
        if self.workspaces.pending.is_some() {
            return;
        }
        let cancel = Cancellation::default();
        let loading = matches!(command, WorkspaceCommand::Load(_));
        let change = if let WorkspaceCommand::Change(change) = &command {
            Some(change.clone())
        } else {
            None
        };
        match self
            .runtime
            .submit(Action::Workspace(Box::new(WorkspaceRequest {
                command,
                cancel: cancel.clone(),
            }))) {
            Ok(()) => {
                self.workspaces.pending = Some(cancel);
                self.workspaces.loading = loading;
                self.workspaces.pending_change = change;
                self.workspaces.error = None;
            }
            Err(error) => self.workspaces.error = Some(error.to_string()),
        }
    }
    pub(super) fn accept_workspace(&mut self, reply: WorkspaceReply) {
        if !self
            .workspaces
            .pending
            .as_ref()
            .is_some_and(|token| token.same_request(&reply.cancel))
        {
            return;
        }
        self.workspaces.pending = None;
        let change = self.workspaces.pending_change.take();
        match reply.result {
            Err(error) => {
                assert!(
                    self.workspaces.preview == 0,
                    "Workspace preview failed: {error}"
                );
                self.workspaces.error = Some(error.to_string());
                if self.workspaces.startup == startup::Startup::Submitted {
                    self.workspaces.startup = startup::Startup::Finished;
                    let _ = self.tabs.model.set_workspace_reference(None);
                    self.error = Some(format!(
                        "{}: {error}",
                        self.tr(
                            "Se han recuperado las lecturas sin vincular el espacio guardado",
                            "Readings were restored without linking the saved workspace"
                        )
                    ));
                }
                if change.is_some() {
                    self.workspaces.close_after = false;
                    if let Some(binding) = &mut self.workspaces.binding
                        && matches!(&change,Some(WorkspaceChange::Replace {expected,..}|WorkspaceChange::Focus {expected,..}|WorkspaceChange::Comparison {expected,..}) if expected.id==binding.workspace.id)
                    {
                        binding.paused = true;
                    }
                    self.error=Some(format!("{}: {error}",self.tr("No se han guardado los cambios del espacio. Las lecturas actuales se conservan.","Workspace changes could not be saved. Current readings are preserved.")));
                }
            }
            Ok(WorkspaceOutcome::Page(WorkspacePage { entries, total })) => {
                self.workspaces.entries = entries;
                self.workspaces.total = total;
                if self.workspaces.offset > 0 && self.workspaces.entries.is_empty() {
                    self.workspaces.offset = self.workspaces.offset.saturating_sub(80);
                    self.workspace_request(WorkspaceCommand::List {
                        offset: self.workspaces.offset,
                    });
                }
            }
            Ok(WorkspaceOutcome::Changed(changed)) => {
                self.accept_workspace_change(changed, change);
                self.workspaces.draft.clear();
                self.workspaces.edit = None;
                self.workspaces.confirm = None;
                self.workspaces.notice = Some(self.tr("Cambios guardados", "Changes saved").into());
                if self.workspaces.open || self.workspaces.preview > 0 {
                    self.workspace_request(WorkspaceCommand::List {
                        offset: self.workspaces.offset,
                    });
                }
            }
            Ok(WorkspaceOutcome::Loaded(saved)) => {
                if self.workspaces.open {
                    if self.workspaces.preview > 0 {
                        self.check_workspace_preview(&saved);
                    }
                    self.open_saved_workspace(*saved);
                }
            }
            Ok(WorkspaceOutcome::Startup(saved)) => self.accept_workspace_startup(saved),
        }
    }
    fn workspace_change(&mut self, change: WorkspaceChange) {
        self.workspace_request(WorkspaceCommand::Change(change));
    }
}
