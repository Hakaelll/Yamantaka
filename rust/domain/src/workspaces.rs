use crate::reader_tabs::DocumentTabs;
mod view;
pub use view::*;

#[derive(Clone, Debug)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub updated_at: String,
    pub is_default: bool,
    pub readings: u64,
}
#[derive(Clone, Debug)]
pub struct WorkspaceRevision {
    pub id: String,
    pub updated_at: String,
}
impl From<&Workspace> for WorkspaceRevision {
    fn from(value: &Workspace) -> Self {
        Self {
            id: value.id.clone(),
            updated_at: value.updated_at.clone(),
        }
    }
}
pub struct WorkspacePage {
    pub entries: Vec<Workspace>,
    pub total: u64,
}
pub struct WorkspaceReadings {
    pub workspace: Workspace,
    pub tabs: DocumentTabs,
    pub captions: Vec<(String, String)>,
    pub view: WorkspaceView,
}
pub enum WorkspaceChanged {
    Saved(Workspace),
    Deleted(String),
}
#[derive(Clone)]
pub enum WorkspaceChange {
    Create {
        name: String,
        tabs: Box<DocumentTabs>,
        view: Box<WorkspaceView>,
    },
    Replace {
        expected: WorkspaceRevision,
        tabs: Box<DocumentTabs>,
        view: Option<Box<WorkspaceView>>,
    },
    Rename {
        expected: WorkspaceRevision,
        name: String,
    },
    Duplicate {
        expected: WorkspaceRevision,
        name: String,
    },
    SetDefault {
        expected: WorkspaceRevision,
    },
    Focus {
        expected: WorkspaceRevision,
        enabled: bool,
    },
    Delete {
        expected: WorkspaceRevision,
    },
}
