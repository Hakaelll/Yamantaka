use super::*;
use crate::app::navigation::PositionAction;

impl YamantakaApp {
    pub(super) fn advance_history_preview(&mut self, ctx: &egui::Context, ready: bool) {
        let preview = &mut self.reference_shortcuts_preview;
        let reference = self.reference.as_mut().unwrap();
        let reference_nav = reference.pane.navigation.as_ref().unwrap();
        match preview.phase {
            20 if ready && reference.pane.page == 1 => {
                assert_eq!(self.reader.page, 1);
                PositionAction::Save(MarkSlot::A).apply(self.reader.navigation.as_mut().unwrap());
                preview.key(21, egui::Key::ArrowLeft, egui::Modifiers::ALT);
            }
            21 if ready && reference.pane.page == 0 => {
                assert!(reference_nav.can_forward());
                assert_eq!(self.reader.page, 1);
                preview.key(22, egui::Key::ArrowRight, egui::Modifiers::ALT);
            }
            22 if ready && reference.pane.page == 1 => {
                assert_eq!(self.reader.page, 1);
                preview.key(23, egui::Key::ArrowLeft, egui::Modifiers::ALT);
            }
            23 if ready && reference.pane.page == 0 => {
                PositionAction::Save(MarkSlot::A)
                    .apply(reference.pane.navigation.as_mut().unwrap());
                preview.key(24, egui::Key::ArrowRight, egui::Modifiers::NONE);
            }
            24 if ready && reference.pane.page == 1 => {
                let nav = reference.pane.navigation.as_mut().unwrap();
                assert!(!nav.can_forward());
                PositionAction::Save(MarkSlot::B).apply(nav);
                assert!(PositionAction::Jump(MarkSlot::A).apply(nav));
                reference.reference_navigation_changed();
                preview.phase = 25;
            }
            25 if ready && reference.pane.page == 0 => {
                assert!(
                    PositionAction::Jump(MarkSlot::B)
                        .apply(reference.pane.navigation.as_mut().unwrap())
                );
                reference.reference_navigation_changed();
                preview.phase = 26;
            }
            26 if ready && reference.pane.page == 1 => {
                assert_eq!(reference_nav.mark(MarkSlot::A).unwrap().page(), 0);
                assert_eq!(reference_nav.mark(MarkSlot::B).unwrap().page(), 1);
                assert_eq!(
                    self.reader
                        .navigation
                        .as_ref()
                        .unwrap()
                        .mark(MarkSlot::A)
                        .unwrap()
                        .page(),
                    1
                );
                assert!(
                    self.reader
                        .navigation
                        .as_ref()
                        .unwrap()
                        .mark(MarkSlot::B)
                        .is_none()
                );
                preview.key(27, egui::Key::F6, egui::Modifiers::NONE);
            }
            27 if !self.reading_focus.reference() && self.reading_focus.canvas_focused(ctx) => {
                preview.key(28, egui::Key::ArrowLeft, egui::Modifiers::ALT);
            }
            28 if page_ready(&self.reader) && self.reader.page == 0 => {
                assert_eq!(reference.pane.page, 1);
                preview.key(29, egui::Key::ArrowRight, egui::Modifiers::ALT);
            }
            29 if page_ready(&self.reader) && self.reader.page == 1 => {
                assert_eq!(reference.pane.page, 1);
                preview.key(30, egui::Key::ArrowLeft, egui::Modifiers::ALT);
            }
            30 if page_ready(&self.reader) && self.reader.page == 0 => {
                preview.key(31, egui::Key::ArrowRight, egui::Modifiers::NONE);
            }
            31 if page_ready(&self.reader) && self.reader.page == 1 => {
                assert!(!self.reader.navigation.as_ref().unwrap().can_forward());
                if preview.primary_positions {
                    self.reader.panel = true;
                    self.reader.reader_panel_tab = outline::ReaderPanelTab::Positions;
                    preview.phase = 34;
                } else {
                    preview.key(32, egui::Key::F6, egui::Modifiers::NONE);
                }
            }
            32 if self.reading_focus.reference() && self.reading_focus.canvas_focused(ctx) => {
                reference.pane.reader_panel_tab = outline::ReaderPanelTab::Positions;
                reference.navigator_requested = true;
                preview.phase = 34;
            }
            _ => {}
        }
    }
}
