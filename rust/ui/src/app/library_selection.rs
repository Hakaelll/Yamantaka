use super::*;
use yamantaka_application::library_favorites::{FavoriteReply, FavoriteRequest};
use yamantaka_domain::library_selection::LibrarySelection;
#[derive(Default)]
pub(super) struct SelectionState {
    pub model: LibrarySelection,
    pending: Option<Cancellation>,
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
            cancel.cancel();
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
            && !self.links.confirmation_open();
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
    pub(super) fn library_selection_control(&mut self, ui: &mut egui::Ui, p: Palette) {
        let count = self.library_selection.model.ids().len();
        let label = if count > 0 {
            format!("{} ({count})", self.tr("Selección", "Selection"))
        } else if self.library_selection.notice.is_some() {
            self.tr("Resultado", "Result").to_owned()
        } else {
            self.tr("Seleccionar", "Select").to_owned()
        };
        let compact = ui.ctx().content_rect().width() < 420.0;
        let active = self.library_selection.model.mode || count > 0;
        let response=if compact {theme::icon_toggle(ui,&label,theme::Icon::Check,active,p)} else {theme::navigation(ui,&label,active,p)}
            .on_hover_text(self.tr("Ctrl + clic para alternar; Mayús + clic para un intervalo. La selección pertenece a esta página.","Ctrl-click to toggle; Shift-click for a range. Selection belongs to this page."));
        let mut popup = egui::Popup::menu(&response);
        if self.library_selection.preview_menu {
            popup = popup.open(true);
        }
        popup.show(|ui| {
            ui.set_max_width((ui.ctx().content_rect().width() - 24.0).max(160.0));
            egui::ScrollArea::vertical()
                .id_salt("library-selection-menu")
                .max_height((ui.ctx().content_rect().height() - 24.0).max(40.0))
                .show(ui, |ui| {
                    if compact {
                        ui.label(&label);
                    }
                    let busy = self.library_selection.pending.is_some();
                    if busy {
                        ui.label(self.tr("Guardando favoritos…", "Saving favorites…"));
                        if theme::navigation(
                            ui,
                            self.tr("Cancelar pendientes", "Cancel remaining"),
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.library_selection.cancel();
                        }
                    } else if count > 0 {
                        for (value, es, en) in [
                            (true, "Añadir a favoritos", "Add to favorites"),
                            (false, "Quitar de favoritos", "Remove from favorites"),
                        ] {
                            if theme::navigation(ui, self.tr(es, en), false, p).clicked() {
                                self.favorite_library_selection(value);
                                ui.close();
                            }
                        }
                    }
                    if let Some(notice) = &self.library_selection.notice {
                        ui.label(notice);
                    }
                    if count > 0 || busy || self.library_selection.notice.is_some() {
                        ui.separator();
                    }
                    ui.add_enabled_ui(!busy, |ui| {
                        let label = self.tr("Modo de selección", "Selection mode");
                        let mut mode = self.library_selection.model.mode;
                        if ui.checkbox(&mut mode, label).changed() {
                            if !mode {
                                self.library_selection.model.clear();
                            } else {
                                self.library_selection.model.mode = true;
                            }
                        }
                        if theme::navigation(
                            ui,
                            self.tr("Seleccionar esta página", "Select this page"),
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.library_selection.model.all(&self.library_page_ids());
                            ui.close();
                        }
                        if theme::navigation(
                            ui,
                            self.tr("Limpiar selección", "Clear selection"),
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.library_selection.model.clear();
                            self.library_selection.notice = None;
                            ui.close();
                        }
                    });
                });
        });
    }
    fn favorite_library_selection(&mut self, value: bool) {
        if self.library_selection.pending.is_some() || self.library_selection.model.ids().is_empty()
        {
            return;
        }
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::FavoriteSelection(Box::new(FavoriteRequest {
                ids: self.library_selection.model.ids().to_vec(),
                value,
                cancel: cancel.clone(),
            }))) {
            Ok(()) => {
                self.library_selection.pending = Some(cancel);
                self.library_selection.notice = None;
            }
            Err(error) => self.library_selection.notice = Some(error.to_string()),
        }
    }
    pub(super) fn accept_selection_favorite(&mut self, reply: FavoriteReply) {
        if !self
            .library_selection
            .pending
            .as_ref()
            .is_some_and(|cancel| cancel.same_request(&reply.cancel))
        {
            return;
        }
        self.library_selection.pending = None;
        match reply.result {
            Ok(report) => {
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
    pub fn preview_library_selection(&mut self, favorite: bool, menu: bool) {
        if self.demo && self.capture.is_some() {
            self.library_selection.preview = Some(0);
            self.library_selection.preview_favorite = favorite;
            self.library_selection.preview_menu = menu;
        }
    }
    pub(super) fn update_library_selection_preview(&mut self) {
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
                    self.favorite_library_selection(true);
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
