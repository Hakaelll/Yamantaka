use super::*;
use yamantaka_application::workspaces::{
    WorkspaceCommand, WorkspaceOutcome, WorkspaceReply, WorkspaceRequest,
};
use yamantaka_domain::workspaces::{Workspace, WorkspaceChange, WorkspacePage, WorkspaceRevision};
mod binding;
mod context;
mod preview;
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
}
impl WorkspaceState {
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && self.pending_open.is_none() && self.preview == 0
    }
    pub(super) fn cancel(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
    }
}
impl YamantakaApp {
    pub(super) fn show_workspaces(&mut self) {
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
                if change.is_some() {
                    self.workspaces.close_after = false;
                    if let Some(binding) = &mut self.workspaces.binding
                        && matches!(&change,Some(WorkspaceChange::Replace {expected,..}|WorkspaceChange::Focus {expected,..}) if expected.id==binding.workspace.id)
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
                if !self.workspaces.open {
                    return;
                }
                if self.workspaces.preview > 0 {
                    self.check_workspace_preview(&saved);
                }
                if self.tabs.busy()
                    || self.opening.busy()
                    || self.reader_preferences_loading
                    || self.library_selection.removing()
                {
                    self.workspaces.error=Some(self.tr("Espera a que termine el cambio de lectura y vuelve a abrir el espacio.","Wait for the reading change to finish, then open the workspace again.").into());
                    return;
                }
                self.sync_tab_context();
                let mut plan = yamantaka_domain::reader_tabs::DocumentTabs::default();
                let mut selected = None;
                let mut readings = Vec::new();
                for tab in saved.tabs.tabs() {
                    let id = uuid::Uuid::new_v4().to_string();
                    if saved
                        .tabs
                        .active()
                        .is_some_and(|active| active.id() == tab.id())
                    {
                        selected = Some(id.clone());
                    }
                    let mapped = yamantaka_domain::reader_tabs::ReaderTab::new(
                        id,
                        tab.document().into(),
                        tab.page(),
                        tab.preferences().clone(),
                        tab.panel(),
                    )
                    .map(|mapped| mapped.with_pinned(tab.pinned()))
                    .and_then(|mapped| mapped.with_queue(tab.queue().clone()));
                    match mapped {
                        Ok(mapped) => readings.push(mapped),
                        Err(error) => {
                            self.workspaces.error = Some(error.to_string());
                            return;
                        }
                    }
                }
                let closed = self.tabs.model.closed().to_vec();
                if let Err(error) = plan.restore_continuity(readings, selected.as_deref(), closed) {
                    self.workspaces.error = Some(error.to_string());
                    return;
                }
                for (id, title) in saved.captions {
                    self.tabs.captions.insert(id, title);
                }
                if self.request_tab_plan(plan) {
                    self.workspaces.pending_open = Some(PendingContext {
                        workspace: saved.workspace.clone(),
                        view: saved.view,
                        target: selected,
                    });
                    self.workspaces.open = false;
                    self.workspaces.notice = Some(format!(
                        "{}: {}",
                        self.tr("Espacio abierto", "Workspace opened"),
                        saved.workspace.name
                    ));
                } else {
                    self.workspaces.error = Some(
                        self.tr(
                            "No se pudo iniciar la apertura. La lectura actual se conserva.",
                            "Opening could not start. The current reading is preserved.",
                        )
                        .into(),
                    );
                }
            }
        }
    }
    fn workspace_change(&mut self, change: WorkspaceChange) {
        self.workspace_request(WorkspaceCommand::Change(change));
    }
}
