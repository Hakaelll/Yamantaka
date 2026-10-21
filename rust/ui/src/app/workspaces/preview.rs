use super::*;
use yamantaka_domain::{
    reader_preferences::{PageRotation, ReaderTheme, ZoomMode},
    reader_tabs::{DocumentTabs, QueueEntry, QueueStatus, ReaderTab},
};
impl YamantakaApp {
    pub fn preview_workspaces(&mut self) {
        if self.demo && self.capture.is_some() {
            self.workspaces.preview = 1;
            self.tabs.continuity.overridden = true;
        }
    }
    fn workspace_fixture(&self) -> DocumentTabs {
        let mut model = DocumentTabs::default();
        for (index, document) in self.documents.iter().take(3).enumerate() {
            let mut preferences = ReaderPreferences::new(&document.id);
            preferences.zoom_mode = ZoomMode::Custom;
            preferences.zoom_level = 1.25;
            preferences.reading_theme = ReaderTheme::Sepia;
            preferences.page_rotation = PageRotation::Deg90;
            preferences.horizontal_margin = 48.0;
            let queue = if index == 0 {
                QueueEntry::new(
                    QueueStatus::Blocked,
                    "Reference notes",
                    "Compare the second chapter",
                )
            } else {
                QueueEntry::default()
            };
            let tab = ReaderTab::new(
                format!("workspace-demo-{index}"),
                document.id.clone(),
                0,
                preferences,
                None,
            )
            .unwrap()
            .with_queue(queue)
            .unwrap();
            model.open(tab, false).unwrap();
        }
        model
    }
    pub(super) fn check_workspace_preview(
        &mut self,
        saved: &yamantaka_domain::workspaces::WorkspaceReadings,
    ) {
        assert_eq!(saved.workspace.name, "Reference copy");
        assert!(saved.workspace.is_default);
        assert_eq!(saved.tabs.tabs().len(), 3);
        let first = &saved.tabs.tabs()[0];
        assert_eq!(first.queue().status(), QueueStatus::Blocked);
        assert_eq!(
            first.queue().next_action(),
            Some("Compare the second chapter")
        );
        assert_eq!(first.preferences().page_rotation, PageRotation::Deg90);
        assert_eq!(first.preferences().reading_theme, ReaderTheme::Sepia);
        assert_eq!(first.preferences().horizontal_margin, 48.0);
        assert_eq!(
            saved.tabs.active().unwrap().document(),
            saved.tabs.tabs()[2].document()
        );
    }
    pub(in crate::app) fn advance_workspace_preview(&mut self) {
        let step = self.workspaces.preview;
        if step == 0
            || !self.library_loaded
            || step <= 2 && self.documents.len() < 3
            || self.workspaces.pending.is_some()
        {
            return;
        }
        if let Some(error) = &self.workspaces.error {
            panic!("Workspace preview failed: {error}");
        }
        let find = |name: &str| {
            self.workspaces
                .entries
                .iter()
                .find(|entry| entry.name == name)
                .map(WorkspaceRevision::from)
                .unwrap()
        };
        match step {
            1 => {
                self.query = "Architecture".into();
                self.list = true;
                self.reader_focus = true;
                self.grouping.mode = super::super::library_grouping::Grouping::Author;
                self.library_density = yamantaka_domain::workspaces::WorkspaceDensity::Compact;
                self.workspaces.preview = 2;
                self.workspace_change(WorkspaceChange::Create {
                    name: "Study".into(),
                    tabs: Box::new(self.workspace_fixture()),
                    view: Box::new(self.capture_workspace_view()),
                });
            }
            2 => {
                self.workspaces.preview = 3;
                self.workspace_change(WorkspaceChange::Replace {
                    expected: find("Study"),
                    tabs: Box::new(self.workspace_fixture()),
                    view: None,
                });
            }
            3 => {
                self.workspaces.preview = 4;
                self.workspace_change(WorkspaceChange::Rename {
                    expected: find("Study"),
                    name: "Reading and study".into(),
                });
            }
            4 => {
                self.workspaces.preview = 5;
                self.workspace_change(WorkspaceChange::Duplicate {
                    expected: find("Reading and study"),
                    name: "Reference copy".into(),
                });
            }
            5 => {
                self.workspaces.preview = 6;
                self.workspace_change(WorkspaceChange::SetDefault {
                    expected: find("Reference copy"),
                });
            }
            6 => {
                self.workspaces.preview = 7;
                self.workspace_change(WorkspaceChange::Delete {
                    expected: find("Reading and study"),
                });
            }
            7 => {
                assert_eq!(self.workspaces.total, 1);
                self.query.clear();
                self.list = false;
                self.grouping.reset();
                self.reader_focus = false;
                self.workspaces.preview = 8;
                self.workspaces.open = true;
                self.workspace_request(WorkspaceCommand::Load(find("Reference copy")));
            }
            8 if self.active.is_some() && self.tabs.ready() && !self.reader_preferences_loading => {
                assert_eq!(self.query, "Architecture");
                assert!(self.list && self.reader_focus && self.workspaces.binding.is_some());
                assert!(self.grouping.mode == super::super::library_grouping::Grouping::Author);
                let active = self.tabs.model.active().unwrap().id().to_owned();
                self.tabs
                    .model
                    .update_queue(
                        &active,
                        QueueEntry::new(QueueStatus::Reviewed, "", "Study key figures"),
                    )
                    .unwrap();
                self.reader_focus = false;
                assert_eq!(self.tabs.model.tabs().len(), 3);
                assert_eq!(
                    self.reader_preferences.as_ref().unwrap().page_rotation,
                    PageRotation::Deg90
                );
                self.workspaces.preview = 9;
                self.show_workspaces();
            }
            9 => {
                use yamantaka_domain::LibraryRepository;
                let database = yamantaka_storage::Database::open_read_only(
                    &self.store.join("export-preview.sqlite3"),
                )
                .unwrap();
                let page = database.workspaces(0, &Cancellation::default()).unwrap();
                assert_eq!(page.total, 1);
                let saved = database
                    .workspace(
                        &WorkspaceRevision::from(&page.entries[0]),
                        &Cancellation::default(),
                    )
                    .unwrap();
                self.check_workspace_preview(&saved);
                assert!(!saved.view.focus);
                assert_eq!(
                    saved.tabs.active().unwrap().queue().status(),
                    QueueStatus::Reviewed
                );
                assert_eq!(
                    saved.tabs.active().unwrap().queue().next_action(),
                    Some("Study key figures")
                );
                self.reader_focus = true;
                self.workspaces.preview = 10;
            }
            10 => {
                if self
                    .workspaces
                    .binding
                    .as_ref()
                    .is_some_and(|binding| binding.focus)
                {
                    self.workspaces.preview = 0;
                }
            }
            _ => {}
        }
    }
}
