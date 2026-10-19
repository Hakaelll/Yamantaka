use super::*;
impl YamantakaApp {
    pub fn preview_library_selection(&mut self, favorite: bool, menu: bool) {
        if self.demo && self.capture.is_some() {
            self.library_selection.preview = Some(0);
            self.library_selection.preview_favorite = favorite;
            self.library_selection.preview_menu = menu;
        }
    }
    pub(in crate::app) fn update_library_selection_preview(&mut self) {
        if !self.library_loaded || self.library_selection.pending.is_some() {
            return;
        }
        match self.library_selection.preview {
            Some(0) if self.documents.len() >= 4 => {
                let page = self.documents.clone();
                self.select_library_book(&page[0], egui::Modifiers::CTRL);
                self.select_library_book(&page[2], egui::Modifiers::SHIFT);
                self.select_library_book(&page[1], egui::Modifiers::CTRL);
                self.select_library_book(&page[3], egui::Modifiers::CTRL);
                assert_eq!(
                    self.library_selection.model.ids(),
                    &[page[0].id.clone(), page[2].id.clone(), page[3].id.clone()]
                );
                if self.library_selection.preview_favorite {
                    self.library_selection.preview_targets =
                        self.library_selection.model.ids().to_vec();
                    self.submit_library_bulk(
                        BulkAction::Favorite(true),
                        self.library_selection.model.ids().to_vec(),
                    );
                    self.library_selection.preview = Some(1);
                } else {
                    self.library_selection.preview = None;
                }
            }
            Some(1) => {
                assert!(self.library_selection.model.ids().is_empty());
                for doc in &self.documents {
                    assert_eq!(
                        doc.favorite,
                        self.library_selection.preview_targets.contains(&doc.id)
                    );
                }
                self.library_selection.preview = None;
            }
            _ => {}
        }
    }
}
