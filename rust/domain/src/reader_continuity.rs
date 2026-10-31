use crate::{Failure, Result, reader_tabs::DocumentTabs, workspaces::WorkspaceComparison};

/// One atomic default-session record; saved workspace views remain independent.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReaderContinuity {
    tabs: DocumentTabs,
    comparison: Option<WorkspaceComparison>,
    page_notes: Vec<crate::annotations::page_note_draft::PageNoteDraft>,
    metadata_drafts: Vec<crate::metadata_review::draft::MetadataDraft>,
    annotation_edits: Vec<crate::annotations::stored_note_draft::StoredNoteDraft>,
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
        Ok(Self {
            tabs,
            comparison,
            page_notes: Vec::new(),
            metadata_drafts: Vec::new(),
            annotation_edits: Vec::new(),
        })
    }
    pub fn with_page_notes(
        mut self,
        drafts: Vec<crate::annotations::page_note_draft::PageNoteDraft>,
    ) -> Result<Self> {
        if drafts.len() > 24 {
            return Err(Failure::Invalid(
                "Too many recovered page-note drafts; original session is preserved".into(),
            ));
        }
        for (index, draft) in drafts.iter().enumerate() {
            draft.validate()?;
            if drafts[..index]
                .iter()
                .any(|other| other.document == draft.document && other.reference == draft.reference)
            {
                return Err(Failure::Invalid(
                    "Duplicate recovered page-note editor".into(),
                ));
            }
        }
        self.page_notes = drafts;
        Ok(self)
    }
    pub fn page_notes(&self) -> &[crate::annotations::page_note_draft::PageNoteDraft] {
        &self.page_notes
    }
    pub fn with_metadata_drafts(
        mut self,
        drafts: Vec<crate::metadata_review::draft::MetadataDraft>,
    ) -> Result<Self> {
        if drafts.len() > 12 {
            return Err(Failure::Invalid("Too many metadata recovery drafts".into()));
        }
        for (index, draft) in drafts.iter().enumerate() {
            draft.validate()?;
            if drafts[..index]
                .iter()
                .any(|old| old.document == draft.document)
            {
                return Err(Failure::Invalid("Duplicate metadata recovery draft".into()));
            }
        }
        self.metadata_drafts = drafts;
        Ok(self)
    }
    pub fn metadata_drafts(&self) -> &[crate::metadata_review::draft::MetadataDraft] {
        &self.metadata_drafts
    }
    pub fn tabs(&self) -> &DocumentTabs {
        &self.tabs
    }
    pub fn with_annotation_edits(
        mut self,
        drafts: Vec<crate::annotations::stored_note_draft::StoredNoteDraft>,
    ) -> Result<Self> {
        if drafts.len() > 24 {
            return Err(Failure::Invalid(
                "Too many stored-note recovery drafts".into(),
            ));
        }
        for (index, draft) in drafts.iter().enumerate() {
            draft.validate()?;
            if drafts[..index].iter().any(|old| old.same_editor(draft)) {
                return Err(Failure::Invalid(
                    "Duplicate stored-note recovery draft".into(),
                ));
            }
        }
        self.annotation_edits = drafts;
        Ok(self)
    }
    pub fn annotation_edits(&self) -> &[crate::annotations::stored_note_draft::StoredNoteDraft] {
        &self.annotation_edits
    }
    pub fn comparison(&self) -> Option<&WorkspaceComparison> {
        self.comparison.as_ref()
    }
    pub fn forget_documents(&mut self, ids: &[String]) {
        self.tabs.forget_documents(ids);
        self.annotation_edits
            .retain(|draft| !ids.contains(&draft.document));
        self.metadata_drafts
            .retain(|draft| !ids.contains(&draft.document));
        self.page_notes
            .retain(|draft| !ids.contains(&draft.document));
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
            page_notes: Vec::new(),
            metadata_drafts: Vec::new(),
            annotation_edits: Vec::new(),
        }
    }
}
