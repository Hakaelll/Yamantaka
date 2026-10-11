use crate::reader_preferences::ReaderPreferences;
use crate::reading_sessions::restoration::SessionPanel;
use crate::{Failure, MAX_HOT_DOCUMENTS, MAX_TABS, Result};

mod commands;
#[cfg(test)]
mod tests;

pub const MAX_CLOSED_TABS: usize = 10;
pub const MAX_PINNED_TABS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabResourceState {
    Active,
    Warm,
    Suspended,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReaderTab {
    id: String,
    document: String,
    page: u32,
    preferences: ReaderPreferences,
    panel: Option<SessionPanel>,
    pinned: bool,
    resource: TabResourceState,
}
impl ReaderTab {
    pub fn new(
        id: String,
        document: String,
        page: u32,
        preferences: ReaderPreferences,
        panel: Option<SessionPanel>,
    ) -> Result<Self> {
        let tab = Self {
            id,
            document,
            page,
            preferences,
            panel,
            pinned: false,
            resource: TabResourceState::Suspended,
        };
        tab.validate()?;
        Ok(tab)
    }
    fn validate(&self) -> Result<()> {
        if self.id.is_empty()
            || self.document.is_empty()
            || self.preferences.document_id != self.document
        {
            return Err(Failure::Invalid(
                "A reader tab requires its own identity and matching document preferences".into(),
            ));
        }
        self.preferences.validate()?;
        let zoom = self.preferences.zoom_level as f32;
        if !zoom.is_finite() || zoom <= 0.0 || !(self.preferences.scroll_offset as f32).is_finite()
        {
            return Err(Failure::Invalid(
                "Reader tab context cannot be represented by the native reader".into(),
            ));
        }
        Ok(())
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn document(&self) -> &str {
        &self.document
    }
    pub fn page(&self) -> u32 {
        self.page
    }
    pub fn preferences(&self) -> &ReaderPreferences {
        &self.preferences
    }
    pub fn panel(&self) -> Option<SessionPanel> {
        self.panel
    }
    pub fn pinned(&self) -> bool {
        self.pinned
    }
    pub fn resource(&self) -> TabResourceState {
        self.resource
    }
    pub fn with_pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct DocumentTabs {
    tabs: Vec<ReaderTab>,
    active: Option<String>,
    closed: Vec<ReaderTab>,
}
pub struct OpenTabOutcome {
    pub selected: String,
    pub evicted: Option<ReaderTab>,
    pub reused: bool,
}
impl DocumentTabs {
    pub fn tabs(&self) -> &[ReaderTab] {
        &self.tabs
    }
    /// Oldest closure first; the last entry is the next one reopened.
    pub fn closed(&self) -> &[ReaderTab] {
        &self.closed
    }
    pub fn active(&self) -> Option<&ReaderTab> {
        self.tabs
            .iter()
            .find(|tab| Some(tab.id.as_str()) == self.active.as_deref())
    }
    pub fn relative(&self, reverse: bool) -> Option<&ReaderTab> {
        let index = self
            .tabs
            .iter()
            .position(|tab| Some(tab.id.as_str()) == self.active.as_deref())?;
        let next = if reverse {
            (index + self.tabs.len() - 1) % self.tabs.len()
        } else {
            (index + 1) % self.tabs.len()
        };
        self.tabs.get(next)
    }
    pub fn hot_documents(&self) -> Vec<&str> {
        let mut documents = Vec::with_capacity(MAX_HOT_DOCUMENTS);
        if let Some(tab) = self.active() {
            documents.push(tab.document.as_str());
        }
        for tab in self
            .tabs
            .iter()
            .rev()
            .filter(|tab| tab.resource == TabResourceState::Warm)
        {
            if !documents.contains(&tab.document.as_str()) {
                documents.push(tab.document.as_str());
            }
        }
        documents
    }
    fn resources(&mut self) {
        let active_document = self.active().map(|tab| tab.document.clone());
        let mut warm_document = None;
        for tab in self.tabs.iter_mut().rev() {
            tab.resource = if Some(tab.id.as_str()) == self.active.as_deref() {
                TabResourceState::Active
            } else if active_document.as_deref() == Some(tab.document.as_str())
                || warm_document.as_deref() == Some(tab.document.as_str())
            {
                TabResourceState::Warm
            } else if warm_document.is_none() && MAX_HOT_DOCUMENTS > 1 {
                warm_document = Some(tab.document.clone());
                TabResourceState::Warm
            } else {
                TabResourceState::Suspended
            };
        }
    }
    fn remember(&mut self, tab: ReaderTab) {
        self.closed.push(tab);
        if self.closed.len() > MAX_CLOSED_TABS {
            self.closed.remove(0);
        }
    }
}
