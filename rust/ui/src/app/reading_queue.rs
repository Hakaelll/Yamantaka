use super::*;
use yamantaka_domain::reader_tabs::{QueueEntry, QueueStatus};
#[derive(Default)]
pub(super) struct QueueState {
    pub(super) open: bool,
    edit: Option<(String, QueueStatus, String, String)>,
    focus_close: bool,
    preview: bool,
}
impl QueueState {
    pub(super) fn ready(&self) -> bool {
        !self.preview
    }
}
fn status_label(status: QueueStatus, english: bool) -> &'static str {
    match (status, english) {
        (QueueStatus::Pending, false) => "Pendiente",
        (QueueStatus::Pending, true) => "Pending",
        (QueueStatus::InReview, false) => "En revisión",
        (QueueStatus::InReview, true) => "In review",
        (QueueStatus::Blocked, false) => "Bloqueada",
        (QueueStatus::Blocked, true) => "Blocked",
        (QueueStatus::Reviewed, false) => "Revisada",
        (QueueStatus::Reviewed, true) => "Reviewed",
        (QueueStatus::Archived, false) => "Archivada",
        (QueueStatus::Archived, true) => "Archived",
    }
}
impl YamantakaApp {
    pub fn preview_reading_queue(&mut self) {
        if self.demo && self.capture.is_some() {
            self.reading_queue.preview = true;
            self.tabs.continuity.overridden = true;
        }
    }
    pub(super) fn advance_queue_preview(&mut self) {
        if !self.reading_queue.preview || !self.library_loaded || self.documents.len() < 3 {
            return;
        }
        use yamantaka_domain::reader_tabs::{DocumentTabs, ReaderTab};
        let mut model = DocumentTabs::default();
        for (index, document) in self.documents.iter().take(3).enumerate() {
            model
                .open(
                    ReaderTab::new(
                        format!("queue-{index}"),
                        document.id.clone(),
                        0,
                        ReaderPreferences::new(document.id.clone()),
                        None,
                    )
                    .unwrap(),
                    false,
                )
                .unwrap();
            self.tabs
                .captions
                .insert(document.id.clone(), document.display_title().into());
        }
        model
            .update_queue(
                "queue-0",
                QueueEntry::new(
                    QueueStatus::Blocked,
                    "Awaiting reference notes",
                    "Read the second chapter",
                ),
            )
            .unwrap();
        model
            .update_queue("queue-2", QueueEntry::new(QueueStatus::InReview, "", ""))
            .unwrap();
        model.reorder(2, 0);
        model
            .update_queue("queue-2", QueueEntry::new(QueueStatus::Reviewed, "", ""))
            .unwrap();
        assert_eq!(model.next_queued().unwrap().id(), "queue-1");
        let active = model.active().unwrap();
        let preferences = active.preferences().clone();
        model.update_active(0, preferences, None).unwrap();
        assert_eq!(
            model.active().unwrap().queue().status(),
            QueueStatus::Reviewed
        );
        assert_eq!(model.next_queued().unwrap().id(), "queue-1");
        let mut closed = model.clone();
        let tab = closed.close("queue-0").unwrap();
        closed
            .refresh_closed(tab.id(), 0, tab.preferences().clone(), None)
            .unwrap();
        assert_eq!(closed.closed().last().unwrap().queue(), tab.queue());
        model
            .update_queue("queue-2", QueueEntry::new(QueueStatus::InReview, "", ""))
            .unwrap();
        let file = self.capture.as_ref().unwrap().with_extension("queue.json");
        yamantaka_storage::reader_continuity::save(&file, &model).unwrap();
        let loaded = yamantaka_storage::reader_continuity::load(&file)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.active().unwrap().id(), "queue-2");
        assert_eq!(loaded.tabs().len(), 3);
        for (saved, original) in loaded.tabs().iter().zip(model.tabs()) {
            assert_eq!(saved.id(), original.id());
            assert_eq!(saved.queue(), original.queue());
        }
        assert_eq!(
            loaded.tabs()[1].queue().blocking_reason(),
            Some("Awaiting reference notes")
        );
        self.tabs.model = loaded;
        self.reading_queue.preview = false;
        self.show_reading_queue();
    }
    pub(super) fn show_reading_queue(&mut self) {
        self.sync_tab_context();
        self.reading_queue.open = true;
        self.reading_queue.focus_close = true;
    }
    pub(super) fn reading_queue_dialog(&mut self, ctx: &egui::Context) {
        if !self.reading_queue.open {
            return;
        }
        self.sync_tab_context();
        let mut state = std::mem::take(&mut self.reading_queue);
        let p = Palette::new(self.dark);
        let mut close = false;
        let mut select = None;
        let mut reorder = None;
        let mut update = None;
        let entries: Vec<_> = self
            .tabs
            .model
            .tabs()
            .iter()
            .map(|tab| {
                (
                    tab.id().to_owned(),
                    self.tabs
                        .captions
                        .get(tab.document())
                        .cloned()
                        .unwrap_or_else(|| self.tr("Documento", "Document").into()),
                    tab.queue().clone(),
                )
            })
            .collect();
        let active = self.tabs.model.active().map(|tab| tab.id().to_owned());
        let busy = self.tabs.busy()
            || self.opening.busy()
            || self.reader_preferences_loading
            || self.library_selection.removing();
        let modal = egui::Modal::new(egui::Id::new("reading-queue"))
            .frame(
                egui::Frame::new()
                    .fill(p.surface)
                    .stroke(egui::Stroke::new(1.0, p.line))
                    .corner_radius(12)
                    .inner_margin(16),
            )
            .show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width() - 56.0).clamp(180.0, 620.0));
                theme::display(ui, self.tr("Cola de lectura", "Reading queue"), 24.0);
                ui.label(self.tr(
                    "Organiza tus lecturas abiertas y elige el siguiente paso.",
                    "Order your open readings and choose the next step.",
                ));
                egui::ScrollArea::vertical()
                    .max_height((ctx.content_rect().height() - 200.0).clamp(40.0, 480.0))
                    .show(ui, |ui| {
                        ui.add_enabled_ui(!busy, |ui| {
                            if entries.is_empty() {
                                ui.label(self.tr(
                                    "Abre un documento para añadirlo a la cola.",
                                    "Open a document to add it to the queue.",
                                ));
                            }
                            for (index, (id, title, entry)) in entries.iter().enumerate() {
                                ui.push_id(id, |ui| {
                                    let selected = active.as_ref() == Some(id);
                                    if theme::bounded_navigation(ui, title, p).clicked() {
                                        select = Some(id.clone());
                                    }
                                    ui.label(
                                        RichText::new(status_label(entry.status(), self.english))
                                            .color(if entry.status() == QueueStatus::Blocked {
                                                p.accent
                                            } else {
                                                p.secondary
                                            }),
                                    );
                                    if let Some(reason) = entry.blocking_reason() {
                                        ui.label(format!(
                                            "{}: {reason}",
                                            self.tr("Bloqueo", "Blocker")
                                        ));
                                    }
                                    if let Some(action) = entry.next_action() {
                                        ui.label(format!(
                                            "{}: {action}",
                                            self.tr("Próximo paso", "Next step")
                                        ));
                                    }
                                    ui.horizontal_wrapped(|ui| {
                                        if theme::bounded_navigation(
                                            ui,
                                            self.tr("Editar", "Edit"),
                                            p,
                                        )
                                        .clicked()
                                        {
                                            state.edit = Some((
                                                id.clone(),
                                                entry.status(),
                                                entry.blocking_reason().unwrap_or_default().into(),
                                                entry.next_action().unwrap_or_default().into(),
                                            ));
                                        }
                                        if ui
                                            .add_enabled_ui(index > 0, |ui| {
                                                theme::bounded_navigation(
                                                    ui,
                                                    self.tr("Antes", "Earlier"),
                                                    p,
                                                )
                                            })
                                            .inner
                                            .clicked()
                                        {
                                            reorder = Some((index, index - 1));
                                        }
                                        if ui
                                            .add_enabled_ui(index + 1 < entries.len(), |ui| {
                                                theme::bounded_navigation(
                                                    ui,
                                                    self.tr("Después", "Later"),
                                                    p,
                                                )
                                            })
                                            .inner
                                            .clicked()
                                        {
                                            reorder = Some((index, index + 1));
                                        }
                                        if selected {
                                            ui.label(self.tr("Lectura actual", "Current reading"));
                                        }
                                    });
                                    if let Some((editing, status, reason, action)) = &mut state.edit
                                        && editing == id
                                    {
                                        ui.horizontal_wrapped(|ui| {
                                            for choice in [
                                                QueueStatus::Pending,
                                                QueueStatus::InReview,
                                                QueueStatus::Blocked,
                                                QueueStatus::Reviewed,
                                                QueueStatus::Archived,
                                            ] {
                                                if theme::navigation(
                                                    ui,
                                                    status_label(choice, self.english),
                                                    *status == choice,
                                                    p,
                                                )
                                                .clicked()
                                                {
                                                    *status = choice;
                                                }
                                            }
                                        });
                                        ui.label(self.tr("Motivo del bloqueo", "Blocking reason"));
                                        ui.add(
                                            egui::TextEdit::singleline(reason)
                                                .char_limit(240)
                                                .desired_width(ui.available_width()),
                                        );
                                        ui.label(self.tr("Próxima acción", "Next action"));
                                        ui.add(
                                            egui::TextEdit::singleline(action)
                                                .char_limit(240)
                                                .desired_width(ui.available_width()),
                                        );
                                        if theme::bounded_navigation(
                                            ui,
                                            self.tr("Guardar cambios", "Save changes"),
                                            p,
                                        )
                                        .clicked()
                                        {
                                            update = Some((
                                                id.clone(),
                                                QueueEntry::new(*status, reason, action),
                                            ));
                                        }
                                    }
                                    ui.add_space(8.0);
                                    ui.separator();
                                    ui.add_space(8.0);
                                });
                            }
                        });
                    });
                ui.horizontal_wrapped(|ui| {
                    let response = theme::navigation(ui, self.tr("Cerrar", "Close"), false, p);
                    if state.focus_close {
                        response.request_focus();
                        state.focus_close = false;
                    }
                    if response.clicked() {
                        close = true;
                    }
                    ui.add_enabled_ui(!busy, |ui| {
                        ui.add_enabled_ui(self.tabs.model.next_queued().is_some(), |ui| {
                            if theme::bounded_navigation(
                                ui,
                                self.tr("Abrir siguiente", "Open next"),
                                p,
                            )
                            .clicked()
                            {
                                select =
                                    self.tabs.model.next_queued().map(|tab| tab.id().to_owned());
                            }
                        });
                        if let Some(id) = &active
                            && theme::bounded_navigation(
                                ui,
                                self.tr("Revisada y siguiente", "Reviewed and next"),
                                p,
                            )
                            .clicked()
                        {
                            let entry = self.tabs.model.active().unwrap().queue();
                            let queue = QueueEntry::new(
                                QueueStatus::Reviewed,
                                entry.blocking_reason().unwrap_or_default(),
                                entry.next_action().unwrap_or_default(),
                            );
                            if let Err(error) = self.tabs.model.update_queue(id, queue) {
                                self.error = Some(error.to_string());
                            }
                            select = self.tabs.model.next_queued().map(|tab| tab.id().to_owned());
                            ctx.request_repaint();
                        }
                    });
                });
            });
        if let Some((from, to)) = reorder {
            self.tabs.model.reorder(from, to);
            ctx.request_repaint();
        }
        if let Some((id, entry)) = update {
            if let Err(error) = self.tabs.model.update_queue(&id, entry) {
                self.error = Some(error.to_string());
            } else {
                state.edit = None;
                ctx.request_repaint();
            }
        }
        if let Some(id) = select
            && self.select_reader_tab(&id)
        {
            close = true;
        }
        if close || modal.should_close() {
            state.open = false;
            state.edit = None;
        }
        self.reading_queue = state;
    }
}
