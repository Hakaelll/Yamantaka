use super::*;
use yamantaka_domain::study_book::{BookStudy, Card, Operation, Reply, Request, Section, Source};

#[derive(Default, Clone, Copy, PartialEq)]
pub(super) enum Tab {
    #[default]
    Summary,
    Cards,
    Review,
}

#[derive(Default)]
pub(super) struct State {
    session: Option<Session>,
    pub(super) open: bool,
}

struct Session {
    document: String,
    title: String,
    reference: bool,
    tab: Tab,
    book: Option<BookStudy>,
    pending: bool,
    dirty: bool,
    error: Option<String>,
    exported: bool,
    seed: Option<(Tab, Seed)>,
    review_index: Option<usize>,
    revealed: bool,
}

pub(super) struct Seed {
    pub source: Source,
    pub text: String,
}

impl Session {
    fn add(&mut self, tab: Tab, seed: Seed) {
        let Some(book) = &mut self.book else {
            self.seed = Some((tab, seed));
            return;
        };
        match tab {
            Tab::Summary => book.sections.push(Section {
                text: seed.text,
                source: Some(seed.source),
                ..Section::default()
            }),
            _ => book.cards.push(Card {
                answer: seed.text,
                source: Some(seed.source),
                ..Card::default()
            }),
        }
        self.dirty = true;
    }
}

impl YamantakaApp {
    pub(crate) fn preview_book_study(&mut self) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        self.book_study.open = true;
        self.book_study.session = Some(Session {
            document: String::new(),
            title: "On the Nature of Things".into(),
            reference: false,
            tab: Tab::Review,
            book: Some(BookStudy {
                cards: vec![Card {
                    question: "¿Qué muestra el esquema de la primera página?".into(),
                    answer: "La relación entre las partes del argumento.".into(),
                    ..Card::default()
                }],
                ..BookStudy::default()
            }),
            pending: false,
            dirty: false,
            error: None,
            exported: false,
            seed: None,
            review_index: Some(0),
            revealed: true,
        });
    }

    pub(super) fn open_book_study(&mut self, reference: bool, tab: Tab, seed: Option<Seed>) {
        let active = if reference {
            self.reference.as_ref().and_then(|r| r.pane.active.as_ref())
        } else {
            self.reader.active.as_ref()
        };
        let Some((document, _)) = active else { return };
        self.book_study.open = true;
        if let Some(session) = &mut self.book_study.session {
            if session.document == document.id {
                session.tab = tab;
                session.reference = reference;
                if let Some(seed) = seed
                    && !session.pending
                {
                    session.add(tab, seed);
                }
                return;
            }
            if session.dirty || session.pending {
                session.error = Some(
                    if self.english {
                        "Save the open book draft before switching books."
                    } else {
                        "Guarda el borrador del libro abierto para cambiar de libro."
                    }
                    .into(),
                );
                return;
            }
        }
        self.book_study.session = Some(Session {
            document: document.id.clone(),
            title: document.display_title().to_string(),
            reference,
            tab,
            book: None,
            pending: false,
            dirty: false,
            error: None,
            exported: false,
            seed: seed.map(|seed| (tab, seed)),
            review_index: None,
            revealed: false,
        });
        self.send_book_study(Operation::Load);
    }

    fn send_book_study(&mut self, operation: Operation) {
        let Some(session) = &mut self.book_study.session else {
            return;
        };
        let action = Action::StudyBook(Box::new(Request {
            document: session.document.clone(),
            operation,
        }));
        session.pending = match self.runtime.submit(action) {
            Ok(()) => {
                session.error = None;
                session.exported = false;
                true
            }
            Err(error) => {
                session.error = Some(error.to_string());
                false
            }
        };
    }

    pub(super) fn accept_book_study(&mut self, reply: Reply) {
        let Some(session) = &mut self.book_study.session else {
            return;
        };
        if session.document != reply.document || !session.pending {
            return;
        }
        session.pending = false;
        match reply.result {
            Ok(book) => {
                session.book = Some(book);
                session.exported = reply.exported;
                session.dirty = false;
                if let Some((tab, seed)) = session.seed.take() {
                    session.tab = tab;
                    session.add(tab, seed);
                }
            }
            Err(error) => session.error = Some(error.to_string()),
        }
    }

    pub(super) fn book_study_window(&mut self, ctx: &egui::Context) {
        let Some(session) = &mut self.book_study.session else {
            return;
        };
        if !self.book_study.open {
            return;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.book_study.open = false;
            return;
        }
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let p = Palette::new(self.dark);
        let mut operation = None;
        let mut source = None;
        let viewport = ctx.content_rect().size();
        let compact = bookmarks::compact_study(ctx);
        let mut dismiss = false;
        egui::Window::new(session.title.clone())
            .id(egui::Id::new("book-study"))
            .open(&mut self.book_study.open)
            .title_bar(!compact)
            .collapsible(false)
            .resizable(!compact)
            .default_pos(egui::pos2(12.0, 12.0))
            .default_width((viewport.x - 40.0).clamp(180.0, 520.0))
            .max_width((viewport.x - 24.0).max(180.0))
            .max_height((viewport.y - 40.0).max(120.0))
            .show(ctx, |ui| {
                if compact {
                    ui.horizontal(|ui| {
                        let close = ui.add(egui::Button::new("×").min_size(egui::vec2(30.0, 28.0)).corner_radius(8));
                        let label = tr("Cerrar", "Close");
                        close.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
                        theme::record_control_bounds(ui, label, close.rect);
                        dismiss = close.on_hover_text(label).clicked();
                        ui.add(egui::Label::new(
                            RichText::new(&session.title).size(14.0).strong(),
                        ).truncate());
                    });
                }
                let study_button = |ui: &mut egui::Ui, label, selected| {
                    if !compact {
                        return theme::navigation(ui, label, selected, p);
                    }
                    let button = ui.add(egui::Button::new(RichText::new(label).size(13.0))
                        .selected(selected).min_size(egui::vec2(30.0, 28.0)).corner_radius(8));
                    theme::record_control_bounds(ui, label, button.rect);
                    button
                };
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for (tab, label) in [
                                (Tab::Summary, tr("Resumen", "Summary")),
                                (Tab::Cards, tr("Tarjetas", "Cards")),
                                (Tab::Review, tr("Repasar", "Review")),
                            ] {
                                if study_button(ui, label, session.tab == tab).clicked() {
                                    session.tab = tab;
                                }
                            }
                        });
                        ui.separator();
                        if session.pending {
                            ui.spinner();
                        }
                        if let Some(error) = &session.error {
                            ui.label(error);
                        }
                        if session.exported {
                            ui.label(tr("Resumen exportado", "Summary exported"));
                        }
                        let Some(book) = &mut session.book else {
                            if !session.pending && ui.button(tr("Reintentar", "Retry")).clicked() {
                                operation = Some(Operation::Load);
                            }
                            return;
                        };
                        ui.add_enabled_ui(!session.pending, |ui| {
                            match session.tab {
                                Tab::Summary => {
                                    let mut remove = None;
                                    let mut swap = None;
                                    for (index, section) in book.sections.iter_mut().enumerate() {
                                        ui.push_id(index, |ui| {
                                            session.dirty |= field(
                                                ui,
                                                tr("Título de la sección", "Section title"),
                                                &mut section.title,
                                                false,
                                            );
                                            session.dirty |= field(
                                                ui,
                                                tr("Contenido", "Content"),
                                                &mut section.text,
                                                true,
                                            );
                                            ui.horizontal_wrapped(|ui| {
                                                if index > 0
                                                    && ui.button(tr("Subir", "Move up")).clicked()
                                                {
                                                    swap = Some(index);
                                                }
                                                if ui
                                                    .button(tr(
                                                        "Eliminar sección",
                                                        "Remove section",
                                                    ))
                                                    .clicked()
                                                {
                                                    remove = Some(index);
                                                }
                                                source_button(
                                                    ui,
                                                    section.source.as_ref(),
                                                    tr("Ver en PDF", "Show in PDF"),
                                                    &mut source,
                                                );
                                            });
                                            ui.separator();
                                        });
                                    }
                                    if let Some(index) = remove {
                                        book.sections.remove(index);
                                        session.dirty = true;
                                    }
                                    if let Some(index) = swap {
                                        book.sections.swap(index, index - 1);
                                        session.dirty = true;
                                    }
                                    if ui.button(tr("Añadir sección", "Add section")).clicked() {
                                        book.sections.push(Section::default());
                                        session.dirty = true;
                                    }
                                    if !book.sections.is_empty()
                                        && !session.dirty
                                        && ui
                                            .button(tr("Exportar Markdown", "Export Markdown"))
                                            .clicked()
                                    {
                                        match yamantaka_windows::annotation_export::pick_annotation_export(
                                            tr("resumen", "summary"),
                                            "md",
                                        ) {
                                            Ok(Some(path)) => operation = Some(Operation::Export(path)),
                                            Ok(None) => {}
                                            Err(error) => session.error = Some(error.to_string()),
                                        }
                                    }
                                }
                                Tab::Cards => {
                                    let mut remove = None;
                                    for (index, card) in book.cards.iter_mut().enumerate() {
                                        ui.push_id(index, |ui| {
                                            session.dirty |= field(
                                                ui,
                                                tr("Pregunta", "Question"),
                                                &mut card.question,
                                                false,
                                            );
                                            session.dirty |= field(
                                                ui,
                                                tr("Respuesta", "Answer"),
                                                &mut card.answer,
                                                true,
                                            );
                                            ui.horizontal_wrapped(|ui| {
                                                source_button(
                                                    ui,
                                                    card.source.as_ref(),
                                                    tr("Ver en PDF", "Show in PDF"),
                                                    &mut source,
                                                );
                                                if ui
                                                    .button(tr("Eliminar tarjeta", "Remove card"))
                                                    .clicked()
                                                {
                                                    remove = Some(index);
                                                }
                                            });
                                            ui.separator();
                                        });
                                    }
                                    if let Some(index) = remove {
                                        book.cards.remove(index);
                                        session.dirty = true;
                                    }
                                    if ui.button(tr("Añadir tarjeta", "Add card")).clicked() {
                                        book.cards.push(Card::default());
                                        session.dirty = true;
                                    }
                                }
                                Tab::Review if session.dirty => {
                                    ui.label(tr(
                                        "Guarda los cambios para empezar el repaso.",
                                        "Save changes to start reviewing.",
                                    ));
                                }
                                Tab::Review => {
                                    let now = chrono::Utc::now();
                                    let due = |card: &Card| {
                                        card.due
                                            .is_none_or(|timestamp| timestamp <= now.timestamp())
                                    };
                                    let start =
                                        session.review_index.unwrap_or(0).min(book.cards.len());
                                    let current = (start..book.cards.len())
                                        .chain(0..start)
                                        .find(|&index| due(&book.cards[index]));
                                    if session.review_index != current {
                                        session.revealed = false;
                                    }
                                    session.review_index = current;
                                    if let Some(index) = current {
                                        let card = &mut book.cards[index];
                                        ui.label(RichText::new(&card.question).size(if compact { 15.0 } else { 18.0 }).strong());
                                        let revealed = session.revealed;
                                        let (again, remembered, skip) = ui.horizontal_wrapped(|ui| {
                                            let (mut again, mut remembered) = (false, false);
                                            if revealed {
                                                again = study_button(ui, tr("Otra vez", "Again"), false).clicked();
                                                remembered = study_button(ui, tr("Recordada", "Remembered"), true).clicked();
                                            } else if study_button(ui, tr("Mostrar respuesta", "Reveal answer"), false).clicked() {
                                                session.revealed = true;
                                            }
                                            let skip = study_button(ui, tr("Saltar", "Skip"), false).clicked();
                                            (again, remembered, skip)
                                        }).inner;
                                        if revealed {
                                            ui.label(RichText::new(&card.answer).size(14.0));
                                            source_button(ui, card.source.as_ref(), tr("Ver en PDF", "Show in PDF"), &mut source);
                                        }
                                        if skip {
                                            session.review_index = Some(index + 1);
                                            session.revealed = false;
                                        } else if again || remembered {
                                            if again {
                                                card.due = Some((now + chrono::Duration::minutes(10)).timestamp());
                                                card.interval = 0;
                                            } else {
                                                schedule_remembered(card, now);
                                            }
                                            session.dirty = true;
                                            session.revealed = false;
                                            operation = Some(Operation::Save(book.clone()));
                                        }
                                    } else {
                                        ui.label(tr(
                                            "No quedan tarjetas pendientes.",
                                            "No cards are due.",
                                        ));
                                    }
                                }
                            }
                            if session.dirty {
                                ui.horizontal_wrapped(|ui| {
                                    if ui.button(tr("Guardar", "Save")).clicked() {
                                        operation = Some(Operation::Save(book.clone()));
                                    }
                                    if ui
                                        .button(tr("Descartar cambios", "Discard changes"))
                                        .clicked()
                                    {
                                        operation = Some(Operation::Load);
                                    }
                                });
                            }
                        });
                    });
            });
        if dismiss {
            self.book_study.open = false;
        }
        if let Some(source) = source {
            let doc = &session.document;
            let matches = |pane: &reader_pane::ReaderPane| {
                pane.active
                    .as_ref()
                    .is_some_and(|(document, _)| &document.id == doc)
            };
            let reference =
                session.reference && self.reference.as_ref().is_some_and(|r| matches(&r.pane));
            if reference || matches(&self.reader) {
                self.book_study.open = false;
                self.apply_notes_action(
                    reference,
                    annotations::NotesAction::Navigate(
                        yamantaka_domain::reader::ReaderPosition::at(source.page, source.point)
                            .expect("validated source anchor"),
                        source.annotation,
                        false,
                    ),
                );
            } else {
                session.error = Some(
                    tr(
                        "Abre el PDF de origen para ver la nota.",
                        "Open the source PDF to see the note.",
                    )
                    .into(),
                );
            }
        }
        if let Some(operation) = operation {
            self.send_book_study(operation);
        }
    }
}

fn field(ui: &mut egui::Ui, label: &str, value: &mut String, multiline: bool) -> bool {
    let label = ui.label(label);
    let text = if multiline {
        egui::TextEdit::multiline(value)
            .desired_rows(3)
            .char_limit(4096)
    } else {
        egui::TextEdit::singleline(value).char_limit(75)
    };
    ui.add(text.desired_width(f32::INFINITY))
        .labelled_by(label.id)
        .changed()
}

fn source_button(
    ui: &mut egui::Ui,
    source: Option<&Source>,
    caption: &str,
    selected: &mut Option<Source>,
) {
    if let Some(source) = source
        && ui.button(caption).clicked()
    {
        *selected = Some(source.clone());
    }
}

fn schedule_remembered(card: &mut Card, now: chrono::DateTime<chrono::Utc>) {
    let steps = [1, 3, 7, 14, 30];
    let days = steps[card.interval.min(steps.len() - 1)];
    let local = now.with_timezone(&chrono::Local);
    card.due = local
        .date_naive()
        .checked_add_days(chrono::Days::new(days))
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .and_then(|date| date.and_local_timezone(chrono::Local).earliest())
        .map(|date| date.timestamp())
        .or_else(|| {
            now.checked_add_signed(chrono::Duration::days(days as i64))
                .map(|date| date.timestamp())
        });
    card.interval = (card.interval + 1).min(steps.len() - 1);
}
