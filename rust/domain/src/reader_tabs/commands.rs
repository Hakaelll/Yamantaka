use super::*;

impl DocumentTabs {
    pub fn open(&mut self, tab: ReaderTab, force_new: bool) -> Result<OpenTabOutcome> {
        tab.validate()?;
        if !force_new
            && let Some(existing) = self
                .tabs
                .iter()
                .find(|existing| existing.document == tab.document)
        {
            let selected = existing.id.clone();
            self.activate(&selected);
            return Ok(OpenTabOutcome {
                selected,
                evicted: None,
                reused: true,
            });
        }
        if self.tabs.iter().any(|existing| existing.id == tab.id) {
            return Err(Failure::Invalid(
                "Reader tab identity is already open".into(),
            ));
        }
        if tab.pinned
            && self.tabs.iter().filter(|existing| existing.pinned).count() >= MAX_PINNED_TABS
        {
            return Err(Failure::Invalid(
                "At most three new reader tabs can be pinned".into(),
            ));
        }
        let evicted = if self.tabs.len() == MAX_TABS {
            let disposable = self
                .tabs
                .iter()
                .rev()
                .find(|existing| {
                    !existing.pinned && Some(existing.id.as_str()) != self.active.as_deref()
                })
                .ok_or_else(|| {
                    Failure::Invalid(
                        "The tab limit is occupied by pinned or active readings".into(),
                    )
                })?
                .id
                .clone();
            self.close(&disposable)
        } else {
            None
        };
        let selected = tab.id.clone();
        self.closed.retain(|closed| closed.id != tab.id);
        self.tabs.push(tab);
        self.active = Some(selected.clone());
        self.resources();
        Ok(OpenTabOutcome {
            selected,
            evicted,
            reused: false,
        })
    }
    pub fn activate(&mut self, id: &str) -> bool {
        if !self.tabs.iter().any(|tab| tab.id == id) {
            return false;
        }
        self.active = Some(id.to_owned());
        self.resources();
        true
    }
    pub fn close(&mut self, id: &str) -> Option<ReaderTab> {
        let index = self.tabs.iter().position(|tab| tab.id == id)?;
        let tab = self.tabs.remove(index);
        self.remember(tab.clone());
        if self.active.as_deref() == Some(id) {
            self.active = self
                .tabs
                .get(index.min(self.tabs.len().saturating_sub(1)))
                .map(|tab| tab.id.clone());
        }
        self.resources();
        Some(tab)
    }
    /// Reconcile actual native closures while retaining contexts which can be reopened.
    pub fn retire_documents(&mut self, documents: &[String]) {
        let tabs: Vec<_> = self
            .tabs
            .iter()
            .filter(|tab| documents.contains(&tab.document))
            .map(|tab| tab.id.clone())
            .collect();
        for id in tabs {
            self.close(&id);
        }
    }
    /// Removed catalogue records must not survive in either continuity stack.
    pub fn forget_documents(&mut self, documents: &[String]) {
        self.tabs.retain(|tab| !documents.contains(&tab.document));
        self.closed.retain(|tab| !documents.contains(&tab.document));
        if self
            .active
            .as_ref()
            .is_some_and(|id| !self.tabs.iter().any(|tab| &tab.id == id))
        {
            self.active = self.tabs.last().map(|tab| tab.id.clone());
        }
        self.resources();
    }
    pub fn close_others(&mut self, id: &str) -> bool {
        if !self.tabs.iter().any(|tab| tab.id == id) {
            return false;
        }
        let ids: Vec<_> = self
            .tabs
            .iter()
            .filter(|tab| tab.id != id && !tab.pinned)
            .map(|tab| tab.id.clone())
            .collect();
        for id in ids {
            self.close(&id);
        }
        self.activate(id);
        true
    }
    pub fn close_to_right(&mut self, id: &str) -> bool {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return false;
        };
        let active_removed = self
            .tabs
            .iter()
            .skip(index + 1)
            .any(|tab| !tab.pinned && Some(tab.id.as_str()) == self.active.as_deref());
        let ids: Vec<_> = self
            .tabs
            .iter()
            .skip(index + 1)
            .filter(|tab| !tab.pinned)
            .map(|tab| tab.id.clone())
            .collect();
        for id in ids {
            self.close(&id);
        }
        if active_removed {
            self.activate(id);
        }
        true
    }
    pub fn reopen_last_closed(&mut self) -> Result<Option<String>> {
        let Some(tab) = self.closed.last() else {
            return Ok(None);
        };
        if self.tabs.len() >= MAX_TABS {
            return Err(Failure::Invalid(
                "Close a tab before reopening the last reading".into(),
            ));
        }
        if self.tabs.iter().any(|open| open.id == tab.id) {
            return Err(Failure::Invalid(
                "Closed tab identity is already open".into(),
            ));
        }
        let tab = self.closed.pop().unwrap();
        let id = tab.id.clone();
        self.tabs.push(tab);
        self.active = Some(id.clone());
        self.resources();
        Ok(Some(id))
    }
    pub fn toggle_pinned(&mut self, id: &str) -> bool {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return false;
        };
        if !self.tabs[index].pinned
            && self.tabs.iter().filter(|tab| tab.pinned).count() >= MAX_PINNED_TABS
        {
            return false;
        }
        self.tabs[index].pinned = !self.tabs[index].pinned;
        true
    }
    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        if from >= self.tabs.len() {
            return false;
        }
        let to = to.min(self.tabs.len() - 1);
        let tab = self.tabs.remove(from);
        self.tabs.insert(to, tab);
        self.resources();
        true
    }
    pub fn update_active(
        &mut self,
        page: u32,
        preferences: ReaderPreferences,
        panel: Option<SessionPanel>,
    ) -> Result<()> {
        let Some(index) = self
            .tabs
            .iter()
            .position(|tab| Some(tab.id.as_str()) == self.active.as_deref())
        else {
            return Err(Failure::Invalid("No active reader tab".into()));
        };
        let tab = &self.tabs[index];
        let mut updated = ReaderTab::new(
            tab.id.clone(),
            tab.document.clone(),
            page,
            preferences,
            panel,
        )?;
        updated.pinned = tab.pinned;
        updated.resource = tab.resource;
        self.tabs[index] = updated;
        Ok(())
    }
    /// Refresh a closure captured while its outgoing reader was still visible.
    pub fn refresh_closed(
        &mut self,
        id: &str,
        page: u32,
        preferences: ReaderPreferences,
        panel: Option<SessionPanel>,
    ) -> Result<bool> {
        let Some(index) = self.closed.iter().position(|tab| tab.id() == id) else {
            return Ok(false);
        };
        let previous = &self.closed[index];
        let mut updated = ReaderTab::new(
            previous.id.clone(),
            previous.document.clone(),
            page,
            preferences,
            panel,
        )?;
        updated.pinned = previous.pinned;
        updated.resource = previous.resource;
        self.closed[index] = updated;
        Ok(true)
    }
    /// Invalid restoration leaves the current tab set and closed stack intact.
    pub fn restore(&mut self, tabs: Vec<ReaderTab>, active: Option<&str>) -> Result<()> {
        if tabs.len() > MAX_TABS {
            return Err(Failure::Invalid(
                "A workspace exceeds the twelve-tab reader limit".into(),
            ));
        }
        for (index, tab) in tabs.iter().enumerate() {
            tab.validate()?;
            if tabs[..index].iter().any(|other| other.id == tab.id) {
                return Err(Failure::Invalid(
                    "A workspace contains duplicate tab identities".into(),
                ));
            }
        }
        let active = tabs
            .iter()
            .find(|tab| Some(tab.id.as_str()) == active)
            .or_else(|| tabs.first())
            .map(|tab| tab.id.clone());
        self.tabs = tabs;
        self.active = active;
        self.closed
            .retain(|closed| !self.tabs.iter().any(|tab| tab.id == closed.id));
        self.resources();
        Ok(())
    }

    /// Restore a complete continuity snapshot without dropping invalid entries.
    pub fn restore_continuity(
        &mut self,
        tabs: Vec<ReaderTab>,
        active: Option<&str>,
        closed: Vec<ReaderTab>,
    ) -> Result<()> {
        if closed.len() > MAX_CLOSED_TABS {
            return Err(Failure::Invalid(
                "A continuity snapshot exceeds ten closed readings".into(),
            ));
        }
        if active.is_some_and(|id| !tabs.iter().any(|tab| tab.id == id)) {
            return Err(Failure::Invalid(
                "The active continuity tab does not exist".into(),
            ));
        }
        for (index, tab) in closed.iter().enumerate() {
            tab.validate()?;
            if tabs
                .iter()
                .chain(&closed[..index])
                .any(|other| other.id == tab.id)
            {
                return Err(Failure::Invalid(
                    "Continuity contains duplicate tab identities".into(),
                ));
            }
        }
        let mut restored = Self::default();
        restored.restore(tabs, active)?;
        restored.closed = closed;
        *self = restored;
        Ok(())
    }
}
