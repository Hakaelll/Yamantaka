use super::panel::{color_label, kind_label, review_label};
use super::*;

pub(in crate::app) enum NotesAction {
    Focus(String),
    ClearFocus,
    RetryDraft,
    Read(usize),
    Change(AnnotationMutation, usize),
    PageNote,
    Selection,
    Navigate(yamantaka_domain::reader::ReaderPosition, String, bool),
    ReloadMarks,
    Export(yamantaka_application::annotation_export::AnnotationExportFormat),
}
pub(in crate::app) struct NotesContext<'a> {
    pub english: bool,
    pub page: u32,
    pub page_count: usize,
    pub busy: bool,
    pub selectable: bool,
    pub reveal_actions: bool,
    pub governor: &'a MemoryGovernor,
    pub marks_error: Option<String>,
}
impl AnnotationState {
    pub(in crate::app) fn show_editor(
        &mut self,
        ui: &mut egui::Ui,
        context: &NotesContext<'_>,
        p: Palette,
        show_intro: bool,
    ) -> Option<NotesAction> {
        let tr = |es, en| if context.english { en } else { es };
        let busy = context.busy || self.pending.is_some() || self.recovered.is_some();
        let offset = self.page.as_ref().map_or(0, |page| page.offset);
        let mut action = None;
        if show_intro {
            ui.label(
                RichText::new(tr("Notas de lectura", "Reading notes"))
                    .size(22.0)
                    .color(p.ink),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(tr(
                    "Conserva una idea y vuelve a su contexto.",
                    "Keep an idea and return to its context.",
                ))
                .color(p.secondary),
            );
            ui.add_space(16.0);
        }
        if self.recovered.is_some() {
            return self.stored_recovery_panel(ui, p, context.english);
        }
        if show_intro || self.history_available(false) || self.history_available(true) {
            ui.horizontal_wrapped(|ui| {
                for (redo, es, en) in [(false, "Deshacer", "Undo"), (true, "Rehacer", "Redo")] {
                    ui.add_enabled_ui(!busy && self.history_available(redo), |ui| {
                        if theme::navigation(ui, tr(es, en), false, p).clicked() {
                            action = Some(NotesAction::Change(
                                if redo {
                                    AnnotationMutation::Redo
                                } else {
                                    AnnotationMutation::Undo
                                },
                                offset,
                            ));
                        }
                    });
                }
            });
        }
        if busy {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(tr("Actualizando notas…", "Updating notes…"));
            });
        }
        if self.saved {
            ui.label(
                RichText::new(tr(
                    "Notas actualizadas en tu biblioteca.",
                    "Notes updated in your library.",
                ))
                .color(p.secondary),
            );
        }
        if let Some(error) = self.error.clone() {
            ui.label(tr(
                "No se pudieron actualizar las notas.",
                "Notes could not be updated.",
            ));
            ui.collapsing(tr("Detalles", "Details"), |ui| {
                ui.label(error);
            });
            if !busy && theme::navigation(ui, tr("Volver a cargar", "Reload"), false, p).clicked() {
                action = Some(NotesAction::Read(offset));
            }
        }
        if let Some(error) = context.marks_error.clone() {
            ui.label(tr(
                "No se pudieron cargar las marcas de página.",
                "Page marks could not be loaded.",
            ));
            ui.collapsing(tr("Detalles de las marcas", "Mark details"), |ui| {
                ui.label(error);
            });
            if theme::navigation(ui, tr("Recargar marcas", "Reload marks"), false, p).clicked() {
                action = Some(NotesAction::ReloadMarks);
            }
        }
        if self.stale
            && self
                .editor
                .as_ref()
                .is_some_and(|editor| editor.stored().is_some())
        {
            ui.label(tr(
                "Las notas han cambiado. Cancela para recargar antes de guardar otra edición.",
                "Notes changed. Cancel to reload before saving another edit.",
            ));
        }
        if self.editor.is_some() {
            let english = context.english;
            let fallback = self.fallback();
            let focus_note = !busy && std::mem::take(&mut self.focus_editor);
            let editor = self.editor.as_mut().unwrap();
            ui.add_enabled_ui(!busy,|ui| {
                if let EditorSource::Selection {snapshot,..}=&editor.source
                    && snapshot.pages().len()>1
                {
                    ui.label(if english {format!("{} pages. Your note applies to each selected passage.",snapshot.pages().len())} else {format!("{} páginas. Tu nota se aplica a cada pasaje seleccionado.",snapshot.pages().len())});
                    ui.add_space(8.0);
                }
                if let EditorSource::PageNote {page,..}=&editor.source {
                    ui.label(if english {format!("Page {}. This note does not require selectable text.",u64::from(*page)+1)} else {format!("Página {}. Esta nota no necesita texto seleccionable.",u64::from(*page)+1)});
                    ui.add_space(8.0);
                }
                let passage=editor.passage();
                if !passage.is_empty() {
                    egui::Frame::new().fill(p.raised).inner_margin(if show_intro { 12 } else { 8 }).corner_radius(8).show(ui,|ui| {
                        egui::ScrollArea::vertical().id_salt("annotation-passage").max_height(110.0).show(ui,|ui| {ui.label(RichText::new(passage).color(p.ink));});
                    });
                    ui.add_space(if show_intro { 14.0 } else { 6.0 });
                }
                let label=ui.label(if english {"Your note"} else {"Tu nota"});
                let note = ui.add(egui::TextEdit::multiline(&mut editor.note).desired_rows(4).desired_width(f32::INFINITY).char_limit(4000).hint_text(if english {"What would you like to remember?"} else {"¿Qué te gustaría recordar?"})).labelled_by(label.id);
                if focus_note { note.request_focus(); }
                let units=editor.note.encode_utf16().count();
                ui.label(RichText::new(format!("{units} / 4000")).size(12.0).color(p.secondary));
                ui.add_space(8.0);
                egui::CollapsingHeader::new(if english {"Details"} else {"Detalles"})
                    .id_salt("annotation-details")
                    .default_open(!editor.tags.is_empty())
                    .show(ui, |ui| {
                let kinds=[AnnotationKind::Highlight,AnnotationKind::Underline,AnnotationKind::Strikeout,AnnotationKind::TextNote,AnnotationKind::PageMarker,AnnotationKind::PageComment];
                let geometry=!matches!(&editor.source,EditorSource::PageNote {..}) && editor.stored().is_none_or(|record|!record.values.rects.is_empty());
                ui.horizontal_wrapped(|ui| {
                    let label=ui.label(if english {"Mark"} else {"Marca"});
                    if editor.kind.is_drawing() {
                        ui.label(kind_label(editor.kind, english)).labelled_by(label.id);
                    } else {
                    let response=egui::ComboBox::from_id_salt("annotation-kind").selected_text(kind_label(editor.kind,english)).show_ui(ui,|ui| {
                        for kind in kinds {
                            if (geometry || kind==AnnotationKind::PageComment)
                                && ui.selectable_value(&mut editor.kind,kind,kind_label(kind,english)).changed()
                            {editor.confirmed=false;}
                        }
                    });
                    response.response.labelled_by(label.id);
                    }
                });
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    for color in [AnnotationColor::Yellow,AnnotationColor::Green,AnnotationColor::Blue,AnnotationColor::Rose,AnnotationColor::Violet] {
                        let response=theme::navigation(ui,color_label(color,english),editor.color==color,p);
                        if response.clicked() {editor.color=color;}
                    }
                });
                ui.add_space(14.0);
                let label = ui.label(if english {"Tags, optional"} else {"Etiquetas, opcionales"});
                ui.add(egui::TextEdit::singleline(&mut editor.tags).desired_width(f32::INFINITY).hint_text(if english {"Study, optics"} else {"Estudio, óptica"})).labelled_by(label.id);
                let valid_tags = editor.tag_changes().is_ok();
                ui.label(RichText::new(if valid_tags {
                    if english {"Separate tags with commas. Up to 12 tags and 160 characters."} else {"Separa con comas. Hasta 12 etiquetas y 160 caracteres."}
                } else {
                    if english {"Shorten the tags to 160 characters and keep at most 12."} else {"Reduce las etiquetas a 160 caracteres y deja un máximo de 12."}
                }).size(12.0).color(if valid_tags {p.secondary} else {p.ink}));
                if editor.stored().is_some() {
                    ui.add_space(8.0);
                    let label=ui.label(if english {"Review"} else {"Revisión"});
                    egui::ComboBox::from_id_salt("annotation-review").selected_text(match editor.review {AnnotationReviewState::Pending=>if english {"Pending"}else{"Pendiente"},AnnotationReviewState::Accepted=>if english {"Accepted"}else{"Aceptada"},AnnotationReviewState::Resolved=>if english {"Resolved"}else{"Resuelta"}}).show_ui(ui,|ui| {
                        for (state,en,es) in [(AnnotationReviewState::Pending,"Pending","Pendiente"),(AnnotationReviewState::Accepted,"Accepted","Aceptada"),(AnnotationReviewState::Resolved,"Resolved","Resuelta")] {ui.selectable_value(&mut editor.review,state,if english {en}else{es});}
                    }).response.labelled_by(label.id);
                }
                });
                if fallback {
                    ui.add_space(12.0);
                    ui.label(if english {"Some selected passages have incomplete geometry. Keep those as page notes with their text."} else {"Algunos pasajes no tienen geometría completa. Consérvalos como notas de página con su texto."});
                    ui.checkbox(&mut editor.confirmed,if english {"Save affected passages as page notes"}else{"Guardar los pasajes afectados como notas de página"});
                }
            });
            ui.add_space(12.0);
            let valid = (!self.stale
                || self
                    .editor
                    .as_ref()
                    .is_none_or(|editor| editor.stored().is_none()))
                && self.editor.as_ref().is_some_and(|editor| {
                    editor.note.encode_utf16().count() <= 4000
                        && editor.tag_changes().is_ok()
                        && (!fallback || editor.confirmed)
                        && (!(matches!(&editor.source, EditorSource::PageNote { .. })
                            || editor.kind == AnnotationKind::TextNote)
                            || !editor.note.trim().is_empty())
                });
            let actions = ui
                .horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled_ui(!busy && valid, |ui| {
                            theme::control(
                                ui,
                                tr("Guardar nota", "Save note"),
                                Some(theme::Icon::Add),
                                false,
                                true,
                                p,
                            )
                        })
                        .inner
                        .clicked()
                    {
                        match self.mutation(context.governor) {
                            Ok(change) => action = Some(NotesAction::Change(change, offset)),
                            Err(error) => self.error = Some(error.to_string()),
                        }
                    }
                    if !busy && theme::navigation(ui, tr("Cancelar", "Cancel"), false, p).clicked()
                    {
                        self.editor = None;
                    }
                })
                .response;
            if context.reveal_actions {
                actions.scroll_to_me(Some(egui::Align::Max));
            }
        } else {
            ui.add_enabled_ui(!busy, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if theme::control(
                        ui,
                        tr("Nota de página", "Page note"),
                        Some(theme::Icon::Add),
                        false,
                        true,
                        p,
                    )
                    .on_hover_text(format!(
                        "{} {}",
                        tr("Añadir nota a la página", "Add note to page"),
                        u64::from(context.page) + 1
                    ))
                    .clicked()
                    {
                        action = Some(NotesAction::PageNote);
                    }
                    if theme::navigation(ui, tr("Anotar selección", "Annotate selection"), false, p)
                        .on_hover_text(tr(
                            "Selecciona un pasaje en el PDF para añadir una nota.",
                            "Select a passage in the PDF to add a note.",
                        ))
                        .clicked()
                    {
                        action = Some(NotesAction::Selection);
                    }
                });
            });
        }
        action
    }
    pub(in crate::app) fn show_list(
        &mut self,
        ui: &mut egui::Ui,
        context: &NotesContext<'_>,
        p: Palette,
    ) -> Option<NotesAction> {
        let tr = |es, en| if context.english { en } else { es };
        let busy = context.busy || self.pending.is_some() || self.editing();
        let mut action = None;
        let page = self.page.clone()?;
        let mut previous_page = None;
        if self.filters.count() > 0 && page.total > 0 {
            let count = match page.total {
                1 => tr("1 resultado", "1 result").to_owned(),
                total => format!("{total} {}", tr("resultados", "results")),
            };
            ui.label(RichText::new(count).size(13.0).color(p.secondary));
        }
        if page.total == 0 {
            ui.label(
                RichText::new(if self.filters.count() > 0 {
                    tr(
                        "No hay notas que coincidan. Prueba otros filtros.",
                        "No notes match. Try different filters.",
                    )
                } else {
                    tr("Tus notas aparecerán aquí.", "Your notes will appear here.")
                })
                .color(p.secondary),
            );
        }
        for (index, record) in page.entries().iter().enumerate() {
            let data = record.data();
            if previous_page != Some(data.values.page_index) {
                previous_page = Some(data.values.page_index);
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        tr("Página", "Page"),
                        data.values.page_index + 1
                    ))
                    .strong()
                    .color(p.ink),
                );
            }
            ui.push_id(&data.id, |ui| {
                ui.separator();
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    if context.selectable {
                        let selected = self.selected.0[index];
                        let caption = format!(
                            "{} {}",
                            tr(
                                if selected {
                                    "Nota seleccionada"
                                } else {
                                    "Seleccionar nota"
                                },
                                if selected {
                                    "Selected note"
                                } else {
                                    "Select note"
                                }
                            ),
                            page.offset.saturating_add(index).saturating_add(1)
                        );
                        if ui
                            .add_enabled_ui(self.pending.is_none() && !self.editing(), |ui| {
                                theme::navigation(ui, &caption, selected, p)
                            })
                            .inner
                            .clicked()
                        {
                            self.selected.0[index] = !selected;
                        }
                    }
                    ui.horizontal(|ui| {
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(10.0, 16.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            rect.center(),
                            4.0,
                            panel::mark_color(data.values.color),
                        );
                        response.on_hover_text(color_label(data.values.color, context.english));
                        ui.label(
                            RichText::new(kind_label(data.values.kind, context.english))
                                .strong()
                                .color(p.ink),
                        );
                        ui.label(
                            RichText::new(review_label(data.values.review_state, context.english))
                                .size(13.0)
                                .color(p.secondary),
                        );
                    });
                });
                if let Some(passage) = &data.values.selected_text {
                    ui.label(RichText::new(passage).italics().color(p.secondary));
                }
                if let Some(note) = &data.values.note {
                    ui.label(RichText::new(note).color(p.ink));
                }
                if data.values.status == AttachmentStatus::ReviewRequired {
                    ui.label(
                        RichText::new(tr(
                            "El PDF cambió · revisa la ubicación",
                            "The PDF changed · review the location",
                        ))
                        .color(p.secondary),
                    );
                }
                if !data.tags.is_empty() {
                    ui.label(
                        RichText::new(data.tags.join(" · "))
                            .size(12.0)
                            .color(p.secondary),
                    );
                }
                ui.horizontal_wrapped(|ui| {
                    let caption = format!(
                        "{} {}",
                        tr("Página", "Page"),
                        u64::from(data.values.page_index) + 1
                    );
                    if ui
                        .add_enabled_ui(
                            (data.values.page_index as usize) < context.page_count,
                            |ui| theme::navigation(ui, &caption, false, p),
                        )
                        .inner
                        .clicked()
                    {
                        let close = bookmarks::compact_study(ui.ctx());
                        let point = data
                            .values
                            .rects
                            .first()
                            .map(|rect| {
                                let rect = rect.region();
                                [(rect.min()[0] + rect.max()[0]) * 0.5, rect.min()[1]]
                            })
                            .or_else(|| {
                                data.values
                                    .drawing
                                    .as_ref()
                                    .and_then(|drawing| drawing.points().first().copied())
                            })
                            .filter(|_| data.values.status == AttachmentStatus::Attached)
                            .unwrap_or([0.5, 0.0]);
                        if let Ok(position) = yamantaka_domain::reader::ReaderPosition::at(
                            data.values.page_index,
                            point,
                        ) {
                            action = Some(NotesAction::Navigate(position, data.id.clone(), close));
                        }
                        if close {
                            ui.close();
                        }
                    }
                    if !busy
                        && ui
                            .add_enabled_ui(!self.editing(), |ui| {
                                theme::navigation(ui, tr("Editar", "Edit"), false, p)
                            })
                            .inner
                            .clicked()
                    {
                        self.edit_entry(index);
                    }
                    if !busy
                        && ui
                            .add_enabled_ui(!self.editing(), |ui| {
                                theme::navigation(ui, tr("Eliminar", "Remove"), false, p)
                            })
                            .inner
                            .clicked()
                    {
                        action = Some(NotesAction::Change(
                            AnnotationMutation::Remove {
                                id: data.id.clone(),
                            },
                            page.offset,
                        ));
                    }
                });
            });
        }
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled_ui(!busy && page.offset > 0, |ui| {
                    theme::navigation(ui, tr("Anteriores", "Previous"), false, p)
                })
                .inner
                .clicked()
            {
                action = Some(NotesAction::Read(page.offset.saturating_sub(80)));
            }
            if ui
                .add_enabled_ui(!busy && (page.offset as u64 + 80) < page.total, |ui| {
                    theme::navigation(ui, tr("Siguientes", "Next"), false, p)
                })
                .inner
                .clicked()
            {
                action = Some(NotesAction::Read(page.offset.saturating_add(80)));
            }
        });
        action
    }
}

impl AnnotationState {
    pub(in crate::app) fn edit_entry(&mut self, index: usize) {
        if self.editing() || self.writing() {
            return;
        }
        let Some(page) = self.page.clone() else {
            return;
        };
        let Some(record) = page.entries().get(index) else {
            return;
        };
        let data = record.data();
        self.editor = Some(AnnotationEditor {
            source: EditorSource::Stored {
                page: page.clone(),
                index,
            },
            kind: data.values.kind,
            color: data.values.color,
            note: data.values.note.clone().unwrap_or_default(),
            tags: data.tags.join(", "),
            review: data.values.review_state,
            confirmed: false,
        });
        self.focus_editor = true;
    }
}
