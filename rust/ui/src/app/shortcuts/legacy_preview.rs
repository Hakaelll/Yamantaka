use super::*;
impl YamantakaApp {
    pub fn preview_legacy_reader_commands(&mut self, review: bool) {
        if self.demo && self.capture.is_some() {
            eprintln!("Reader command preview: phase 40");
            self.reference_shortcuts_preview.phase = 40;
            self.reference_shortcuts_preview.legacy = true;
            self.reference_shortcuts_preview.jump_review = review;
        }
    }
    pub(super) fn advance_legacy_reader_preview(&mut self, ctx: &egui::Context, ready: bool) {
        let preview = &mut self.reference_shortcuts_preview;
        let Some(reference) = self.reference.as_mut() else {
            return;
        };
        match preview.phase {
            40 if ready && self.reference_demo == 3 => {
                preview.key(41, egui::Key::J, egui::Modifiers::CTRL)
            }
            41 if self.reader.panel => {
                assert!(matches!(
                    self.reader.reader_panel_tab,
                    outline::ReaderPanelTab::Thumbnails
                ));
                preview.key(
                    42,
                    egui::Key::J,
                    egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                );
            }
            42 if !self.reader.panel => preview.key(43, egui::Key::B, egui::Modifiers::CTRL),
            43 if self.reader.bookmarks.ready() => {
                assert_eq!(self.reader.bookmarks.current().unwrap().total, 1);
                preview.key(44, egui::Key::B, egui::Modifiers::CTRL);
            }
            44 if self.reader.bookmarks.ready() => {
                assert_eq!(self.reader.bookmarks.current().unwrap().total, 0);
                preview.key(45, egui::Key::F6, egui::Modifiers::NONE);
            }
            45 if self.reading_focus.reference() && self.reading_focus.canvas_focused(ctx) => {
                preview.key(46, egui::Key::G, egui::Modifiers::CTRL);
                preview.key(46, egui::Key::PageDown, egui::Modifiers::NONE);
            }
            46 if self.page_jump.is_some() => {
                assert_eq!(self.reader.page, 0);
                assert_eq!(reference.pane.page, 0);
                if preview.jump_review {
                    preview.phase = 50;
                } else {
                    preview.key(47, egui::Key::PageDown, egui::Modifiers::NONE);
                }
            }
            47 => {
                assert_eq!(self.reader.page, 0);
                assert_eq!(reference.pane.page, 0);
                assert!(self.page_jump.is_some());
                preview.key(55, egui::Key::A, egui::Modifiers::CTRL);
                preview.pending.push(egui::Event::Text("0".into()));
                preview.key(55, egui::Key::Enter, egui::Modifiers::NONE);
            }
            55 if self
                .page_jump
                .as_ref()
                .is_some_and(|jump| jump.value == "0") =>
            {
                assert_eq!(self.reader.page, 0);
                assert_eq!(reference.pane.page, 0);
                preview.key(48, egui::Key::A, egui::Modifiers::CTRL);
                preview.pending.push(egui::Event::Text("2".into()));
                preview.key(48, egui::Key::Enter, egui::Modifiers::NONE);
            }
            48 if self.page_jump.is_none() && ready => {
                assert_eq!(self.reader.page, 0);
                assert_eq!(reference.pane.page, 1);
                preview.key(49, egui::Key::B, egui::Modifiers::CTRL);
            }
            49 if reference.pane.bookmarks.ready() => {
                assert_eq!(reference.pane.bookmarks.current().unwrap().total, 1);
                assert_eq!(
                    reference.pane.bookmarks.current().unwrap().entries[0].page_index,
                    1
                );
                assert_eq!(self.reader.bookmarks.current().unwrap().total, 0);
                self.reader.panel = true;
                preview.key(51, egui::Key::H, egui::Modifiers::CTRL);
            }
            51 if self.reader_chrome_hidden => {
                assert!(self.reader.panel);
                assert!(!self.reader_focus);
                preview.key(52, egui::Key::H, egui::Modifiers::CTRL);
            }
            52 if !self.reader_chrome_hidden => {
                preview.key(53, egui::Key::J, egui::Modifiers::CTRL)
            }
            53 if reference.pane.panel => {
                assert!(matches!(
                    reference.pane.reader_panel_tab,
                    outline::ReaderPanelTab::Thumbnails
                ));
                preview.key(
                    54,
                    egui::Key::I,
                    egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                );
            }
            54 if self.reader_focus && ready => {
                assert!(!self.reader.panel && !reference.pane.panel);
                assert_eq!(self.reader.page, 0);
                assert_eq!(reference.pane.page, 1);
                preview.phase = 50;
            }
            _ => {}
        }
    }
}
