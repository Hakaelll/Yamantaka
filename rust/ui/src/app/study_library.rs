use super::*;
use yamantaka_domain::{
    Failure, Result,
    annotations::{AnnotationKind, repository::AnnotationRecordData},
    reader::ReaderPosition,
    study_library::{Library, Notebook, Operation, Outcome, Passage, PassageLink},
};

#[derive(Default)]
pub(super) struct State {
    open: bool,
    links: bool,
    library: Option<Library>,
    notebook: usize,
    new_title: String,
    incoming: Option<Passage>,
    first_link: Option<(bool, Passage)>,
    pair: Option<[Passage; 2]>,
    observation: String,
    pending: bool,
    dirty: bool,
    error: Option<String>,
    exported: bool,
    opening: Option<Opening>,
}
impl State {
    pub(super) fn open(&self) -> bool {
        self.open
    }
}

struct Opening {
    targets: Vec<(Passage, Document)>,
    step: u8,
    started: Instant,
}

pub(in crate::app) fn annotation_passage(
    record: &AnnotationRecordData,
    point: [f64; 2],
) -> Passage {
    let value = &record.values;
    let crop = value
        .drawing
        .as_ref()
        .filter(|shape| shape.kind() == AnnotationKind::Crop)
        .map(|shape| shape.bounds());
    let text = match (value.selected_text.as_deref(), value.note.as_deref()) {
        (Some(quote), Some(note)) => format!("{quote}\n\n{note}"),
        (Some(text), None) | (None, Some(text)) => text.to_owned(),
        _ => String::new(),
    };
    let mut rects: Vec<_> = value.rects.iter().copied().take(128).collect();
    if let Some([a, b]) = crop
        && let Ok(region) = yamantaka_domain::pdf_geometry::NormalizedPageRect::new(a, b)
    {
        rects.push(region.into());
    }
    Passage {
        document: value.document_id.clone(),
        fingerprint: value.document_fingerprint.clone(),
        title: record.document_title.clone().unwrap_or_default(),
        page: value.page_index,
        last_page: value.page_index,
        point,
        rects,
        text,
        annotation: Some(record.id.clone()),
        crop,
    }
}

impl YamantakaApp {
    pub(crate) fn preview_study_notebooks(&mut self, links: bool) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        let passage = |id: &str, title: &str, page, text: &str| Passage {
            document: id.into(),
            fingerprint: "synthetic".into(),
            title: title.into(),
            page,
            last_page: page,
            point: [0.3, 0.4],
            rects: vec![],
            text: text.into(),
            annotation: None,
            crop: None,
        };
        let first = passage(
            "demo-0",
            "La lectura atenta",
            1,
            "Volver al texto cambia la lectura de una idea.",
        );
        let second = passage(
            "demo-1",
            "Cuaderno de campo",
            3,
            "Una observación conecta dos fragmentos.",
        );
        self.study_library.open = true;
        self.study_library.links = links;
        self.study_library.library = Some(Library {
            revision: 0,
            notebooks: vec![Notebook {
                title: "Lectura y método".into(),
                entries: vec![first.clone(), second.clone()],
            }],
            links: vec![PassageLink {
                passages: [first, second],
                note: "Contrastar ambos pasajes en el repaso.".into(),
            }],
        });
    }
    pub(super) fn send_passage_to_notebook(&mut self, reference: bool) {
        match self.selected_passage(reference) {
            Ok(passage) => self.open_study_library(false, Some(passage)),
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub(super) fn capture_link_passage(&mut self, reference: bool) {
        if self.reference.is_none() {
            self.error = Some(
                self.tr(
                    "Abre Comparar para vincular dos pasajes.",
                    "Open comparison to link two passages.",
                )
                .into(),
            );
            return;
        }
        let passage = match self.selected_passage(reference) {
            Ok(passage) => passage,
            Err(error) => {
                self.error = Some(error.to_string());
                return;
            }
        };
        let first = self.study_library.first_link.take();
        match first {
            Some((side, first)) if side != reference => {
                let pane = self.study_pane(side);
                if pane.viewport.source_changed
                    || pane.active.as_ref().is_none_or(|(doc, _)| {
                        doc.id != first.document || doc.fingerprint != first.fingerprint
                    })
                {
                    self.error = Some(
                        self.tr(
                            "Cambió el PDF del primer pasaje. Vuelve a seleccionarlo.",
                            "The first passage's PDF changed. Select it again.",
                        )
                        .into(),
                    );
                    return;
                }
                self.study_library.pair = Some(if reference {
                    [first, passage]
                } else {
                    [passage, first]
                });
                self.open_study_library(true, None);
            }
            _ => {
                self.study_library.first_link = Some((reference, passage));
                self.error = Some(
                    self.tr(
                        "Primer pasaje listo. Selecciona y vincula el pasaje del otro panel.",
                        "First passage ready. Select and link the passage in the other pane.",
                    )
                    .into(),
                );
            }
        }
    }
    fn selected_passage(&self, reference: bool) -> Result<Passage> {
        let pane = self.study_pane(reference);
        let (document, _) = pane
            .active
            .as_ref()
            .filter(|_| !pane.viewport.source_changed)
            .ok_or_else(|| Failure::Invalid("The PDF source is unavailable".into()))?;
        let snapshot = pane.selection.annotation_source(&self.governor)?;
        if snapshot.text().len() > 16_384 {
            return Err(Failure::Invalid(
                self.tr(
                    "Selecciona un fragmento más corto.",
                    "Select a shorter passage.",
                )
                .into(),
            ));
        }
        let range = snapshot.selection();
        let part = range
            .page(&snapshot.pages()[0])?
            .ok_or_else(|| Failure::Invalid("The selection is empty".into()))?;
        let rects: Vec<_> = part.ink_regions().take(128).map(Into::into).collect();
        let point = rects
            .first()
            .map(|rect: &yamantaka_domain::annotations::AnnotationRect| {
                let region = rect.region();
                [(region.min()[0] + region.max()[0]) * 0.5, region.min()[1]]
            })
            .unwrap_or([0.5, 0.0]);
        Ok(Passage {
            document: document.id.clone(),
            fingerprint: document.fingerprint.clone(),
            title: document.display_title().to_owned(),
            page: range.start().page(),
            last_page: range.end().page(),
            point,
            rects,
            text: snapshot.text().to_owned(),
            annotation: None,
            crop: None,
        })
    }

    pub(super) fn study_library_menu(&mut self, ui: &mut egui::Ui, p: Palette, reference: bool) {
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let label = tr("Acciones de estudio", "Study actions");
        let button = theme::icon_button(ui, label, theme::Icon::Menu, p);
        egui::Popup::menu(&button).show(|ui| {
            ui.set_max_width((ui.ctx().content_rect().width() - 24.0).clamp(140.0, 270.0));
            egui::ScrollArea::vertical()
                .max_height((ui.ctx().content_rect().height() * 0.54).max(75.0))
                .show(ui, |ui| {
                    if ui.button(tr("Cuadernos", "Notebooks")).clicked() {
                        self.open_study_library(false, None);
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            self.study_pane(reference).selection.has_passage(),
                            egui::Button::new(tr(
                                "Enviar selección a cuaderno",
                                "Send selection to notebook",
                            )),
                        )
                        .clicked()
                    {
                        match self.selected_passage(reference) {
                            Ok(passage) => self.open_study_library(false, Some(passage)),
                            Err(error) => self.error = Some(error.to_string()),
                        }
                        ui.close();
                    }
                    if ui
                        .button(tr("Vínculos entre pasajes", "Passage links"))
                        .clicked()
                    {
                        self.open_study_library(true, None);
                        ui.close();
                    }
                    let can_link = self.reference.is_some()
                        && self.study_pane(reference).selection.has_passage();
                    if ui
                        .add_enabled(
                            can_link,
                            egui::Button::new(tr("Vincular pasaje", "Link passage")),
                        )
                        .on_disabled_hover_text(tr(
                            "Selecciona un pasaje en Comparar.",
                            "Select a passage in comparison view.",
                        ))
                        .clicked()
                    {
                        self.capture_link_passage(reference);
                        ui.close();
                    }
                    self.study_document_actions(ui, reference);
                });
        });
    }

    pub(in crate::app) fn open_study_library(&mut self, links: bool, incoming: Option<Passage>) {
        self.study_library.open = true;
        self.study_library.links = links;
        if incoming.is_some() && self.study_library.incoming.is_some() {
            self.study_library.error = Some(
                self.tr(
                    "Añade la entrada pendiente antes de enviar otra.",
                    "Add the pending entry before sending another.",
                )
                .into(),
            );
        } else if let Some(incoming) = incoming {
            self.study_library.incoming = Some(incoming);
        }
        if self.study_library.library.is_none() && !self.study_library.pending {
            self.send_study_library(Operation::Load);
        }
    }

    fn send_study_library(&mut self, operation: Operation) {
        self.study_library.error = None;
        self.study_library.exported = false;
        match self
            .runtime
            .submit(Action::StudyLibrary(Box::new(operation)))
        {
            Ok(()) => self.study_library.pending = true,
            Err(error) => self.study_library.error = Some(error.to_string()),
        }
    }

    pub(super) fn accept_study_library(&mut self, reply: Result<Outcome>) {
        let state = &mut self.study_library;
        state.pending = false;
        match reply {
            Ok(Outcome::Library(library)) => {
                state.library = Some(library);
                state.dirty = false;
            }
            Ok(Outcome::Exported) => state.exported = true,
            Ok(Outcome::Resolved(targets)) => {
                state.open = false;
                state.opening = Some(Opening {
                    targets,
                    step: 0,
                    started: Instant::now(),
                });
            }
            Err(error) => state.error = Some(error.to_string()),
        }
    }

    pub(super) fn study_library_window(&mut self, ctx: &egui::Context) {
        self.advance_study_destination(ctx);
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let state = &mut self.study_library;
        if !state.open {
            return;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            state.open = false;
            return;
        }
        let p = Palette::new(self.dark);
        let mut operation = None;
        let mut export = false;
        let mut open = state.open;
        let viewport = ctx.content_rect();
        egui::Window::new(tr("Estudio", "Study"))
            .id(egui::Id::new("study-library"))
            .open(&mut open)
            .collapsible(false)
            .default_width((viewport.width() - 36.0).clamp(175.0, 520.0))
            .max_width((viewport.width() - 22.0).max(175.0))
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if theme::navigation(ui, tr("Cuadernos", "Notebooks"), !state.links, p)
                        .clicked()
                    {
                        state.links = false;
                    }
                    if theme::navigation(ui, tr("Vínculos", "Links"), state.links, p).clicked() {
                        state.links = true;
                    }
                });
                if state.pending {
                    ui.spinner();
                }
                if let Some(error) = &state.error {
                    ui.label(error);
                }
                if state.exported {
                    ui.label(tr("Markdown guardado.", "Markdown saved."));
                }
                let Some(library) = &mut state.library else {
                    if state.error.is_some() && ui.button(tr("Reintentar", "Retry")).clicked() {
                        operation = Some(Operation::Load);
                    }
                    return;
                };
                ui.add_enabled_ui(!state.pending, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt(state.links)
                        .max_height((viewport.height() * 0.5).max(70.0))
                        .show(ui, |ui| {
                            if state.links {
                                if let Some(pair) = &state.pair {
                                    for passage in pair {
                                        passage_label(ui, passage, tr, p);
                                    }
                                    label_field(
                                        ui,
                                        tr("Observación", "Observation"),
                                        &mut state.observation,
                                        2048,
                                        true,
                                    );
                                    ui.horizontal_wrapped(|ui| {
                                        if ui
                                            .add_enabled(
                                                library.links.len() < 200,
                                                egui::Button::new(tr(
                                                    "Guardar vínculo",
                                                    "Save link",
                                                )),
                                            )
                                            .clicked()
                                        {
                                            library.links.push(PassageLink {
                                                passages: state.pair.take().unwrap(),
                                                note: std::mem::take(&mut state.observation),
                                            });
                                            state.dirty = true;
                                            operation = Some(Operation::Save(library.clone()));
                                        }
                                        if ui.button(tr("Cancelar", "Cancel")).clicked() {
                                            state.pair = None;
                                            state.observation.clear();
                                        }
                                    });
                                    ui.separator();
                                }
                                if library.links.is_empty() {
                                    ui.label(tr(
                                        "Los vínculos de comparación aparecerán aquí.",
                                        "Your comparison links appear here.",
                                    ));
                                }
                                let mut remove = None;
                                for (index, link) in library.links.iter_mut().enumerate() {
                                    ui.push_id(index, |ui| {
                                        for passage in &link.passages {
                                            passage_label(ui, passage, tr, p);
                                        }
                                        state.dirty |= label_field(
                                            ui,
                                            tr("Observación", "Observation"),
                                            &mut link.note,
                                            2048,
                                            true,
                                        );
                                        ui.horizontal_wrapped(|ui| {
                                            if ui
                                                .button(tr("Abrir comparación", "Open comparison"))
                                                .clicked()
                                            {
                                                operation = Some(Operation::Resolve(
                                                    link.passages.to_vec(),
                                                ));
                                            }
                                            if ui
                                                .button(tr("Quitar vínculo", "Remove link"))
                                                .clicked()
                                            {
                                                remove = Some(index);
                                            }
                                        });
                                        ui.separator();
                                    });
                                }
                                if let Some(index) = remove {
                                    library.links.remove(index);
                                    state.dirty = true;
                                }
                            } else {
                                ui.horizontal_wrapped(|ui| {
                                    for (index, notebook) in library.notebooks.iter().enumerate() {
                                        if theme::navigation(
                                            ui,
                                            &notebook.title,
                                            state.notebook == index,
                                            p,
                                        )
                                        .clicked()
                                        {
                                            state.notebook = index;
                                        }
                                    }
                                });
                                label_field(
                                    ui,
                                    tr("Nuevo cuaderno", "New notebook"),
                                    &mut state.new_title,
                                    75,
                                    false,
                                );
                                if ui
                                    .add_enabled(
                                        !state.new_title.trim().is_empty()
                                            && library.notebooks.len() < 40,
                                        egui::Button::new(tr("Crear", "Create")),
                                    )
                                    .clicked()
                                {
                                    state.notebook = library.notebooks.len();
                                    library.notebooks.push(Notebook {
                                        title: state.new_title.trim().to_owned(),
                                        entries: Vec::new(),
                                    });
                                    state.new_title.clear();
                                    state.dirty = true;
                                }
                                if let Some(notebook) = library.notebooks.get_mut(state.notebook) {
                                    state.dirty |= label_field(
                                        ui,
                                        tr("Nombre del cuaderno", "Notebook name"),
                                        &mut notebook.title,
                                        75,
                                        false,
                                    );
                                    if let Some(passage) = state.incoming.as_ref() {
                                        passage_label(ui, passage, tr, p);
                                        ui.horizontal_wrapped(|ui| {
                                            if ui
                                                .add_enabled(
                                                    notebook.entries.len() < 200,
                                                    egui::Button::new(tr(
                                                        "Añadir al cuaderno",
                                                        "Add to notebook",
                                                    )),
                                                )
                                                .clicked()
                                            {
                                                notebook
                                                    .entries
                                                    .push(state.incoming.take().unwrap());
                                                state.dirty = true;
                                            }
                                            if ui
                                                .button(tr(
                                                    "Descartar entrada pendiente",
                                                    "Discard pending entry",
                                                ))
                                                .clicked()
                                            {
                                                state.incoming = None;
                                            }
                                        });
                                    }
                                    let mut reorder = None;
                                    let mut remove = None;
                                    let count = notebook.entries.len();
                                    for (index, passage) in notebook.entries.iter_mut().enumerate()
                                    {
                                        ui.push_id(index, |ui| {
                                            passage_label(ui, passage, tr, p);
                                            state.dirty |= label_field(
                                                ui,
                                                tr("Contenido", "Content"),
                                                &mut passage.text,
                                                4096,
                                                true,
                                            );
                                            ui.horizontal_wrapped(|ui| {
                                                if ui
                                                    .button(tr("Ir al pasaje", "Go to passage"))
                                                    .clicked()
                                                {
                                                    operation = Some(Operation::Resolve(vec![
                                                        passage.clone(),
                                                    ]));
                                                }
                                                if ui
                                                    .add_enabled(
                                                        index > 0,
                                                        egui::Button::new(tr("Subir", "Move up")),
                                                    )
                                                    .clicked()
                                                {
                                                    reorder = Some((index, index - 1));
                                                }
                                                if ui
                                                    .add_enabled(
                                                        index + 1 < count,
                                                        egui::Button::new(tr("Bajar", "Move down")),
                                                    )
                                                    .clicked()
                                                {
                                                    reorder = Some((index, index + 1));
                                                }
                                                if ui.button(tr("Quitar", "Remove")).clicked() {
                                                    remove = Some(index);
                                                }
                                            });
                                            ui.separator();
                                        });
                                    }
                                    if let Some((a, b)) = reorder {
                                        notebook.entries.swap(a, b);
                                        state.dirty = true;
                                    }
                                    if let Some(index) = remove {
                                        notebook.entries.remove(index);
                                        state.dirty = true;
                                    }
                                    export = ui
                                        .add_enabled(
                                            !state.dirty && !notebook.entries.is_empty(),
                                            egui::Button::new(tr(
                                                "Exportar Markdown",
                                                "Export Markdown",
                                            )),
                                        )
                                        .clicked();
                                }
                            }
                        });
                    if state.dirty
                        && theme::navigation(ui, tr("Guardar cambios", "Save changes"), true, p)
                            .clicked()
                    {
                        operation = Some(Operation::Save(library.clone()));
                    }
                });
            });
        state.open = open;
        if export {
            match yamantaka_windows::annotation_export::pick_annotation_export(
                tr("Exportar cuaderno", "Export notebook"),
                "md",
            ) {
                Ok(Some(path)) => operation = Some(Operation::Export(state.notebook, path)),
                Ok(None) => {}
                Err(error) => state.error = Some(error.to_string()),
            }
        }
        if let Some(operation) = operation {
            self.send_study_library(operation);
        }
    }

    fn advance_study_destination(&mut self, ctx: &egui::Context) {
        let Some(mut opening) = self.study_library.opening.take() else {
            return;
        };
        let Some((primary, document)) = opening.targets.first() else {
            return;
        };
        let same = |pane: &reader_pane::ReaderPane, passage: &Passage| {
            pane.active.as_ref().is_some_and(|(doc, _)| {
                doc.id == passage.document && doc.fingerprint == passage.fingerprint
            }) && !pane.viewport.source_changed
        };
        let failed = opening.started.elapsed() > Duration::from_secs(45);
        if failed {
            self.error = Some(
                self.tr(
                    "No se pudieron abrir los pasajes guardados.",
                    "The saved passages could not be opened.",
                )
                .into(),
            );
            return;
        }
        if !same(&self.reader, primary) {
            if self.reference.is_some() {
                if opening.step == 0 && self.request_reference_close() {
                    opening.step = 1;
                }
            } else if opening.step <= 1 {
                if !self.open(document.clone()) {
                    return;
                }
                self.open_intent.set_page(document.id.clone(), primary.page);
                opening.step = 2;
            }
        } else if let Some((secondary, document)) = opening.targets.get(1) {
            if let Some(view) = &self.reference {
                if same(&view.pane, secondary) {
                    self.go_to_study_passage(primary.clone(), false);
                    self.go_to_study_passage(secondary.clone(), true);
                    return;
                }
                let replace = view
                    .pane
                    .active
                    .as_ref()
                    .is_some_and(|(doc, _)| doc.id != primary.document);
                if replace && opening.step != 3 {
                    if self.request_reference_close() {
                        opening.step = 3;
                    }
                } else if !replace
                    && opening.step != 5
                    && !view.busy()
                    && !self.reader.reader_preferences_loading
                {
                    self.open_reference(document.clone());
                    opening.step = 5;
                }
            } else if self.reference_mount.is_none()
                && opening.step != 4
                && !self.reader.reader_preferences_loading
                && !self.opening.busy()
                && !self.tabs.busy()
            {
                self.toggle_reference(ctx);
                if self.reference_mount.is_some() {
                    opening.step = 4;
                }
            }
        } else {
            self.go_to_study_passage(primary.clone(), false);
            return;
        }
        self.study_library.opening = Some(opening);
        ctx.request_repaint_after(Duration::from_millis(80));
    }

    fn go_to_study_passage(&mut self, passage: Passage, reference: bool) {
        let pane = self.study_pane_mut(reference);
        let Ok(position) = ReaderPosition::at(passage.page, passage.point) else {
            return;
        };
        if let Some(nav) = &mut pane.navigation {
            nav.navigate_to(position);
        }
        pane.annotation_paint.focused = passage.annotation.clone();
        pane.study.passage = Some(passage);
        if reference {
            let reference = self.reference.as_mut().unwrap();
            reference.follow_page = false;
            reference.follow_position = false;
            reference.reference_navigation_changed();
        } else {
            self.surface = Surface::Reader;
            self.apply_navigation();
        }
    }
}

impl study::Pane {
    pub(super) fn paint_passage(
        &self,
        ui: &egui::Ui,
        page: u32,
        paper: egui::Rect,
        rotation: yamantaka_domain::reader_preferences::PageRotation,
        palette: Palette,
    ) {
        if let Some(passage) = self.passage.as_ref().filter(|passage| passage.page == page) {
            for rect in &passage.rects {
                let rect = reader_viewport::page_region(paper, rotation, rect.region());
                ui.painter().rect_stroke(
                    rect,
                    2.0,
                    egui::Stroke::new(1.5, palette.accent),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
}

fn passage_label(
    ui: &mut egui::Ui,
    passage: &Passage,
    tr: impl Fn(&'static str, &'static str) -> &'static str,
    p: Palette,
) {
    let first = passage.page + 1;
    let last = passage.last_page + 1;
    let pages = if first == last {
        first.to_string()
    } else {
        format!("{first}–{last}")
    };
    ui.label(
        RichText::new(format!("{} · {} {pages}", passage.title, tr("pág.", "p.")))
            .strong()
            .color(p.ink),
    );
    if passage.crop.is_some() {
        ui.label(RichText::new(tr("Recorte vinculado", "Linked crop")).color(p.secondary));
    }
    if !passage.text.is_empty() {
        let display = passage.text.chars().take(240).collect::<String>();
        ui.label(RichText::new(display).color(p.secondary));
    }
}

fn label_field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    max: usize,
    multi: bool,
) -> bool {
    let label = ui.label(label);
    let edit = if multi {
        egui::TextEdit::multiline(value).desired_rows(2)
    } else {
        egui::TextEdit::singleline(value)
    };
    ui.add(edit.desired_width(f32::INFINITY).char_limit(max))
        .labelled_by(label.id)
        .changed()
}
