use super::*;

impl YamantakaApp {
    pub(in crate::app) fn sync_tab_context(&mut self) {
        let (Some((document, _)), Some(preferences)) =
            (&self.reader.active, &self.reader.reader_preferences)
        else {
            return;
        };
        if preferences.document_id != document.id || self.reader.reader_preferences_loading {
            return;
        }
        let panel = self
            .reader
            .panel
            .then(|| session_panel(self.reader.reader_panel_tab));
        if !self
            .tabs
            .model
            .active()
            .is_some_and(|t| t.document() == document.id)
        {
            let tab = ReaderTab::new(
                uuid::Uuid::new_v4().to_string(),
                document.id.clone(),
                self.reader.page,
                preferences.clone(),
                panel,
            );
            match tab.and_then(|tab| self.tabs.model.open(tab, false)) {
                Ok(_) => {}
                Err(error) => {
                    self.error = Some(error.to_string());
                    return;
                }
            }
        }
        if let Err(error) =
            self.tabs
                .model
                .update_active(self.reader.page, preferences.clone(), panel)
        {
            self.error = Some(error.to_string());
        }
        self.tabs.captions.insert(
            document.id.clone(),
            document.display_title().chars().take(512).collect(),
        );
        self.tabs.captions.retain(|id, _| {
            self.tabs
                .model
                .tabs()
                .iter()
                .chain(self.tabs.model.closed())
                .any(|t| t.document() == id)
        });
    }
    pub(in crate::app) fn select_reader_tab(&mut self, id: &str) -> bool {
        self.select_tab_policy(id, false)
    }
    pub(in crate::app) fn restore_reader_tab(&mut self, id: &str) -> bool {
        self.select_tab_policy(id, true)
    }
    fn select_tab_policy(&mut self, id: &str, automatic: bool) -> bool {
        self.sync_tab_context();
        let mut plan = self.tabs.model.clone();
        if !plan.activate(id) {
            return false;
        }
        self.request_tab_plan_policy(plan, automatic)
    }
    pub(in crate::app) fn request_tab_plan(&mut self, plan: DocumentTabs) -> bool {
        self.request_tab_plan_policy(plan, false)
    }
    pub(in crate::app) fn request_restored_tab_plan(&mut self, plan: DocumentTabs) -> bool {
        self.request_tab_plan_policy(plan, true)
    }
    fn request_tab_plan_policy(&mut self, plan: DocumentTabs, automatic: bool) -> bool {
        if self.reference_blocks_switch() {
            return false;
        }
        if self.library_selection.removing() {
            return false;
        }
        if self.tabs.closing.busy() {
            return false;
        }
        if let Err(error) = self
            .tabs
            .closing
            .can_enqueue(&removed_documents(&self.tabs.model, &plan))
        {
            self.error = Some(error.to_string());
            return false;
        }
        if self.reader.annotations.editing() {
            self.error = Some(
                self.tr(
                    "Termina o cancela la nota antes de cambiar de lectura.",
                    "Finish or cancel the note before switching readings.",
                )
                .into(),
            );
            return false;
        }
        let Some(target) = plan.active().cloned() else {
            return self.request_empty_tab_plan(plan);
        };
        if self
            .reader
            .active
            .as_ref()
            .is_some_and(|(d, _)| d.id == target.document())
            && self.reader.reader_preferences.is_some()
        {
            self.cancel_reader_opening();
            self.tabs.begin(plan, Cancellation::default());
            self.tabs
                .adopt(self.reader.reader_preferences_cancel.clone());
            let preferences = self.reader.reader_preferences.clone().unwrap();
            self.navigate(target.page());
            self.load_reader_preferences(preferences);
            self.show_surface(Surface::Reader);
            return true;
        }
        let token = Cancellation::default();
        if let Err(error) = self.runtime.submit(if automatic {
            Action::GetRestorableDocument(target.document().into(), token.clone())
        } else {
            Action::GetDocument(target.document().into(), token.clone())
        }) {
            self.error = Some(error.to_string());
            return false;
        }
        self.cancel_reader_opening();
        self.tabs.begin(plan, token);
        true
    }
    pub(in crate::app) fn duplicate_reader_tab(&mut self) {
        if let Some(id) = self.tabs.model.active().map(|t| t.id().to_owned()) {
            self.duplicate_reader_tab_from(&id);
        }
    }
    pub(in crate::app) fn duplicate_reader_tab_from(&mut self, id: &str) {
        self.sync_tab_context();
        let Some(current) = self.tabs.model.tabs().iter().find(|t| t.id() == id) else {
            return;
        };
        let tab = ReaderTab::new(
            uuid::Uuid::new_v4().to_string(),
            current.document().into(),
            current.page(),
            current.preferences().clone(),
            current.panel(),
        );
        let mut plan = self.tabs.model.clone();
        match tab
            .and_then(|tab| tab.with_queue(current.queue().clone()))
            .and_then(|tab| plan.open(tab, true))
        {
            Ok(_) => {
                self.request_tab_plan(plan);
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(in crate::app) fn relative_reader_tab(&mut self, reverse: bool) {
        if let Some(id) = self
            .tabs
            .model
            .relative(reverse)
            .map(|tab| tab.id().to_owned())
        {
            self.select_reader_tab(&id);
        }
    }
    pub(in crate::app) fn accept_tab_lookup(
        &mut self,
        id: &str,
        document: Option<Document>,
        cancel: &Cancellation,
    ) -> bool {
        let Some(target) = self.tabs.lookup_target(id, cancel) else {
            return false;
        };
        let Some(document) = document.filter(|d| d.id == id && !d.missing) else {
            self.tabs.cancel();
            self.error = Some(
                self.tr(
                    "El documento ya no está disponible. Localízalo desde la biblioteca.",
                    "The document is unavailable. Locate it from the library.",
                )
                .into(),
            );
            return true;
        };
        if self.open_candidate(document) {
            self.open_intent.set_page(id.into(), target.page());
            self.tabs
                .bind_open(self.opening.token().expect("Admitted opening has a token"));
        } else {
            self.tabs.cancel();
        }
        true
    }
    pub(in crate::app) fn restore_tab_preferences(
        &mut self,
        preferences: ReaderPreferences,
    ) -> ReaderPreferences {
        let Some(tab) = self.tabs.take_restoration(
            &preferences.document_id,
            &self.reader.reader_preferences_cancel,
        ) else {
            return preferences;
        };
        self.reader.panel = tab.panel().is_some();
        if let Some(panel) = tab.panel() {
            self.reader.reader_panel_tab = reader_panel(panel);
        }
        self.reader.restoration.scroll = Some(tab.preferences().scroll_offset);
        tab.preferences().clone()
    }
}
