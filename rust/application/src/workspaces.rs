use yamantaka_domain::{Cancellation, LibraryRepository, Result, workspaces::*};
pub enum WorkspaceCommand {
    List { offset: usize },
    Load(WorkspaceRevision),
    Change(WorkspaceChange),
}
pub struct WorkspaceRequest {
    pub command: WorkspaceCommand,
    pub cancel: Cancellation,
}
pub enum WorkspaceOutcome {
    Page(WorkspacePage),
    Loaded(Box<WorkspaceReadings>),
    Changed(String),
}
pub struct WorkspaceReply {
    pub result: Result<WorkspaceOutcome>,
    pub cancel: Cancellation,
}
pub(crate) fn execute(
    repository: &mut impl LibraryRepository,
    request: WorkspaceRequest,
) -> WorkspaceReply {
    let result = match request.command {
        WorkspaceCommand::List { offset } => repository
            .workspaces(offset, &request.cancel)
            .map(WorkspaceOutcome::Page),
        WorkspaceCommand::Load(expected) => repository
            .workspace(&expected, &request.cancel)
            .map(|value| WorkspaceOutcome::Loaded(Box::new(value))),
        WorkspaceCommand::Change(change) => repository
            .change_workspace(&change, &request.cancel)
            .map(WorkspaceOutcome::Changed),
    };
    WorkspaceReply {
        result,
        cancel: request.cancel,
    }
}
