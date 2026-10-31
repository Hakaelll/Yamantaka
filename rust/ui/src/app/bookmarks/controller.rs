use super::*;

#[derive(Default)]
pub(in crate::app) struct BookmarkState {
    pub(super) document: Option<String>,
    pub(super) page: Option<BookmarkPage>,
    pub(super) pending: Option<(Cancellation, bool)>,
    pub(super) clear_edit: bool,
    pub(super) saved: bool,
    pub(super) error: Option<String>,
    pub(super) editing: Option<Bookmark>,
    pub(super) label: String,
    stale: bool,
}

impl BookmarkState {
    pub(in crate::app) fn current(&self) -> Option<&BookmarkPage> {
        self.page.as_ref()
    }

    pub(in crate::app) fn ready(&self) -> bool {
        self.page.is_some() && self.pending.is_none() && !self.stale
    }
    fn bind(&mut self, document: &str) {
        if self.document.as_deref() == Some(document) {
            return;
        }
        *self = Self::default();
        self.document = Some(document.into());
    }

    pub(in crate::app) fn accept(
        &mut self,
        document: &str,
        cancel: &Cancellation,
        result: Result<BookmarkPage, String>,
    ) -> bool {
        if !self.owns(document, cancel) {
            return false;
        }
        let writing = self.pending.take().unwrap().1;
        match result {
            Ok(page) => {
                self.stale = false;
                self.page = Some(page);
                self.error = None;
                self.saved = writing;
                if writing && self.clear_edit {
                    self.editing = None;
                    self.label.clear();
                }
            }
            Err(error) => self.error = Some(error),
        }
        true
    }
}

impl Drop for BookmarkState {
    fn drop(&mut self) {
        self.cancel_query();
    }
}

impl BookmarkState {
    pub(in crate::app) fn owns(&self, document: &str, cancel: &Cancellation) -> bool {
        !cancel.is_cancelled()
            && self.document.as_deref() == Some(document)
            && self
                .pending
                .as_ref()
                .is_some_and(|(pending, _)| pending.same_request(cancel))
    }

    pub(in crate::app) fn prepare(&mut self, runtime: &Runtime, document: &str) {
        self.bind(document);
        if (self.page.is_none() || self.stale) && self.pending.is_none() && self.error.is_none() {
            let offset = self.page.as_ref().map_or(0, |page| page.offset);
            self.command(runtime, offset, None);
        }
    }

    pub(in crate::app) fn cancel_query(&mut self) {
        if self.pending.as_ref().is_some_and(|(_, writing)| !writing) {
            self.pending.take().unwrap().0.cancel();
            self.stale = true;
        }
    }

    pub(in crate::app) fn invalidate(&mut self, document: &str, origin: &Cancellation) {
        if self.document.as_deref() != Some(document) || self.owns(document, origin) {
            return;
        }
        self.cancel_query();
        self.stale = true;
        self.saved = false;
    }

    pub(in crate::app) fn capture_summary(&self) -> serde_json::Value {
        serde_json::json!({"document":self.document,"pending":self.pending.is_some(),
            "saved":self.saved,"total":self.page.as_ref().map(|page|page.total),
            "offset":self.page.as_ref().map(|page|page.offset),
            "entries":self.page.as_ref().map(|page|&page.entries)})
    }
    pub(in crate::app) fn toggle(&mut self, runtime: &Runtime, document: &str, page: u32) {
        self.bind(document);
        if self.pending.as_ref().is_some_and(|(_, writing)| *writing) {
            return;
        }
        if let Some((cancel, _)) = self.pending.take() {
            cancel.cancel();
        }
        let offset = self.page.as_ref().map_or(0, |page| page.offset);
        self.command(runtime, offset, Some(BookmarkChange::Toggle { page }));
    }
    pub(in crate::app) fn command(
        &mut self,
        runtime: &Runtime,
        offset: usize,
        change: Option<BookmarkChange>,
    ) {
        if self.pending.is_some() {
            return;
        }
        let Some(document) = self.document.clone() else {
            return;
        };
        let cancel = Cancellation::default();
        let writing = change.is_some();
        self.clear_edit = self.editing.as_ref().is_some_and(|editing| {
            matches!(&change, Some(BookmarkChange::Rename { id, .. } | BookmarkChange::Remove { id }) if id == &editing.id) || matches!(&change,Some(BookmarkChange::Toggle {page}) if *page==editing.page_index)
        });
        let action = match change {
            Some(change) => Action::ChangeBookmark(document, change, offset, cancel.clone()),
            None => Action::Bookmarks(document, offset, cancel.clone()),
        };
        self.error = None;
        self.saved = false;
        match runtime.submit(action) {
            Ok(()) => {
                self.pending = Some((cancel, writing));
                if !writing {
                    self.stale = false;
                }
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
}
