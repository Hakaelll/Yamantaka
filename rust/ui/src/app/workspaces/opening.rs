use super::*;
impl YamantakaApp {
    pub(super) fn open_saved_workspace(
        &mut self,
        saved: yamantaka_domain::workspaces::WorkspaceReadings,
    ) {
        if self.tabs.busy()
            || self.opening.busy()
            || self.reader.reader_preferences_loading
            || self.library_selection.removing()
        {
            self.workspaces.error = Some(
                self.tr(
                    "Espera a que termine el cambio de lectura y vuelve a abrir el espacio.",
                    "Wait for the reading change to finish, then open the workspace again.",
                )
                .into(),
            );
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
