use super::*;
#[cfg(test)]
mod tests;
mod views;
use yamantaka_application::reading_sessions::{ReadingHistoryReply, ReadingHistoryRequest};
use yamantaka_domain::reading_sessions::restoration::ReaderSessionRestoration;
type ContextResult = std::result::Result<Option<ReaderSessionRestoration>, String>;
use yamantaka_domain::reading_sessions::history::{
    READING_HISTORY_PAGE_SIZE, ReadingHistoryPage, ReadingHistoryQuery,
};

#[derive(Default)]
pub(super) struct HistoryState {
    page: Option<Arc<ReadingHistoryPage>>,
    pending: Option<(ReadingHistoryQuery, Cancellation)>,
    cursor: usize,
    error: Option<String>,
    context: Option<(usize, ContextResult)>,
    preview_restore: bool,
    preview_target: Option<(String, u32)>,
}
impl HistoryState {
    fn context_for(&mut self, page: &ReadingHistoryPage) -> ContextResult {
        if !self
            .context
            .as_ref()
            .is_some_and(|(cursor, _)| *cursor == self.cursor)
        {
            let decoded = page.entries().get(self.cursor).map_or(Ok(None), |entry| {
                yamantaka_application::reading_sessions::restoration::decode_session_context(
                    entry.context_json.as_deref(),
                )
                .map_err(|error| error.to_string())
            });
            self.context = Some((self.cursor, decoded));
        }
        self.context.as_ref().unwrap().1.clone()
    }
    pub(super) fn source_ready(&self, current: Option<(&str, u32)>) -> bool {
        self.preview_target
            .as_ref()
            .is_none_or(|(id, page)| current == Some((id.as_str(), *page)))
    }
    pub(super) fn ready(&self) -> bool {
        self.pending.is_none() && self.page.is_some() && self.error.is_none()
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"ready":self.ready(), "resident":self.page.as_ref().map_or(0,|page|page.entries().len()),"total":self.page.as_ref().map_or(0,|page|page.total()), "offset":self.page.as_ref().map_or(0,|page|page.offset()),"cursor":self.cursor,"pending":self.pending.is_some()})
    }

    fn request(&mut self, offset: usize, governor: &MemoryGovernor) -> ReadingHistoryRequest {
        if let Some((_, cancel)) = self.pending.take() {
            cancel.cancel();
        }
        let query = ReadingHistoryQuery {
            offset,
            ..Default::default()
        };
        let cancel = Cancellation::default();
        self.pending = Some((query.clone(), cancel.clone()));
        self.error = None;
        ReadingHistoryRequest {
            query,
            governor: governor.clone(),
            cancel,
        }
    }
    pub(super) fn accept(&mut self, reply: ReadingHistoryReply) {
        if reply.cancel.is_cancelled()
            || !self.pending.as_ref().is_some_and(|(query, cancel)| {
                cancel.same_request(&reply.cancel)
                    && query.offset == reply.query.offset
                    && query.document == reply.query.document
            })
        {
            return;
        }
        self.pending = None;
        match reply.result {
            Ok(page) => {
                self.page = Some(page);
                self.context = None;
                self.cursor = 0;
                self.error = None;
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(super) fn leave(&mut self) {
        if let Some((_, cancel)) = self.pending.take() {
            cancel.cancel();
        }
        self.page = None;
        self.context = None;
        self.cursor = 0;
        self.error = None;
    }
}
impl YamantakaApp {
    pub fn preview_history(&mut self, next: bool, restore: bool) {
        if self.demo && self.capture.is_some() {
            self.show_history();
            self.history.preview_restore = restore;
            if next {
                self.request_history(80);
            }
        }
    }
    pub(super) fn show_history(&mut self) {
        self.show_surface(Surface::History);
        self.request_history(0);
    }
    pub(super) fn restore_history_entry(
        &mut self,
        id: String,
        page: u32,
        context: ReaderSessionRestoration,
    ) {
        if self.annotations.editing() {
            return;
        }
        let cancel = self.open_intent.lookup_context(id.clone(), page, context);
        if let Err(error) = self.runtime.submit(Action::GetDocument(id, cancel)) {
            self.open_intent.leave_destination();
            self.error = Some(error.to_string());
        }
    }
    fn request_history(&mut self, offset: usize) {
        let request = self.history.request(offset, &self.governor);
        let query = request.query.clone();
        let cancel = request.cancel.clone();
        if let Err(error) = self
            .runtime
            .submit(Action::ReadingHistory(Box::new(request)))
        {
            self.history.accept(ReadingHistoryReply {
                query,
                cancel,
                result: Err(error),
            });
        }
    }
    pub(super) fn history_panel(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let compact = ui.available_width() < 600.0;
        egui::Frame::new().inner_margin(if compact { 12 } else { 24 }).show(ui, |ui| {
        egui::ScrollArea::vertical().id_salt("reading-journal").show(ui, |ui| {
            if !compact {
                ui.add(egui::Label::new(RichText::new(self.tr("Diario de lectura", "Reading journal"))
                    .font(egui::FontId::new(34.0, egui::FontFamily::Name("Editorial".into())))).wrap());
                ui.label(self.tr("Vuelve a las lecturas que forman tu biblioteca.", "Return to the readings that shape your library."));
                ui.add_space(16.0);
            }
            let offset = self.history.page.as_ref().map_or(0, |page| page.offset());
            if !compact {
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled_ui(self.history.pending.is_none(), |ui| {
                    if theme::navigation(ui, self.tr("Actualizar", "Refresh"), false, p).clicked() { self.request_history(offset); }
                });
                if self.history.pending.is_some() { ui.spinner(); ui.label(self.tr("Cargando lecturas…", "Loading readings…")); }
            });
            }
            if let Some(error) = &self.history.error { ui.label(error); }
            let Some(page) = self.history.page.clone() else { return; };
            if self.history.preview_restore {
                self.history.preview_restore = false;
                if let Some(entry) = page.entries().first() {
                    self.history.preview_target = Some((entry.document.clone(), entry.end_page));
                    if let Ok(Some(context)) = self.history.context_for(&page) {
                        self.restore_history_entry(entry.document.clone(),entry.end_page,context);
                    }
                }
            }
            if page.entries().is_empty() {
                if page.offset() > 0 {
                    ui.label(self.tr("Esta página ya no contiene sesiones.", "This page no longer contains sessions."));
                    ui.add_enabled_ui(self.history.pending.is_none(), |ui| {
                        if theme::navigation(ui, self.tr("Volver al inicio", "Back to first page"), false, p).clicked() { self.request_history(0); }
                    });
                    return;
                }
                ui.add_space(24.0);
                ui.heading(self.tr("Tu próxima lectura empieza aquí", "Your next reading starts here"));
                ui.label(self.tr("Las sesiones finalizadas aparecen aquí cuando activas su registro en Privacidad.", "Completed sessions appear here when recording is enabled in Privacy."));
                return;
            }
            if !compact {
            ui.add_space(12.0);
            ui.label(format!("{} · {}–{} / {}", self.tr("Sesiones", "Sessions"), page.offset().saturating_add(1), page.offset().saturating_add(page.entries().len()), page.total()));
            ui.add_space(12.0);
            }
            if ui.available_width() >= 900.0 {
                ui.columns(2, |columns| {
                    self.history_list(&mut columns[0], p, &page);
                    self.history_detail(&mut columns[1], p, &page);
                });
            } else {
                self.history_detail(ui, p, &page);
                ui.add_space(20.0);
                self.history_list(ui, p, &page);
            }
            if compact {
                ui.add_space(12.0);
                ui.label(format!("{}–{} / {}", page.offset().saturating_add(1), page.offset().saturating_add(page.entries().len()), page.total()));
                ui.add_enabled_ui(self.history.pending.is_none(), |ui| {
                    if theme::navigation(ui, self.tr("Actualizar", "Refresh"), false, p).clicked() { self.request_history(offset); }
                });
                if self.history.pending.is_some() { ui.spinner(); }
            }
            ui.add_space(16.0);
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled_ui(self.history.pending.is_none(), |ui| {
                    ui.add_enabled_ui(offset > 0, |ui| {
                        if theme::navigation(ui, self.tr("Anterior", "Previous"), false, p).clicked() { self.request_history(offset.saturating_sub(READING_HISTORY_PAGE_SIZE)); }
                    });
                    ui.add_enabled_ui((offset as u64).saturating_add(page.entries().len() as u64) < page.total(), |ui| {
                        if theme::navigation(ui, self.tr("Siguiente", "Next"), false, p).clicked() { self.request_history(offset.saturating_add(READING_HISTORY_PAGE_SIZE)); }
                    });
                });
            });
        });
        });
    }
}
