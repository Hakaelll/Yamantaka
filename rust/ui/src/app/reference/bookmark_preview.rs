use super::*;
use yamantaka_domain::bookmarks::BookmarkChange;

impl YamantakaApp {
    pub fn preview_reference_bookmarks(&mut self, shared: bool) {
        if self.demo {
            self.reference_bookmarks_demo = true;
            if shared {
                self.reference_demo = 29;
            }
        }
    }

    pub(super) fn advance_reference_bookmarks_demo(&mut self) -> bool {
        if !(20..=35).contains(&self.reference_demo) {
            return false;
        }
        if self.reference_demo == 32 {
            self.reader.bookmarks.prepare(&self.runtime, "demo-0");
        }
        let ready = self
            .reference
            .as_ref()
            .is_some_and(|reference| reference.pane.bookmarks.ready() && !reference.pane.loading);
        match self.reference_demo {
            20 if ready && self.reader.bookmarks.ready() => {
                assert_eq!(self.reader.bookmarks.current().unwrap().total, 2);
                let reference = self.reference.as_mut().unwrap();
                assert_eq!(reference.pane.bookmarks.current().unwrap().total, 0);
                reference.pane.bookmarks.command(
                    &self.runtime,
                    0,
                    Some(BookmarkChange::Ensure { page: 1 }),
                );
                self.reference_demo = 21;
            }
            21 if ready => {
                let reference = self.reference.as_mut().unwrap();
                let page = reference.pane.bookmarks.current().unwrap();
                assert_eq!(page.total, 1);
                assert_eq!(page.entries[0].document_id, "demo-1");
                assert_eq!(page.entries[0].page_index, 1);
                let id = page.entries[0].id.clone();
                reference.pane.bookmarks.command(
                    &self.runtime,
                    0,
                    Some(BookmarkChange::Rename {
                        id,
                        label: Some("A passage worth keeping".into()),
                    }),
                );
                self.reference_demo = 22;
            }
            22 if ready => {
                let reference = self.reference.as_mut().unwrap();
                assert_eq!(
                    reference.pane.bookmarks.current().unwrap().entries[0]
                        .label
                        .as_deref(),
                    Some("A passage worth keeping")
                );
                reference.set_page(0);
                self.reference_demo = 23;
            }
            23 if ready => {
                let reference = self.reference.as_mut().unwrap();
                assert_eq!(reference.pane.page, 0);
                let page = reference.pane.bookmarks.current().unwrap().entries[0].page_index;
                reference.set_page(page);
                self.reference_demo = 24;
            }
            24 if ready => {
                let reference = self.reference.as_mut().unwrap();
                assert_eq!(reference.pane.page, 1);
                assert_eq!(self.reader.page, 0);
                let id = reference.pane.bookmarks.current().unwrap().entries[0]
                    .id
                    .clone();
                reference.pane.bookmarks.command(
                    &self.runtime,
                    0,
                    Some(BookmarkChange::Remove { id }),
                );
                self.reference_demo = 25;
            }
            25 if ready => {
                let reference = self.reference.as_mut().unwrap();
                assert_eq!(reference.pane.bookmarks.current().unwrap().total, 0);
                reference.pane.bookmarks.command(
                    &self.runtime,
                    0,
                    Some(BookmarkChange::Ensure { page: 1 }),
                );
                self.reference_demo = 26;
            }
            26 if ready => {
                let reference = self.reference.as_mut().unwrap();
                let id = reference.pane.bookmarks.current().unwrap().entries[0]
                    .id
                    .clone();
                reference.pane.bookmarks.command(
                    &self.runtime,
                    0,
                    Some(BookmarkChange::Rename {
                        id,
                        label: Some("Compare this passage".into()),
                    }),
                );
                self.reference_demo = 27;
            }
            27 if ready => {
                let reference = self.reference.as_ref().unwrap();
                let page = reference.pane.bookmarks.current().unwrap();
                assert_eq!(page.total, 1);
                assert_eq!(
                    page.entries[0].label.as_deref(),
                    Some("Compare this passage")
                );
                assert_eq!(self.reader.bookmarks.current().unwrap().total, 2);
                assert_eq!(self.reader.page, 0);
                self.reference_demo = 28;
            }
            29 if self.reference.as_ref().is_some_and(|reference| {
                !reference.busy()
                    && !reference.pane.loading
                    && !reference.pane.viewport.pages.is_empty()
            }) =>
            {
                self.reference.as_mut().unwrap().bookmarks_preview = true;
                self.reader.bookmarks.prepare(&self.runtime, "demo-0");
                self.reference_demo = 30;
            }
            30 if ready && self.reader.bookmarks.ready() => {
                let reference = self.reference.as_mut().unwrap();
                let page = reference.pane.bookmarks.current().unwrap();
                assert_eq!(page.total, 2);
                let id = page
                    .entries
                    .iter()
                    .find(|entry| entry.page_index == 1)
                    .unwrap()
                    .id
                    .clone();
                reference.pane.bookmarks.command(
                    &self.runtime,
                    0,
                    Some(BookmarkChange::Rename {
                        id,
                        label: Some("Shared reading note".into()),
                    }),
                );
                self.reader.bookmarks.command(
                    &self.runtime,
                    0,
                    Some(BookmarkChange::Ensure { page: 0 }),
                );
                self.reference_demo = 31;
            }
            31 if ready => {
                assert!(!self.reader.bookmarks.ready());
                self.reference_demo = 32;
            }
            32 if ready && self.reader.bookmarks.ready() => {
                let page = self.reader.bookmarks.current().unwrap();
                assert_eq!(page.total, 2);
                assert!(
                    page.entries
                        .iter()
                        .any(|entry| entry.label.as_deref() == Some("Shared reading note"))
                );
                let reference = self.reference.as_mut().unwrap();
                let id = reference
                    .pane
                    .bookmarks
                    .current()
                    .unwrap()
                    .entries
                    .iter()
                    .find(|entry| entry.page_index == 0)
                    .unwrap()
                    .id
                    .clone();
                reference.pane.bookmarks.command(
                    &self.runtime,
                    0,
                    Some(BookmarkChange::Remove { id }),
                );
                self.request_reference_close();
                self.reference_demo = 33;
            }
            33 if self.reference.is_none() && !self.reader.bookmarks.ready() => {
                self.reader.panel = true;
                self.reader.reader_panel_tab = outline::ReaderPanelTab::Bookmarks;
                self.reader.bookmarks.prepare(&self.runtime, "demo-0");
                self.reference_demo = 34;
            }
            34 if self.reader.bookmarks.ready() => {
                let page = self.reader.bookmarks.current().unwrap();
                assert_eq!(page.total, 1);
                assert_eq!(page.entries[0].page_index, 1);
                assert_eq!(
                    page.entries[0].label.as_deref(),
                    Some("Shared reading note")
                );
                assert!(self.reference.is_none());
                assert_eq!(self.reader.page, 0);
                self.reference_demo = 35;
            }
            _ => {}
        }
        true
    }
}
