use super::*;

fn choose<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    label: &str,
    id: &str,
    value: &mut T,
    choices: &[(T, &str)],
) {
    let label = ui.label(label);
    let caption = choices
        .iter()
        .find(|(candidate, _)| candidate == value)
        .map_or("", |(_, text)| *text);
    egui::ComboBox::from_id_salt(id)
        .selected_text(caption)
        .show_ui(ui, |ui| {
            for (candidate, text) in choices {
                ui.selectable_value(value, *candidate, *text);
            }
        })
        .response
        .labelled_by(label.id);
}
impl YamantakaApp {
    pub(super) fn global_notes_filters(&mut self, ui: &mut egui::Ui, p: Palette) {
        let english = self.english;
        let mut apply = false;
        ui.add_enabled_ui(!self.global_notes.writing(), |ui| {
            let label = ui.label(self.tr("Buscar en pasajes y notas", "Search passages and notes"));
            ui.add(
                egui::TextEdit::singleline(&mut self.global_notes.draft.text)
                    .desired_width(f32::INFINITY)
                    .char_limit(160),
            )
            .labelled_by(label.id);
            ui.horizontal_wrapped(|ui| {
                if theme::navigation(
                    ui,
                    self.tr("Filtros", "Filters"),
                    self.global_notes.expanded,
                    p,
                )
                .clicked()
                {
                    self.global_notes.expanded = !self.global_notes.expanded;
                }
                if theme::navigation(ui, self.tr("Restablecer", "Reset"), false, p).clicked() {
                    self.global_notes.draft = Default::default();
                    apply = true;
                }
                let draft = &mut self.global_notes.draft;
                choose(
                    ui,
                    if english { "Group by" } else { "Agrupar por" },
                    "global-notes-group",
                    &mut draft.grouping,
                    &[
                        (
                            AnnotationGrouping::Document,
                            if english { "Document" } else { "Documento" },
                        ),
                        (
                            AnnotationGrouping::Page,
                            if english { "Page" } else { "Página" },
                        ),
                        (
                            AnnotationGrouping::Review,
                            if english { "Review" } else { "Revisión" },
                        ),
                    ],
                );
            });
            if self.global_notes.expanded {
                let draft = &mut self.global_notes.draft;
                let label = ui.label(if english {
                    "Exact tag name"
                } else {
                    "Nombre exacto de etiqueta"
                });
                ui.add(
                    egui::TextEdit::singleline(&mut draft.tag)
                        .desired_width(260.0)
                        .char_limit(160),
                )
                .labelled_by(label.id);
                ui.horizontal_wrapped(|ui| {
                    choose(
                        ui,
                        if english { "Mark" } else { "Marca" },
                        "global-notes-kind",
                        &mut draft.kind,
                        &[
                            (None, if english { "All" } else { "Todas" }),
                            (
                                Some(AnnotationKind::Highlight),
                                kind_label(AnnotationKind::Highlight, english),
                            ),
                            (
                                Some(AnnotationKind::Underline),
                                kind_label(AnnotationKind::Underline, english),
                            ),
                            (
                                Some(AnnotationKind::Strikeout),
                                kind_label(AnnotationKind::Strikeout, english),
                            ),
                            (
                                Some(AnnotationKind::TextNote),
                                kind_label(AnnotationKind::TextNote, english),
                            ),
                            (
                                Some(AnnotationKind::PageMarker),
                                kind_label(AnnotationKind::PageMarker, english),
                            ),
                            (
                                Some(AnnotationKind::PageComment),
                                kind_label(AnnotationKind::PageComment, english),
                            ),
                        ],
                    );
                    choose(
                        ui,
                        "Color",
                        "global-notes-color",
                        &mut draft.color,
                        &[
                            (None, if english { "All" } else { "Todos" }),
                            (
                                Some(AnnotationColor::Yellow),
                                color_label(AnnotationColor::Yellow, english),
                            ),
                            (
                                Some(AnnotationColor::Green),
                                color_label(AnnotationColor::Green, english),
                            ),
                            (
                                Some(AnnotationColor::Blue),
                                color_label(AnnotationColor::Blue, english),
                            ),
                            (
                                Some(AnnotationColor::Rose),
                                color_label(AnnotationColor::Rose, english),
                            ),
                            (
                                Some(AnnotationColor::Violet),
                                color_label(AnnotationColor::Violet, english),
                            ),
                        ],
                    );
                });
                ui.horizontal_wrapped(|ui| {
                    choose(
                        ui,
                        if english { "Review" } else { "Revisión" },
                        "global-notes-review",
                        &mut draft.review,
                        &[
                            (None, if english { "All" } else { "Todos" }),
                            (
                                Some(AnnotationReviewState::Pending),
                                review_label(AnnotationReviewState::Pending, english),
                            ),
                            (
                                Some(AnnotationReviewState::Accepted),
                                review_label(AnnotationReviewState::Accepted, english),
                            ),
                            (
                                Some(AnnotationReviewState::Resolved),
                                review_label(AnnotationReviewState::Resolved, english),
                            ),
                        ],
                    );
                    choose(
                        ui,
                        if english { "Updated" } else { "Actualizadas" },
                        "global-notes-date",
                        &mut draft.days,
                        &[
                            (None, if english { "Any time" } else { "Siempre" }),
                            (
                                Some(7),
                                if english {
                                    "Last 7 days"
                                } else {
                                    "Últimos 7 días"
                                },
                            ),
                            (
                                Some(30),
                                if english {
                                    "Last 30 days"
                                } else {
                                    "Últimos 30 días"
                                },
                            ),
                        ],
                    );
                });
            }
            if theme::navigation(ui, self.tr("Aplicar", "Apply"), false, p).clicked() {
                apply = true;
            }
        });
        if apply {
            self.global_notes.filters = self.global_notes.draft.clone();
            self.global_note_command(None, 0);
        }
    }
}
