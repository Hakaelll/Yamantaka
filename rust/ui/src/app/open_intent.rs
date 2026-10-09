use yamantaka_domain::Cancellation;

/// Presentation intent is separate from native document ownership: leaving the
/// reader must not interrupt a PDF handle swap that the actor already accepted.
#[derive(Default)]
pub(super) struct OpenIntent {
    lookup_cancel: Cancellation,
    lookup: Option<(String, Option<u32>)>,
    pending_page: Option<(String, u32)>,
    reveal_reader: bool,
}

impl OpenIntent {
    pub fn leave_destination(&mut self) {
        self.lookup_cancel.cancel();
        self.lookup = None;
        self.reveal_reader = false;
    }

    pub fn lookup(&mut self, id: String, page: Option<u32>) -> Cancellation {
        self.leave_destination();
        self.lookup_cancel = Cancellation::default();
        self.lookup = Some((id, page));
        self.lookup_cancel.clone()
    }

    pub fn accept_lookup(&mut self, id: &str, cancel: &Cancellation) -> Option<Option<u32>> {
        if cancel.is_cancelled() || !self.lookup.as_ref().is_some_and(|(target, _)| target == id) {
            return None;
        }
        self.lookup.take().map(|(_, page)| page)
    }

    pub fn begin_open(&mut self) {
        self.leave_destination();
        self.pending_page = None;
        self.reveal_reader = true;
    }

    pub fn set_page(&mut self, id: String, page: u32) {
        self.pending_page = Some((id, page));
    }

    pub fn wants_reader(&self) -> bool {
        self.reveal_reader
    }

    pub fn finish_open(&mut self, id: &str) -> (Option<u32>, bool) {
        let page = if self
            .pending_page
            .as_ref()
            .is_some_and(|(target, _)| target == id)
        {
            self.pending_page.take().map(|(_, page)| page)
        } else {
            None
        };
        (page, std::mem::take(&mut self.reveal_reader))
    }
}

impl Drop for OpenIntent {
    fn drop(&mut self) {
        self.lookup_cancel.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaving_search_rejects_an_already_delivered_lookup() {
        let mut intent = OpenIntent::default();
        let reply = intent.lookup("a".into(), Some(12));
        intent.leave_destination();
        assert!(reply.is_cancelled());
        assert_eq!(intent.accept_lookup("a", &reply), None);
    }

    #[test]
    fn repeated_same_document_lookup_keeps_only_the_latest_page() {
        let mut intent = OpenIntent::default();
        let first = intent.lookup("a".into(), Some(12));
        let second = intent.lookup("a".into(), Some(37));
        assert_eq!(intent.accept_lookup("a", &first), None);
        assert_eq!(intent.accept_lookup("other", &second), None);
        assert_eq!(intent.accept_lookup("a", &second), Some(Some(37)));
        assert_eq!(intent.accept_lookup("a", &second), None);
    }

    #[test]
    fn changing_surface_during_open_preserves_page_without_stealing_focus() {
        let mut intent = OpenIntent::default();
        intent.begin_open();
        intent.set_page("a".into(), 37);
        intent.leave_destination();
        assert_eq!(intent.finish_open("a"), (Some(37), false));
    }

    #[test]
    fn opening_another_book_revokes_lookup_and_old_page() {
        let mut intent = OpenIntent::default();
        let reply = intent.lookup("a".into(), Some(12));
        intent.set_page("a".into(), 12);
        intent.begin_open();
        assert_eq!(intent.accept_lookup("a", &reply), None);
        assert_eq!(intent.finish_open("b"), (None, true));
        assert_eq!(intent.finish_open("b"), (None, false));
    }
}
