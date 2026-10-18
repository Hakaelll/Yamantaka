use yamantaka_domain::Cancellation;
use yamantaka_domain::reading_sessions::restoration::ReaderSessionRestoration;

pub(super) struct OpenTarget {
    pub page: Option<u32>,
    pub context: Option<ReaderSessionRestoration>,
}

/// Presentation intent is separate from native document ownership: leaving the
/// reader must not interrupt a PDF handle swap that the actor already accepted.
#[derive(Default)]
pub(super) struct OpenIntent {
    lookup_cancel: Cancellation,
    lookup: Option<(String, OpenTarget)>,
    pending_page: Option<(String, u32)>,
    inherited_navigation: Option<(String, yamantaka_domain::reader::ReaderNavigation)>,
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
        self.lookup = Some((
            id,
            OpenTarget {
                page,
                context: None,
            },
        ));
        self.lookup_cancel.clone()
    }
    pub fn lookup_context(
        &mut self,
        id: String,
        page: u32,
        context: ReaderSessionRestoration,
    ) -> Cancellation {
        let cancel = self.lookup(id, Some(page));
        if let Some((_, target)) = &mut self.lookup {
            target.context = Some(context);
        }
        cancel
    }

    pub fn accept_lookup(&mut self, id: &str, cancel: &Cancellation) -> Option<OpenTarget> {
        if cancel.is_cancelled()
            || !self.lookup_cancel.same_request(cancel)
            || !self.lookup.as_ref().is_some_and(|(target, _)| target == id)
        {
            return None;
        }
        self.lookup.take().map(|(_, page)| page)
    }

    pub fn begin_open(&mut self) {
        self.leave_destination();
        self.pending_page = None;
        self.inherited_navigation = None;
        self.reveal_reader = true;
    }

    pub fn inherit_navigation(
        &mut self,
        id: String,
        navigation: yamantaka_domain::reader::ReaderNavigation,
    ) {
        self.inherited_navigation = Some((id, navigation));
    }
    pub fn take_navigation(
        &mut self,
        id: &str,
    ) -> Option<yamantaka_domain::reader::ReaderNavigation> {
        self.inherited_navigation
            .take()
            .filter(|(target, _)| target == id)
            .map(|(_, nav)| nav)
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
    use yamantaka_application::reading_sessions::restoration::decode_session_context;
    use yamantaka_domain::reader_preferences::ReaderPreferences;

    #[test]
    fn source_navigation_retains_history_and_marks_and_transfers_once() {
        use yamantaka_domain::reader::{MarkSlot, ReaderNavigation};
        let mut intent = OpenIntent::default();
        let mut navigation = ReaderNavigation::new(10, 0).unwrap();
        navigation.go_to_page(4);
        navigation.capture_mark(MarkSlot::A);
        navigation.go_to_page(7);
        intent.begin_open();
        intent.inherit_navigation("a".into(), navigation);
        let mut restored = intent.take_navigation("a").unwrap();
        assert!(restored.go_to_page(1));
        assert!(restored.back());
        assert_eq!(restored.position().page(), 7);
        assert!(restored.jump_to_mark(MarkSlot::A));
        assert_eq!(restored.position().page(), 4);
        assert!(intent.take_navigation("a").is_none());
        intent.inherit_navigation("a".into(), restored.clone());
        assert!(intent.take_navigation("b").is_none());
        assert!(intent.take_navigation("a").is_none());
        intent.inherit_navigation("a".into(), restored);
        intent.begin_open();
        assert!(intent.take_navigation("a").is_none());
    }

    #[test]
    fn context_lookup_rejects_an_unrelated_token_and_transfers_once() {
        let mut intent = OpenIntent::default();
        let context = decode_session_context(Some(r#"{"zoomLevel":1.25}"#))
            .unwrap()
            .unwrap();
        let token = intent.lookup_context("a".into(), 1, context);
        assert!(
            intent
                .accept_lookup("a", &Cancellation::default())
                .is_none()
        );
        let target = intent.accept_lookup("a", &token).unwrap();
        assert_eq!(target.page, Some(1));
        assert_eq!(
            target
                .context
                .unwrap()
                .apply(&ReaderPreferences::new("a"))
                .unwrap()
                .zoom_level,
            1.25
        );
        assert!(intent.accept_lookup("a", &token).is_none());
    }
    #[test]
    fn new_page_lookup_and_leaving_revoke_the_old_context() {
        let mut intent = OpenIntent::default();
        let context = decode_session_context(Some("{}")).unwrap().unwrap();
        let old = intent.lookup_context("a".into(), 1, context.clone());
        let current = intent.lookup("a".into(), Some(4));
        assert!(intent.accept_lookup("a", &old).is_none());
        assert!(
            intent
                .accept_lookup("a", &current)
                .unwrap()
                .context
                .is_none()
        );
        let token = intent.lookup_context("a".into(), 1, context);
        intent.leave_destination();
        assert!(token.is_cancelled());
        assert!(intent.accept_lookup("a", &token).is_none());
    }

    #[test]
    fn leaving_search_rejects_an_already_delivered_lookup() {
        let mut intent = OpenIntent::default();
        let reply = intent.lookup("a".into(), Some(12));
        intent.leave_destination();
        assert!(reply.is_cancelled());
        assert_eq!(
            intent.accept_lookup("a", &reply).map(|target| target.page),
            None
        );
    }

    #[test]
    fn repeated_same_document_lookup_keeps_only_the_latest_page() {
        let mut intent = OpenIntent::default();
        let first = intent.lookup("a".into(), Some(12));
        let second = intent.lookup("a".into(), Some(37));
        assert_eq!(
            intent.accept_lookup("a", &first).map(|target| target.page),
            None
        );
        assert_eq!(
            intent
                .accept_lookup("other", &second)
                .map(|target| target.page),
            None
        );
        assert_eq!(
            intent.accept_lookup("a", &second).map(|target| target.page),
            Some(Some(37))
        );
        assert_eq!(
            intent.accept_lookup("a", &second).map(|target| target.page),
            None
        );
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
        assert_eq!(
            intent.accept_lookup("a", &reply).map(|target| target.page),
            None
        );
        assert_eq!(intent.finish_open("b"), (None, true));
        assert_eq!(intent.finish_open("b"), (None, false));
    }
}
