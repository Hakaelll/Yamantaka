use super::*;

impl YamantakaApp {
    pub(in crate::app) fn annotation_history_command(&mut self, redo: bool) {
        if !matches!(self.surface, Surface::Reader) {
            return;
        }
        let reference = self.reading_focus.reference();
        let pane = if reference {
            let Some(reference) = self
                .reference
                .as_mut()
                .filter(|reference| !reference.busy())
            else {
                return;
            };
            &mut reference.pane
        } else {
            &mut self.reader
        };
        let Some((document, _)) = &pane.active else {
            return;
        };
        pane.annotations.bind(&document.id);
        if pane.annotations.editing() {
            return;
        }
        pane.annotations.cancel_query();
        pane.reader_panel_tab = outline::ReaderPanelTab::Annotations;
        if !reference {
            pane.panel = true;
        }
        if reference && let Some(reference) = &mut self.reference {
            reference.navigator_requested = true;
        }
        self.apply_notes_action(
            reference,
            NotesAction::Change(
                if redo {
                    AnnotationMutation::Redo
                } else {
                    AnnotationMutation::Undo
                },
                0,
            ),
        );
    }
}
