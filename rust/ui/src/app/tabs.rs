use super::*;
use std::collections::HashMap;
use yamantaka_domain::reader_tabs::{DocumentTabs, ReaderTab};
use yamantaka_domain::reading_sessions::restoration::SessionPanel;

mod bulk;
mod closing;
mod commands;
mod continuity;
mod preview;
mod preview_bulk;

mod view;

struct PendingSwitch {
    plan: DocumentTabs,
    target: ReaderTab,
    lookup: Cancellation,
    opening: Option<Cancellation>,
    retire: Vec<String>,
}
#[derive(Default)]
pub(super) struct TabState {
    pub(in crate::app) continuity: continuity::ContinuityState,
    pub(super) model: DocumentTabs,
    pub(super) captions: HashMap<String, String>,
    pending: Option<PendingSwitch>,
    restoration: Option<(ReaderTab, Cancellation)>,
    preview: preview::TabPreview,
    preview_first: Option<String>,
    pub(super) closing: closing::CloseState,
    retire_after_switch: Vec<String>,
    preview_close: usize,
    bulk_preview: preview_bulk::BulkPreview,
}
impl TabState {
    pub(super) fn cancel(&mut self) {
        if let Some(pending) = self.pending.take() {
            pending.lookup.cancel();
            if let Some(opening) = pending.opening {
                opening.cancel();
            }
        }
        self.restoration = None;
    }
    fn begin(&mut self, plan: DocumentTabs, lookup: Cancellation) {
        self.cancel();
        let target = plan.active().expect("A switch plan has a target").clone();
        let retire = removed_documents(&self.model, &plan);
        self.pending = Some(PendingSwitch {
            retire,
            plan,
            target,
            lookup,
            opening: None,
        });
    }
    fn lookup_target(&self, document: &str, token: &Cancellation) -> Option<ReaderTab> {
        self.pending
            .as_ref()
            .filter(|p| {
                p.opening.is_none()
                    && !token.is_cancelled()
                    && p.target.document() == document
                    && p.lookup.same_request(token)
            })
            .map(|p| p.target.clone())
    }
    pub(super) fn bind_open(&mut self, token: Cancellation) {
        if let Some(pending) = &mut self.pending {
            pending.opening = Some(token);
        }
    }
    pub(super) fn accept_open(
        &mut self,
        document: &str,
        token: &Cancellation,
        preferences: Cancellation,
    ) -> bool {
        if token.is_cancelled()
            || !self.pending.as_ref().is_some_and(|p| {
                p.target.document() == document
                    && p.opening.as_ref().is_some_and(|t| t.same_request(token))
            })
        {
            return false;
        }
        self.adopt(preferences);
        true
    }
    fn adopt(&mut self, preferences: Cancellation) {
        let Some(mut pending) = self.pending.take() else {
            return;
        };
        self.refresh_plan_context(&mut pending.plan);
        self.retire_after_switch = pending.retire;
        self.model = pending.plan;
        self.restoration = Some((pending.target, preferences));
    }
    pub(super) fn fail_lookup(&mut self, token: &Cancellation) -> bool {
        if self
            .pending
            .as_ref()
            .is_some_and(|p| p.opening.is_none() && p.lookup.same_request(token))
        {
            self.cancel();
            return true;
        }
        false
    }
    pub(super) fn fail_preferences(&mut self, token: &Cancellation) {
        if self
            .restoration
            .as_ref()
            .is_some_and(|(_, t)| t.same_request(token))
        {
            self.restoration = None;
        }
    }
    pub(super) fn fail(&mut self, document: &str, token: &Cancellation) {
        if self.pending.as_ref().is_some_and(|p| {
            p.target.document() == document
                && p.opening.as_ref().is_some_and(|t| t.same_request(token))
        }) {
            self.cancel();
        }
    }
    fn take_restoration(&mut self, document: &str, token: &Cancellation) -> Option<ReaderTab> {
        if token.is_cancelled()
            || !self
                .restoration
                .as_ref()
                .is_some_and(|(tab, t)| tab.document() == document && t.same_request(token))
        {
            return None;
        }
        self.restoration.take().map(|(tab, _)| tab)
    }
    pub(super) fn busy(&self) -> bool {
        self.pending.is_some() || self.closing.busy() || !self.retire_after_switch.is_empty()
    }
    pub(super) fn leave_destination(&mut self) {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.opening.is_none())
        {
            self.cancel();
        }
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none()
            && self.restoration.is_none()
            && !self.closing.busy()
            && self.retire_after_switch.is_empty()
            && self.preview.ready()
            && self.bulk_preview.ready()
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"bulkPreview":format!("{:?}",self.bulk_preview),"nativeClosures":self.closing.summary(),"closePending":self.closing.busy(),"closed":self.model.closed().iter().map(|t|serde_json::json!({"id":t.id(),"document":t.document(),"page":t.page(),"zoom":t.preferences().zoom_level,"scroll":t.preferences().scroll_offset})).collect::<Vec<_>>(),"preview":format!("{:?}",self.preview),"pending":self.pending.as_ref().map(|p|p.target.id()),
            "active":self.model.active().map(ReaderTab::id),
            "tabs":self.model.tabs().iter().map(|t|serde_json::json!({"id":t.id(),"document":t.document(),"page":t.page(),"pinned":t.pinned(),"zoom":t.preferences().zoom_level,"zoomMode":t.preferences().zoom_mode.persisted_name(),"rotation":t.preferences().page_rotation.degrees(),"horizontalMargin":t.preferences().horizontal_margin,"verticalMargin":t.preferences().vertical_margin,"scroll":t.preferences().scroll_offset})).collect::<Vec<_>>()})
    }
}
impl Drop for TabState {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn session_panel(panel: outline::ReaderPanelTab) -> SessionPanel {
    match panel {
        outline::ReaderPanelTab::Outline => SessionPanel::Outline,
        outline::ReaderPanelTab::Thumbnails => SessionPanel::Thumbnails,
        outline::ReaderPanelTab::Find => SessionPanel::Search,
        outline::ReaderPanelTab::Bookmarks => SessionPanel::Bookmarks,
        outline::ReaderPanelTab::Annotations => SessionPanel::Annotations,
        outline::ReaderPanelTab::Text => SessionPanel::Text,
    }
}
fn reader_panel(panel: SessionPanel) -> outline::ReaderPanelTab {
    match panel {
        SessionPanel::Outline => outline::ReaderPanelTab::Outline,
        SessionPanel::Thumbnails => outline::ReaderPanelTab::Thumbnails,
        SessionPanel::Search => outline::ReaderPanelTab::Find,
        SessionPanel::Bookmarks => outline::ReaderPanelTab::Bookmarks,
        SessionPanel::Annotations => outline::ReaderPanelTab::Annotations,
        SessionPanel::Text => outline::ReaderPanelTab::Text,
    }
}

fn removed_documents(before: &DocumentTabs, after: &DocumentTabs) -> Vec<String> {
    let mut documents = Vec::new();
    for tab in before.tabs() {
        if !after
            .tabs()
            .iter()
            .any(|live| live.document() == tab.document())
            && !documents.iter().any(|id| id == tab.document())
        {
            documents.push(tab.document().to_owned());
        }
    }
    documents
}
