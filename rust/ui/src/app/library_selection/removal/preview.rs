use super::*;
use yamantaka_domain::{reader_preferences::ReaderPreferences, reader_tabs::ReaderTab};
pub(in crate::app::library_selection) struct RemovalPreview {
    stage: u8,
    mode: String,
}
impl YamantakaApp {
    pub fn preview_library_removal(&mut self, mode: &str) {
        if self.demo && self.capture.is_some() {
            self.preview_sessions();
            self.enable_continuity_capture();
            self.library_selection.removal_preview = Some(RemovalPreview {
                stage: 0,
                mode: mode.into(),
            });
        }
    }
    pub(in crate::app) fn update_removal_preview(&mut self) {
        let Some(mut preview) = self.library_selection.removal_preview.take() else {
            return;
        };
        let targets = vec![
            "demo-0".to_owned(),
            "demo-1".to_owned(),
            "demo-2".to_owned(),
        ];
        let reader_ready = |app: &Self, id: &str| {
            app.reader
                .active
                .as_ref()
                .is_some_and(|(document, _)| document.id == id)
                && !app.reader.reader_preferences_loading
                && app.reader.reader_preferences.is_some()
                && app.tabs.ready()
        };
        match preview.stage {
            0 if self.library_loaded => {
                if preview.mode == "last-page" {
                    assert_eq!(self.count, 81);
                    self.offset = 80;
                    self.refresh();
                    preview.stage = 10;
                } else {
                    if preview.mode == "review" {
                        self.library_selection.model.all(&targets);
                        self.open_library_removal_review();
                        self.library_selection.removal_verified =
                            Some(serde_json::json!({"mode":"review","selected":3,"remaining":10}));
                        return;
                    }
                    let document = self
                        .documents
                        .iter()
                        .find(|document| document.id == "demo-3")
                        .unwrap()
                        .clone();
                    assert!(self.open(document));
                    preview.stage = 1;
                }
            }
            1 if reader_ready(self, "demo-3") => {
                let mut preferences = self.reader.reader_preferences.clone().unwrap();
                preferences.zoom_mode = ZoomMode::Custom;
                preferences.zoom_level = 1.25;
                self.load_reader_preferences(preferences);
                self.sync_tab_context();
                let document = self
                    .documents
                    .iter()
                    .find(|document| document.id == "demo-0")
                    .unwrap()
                    .clone();
                assert!(self.open(document));
                preview.stage = 2;
            }
            2 if reader_ready(self, "demo-0") => {
                self.sync_tab_context();
                let duplicate = ReaderTab::new(
                    "duplicate-selected".into(),
                    "demo-0".into(),
                    0,
                    self.reader.reader_preferences.clone().unwrap(),
                    None,
                )
                .unwrap();
                self.tabs.model.open(duplicate, true).unwrap();
                let active = self.tabs.model.active().unwrap().id().to_owned();
                let closed = [
                    ("selected-closed", "demo-1"),
                    ("unselected-closed", "demo-3"),
                ]
                .map(|(id, document)| {
                    ReaderTab::new(
                        id.into(),
                        document.into(),
                        0,
                        ReaderPreferences::new(document),
                        None,
                    )
                    .unwrap()
                })
                .to_vec();
                self.tabs
                    .model
                    .restore_continuity(self.tabs.model.tabs().to_vec(), Some(&active), closed)
                    .unwrap();
                self.show_surface(Surface::Library);
                let source = self
                    .documents
                    .iter()
                    .find(|document| document.id == "demo-0")
                    .unwrap()
                    .path
                    .to_string_lossy()
                    .into_owned();
                self.preferences
                    .rust
                    .insert("lastDocumentPath".into(), serde_json::json!(source));
                self.library_selection.model.all(&targets);
                assert!(self.prepare_library_removal(targets.clone()));
                assert!(!self.sessions.lifecycle.settled_for(&targets).unwrap());
                if preview.mode == "cancel" {
                    self.library_selection.cancel();
                    self.runtime.wake();
                }
                preview.stage = 3;
            }
            3 if !self.library_selection.writing()
                && self.library_loaded
                && self.continuity_settled() =>
            {
                let expected = match preview.mode.as_str() {
                    "cancel" => 10,
                    "failure" => 8,
                    _ => 7,
                };
                assert_eq!(self.documents.len(), expected);
                assert!(
                    self.documents
                        .iter()
                        .any(|document| document.id == "demo-3")
                );
                let retained = self
                    .tabs
                    .model
                    .tabs()
                    .iter()
                    .find(|tab| tab.document() == "demo-3")
                    .unwrap();
                assert_eq!(retained.preferences().zoom_level, 1.25);
                assert!(
                    self.tabs
                        .model
                        .closed()
                        .iter()
                        .any(|tab| tab.id() == "unselected-closed")
                );
                if preview.mode != "cancel" {
                    assert!(
                        self.tabs
                            .model
                            .tabs()
                            .iter()
                            .all(|tab| !targets.iter().any(|id| id == tab.document()))
                    );
                    assert!(self.sessions.lifecycle.settled_for(&targets).unwrap());
                    assert!(self.reader.active.is_none());
                    if preview.mode == "failure" {
                        assert_eq!(self.library_selection.model.ids(), ["demo-0"]);
                        assert!(
                            self.tabs
                                .model
                                .closed()
                                .iter()
                                .any(|tab| tab.document() == "demo-0")
                        );
                    } else {
                        assert!(self.library_selection.model.ids().is_empty());
                        assert!(
                            self.tabs
                                .model
                                .closed()
                                .iter()
                                .all(|tab| !targets.iter().any(|id| id == tab.document()))
                        );
                        assert_eq!(
                            self.preferences
                                .get("lastDocumentPath")
                                .and_then(serde_json::Value::as_str),
                            Some("")
                        );
                    }
                } else {
                    assert_eq!(self.library_selection.model.ids(), targets);
                    assert!(
                        self.tabs
                            .model
                            .tabs()
                            .iter()
                            .any(|tab| tab.document() == "demo-0")
                    );
                }
                self.library_selection.removal_verified = Some(
                    serde_json::json!({"mode":preview.mode,"selected":3,"remaining":expected,"continuitySaved":true,"otherReadingKept":true,"sessionsSettled":self.sessions.lifecycle.settled_for(&targets).unwrap_or(false)}),
                );
                if preview.mode == "cancel" {
                    return;
                }
                let tab = self
                    .tabs
                    .model
                    .tabs()
                    .iter()
                    .find(|tab| tab.document() == "demo-3")
                    .unwrap()
                    .id()
                    .to_owned();
                assert!(self.select_reader_tab(&tab));
                preview.stage = 4;
            }
            4 if reader_ready(self, "demo-3") => {
                assert_eq!(
                    self.reader.reader_preferences.as_ref().unwrap().zoom_level,
                    1.25
                );
                self.show_surface(Surface::Library);
                self.refresh();
                preview.stage = 5;
            }
            5 if self.library_loaded && self.continuity_settled() => {
                self.library_selection.removal_verified.as_mut().unwrap()["otherReadingReopened"] =
                    serde_json::json!(true);
                assert_eq!(self.count, if preview.mode == "failure" { 8 } else { 7 });
                return;
            }
            10 if self.library_loaded => {
                assert_eq!(self.documents.len(), 1);
                let ids = vec![self.documents[0].id.clone()];
                self.library_selection.model.all(&ids);
                assert!(self.prepare_library_removal(ids));
                preview.stage = 11;
            }
            11 if !self.library_selection.writing()
                && self.library_loaded
                && self.continuity_settled() =>
            {
                assert_eq!(self.count, 80);
                assert_eq!(self.offset, 0);
                assert_eq!(self.documents.len(), 80);
                self.library_selection.removal_verified = Some(
                    serde_json::json!({"mode":"last-page","selected":1,"remaining":80,"recoveredOffset":0,"continuitySaved":true}),
                );
                return;
            }
            _ => {}
        }
        self.library_selection.removal_preview = Some(preview);
    }
}
