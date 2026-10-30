use super::*;
mod views;
use annotations::panel::{color_label, kind_label, review_label};
use unicode_segmentation::UnicodeSegmentation;
use yamantaka_application::annotations::{
    AnnotationBatchMutation, AnnotationOperation, AnnotationReply, AnnotationRequest,
};
use yamantaka_domain::annotations::{repository::*, *};

#[derive(Clone, PartialEq, Eq)]
struct Filters {
    text: String,
    tag: String,
    kind: Option<AnnotationKind>,
    color: Option<AnnotationColor>,
    review: Option<AnnotationReviewState>,
    days: Option<u8>,
    grouping: AnnotationGrouping,
}
impl Default for Filters {
    fn default() -> Self {
        Self {
            text: String::new(),
            tag: String::new(),
            kind: None,
            color: None,
            review: None,
            days: None,
            grouping: AnnotationGrouping::Document,
        }
    }
}
impl Filters {
    fn query(&self, offset: usize) -> AnnotationQuery {
        AnnotationQuery {
            text: self.text.trim().into(),
            tag: (!self.tag.trim().is_empty()).then(|| self.tag.trim().into()),
            kind: self.kind,
            color: self.color,
            review: self.review,
            grouping: Some(self.grouping),
            updated_after: self.days.map(|days| {
                (chrono::Utc::now() - chrono::Duration::days(i64::from(days)))
                    .to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
            }),
            offset,
            ..Default::default()
        }
    }
}
struct Pending {
    cancel: Cancellation,
    writing: bool,
}
pub(super) struct GlobalNotes {
    page: Option<Arc<AnnotationPage>>,
    pending: Option<Pending>,
    draft: Filters,
    filters: Filters,
    selected: [bool; 80],
    cursor: usize,
    tag: String,
    expanded: bool,
    error: Option<String>,
    saved: bool,
    preview: Option<&'static str>,
    preview_source: Option<(String, u32)>,
}
impl Default for GlobalNotes {
    fn default() -> Self {
        Self {
            page: None,
            pending: None,
            draft: Default::default(),
            filters: Default::default(),
            selected: [false; 80],
            cursor: 0,
            tag: String::new(),
            expanded: false,
            error: None,
            saved: false,
            preview: None,
            preview_source: None,
        }
    }
}
impl GlobalNotes {
    pub(in crate::app) fn export_ready(&self) -> bool {
        self.pending.is_none()
            && self
                .page
                .as_ref()
                .is_some_and(|page| !page.entries().is_empty())
    }
    pub(in crate::app) fn export_query(&self) -> Option<AnnotationQuery> {
        let page = self.page.as_ref()?;
        self.export_ready().then(|| self.filters.query(page.offset))
    }
    pub(super) fn source_ready(&self, active: Option<(&str, u32)>) -> bool {
        self.preview_source
            .as_ref()
            .is_none_or(|(id, page)| active == Some((id.as_str(), *page)))
    }
    pub(super) fn writing(&self) -> bool {
        self.pending.as_ref().is_some_and(|p| p.writing)
    }
    pub(super) fn ready(&self) -> bool {
        self.page.is_some() && self.pending.is_none() && self.preview.is_none()
    }
    pub(super) fn matches(&self, reply: &AnnotationReply) -> bool {
        reply.document_id.is_none()
            && self
                .pending
                .as_ref()
                .is_some_and(|p| p.cancel.same_request(&reply.cancel))
    }
    fn count(&self) -> usize {
        self.selected.iter().filter(|v| **v).count()
    }
    fn targets(&self) -> Vec<AnnotationTarget> {
        self.page.as_ref().map_or_else(Vec::new, |page| {
            page.entries()
                .iter()
                .zip(self.selected)
                .filter(|(_, v)| *v)
                .map(|(r, _)| AnnotationTarget {
                    annotation_id: r.data().id.clone(),
                    document_id: r.data().values.document_id.clone(),
                })
                .collect()
        })
    }
    fn request(
        &mut self,
        mutation: Option<AnnotationBatchMutation>,
        offset: usize,
        governor: &yamantaka_domain::memory::MemoryGovernor,
    ) -> Option<AnnotationRequest> {
        if self.writing() {
            return None;
        }
        if let Some(pending) = self.pending.take() {
            pending.cancel.cancel();
        }
        let writing = mutation.is_some();
        if !writing {
            self.page = None;
            self.selected.fill(false);
            self.tag.clear();
        }
        let cancel = Cancellation::default();
        self.pending = Some(Pending {
            cancel: cancel.clone(),
            writing,
        });
        self.error = None;
        self.saved = false;
        Some(AnnotationRequest {
            operation: mutation.map_or(AnnotationOperation::Read, |mutation| {
                AnnotationOperation::Batch { mutation }
            }),
            query: self.filters.query(offset),
            governor: governor.clone(),
            cancel,
        })
    }
    pub(super) fn accept(&mut self, reply: AnnotationReply) -> bool {
        if !self.matches(&reply) {
            return false;
        }
        let writing = self.pending.take().unwrap().writing;
        match reply.result {
            Ok(page) => {
                self.page = Some(Arc::new(page));
                self.cursor = 0;
                self.selected.fill(false);
                self.tag.clear();
                self.error = None;
                self.saved = writing;
                writing
            }
            Err(error) => {
                self.error = Some(error.to_string());
                false
            }
        }
    }
    pub(super) fn leave(&mut self) {
        if !self.writing() {
            if let Some(p) = self.pending.take() {
                p.cancel.cancel();
            }
            self.page = None;
            self.selected.fill(false);
            self.tag.clear();
        }
    }
    pub(super) fn cancel(&self) {
        if let Some(p) = &self.pending {
            p.cancel.cancel();
        }
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"accepted":self.page.as_ref().map_or(0,|p|p.entries().iter().filter(|r|r.data().values.review_state==AnnotationReviewState::Accepted).count()),"studyTagged":self.page.as_ref().map_or(0,|p|p.entries().iter().filter(|r|r.data().tags.iter().any(|tag|tag=="Study")).count()),"total":self.page.as_ref().map_or(0,|p|p.total),"resident":self.page.as_ref().map_or(0,|p|p.entries().len()),"selected":self.count(),"pending":self.pending.is_some(),"saved":self.saved,"ready":self.ready(),"documents":self.page.as_ref().map_or(0,|p|p.entries().iter().map(|r|&r.data().values.document_id).collect::<std::collections::BTreeSet<_>>().len())})
    }
}
impl Drop for GlobalNotes {
    fn drop(&mut self) {
        self.cancel();
    }
}
impl YamantakaApp {
    pub(super) fn accept_global_note_reply(&mut self, reply: AnnotationReply) {
        let writing = self.global_notes.writing();
        let failure = if writing && !matches!(self.surface, Surface::Notes) {
            reply.result.as_ref().err().map(ToString::to_string)
        } else {
            None
        };
        if self.global_notes.accept(reply) {
            self.reader.annotations.invalidate_records();
        }
        if writing {
            self.reader.annotations.refresh(None, None);
            if let Some(view) = &mut self.reference {
                view.pane.annotations.refresh(None, None);
            }
            self.invalidate_annotation_marks(None);
        }
        if let Some(error) = failure {
            self.error = Some(format!(
                "{}: {error}",
                self.tr(
                    "No se pudieron actualizar las notas de la biblioteca",
                    "Could not update library notes"
                )
            ));
        }
    }
    pub(super) fn show_global_notes(&mut self) {
        self.show_surface(Surface::Notes);
        self.global_note_command(None, 0);
    }
    fn global_note_command(&mut self, mutation: Option<AnnotationBatchMutation>, offset: usize) {
        if mutation.is_some()
            && (self.library_selection.removing()
                || self.library_tags.writing()
                || self.reader.annotations.editing()
                || self.reader.annotations.writing()
                || self.reference.as_ref().is_some_and(|view| {
                    view.pane.annotations.editing() || view.pane.annotations.writing()
                }))
        {
            return;
        }
        let Some(request) = self.global_notes.request(mutation, offset, &self.governor) else {
            return;
        };
        let cancel = request.cancel.clone();
        if let Err(error) = self.runtime.submit(Action::Annotation(Box::new(request))) {
            self.global_notes.accept(AnnotationReply {
                document_id: None,
                result: Err(error),
                cancel,
            });
        }
    }
    pub fn preview_global_notes(&mut self, action: &'static str) {
        if self.demo && self.capture.is_some() {
            self.show_global_notes();
            self.global_notes.preview = Some(action);
        }
    }
    pub(super) fn apply_global_note_preview(&mut self) {
        if self.global_notes.pending.is_some() || self.global_notes.page.is_none() {
            return;
        }
        let Some(action) = self.global_notes.preview.take() else {
            return;
        };
        match action {
            "export-json" | "export-markdown" => {
                self.preview_review_export(action == "export-markdown")
            }
            "review" | "tag" => {
                for selected in self.global_notes.selected.iter_mut().take(3) {
                    *selected = true;
                }
                let targets = self.global_notes.targets();
                let mutation = if action == "review" {
                    AnnotationBatchMutation::Review {
                        targets,
                        state: AnnotationReviewState::Accepted,
                    }
                } else {
                    AnnotationBatchMutation::AddTag {
                        targets,
                        tag: "Study".into(),
                    }
                };
                self.global_note_command(Some(mutation), 0);
            }
            "filters" => {
                self.global_notes.expanded = true;
            }
            "open" => {
                if let Some((id, page)) = self
                    .global_notes
                    .page
                    .as_ref()
                    .and_then(|p| p.entries().first())
                    .map(|r| {
                        (
                            r.data().values.document_id.clone(),
                            r.data().values.page_index,
                        )
                    })
                {
                    self.global_notes.preview_source = Some((id.clone(), page));
                    self.activate_global_note(id, page);
                }
            }
            "empty" => {
                self.global_notes.filters.text = "No matching synthetic note".into();
                self.global_notes.draft = self.global_notes.filters.clone();
                self.global_note_command(None, 0);
            }
            _ => {}
        }
    }
    pub(super) fn global_notes_panel(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        egui::ScrollArea::vertical()
            .id_salt("global-notes-screen")
            .show(ui, |ui| {
                egui::Frame::new().inner_margin(20).show(ui, |ui| {
                    let compact = ui.available_width() < 600.0;
                    theme::display(
                        ui,
                        self.tr("Notas de tu biblioteca", "Library notes"),
                        if compact { 24.0 } else { 32.0 },
                    );
                    ui.label(
                        RichText::new(self.tr(
                            "Vuelve a las ideas que merecen una segunda lectura.",
                            "Return to ideas worth reading again.",
                        ))
                        .color(p.secondary),
                    );
                    ui.add_space(16.0);
                    self.global_notes_filters(ui, p);
                    ui.add_space(12.0);
                    if self.global_notes.pending.is_some() {
                        ui.horizontal_wrapped(|ui| {
                            ui.spinner();
                            ui.label(self.tr("Actualizando notas…", "Updating notes…"));
                            if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p)
                                .clicked()
                            {
                                self.global_notes.cancel();
                            }
                        });
                    }
                    if let Some(error) = self.global_notes.error.clone() {
                        ui.label(self.tr(
                            "No se pudieron actualizar las notas.",
                            "Notes could not be updated.",
                        ));
                        ui.collapsing(self.tr("Detalles", "Details"), |ui| {
                            ui.label(error);
                        });
                        if theme::navigation(ui, self.tr("Volver a cargar", "Reload"), false, p)
                            .clicked()
                        {
                            self.global_note_command(None, 0);
                        }
                    }
                    if self.global_notes.saved {
                        ui.label(
                            RichText::new(self.tr(
                                "Notas actualizadas en tu biblioteca.",
                                "Notes updated in your library.",
                            ))
                            .color(p.secondary),
                        );
                    }
                    let Some(page) = self.global_notes.page.clone() else {
                        return;
                    };
                    ui.label(
                        RichText::new(format!("{} {}", page.total, self.tr("notas", "notes")))
                            .color(p.secondary),
                    );
                    self.global_notes_actions(ui, p, page.offset);
                    self.annotation_export_panel(ui, p);
                    ui.add_space(16.0);
                    if page.total == 0 {
                        ui.label(self.tr(
                            "No hay notas que coincidan. Prueba otros filtros.",
                            "No notes match. Try different filters.",
                        ));
                        return;
                    }
                    if ui.available_width() >= 900.0 {
                        ui.columns(2, |cols| {
                            self.global_notes_list(&mut cols[0], p, &page);
                            self.global_notes_detail(&mut cols[1], p, &page);
                        });
                    } else {
                        self.global_notes_list(ui, p, &page);
                        ui.add_space(16.0);
                        self.global_notes_detail(ui, p, &page);
                    }
                    ui.add_space(12.0);
                    ui.horizontal_wrapped(|ui| {
                        let idle = self.global_notes.pending.is_none();
                        if ui
                            .add_enabled_ui(idle && page.offset > 0, |ui| {
                                theme::navigation(ui, self.tr("Anteriores", "Previous"), false, p)
                            })
                            .inner
                            .clicked()
                        {
                            self.global_note_command(None, page.offset.saturating_sub(80));
                        }
                        if ui
                            .add_enabled_ui(idle && (page.offset as u64 + 80) < page.total, |ui| {
                                theme::navigation(ui, self.tr("Siguientes", "Next"), false, p)
                            })
                            .inner
                            .clicked()
                        {
                            self.global_note_command(None, page.offset.saturating_add(80));
                        }
                    });
                });
            });
    }
    fn activate_global_note(&mut self, id: String, page: u32) {
        if self.reader.annotations.editing() {
            return;
        }
        let cancel = self.open_intent.lookup(id.clone(), Some(page));
        self.send(Action::GetDocument(id, cancel));
    }
    fn global_notes_actions(&mut self, ui: &mut egui::Ui, p: Palette, offset: usize) {
        let editing = self.reader.annotations.editing()
            || self
                .reference
                .as_ref()
                .is_some_and(|view| view.pane.annotations.editing());
        let enabled = self.global_notes.pending.is_none()
            && !editing
            && !self.reader.annotations.writing()
            && !self
                .reference
                .as_ref()
                .is_some_and(|view| view.pane.annotations.writing())
            && !self.library_tags.writing();
        if editing {
            ui.label(self.tr(
                "Termina o cancela tu nota en el lector para editar estas notas.",
                "Finish or cancel your note in the reader to edit these notes.",
            ));
        }
        ui.add_enabled_ui(enabled, |ui| {
            ui.horizontal_wrapped(|ui| {
                if theme::navigation(ui, self.tr("Seleccionar página", "Select page"), false, p)
                    .clicked()
                {
                    let n = self
                        .global_notes
                        .page
                        .as_ref()
                        .map_or(0, |p| p.entries().len());
                    self.global_notes.selected.fill(false);
                    self.global_notes
                        .selected
                        .iter_mut()
                        .take(n)
                        .for_each(|v| *v = true);
                }
                if self.global_notes.count() > 0
                    && theme::navigation(
                        ui,
                        self.tr("Quitar selección", "Clear selection"),
                        false,
                        p,
                    )
                    .clicked()
                {
                    self.global_notes.selected.fill(false);
                }
            });
            let count = self.global_notes.count();
            if count == 0 {
                return;
            }
            ui.label(format!("{count} {}", self.tr("seleccionadas", "selected")));
            ui.horizontal_wrapped(|ui| {
                for (state, en, es) in [
                    (
                        AnnotationReviewState::Pending,
                        "Mark pending",
                        "Marcar pendientes",
                    ),
                    (AnnotationReviewState::Accepted, "Accept", "Aceptar"),
                    (AnnotationReviewState::Resolved, "Resolve", "Resolver"),
                ] {
                    if theme::navigation(ui, if self.english { en } else { es }, false, p).clicked()
                    {
                        let targets = self.global_notes.targets();
                        self.global_note_command(
                            Some(AnnotationBatchMutation::Review { targets, state }),
                            offset,
                        );
                    }
                }
            });
            let label = ui.label(self.tr("Añadir una etiqueta", "Add one tag"));
            ui.add(
                egui::TextEdit::singleline(&mut self.global_notes.tag)
                    .desired_width(260.0)
                    .char_limit(4096),
            )
            .labelled_by(label.id);
            let tag = self.global_notes.tag.trim();
            let valid = !tag.is_empty() && tag.len() <= 4096 && tag.graphemes(true).count() <= 80;
            if ui
                .add_enabled_ui(valid, |ui| {
                    theme::navigation(
                        ui,
                        self.tr("Añadir a las seleccionadas", "Add to selected"),
                        false,
                        p,
                    )
                })
                .inner
                .clicked()
            {
                let tag = self.global_notes.tag.trim().to_owned();
                let targets = self.global_notes.targets();
                self.global_note_command(
                    Some(AnnotationBatchMutation::AddTag { targets, tag }),
                    offset,
                );
            }
        });
    }
}
mod filters;
