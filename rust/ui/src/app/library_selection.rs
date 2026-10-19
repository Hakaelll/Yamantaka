mod covers;
mod menu;
mod preview;
use super::*;
use yamantaka_application::library_bulk::{BulkAction, BulkReply, BulkRequest};
use yamantaka_domain::library_selection::LibrarySelection;
struct PendingBulk {
    cancel: Cancellation,
    action: PendingAction,
}
enum PendingAction {
    Data(BulkAction),
    Covers {
        ids: Vec<String>,
        refreshed: std::collections::HashSet<String>,
    },
}
struct CoverReview {
    ids: Vec<String>,
    focus_cancel: bool,
}
#[derive(Default)]
pub(super) struct SelectionState {
    pub model: LibrarySelection,
    pending: Option<PendingBulk>,
    cover_review: Option<CoverReview>,
    notice: Option<String>,
    pub(super) focus: Option<String>,
    pub columns: usize,
    preview: Option<u8>,
    preview_favorite: bool,
    preview_menu: bool,
    preview_targets: Vec<String>,
}
impl SelectionState {
    pub(super) fn writing(&self) -> bool {
        self.pending.is_some()
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && self.preview.is_none()
    }
    pub(super) fn cancel(&self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel.cancel();
        }
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"mode":self.model.mode,"ids":self.model.ids(),"writing":self.pending.is_some(),"notice":self.notice,"preview":self.preview,"verifiedFavoriteTargets":self.preview_targets})
    }
}
impl YamantakaApp {
    pub(super) fn library_page_ids(&self) -> Vec<String> {
        self.documents.iter().map(|doc| doc.id.clone()).collect()
    }
    fn select_library_book(&mut self, document: &Document, modifiers: egui::Modifiers) {
        if self.library_selection.pending.is_none() {
            self.library_selection.model.select(
                &self.library_page_ids(),
                &document.id,
                modifiers.ctrl || modifiers.command,
                modifiers.shift,
            );
            self.library_selection.notice = None;
        }
        self.selected = Some(document.id.clone());
    }
    pub(super) fn library_book_response(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        document: &Document,
    ) {
        if self.library_selection.focus.as_deref() == Some(&document.id) {
            response.request_focus();
            response.scroll_to_me(Some(egui::Align::Center));
            self.library_selection.focus = None;
        }
        let keyboard_space =
            response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Space));
        if response.clicked() && !keyboard_space {
            self.select_library_book(document, ui.input(|input| input.modifiers));
        }
        let modifiers = ui.input(|input| input.modifiers);
        let keyboard_allowed = !egui::Popup::is_any_open(ui.ctx())
            && self.password.is_none()
            && !self.links.confirmation_open()
            && self.bulk_picker.is_none()
            && self.library_selection.cover_review.is_none();
        if response.has_focus() && keyboard_allowed {
            if ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::A))
                && self.library_selection.pending.is_none()
            {
                self.library_selection.model.all(&self.library_page_ids());
            }
            if ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
                && self.library_selection.pending.is_none()
            {
                self.library_selection.model.clear();
            }
            if ui.input_mut(|input| input.consume_key(modifiers, egui::Key::Space)) {
                self.select_library_book(
                    document,
                    egui::Modifiers {
                        ctrl: true,
                        ..modifiers
                    },
                );
            }
            let page = self.library_page_ids();
            if let Some(index) = page.iter().position(|id| id == &document.id) {
                let columns = self.library_selection.columns.max(1);
                for (key, target) in [
                    (egui::Key::ArrowLeft, index.saturating_sub(1)),
                    (egui::Key::ArrowRight, (index + 1).min(page.len() - 1)),
                    (egui::Key::ArrowUp, index.saturating_sub(columns)),
                    (egui::Key::ArrowDown, (index + columns).min(page.len() - 1)),
                    (egui::Key::Home, 0),
                    (egui::Key::End, page.len() - 1),
                ] {
                    if ui.input_mut(|input| input.consume_key(modifiers, key)) {
                        if modifiers.shift {
                            self.select_library_book(&self.documents[target].clone(), modifiers);
                        }
                        self.selected = Some(page[target].clone());
                        self.library_selection.focus = Some(page[target].clone());
                        ui.ctx().request_repaint();
                        break;
                    }
                }
            }
        }
        if keyboard_allowed
            && !self.library_selection.model.mode
            && !(modifiers.ctrl || modifiers.command || modifiers.shift)
            && (response.double_clicked()
                || response.has_focus()
                    && ui.input_mut(|input| {
                        input.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                    }))
        {
            self.open(document.clone());
        }
    }
    pub(super) fn library_selection_shortcuts(&mut self, ctx: &egui::Context) {
        if !matches!(self.surface, Surface::Library)
            || self.password.is_some()
            || self.links.confirmation_open()
            || self.bulk_picker.is_some()
            || self.library_selection.cover_review.is_some()
            || ctx.egui_wants_keyboard_input()
            || egui::Popup::is_any_open(ctx)
            || self.library_selection.pending.is_some()
        {
            return;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::A)) {
            self.library_selection.model.all(&self.library_page_ids());
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.library_selection.model.clear();
        }
    }
    pub(super) fn submit_library_bulk(&mut self, action: BulkAction, ids: Vec<String>) -> bool {
        if self.library_selection.pending.is_some() || ids.is_empty() {
            return false;
        }
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::BulkSelection(Box::new(BulkRequest {
                ids,
                action: action.clone(),
                cancel: cancel.clone(),
            }))) {
            Ok(()) => {
                self.library_selection.pending = Some(PendingBulk {
                    cancel,
                    action: PendingAction::Data(action),
                });
                self.library_selection.notice = None;
                true
            }
            Err(error) => {
                self.library_selection.notice = Some(error.to_string());
                false
            }
        }
    }
    pub(super) fn accept_library_bulk(&mut self, reply: BulkReply) {
        if !self
            .library_selection
            .pending
            .as_ref()
            .is_some_and(|cancel| cancel.cancel.same_request(&reply.cancel))
        {
            return;
        }
        let pending = self.library_selection.pending.take().unwrap();
        if let PendingAction::Covers { ids, .. } = &pending.action {
            self.library_covers.resume(ids);
        }
        match reply.result {
            Ok(report) => {
                match pending.action {
                    PendingAction::Data(BulkAction::Tag(_)) => {
                        self.bulk_tags_changed();
                        if self.search.enabled {
                            self.send(Action::IndexStatistics);
                        }
                    }
                    PendingAction::Data(BulkAction::Collection { .. } | BulkAction::Archive) => {
                        self.bulk_collections_changed()
                    }
                    PendingAction::Data(BulkAction::Favorite(_)) | PendingAction::Covers { .. } => {
                    }
                }

                let message = format!(
                    "{}: {} · {}: {} · {}: {}",
                    self.tr("Guardados", "Saved"),
                    report.succeeded.len(),
                    self.tr("Fallidos", "Failed"),
                    report.failed.len(),
                    self.tr("Pendientes", "Remaining"),
                    report.remaining.len()
                );
                let retry: Vec<_> = report
                    .failed
                    .into_iter()
                    .chain(report.remaining)
                    .filter(|id| self.documents.iter().any(|doc| &doc.id == id))
                    .collect();
                self.library_selection.model.all(&retry);
                self.library_selection.model.mode = !retry.is_empty();
                self.library_selection.notice = Some(message);
                self.refresh();
            }
            Err(error) => self.library_selection.notice = Some(error.to_string()),
        }
    }
}
