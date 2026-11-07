use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_scope_combines_with_review_and_exact_note_lookup_takes_priority() {
        let mut filters = Filters {
            page: Some(3),
            review: Some(AnnotationReviewState::Pending),
            ..Default::default()
        };
        let query = filters.query("book".into(), 80);
        assert_eq!(query.page, Some(3));
        assert_eq!(query.review, Some(AnnotationReviewState::Pending));
        assert_eq!(filters.count(), 2);
        filters.focused = Some("exact".into());
        let exact = filters.query("book".into(), 80);
        assert_eq!(exact.annotation_id.as_deref(), Some("exact"));
        assert_eq!(exact.page, None);
        assert_eq!(exact.offset, 0);
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct Filters {
    pub focused: Option<String>,
    pub page: Option<u32>,
    pub text: String,
    pub tag: String,
    pub kind: Option<AnnotationKind>,
    pub review: Option<AnnotationReviewState>,
}
impl Filters {
    pub(super) fn count(&self) -> usize {
        usize::from(!self.text.trim().is_empty())
            + usize::from(!self.tag.trim().is_empty())
            + usize::from(self.kind.is_some())
            + usize::from(self.review.is_some())
            + usize::from(self.page.is_some())
    }
    pub(super) fn query(&self, document: String, offset: usize) -> AnnotationQuery {
        if let Some(id) = &self.focused {
            return AnnotationQuery {
                document_id: Some(document),
                annotation_id: Some(id.clone()),
                ..Default::default()
            };
        }
        AnnotationQuery {
            document_id: Some(document),
            offset,
            text: self.text.trim().into(),
            tag: (!self.tag.trim().is_empty()).then(|| self.tag.trim().into()),
            kind: self.kind,
            review: self.review,
            page: self.page,
            grouping: Some(AnnotationGrouping::Page),
            ..Default::default()
        }
    }
}
impl AnnotationState {
    fn toggle_page_filter(&mut self, page: u32) {
        self.filter_draft = self.filters.clone();
        self.filter_draft.page = self.filters.page.is_none().then_some(page);
    }

    pub(super) fn apply_filters(&mut self) -> bool {
        if self.editor.is_some()
            || self.pending.as_ref().is_some_and(|pending| pending.writing)
            || self.filters == self.filter_draft
        {
            return false;
        }
        if let Some(pending) = self.pending.take() {
            pending.cancel.cancel();
        }
        self.selected = Default::default();
        self.bulk_tag.clear();
        self.filters = self.filter_draft.clone();
        self.page = None;
        self.error = None;
        self.saved = false;
        true
    }
}

pub(super) enum FilterPreview {
    Matching,
    Empty,
    Page,
}

impl YamantakaApp {
    pub fn preview_annotation_filters(&mut self, empty: bool) {
        if self.demo && self.capture.is_some() {
            self.preview_annotation_list();
            self.reader.annotations.filter_preview = Some(if empty {
                FilterPreview::Empty
            } else {
                FilterPreview::Matching
            });
        }
    }

    pub fn preview_annotation_page_filter(&mut self) {
        if self.demo && self.capture.is_some() {
            self.preview_annotation_list();
            self.reader.annotations.filter_preview = Some(FilterPreview::Page);
        }
    }
    pub(in crate::app) fn apply_annotation_filter_preview(&mut self) {
        if !self.reader.annotations.ready() {
            return;
        }
        let Some(preview) = self.reader.annotations.filter_preview.take() else {
            return;
        };
        let empty = matches!(preview, FilterPreview::Empty);
        let details = !empty && !matches!(preview, FilterPreview::Page);
        if matches!(preview, FilterPreview::Page) {
            self.reader.annotations.toggle_page_filter(self.reader.page);
        } else {
            self.reader.annotations.filter_draft = Filters {
                text: if empty {
                    "No matching study note"
                } else {
                    "Synthetic"
                }
                .into(),
                tag: "Study".into(),
                kind: Some(AnnotationKind::Highlight),
                review: Some(AnnotationReviewState::Pending),
                ..Default::default()
            };
        }
        if self.reader.annotations.apply_filters() {
            self.reader.annotations.filters_open = details;
            self.annotation_command(None, 0);
        }
    }
    pub(in crate::app) fn annotation_filters_panel(&mut self, ui: &mut egui::Ui, p: Palette) {
        if let Some(action) =
            self.reader
                .annotations
                .show_filters(ui, self.english, self.reader.page, p)
        {
            self.apply_notes_action(false, action);
        }
    }
}
impl AnnotationState {
    pub(in crate::app) fn show_filters(
        &mut self,
        ui: &mut egui::Ui,
        english: bool,
        current_page: u32,
        p: Palette,
    ) -> Option<NotesAction> {
        let tr = |es, en| if english { en } else { es };
        if self.filters.focused.is_some() {
            return ui
                .add_enabled_ui(!self.editing() && !self.writing(), |ui| {
                    theme::navigation(ui, tr("Volver a notas", "Back to notes"), false, p)
                })
                .inner
                .clicked()
                .then_some(NotesAction::ClearFocus);
        }
        let count = self.filters.count();
        let caption = if count == 0 {
            tr("Filtrar notas", "Filter notes").to_owned()
        } else {
            format!("{} ({count})", tr("Filtros activos", "Active filters"))
        };
        let enabled =
            !self.editing() && !self.pending.as_ref().is_some_and(|pending| pending.writing);
        let mut apply = false;
        ui.add_enabled_ui(enabled, |ui| {
            ui.horizontal_wrapped(|ui| {
                let page = self.filters.page;
                let page_label = page.map_or_else(
                    || tr("Esta página", "This page").to_owned(),
                    |page| format!("{} {}", tr("Página", "Page"), u64::from(page) + 1),
                );
                if theme::navigation(ui, &page_label, page.is_some(), p)
                    .on_hover_text(if page.is_some() {
                        tr("Quitar filtro de página", "Clear page filter")
                    } else {
                        tr(
                            "Mostrar solo las anotaciones de esta página",
                            "Show only annotations on this page",
                        )
                    })
                    .clicked()
                {
                    self.toggle_page_filter(current_page);
                    apply = true;
                }
                let pending = self.filters.review == Some(AnnotationReviewState::Pending);
                if theme::navigation(ui, tr("Pendientes", "Pending"), pending, p).clicked() {
                    self.filter_draft = self.filters.clone();
                    self.filter_draft.review = (!pending).then_some(AnnotationReviewState::Pending);
                    apply = true;
                }
                if theme::navigation(ui, &caption, self.filters_open, p).clicked() {
                    self.filters_open = !self.filters_open;
                }
                if count > 0
                    && theme::navigation(ui, tr("Restablecer", "Reset"), false, p).clicked()
                {
                    self.filter_draft = Filters::default();
                    apply = true;
                }
            });
            if self.filters_open {
                let draft = &mut self.filter_draft;
                let label = ui.label(if english { "Text" } else { "Texto" });
                ui.add(
                    egui::TextEdit::singleline(&mut draft.text)
                        .desired_width(f32::INFINITY)
                        .char_limit(160)
                        .hint_text(if english {
                            "Passage or note"
                        } else {
                            "Pasaje o nota"
                        }),
                )
                .labelled_by(label.id);
                let label = ui.label(if english { "Tag" } else { "Etiqueta" });
                ui.add(
                    egui::TextEdit::singleline(&mut draft.tag)
                        .desired_width(f32::INFINITY)
                        .char_limit(160)
                        .hint_text(if english {
                            "Exact tag name"
                        } else {
                            "Nombre exacto"
                        }),
                )
                .labelled_by(label.id);
                ui.horizontal_wrapped(|ui| {
                    let label = ui.label(if english { "Mark" } else { "Marca" });
                    egui::ComboBox::from_id_salt("annotation-filter-kind")
                        .selected_text(
                            draft
                                .kind
                                .map_or(if english { "All" } else { "Todas" }, |kind| {
                                    super::panel::kind_label(kind, english)
                                }),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut draft.kind,
                                None,
                                if english {
                                    "All marks"
                                } else {
                                    "Todas las marcas"
                                },
                            );
                            for kind in [
                                AnnotationKind::Highlight,
                                AnnotationKind::Underline,
                                AnnotationKind::Strikeout,
                                AnnotationKind::TextNote,
                                AnnotationKind::PageMarker,
                                AnnotationKind::PageComment,
                                AnnotationKind::Ink,
                                AnnotationKind::Arrow,
                                AnnotationKind::Rectangle,
                            ] {
                                ui.selectable_value(
                                    &mut draft.kind,
                                    Some(kind),
                                    super::panel::kind_label(kind, english),
                                );
                            }
                        })
                        .response
                        .labelled_by(label.id);
                });
                ui.horizontal_wrapped(|ui| {
                    let label = ui.label(if english { "Review" } else { "Revisión" });
                    let caption = match draft.review {
                        None => {
                            if english {
                                "All"
                            } else {
                                "Todos"
                            }
                        }
                        Some(AnnotationReviewState::Pending) => {
                            if english {
                                "Pending"
                            } else {
                                "Pendiente"
                            }
                        }
                        Some(AnnotationReviewState::Accepted) => {
                            if english {
                                "Accepted"
                            } else {
                                "Aceptada"
                            }
                        }
                        Some(AnnotationReviewState::Resolved) => {
                            if english {
                                "Resolved"
                            } else {
                                "Resuelta"
                            }
                        }
                    };
                    egui::ComboBox::from_id_salt("annotation-filter-review")
                        .selected_text(caption)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut draft.review,
                                None,
                                if english {
                                    "All states"
                                } else {
                                    "Todos los estados"
                                },
                            );
                            for (state, en, es) in [
                                (AnnotationReviewState::Pending, "Pending", "Pendiente"),
                                (AnnotationReviewState::Accepted, "Accepted", "Aceptada"),
                                (AnnotationReviewState::Resolved, "Resolved", "Resuelta"),
                            ] {
                                ui.selectable_value(
                                    &mut draft.review,
                                    Some(state),
                                    if english { en } else { es },
                                );
                            }
                        })
                        .response
                        .labelled_by(label.id);
                });
                if theme::navigation(ui, tr("Aplicar filtros", "Apply filters"), false, p).clicked()
                {
                    apply = true;
                }
            }
        });
        (apply && self.apply_filters()).then_some(NotesAction::Read(0))
    }
}
