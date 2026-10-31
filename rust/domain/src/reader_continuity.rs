use crate::{Failure, Result, reader_tabs::DocumentTabs, workspaces::WorkspaceComparison};

/// One atomic default-session record; saved workspace views remain independent.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReaderContinuity {
    tabs: DocumentTabs,
    comparison: Option<WorkspaceComparison>,
}
impl ReaderContinuity {
    pub fn new(tabs: DocumentTabs, comparison: Option<WorkspaceComparison>) -> Result<Self> {
        if let Some(saved) = &comparison {
            saved.validate()?;
            if tabs
                .active()
                .is_none_or(|tab| tab.document() != saved.primary)
            {
                return Err(Failure::Invalid("Saved comparison does not belong to the active reading; original continuity is preserved".into()));
            }
        }
        Ok(Self { tabs, comparison })
    }
    pub fn tabs(&self) -> &DocumentTabs {
        &self.tabs
    }
    pub fn comparison(&self) -> Option<&WorkspaceComparison> {
        self.comparison.as_ref()
    }
    pub fn forget_documents(&mut self, ids: &[String]) {
        self.tabs.forget_documents(ids);
        if self.comparison.as_ref().is_some_and(|saved| {
            ids.contains(&saved.document)
                || self
                    .tabs
                    .active()
                    .is_none_or(|tab| tab.document() != saved.primary)
        }) {
            self.comparison = None;
        }
    }
}
impl From<DocumentTabs> for ReaderContinuity {
    fn from(tabs: DocumentTabs) -> Self {
        Self {
            tabs,
            comparison: None,
        }
    }
}
