mod covers;
mod menu;
mod preview;
mod removal;
mod review;
use super::*;
use yamantaka_application::library_bulk::{BulkAction, BulkReply, BulkRequest};
use yamantaka_domain::library_selection::LibrarySelection;
struct PendingBulk {
    cancel: Cancellation,
    action: PendingAction,
}
enum RemovalPhase {
    Preparing,
    Running,
}
enum PendingAction {
    Removal {
        ids: Vec<String>,
        phase: RemovalPhase,
    },
    Data(BulkAction),
    Covers {
        ids: Vec<String>,
        refreshed: std::collections::HashSet<String>,
    },
}
#[derive(Clone, Copy)]
enum ReviewAction {
    Covers,
    Remove,
}
struct SelectionReview {
    action: ReviewAction,
    ids: Vec<String>,
    focus_cancel: bool,
}
#[derive(Default)]
pub(super) struct SelectionState {
    pub model: LibrarySelection,
    pending: Option<PendingBulk>,
    review: Option<SelectionReview>,
    pub(super) close_after_removal: bool,
    removal_preview: Option<removal::preview::RemovalPreview>,
    removal_verified: Option<serde_json::Value>,
    notice: Option<String>,
    pub(super) focus: Option<String>,
    pub columns: usize,
    preview: Option<u8>,
    preview_favorite: bool,
    preview_menu: bool,
    preview_targets: Vec<String>,
}
impl SelectionState {
    pub(super) fn removing(&self) -> bool {
        self.pending
            .as_ref()
            .is_some_and(|pending| matches!(pending.action, PendingAction::Removal { .. }))
    }
    pub(super) fn removing_document(&self, id: &str) -> bool {
        self.pending.as_ref().is_some_and(|pending|matches!(&pending.action,PendingAction::Removal{ids,..} if ids.iter().any(|selected|selected==id)))
    }
    pub(super) fn writing(&self) -> bool {
        self.pending.is_some()
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && self.preview.is_none() && self.removal_preview.is_none()
    }
    pub(super) fn cancel(&self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel.cancel();
        }
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"removal":self.removal_verified,"mode":self.model.mode,"ids":self.model.ids(),"writing":self.pending.is_some(),"notice":self.notice,"preview":self.preview,"verifiedFavoriteTargets":self.preview_targets})
    }
}
impl YamantakaApp {
    pub(super) fn library_page_ids(&self) -> Vec<String> {
        self.library_visible_ids()
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
            && self.library_selection.review.is_none();
        if response.has_focus() && keyboard_allowed {
            if self.library_selection.pending.is_none()
                && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Delete))
            {
                let ids = if self.library_selection.model.ids().is_empty() {
                    vec![document.id.clone()]
                } else {
                    self.library_selection.model.ids().to_vec()
                };
                self.review_removal_ids(ids);
            }
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
                for (key, target) in [
                    (egui::Key::ArrowLeft, index.saturating_sub(1)),
                    (egui::Key::ArrowRight, (index + 1).min(page.len() - 1)),
                    (
                        egui::Key::ArrowUp,
                        self.library_vertical_index(&page, index, false),
                    ),
                    (
                        egui::Key::ArrowDown,
                        self.library_vertical_index(&page, index, true),
                    ),
                    (egui::Key::Home, 0),
                    (egui::Key::End, page.len() - 1),
                ] {
                    if ui.input_mut(|input| input.consume_key(modifiers, key)) {
                        if modifiers.shift
                            && let Some(document) = self
                                .documents
                                .iter()
                                .find(|doc| doc.id == page[target])
                                .cloned()
                        {
                            self.select_library_book(&document, modifiers);
                        }
                        self.selected = Some(page[target].clone());
                        self.library_selection.focus = Some(page[target].clone());
                        ui.ctx().request_repaint();
                        break;
                    }
                }
            }
        }
        response.context_menu(|ui| {
            ui.set_max_width((ui.ctx().content_rect().width() - 24.0).max(160.0));
            let ids = if self.library_selection.model.contains(&document.id) {
                self.library_selection.model.ids().to_vec()
            } else {
                vec![document.id.clone()]
            };
            let label = if ids.len() > 1 {
                format!(
                    "{} ({})…",
                    self.tr("Retirar selección", "Remove selection"),
                    ids.len()
                )
            } else {
                self.tr("Retirar de la biblioteca…", "Remove from library…")
                    .into()
            };
            if ui
                .add_enabled_ui(!self.library_selection.writing(), |ui| {
                    theme::bounded_navigation(ui, &label, Palette::new(self.dark))
                })
                .inner
                .clicked()
            {
                self.review_removal_ids(ids);
                ui.close();
            }
        });
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
            || self.library_selection.review.is_some()
            || self.tag_dialog_open()
            || self.legacy_import.open
            || self.backup_restore.open
            || ctx.egui_wants_keyboard_input()
            || egui::Popup::is_any_open(ctx)
            || self.library_selection.pending.is_some()
        {
            return;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Delete)) {
            let ids = if self.library_selection.model.ids().is_empty() {
                self.selected
                    .iter()
                    .filter(|id| self.library_page_ids().contains(id))
                    .cloned()
                    .collect()
            } else {
                self.library_selection.model.ids().to_vec()
            };
            self.review_removal_ids(ids);
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::A)) {
            self.library_selection.model.all(&self.library_page_ids());
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.library_selection.model.clear();
        }
    }
    pub(super) fn submit_library_bulk(&mut self, action: BulkAction, ids: Vec<String>) -> bool {
        if self.library_selection.pending.is_some() || self.library_tags.writing() || ids.is_empty()
        {
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
    pub(super) fn accept_library_bulk(&mut self, ctx: &egui::Context, reply: BulkReply) {
        if !self
            .library_selection
            .pending
            .as_ref()
            .is_some_and(|cancel| cancel.cancel.same_request(&reply.cancel))
        {
            return;
        }
        let pending = self.library_selection.pending.take().unwrap();
        if let PendingAction::Covers { ids, .. } | PendingAction::Removal { ids, .. } =
            &pending.action
        {
            self.library_covers.resume(ids);
        }
        let removing = matches!(pending.action, PendingAction::Removal { .. });
        match reply.result {
            Ok(report) => {
                if removing {
                    self.reconcile_removal(ctx, &report);
                    if !report.succeeded.is_empty() {
                        self.removal_views_changed();
                    }
                }
                match pending.action {
                    PendingAction::Removal { .. } => {}
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
                    if removing {
                        self.tr("Retirados", "Removed")
                    } else {
                        self.tr("Guardados", "Saved")
                    },
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
            Err(error) => {
                self.library_selection.notice = Some(if removing {
                    match error {
                    yamantaka_domain::Failure::Cancelled=>self.tr("Retirada cancelada. No se han retirado libros.","Removal cancelled. No books were removed.").into(),
                    yamantaka_domain::Failure::QueueFull=>self.tr("La biblioteca está ocupada. Reintenta cuando termine la operación actual.","The library is busy. Retry after the current operation finishes.").into(),
                    _=>error.to_string(),
                }
                } else {
                    error.to_string()
                })
            }
        }
    }
}
