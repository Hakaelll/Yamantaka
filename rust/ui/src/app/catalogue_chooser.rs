use super::*;
#[derive(Clone, Copy)]
pub(in crate::app) enum ChooserPurpose<'a> {
    Reference(Option<&'a str>),
    Relation(&'a str),
}

pub(in crate::app) struct CatalogueChooser {
    pub text: String,
    pub offset: usize,
    displayed_offset: usize,
    pub entries: Vec<Document>,
    pub total: u64,
    pub loaded: bool,
    pending: Option<Cancellation>,
    due: Option<Instant>,
    error: Option<String>,
    pub preview: bool,
    focus: bool,
}

impl Default for CatalogueChooser {
    fn default() -> Self {
        Self {
            text: String::new(),
            offset: 0,
            displayed_offset: 0,
            entries: Vec::new(),
            total: 0,
            loaded: false,
            pending: None,
            due: Some(Instant::now()),
            error: None,
            preview: false,
            focus: true,
        }
    }
}

impl CatalogueChooser {
    pub fn reopen(&mut self) {
        self.focus = true;
        self.schedule(self.offset, Duration::ZERO);
    }
    pub fn schedule(&mut self, offset: usize, delay: Duration) {
        if let Some(token) = self.pending.take() {
            token.cancel();
        }
        self.offset = offset;
        self.loaded = false;
        self.error = None;
        self.due = Some(Instant::now() + delay);
    }

    pub fn accept(&mut self, documents: &[Document], total: u64, cancel: &Cancellation) -> bool {
        if cancel.is_cancelled()
            || !self
                .pending
                .as_ref()
                .is_some_and(|token| token.same_request(cancel))
        {
            return false;
        }
        self.pending = None;
        if self.offset > 0 && (total == 0 || self.offset as u64 >= total) {
            self.schedule(
                usize::try_from(total.saturating_sub(1) / 80 * 80).unwrap_or(0),
                Duration::ZERO,
            );
            return true;
        }
        self.entries = documents.to_vec();
        self.displayed_offset = self.offset;
        self.total = total;
        self.loaded = true;
        true
    }

    pub fn fail(&mut self, cancel: &Cancellation, error: &yamantaka_domain::Failure) -> bool {
        if !self
            .pending
            .as_ref()
            .is_some_and(|token| token.same_request(cancel))
        {
            return false;
        }
        self.pending = None;
        self.error = Some(error.to_string());
        true
    }

    fn pump(&mut self, runtime: &Runtime, ctx: &egui::Context) {
        let Some(due) = self.due else {
            return;
        };
        let now = Instant::now();
        if now < due {
            ctx.request_repaint_after(due - now);
            return;
        }
        let cancel = Cancellation::default();
        let query = LibraryQuery {
            text: self.text.clone(),
            offset: self.offset,
            page_size: 80,
            sort: LibrarySort::Title,
            sort_direction: SortDirection::Ascending,
            ..Default::default()
        };
        match runtime.submit(Action::Query(query, cancel.clone())) {
            Ok(()) => {
                self.pending = Some(cancel);
                self.due = None;
            }
            Err(yamantaka_domain::Failure::QueueFull) => {
                self.due = Some(now + Duration::from_millis(100));
                ctx.request_repaint_after(Duration::from_millis(100));
            }
            Err(error) => {
                self.due = None;
                self.error = Some(error.to_string());
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        english: bool,
        p: Palette,
        purpose: ChooserPurpose<'_>,
    ) -> Option<Document> {
        let tr = |es, en| if english { en } else { es };
        ui.set_width((ui.ctx().content_rect().width() - 40.0).clamp(160.0, 420.0));
        let search = ui.add(
            egui::TextEdit::singleline(&mut self.text)
                .hint_text(tr(
                    "Buscar título, autor o archivo",
                    "Search title, author or filename",
                ))
                .char_limit(512)
                .desired_width(f32::INFINITY),
        );
        search.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::TextEdit,
                ui.is_enabled(),
                match purpose {
                    ChooserPurpose::Reference(_) => {
                        tr("Buscar libro de referencia", "Search reference books")
                    }
                    ChooserPurpose::Relation(_) => {
                        tr("Buscar documento relacionado", "Search related documents")
                    }
                },
            )
        });
        if self.focus {
            search.request_focus();
            self.focus = false;
        }
        if search.changed() {
            self.schedule(0, Duration::from_millis(180));
        }
        self.pump(runtime, ui.ctx());
        if let Some(error) = self.error.clone() {
            ui.label(error);
            if theme::navigation(ui, tr("Reintentar", "Retry"), false, p).clicked() {
                self.schedule(self.offset, Duration::ZERO);
            }
        }
        let mut selected = None;
        let ready = self.loaded && self.pending.is_none() && self.due.is_none();
        let height =
            (ui.ctx().content_rect().bottom() - ui.cursor().top() - 72.0).clamp(60.0, 340.0);
        egui::ScrollArea::vertical()
            .max_height(height)
            .show(ui, |ui| {
                for document in &self.entries {
                    if ui
                        .add_enabled_ui(ready && match purpose { ChooserPurpose::Reference(_) => !document.missing, ChooserPurpose::Relation(source) => document.id != source }, |ui| {
                            theme::navigation(
                                ui,
                                document.display_title(),
                                matches!(purpose, ChooserPurpose::Reference(Some(active)) if active == document.id),
                                p,
                            )
                        })
                        .inner
                        .clicked()
                    {
                        selected = Some(document.clone());
                        ui.close();
                    }
                }
                if ready && self.entries.is_empty() {
                    ui.label(tr("No se han encontrado libros.", "No books found."));
                }
            });
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled_ui(ready && self.offset > 0, |ui| {
                    theme::navigation(ui, tr("Anterior", "Previous"), false, p)
                })
                .inner
                .clicked()
            {
                self.schedule(self.offset.saturating_sub(80), Duration::ZERO);
            }
            let end = (self.displayed_offset as u64 + self.entries.len() as u64).min(self.total);
            ui.label(if self.entries.is_empty() {
                format!("0 / {}", self.total)
            } else {
                format!("{}–{} / {}", self.displayed_offset + 1, end, self.total)
            });
            if ui
                .add_enabled_ui(ready && end < self.total, |ui| {
                    theme::navigation(ui, tr("Siguiente", "Next"), false, p)
                })
                .inner
                .clicked()
            {
                self.schedule(self.offset.saturating_add(80), Duration::ZERO);
            }
        });
        selected
    }
}

impl Drop for CatalogueChooser {
    fn drop(&mut self) {
        if let Some(token) = &self.pending {
            token.cancel();
        }
    }
}
