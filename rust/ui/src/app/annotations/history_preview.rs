use super::*;
impl YamantakaApp {
    pub(in crate::app) fn advance_annotation_history_preview(&mut self, ctx: &egui::Context) {
        let phase = self.command_palette_preview.phase;
        if phase == 1 {
            if self.reference_notes_preview.phase() == 12 {
                shortcuts::verify_annotation_text_focus();
                self.command_palette_preview
                    .key(51, egui::Key::F6, egui::Modifiers::NONE);
            }
            return;
        }
        let Some(reference) = &self.reference else {
            return;
        };
        if reference.busy()
            || !reference.pane.annotations.ready()
            || !self.reader.annotations.ready()
        {
            return;
        }
        assert_eq!(self.reader.annotations.page.as_ref().unwrap().total, 2);
        assert_eq!(self.reader.page, 0);
        let page = reference.pane.annotations.page.as_ref().unwrap();
        let note = page
            .entries()
            .first()
            .and_then(|entry| entry.data().values.note.as_deref());
        match phase {
            51 if self.reading_focus.reference() => {
                self.command_palette_preview
                    .key(52, egui::Key::Z, egui::Modifiers::CTRL)
            }
            52 if page.total == 0 => {
                self.command_palette_preview
                    .key(53, egui::Key::Z, egui::Modifiers::CTRL)
            }
            53 if note == Some("Updated reference note") => {
                self.command_palette_preview
                    .key(54, egui::Key::Z, egui::Modifiers::CTRL)
            }
            54 if note == Some("Reference study note") => self.command_palette_preview.key(
                55,
                egui::Key::Z,
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
            ),
            55 if note == Some("Updated reference note") => self.command_palette_preview.key(
                56,
                egui::Key::Z,
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
            ),
            56 if page.total == 0 => self.command_palette_preview.key(
                57,
                egui::Key::Z,
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
            ),
            57 if note == Some("Final reference note") => {
                self.command_palette_preview
                    .key(58, egui::Key::Z, egui::Modifiers::CTRL)
            }
            58 if page.total == 0 => {
                let reference = self.reference.as_mut().unwrap();
                reference.pane.begin_page_note();
                reference.pane.annotations.editor.as_mut().unwrap().note =
                    "Branched reference note".into();
                let change = reference.pane.annotations.mutation(&self.governor).unwrap();
                self.apply_notes_action(true, NotesAction::Change(change, 0));
                self.command_palette_preview.phase = 59;
            }
            59 if note == Some("Branched reference note") => {
                assert_eq!(page.history.redo, 0);
                assert!(page.history.undo > 0);
                self.command_palette_preview.phase = 90;
            }
            _ => {}
        }
        ctx.request_repaint();
    }
}
