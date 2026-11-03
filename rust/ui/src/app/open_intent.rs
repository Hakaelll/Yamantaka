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

    pub fn pending_page(&self, id: &str) -> Option<u32> {
        self.pending_page
            .as_ref()
            .filter(|(target, _)| target == id)
            .map(|(_, page)| *page)
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
