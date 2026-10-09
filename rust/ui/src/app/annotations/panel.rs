use super::*;

fn kind_label(kind: AnnotationKind, english: bool) -> &'static str {
    match (kind, english) {
        (AnnotationKind::Highlight, true) => "Highlight",
        (AnnotationKind::Highlight, false) => "Resaltado",
        (AnnotationKind::Underline, true) => "Underline",
        (AnnotationKind::Underline, false) => "Subrayado",
        (AnnotationKind::Strikeout, true) => "Strikeout",
        (AnnotationKind::Strikeout, false) => "Tachado",
        (AnnotationKind::TextNote, true) => "Text note",
        (AnnotationKind::TextNote, false) => "Nota de texto",
        (AnnotationKind::PageMarker, true) => "Page marker",
        (AnnotationKind::PageMarker, false) => "Marca de página",
        (AnnotationKind::PageComment, true) => "Page note",
        (AnnotationKind::PageComment, false) => "Nota de página",
    }
}
fn color_label(color: AnnotationColor, english: bool) -> &'static str {
    match (color, english) {
        (AnnotationColor::Yellow, true) => "Yellow",
        (AnnotationColor::Yellow, false) => "Amarillo",
        (AnnotationColor::Green, true) => "Green",
        (AnnotationColor::Green, false) => "Verde",
        (AnnotationColor::Blue, true) => "Blue",
        (AnnotationColor::Blue, false) => "Azul",
        (AnnotationColor::Rose, true) => "Rose",
        (AnnotationColor::Rose, false) => "Rosa",
        (AnnotationColor::Violet, true) => "Violet",
        (AnnotationColor::Violet, false) => "Violeta",
    }
}
fn mark_color(color: AnnotationColor) -> Color32 {
    match color {
        AnnotationColor::Yellow => Color32::from_rgb(190, 139, 20),
        AnnotationColor::Green => Color32::from_rgb(35, 141, 87),
        AnnotationColor::Blue => Color32::from_rgb(41, 120, 188),
        AnnotationColor::Rose => Color32::from_rgb(190, 69, 100),
        AnnotationColor::Violet => Color32::from_rgb(132, 84, 190),
    }
}
impl YamantakaApp {
    pub(in crate::app) fn prepare_annotations(&mut self, document: &str) {
        self.annotations.bind(document);
        if self.annotations.page.is_none()
            && self.annotations.pending.is_none()
            && self.annotations.error.is_none()
        {
            self.annotation_command(None, 0);
        }
    }
    pub(in crate::app) fn annotation_panel(
        &mut self,
        ui: &mut egui::Ui,
        info: &PdfInfo,
        p: Palette,
    ) {
        let busy = self.annotations.pending.is_some();
        let offset = self.annotations.page.as_ref().map_or(0, |page| page.offset);
        ui.label(
            RichText::new(self.tr("Notas de lectura", "Reading notes"))
                .size(22.0)
                .color(p.ink),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(self.tr(
                "Conserva una idea y vuelve a su contexto.",
                "Keep an idea and return to its context.",
            ))
            .color(p.secondary),
        );
        ui.add_space(16.0);
        if busy {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(self.tr("Actualizando notas…", "Updating notes…"));
            });
        }
        if self.annotations.saved {
            ui.label(
                RichText::new(self.tr(
                    "Nota guardada en tu biblioteca.",
                    "Note saved in your library.",
                ))
                .color(p.secondary),
            );
        }
        if let Some(error) = self.annotations.error.clone() {
            ui.label(self.tr(
                "No se pudieron actualizar las notas.",
                "Notes could not be updated.",
            ));
            ui.collapsing(self.tr("Detalles", "Details"), |ui| {
                ui.label(error);
            });
            if !busy
                && theme::navigation(ui, self.tr("Volver a cargar", "Reload"), false, p).clicked()
            {
                self.annotation_command(None, offset);
            }
        }
        if self.annotations.editor.is_some() {
            let english = self.english;
            let fallback = self.annotations.fallback();
            let editor = self.annotations.editor.as_mut().unwrap();
            ui.add_enabled_ui(!busy,|ui| {
                let passage=editor.passage();
                if !passage.is_empty() {
                    egui::Frame::new().fill(p.raised).inner_margin(12).corner_radius(8).show(ui,|ui| {
                        egui::ScrollArea::vertical().id_salt("annotation-passage").max_height(110.0).show(ui,|ui| {ui.label(RichText::new(passage).color(p.ink));});
                    });
                    ui.add_space(14.0);
                }
                let kinds=[AnnotationKind::Highlight,AnnotationKind::Underline,AnnotationKind::Strikeout,AnnotationKind::TextNote,AnnotationKind::PageMarker,AnnotationKind::PageComment];
                let geometry=editor.stored().is_none_or(|record|!record.values.rects.is_empty());
                ui.horizontal_wrapped(|ui| {
                    let label=ui.label(if english {"Mark"} else {"Marca"});
                    let response=egui::ComboBox::from_id_salt("annotation-kind").selected_text(kind_label(editor.kind,english)).show_ui(ui,|ui| {
                        for kind in kinds {
                            if (geometry || kind==AnnotationKind::PageComment)
                                && ui.selectable_value(&mut editor.kind,kind,kind_label(kind,english)).changed()
                            {editor.confirmed=false;}
                        }
                    });
                    response.response.labelled_by(label.id);
                });
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    for color in [AnnotationColor::Yellow,AnnotationColor::Green,AnnotationColor::Blue,AnnotationColor::Rose,AnnotationColor::Violet] {
                        let response=theme::navigation(ui,color_label(color,english),editor.color==color,p);
                        if response.clicked() {editor.color=color;}
                    }
                });
                ui.add_space(14.0);
                let label=ui.label(if english {"Your note"} else {"Tu nota"});
                ui.add(egui::TextEdit::multiline(&mut editor.note).desired_rows(4).desired_width(f32::INFINITY).char_limit(4000).hint_text(if english {"What would you like to remember?"} else {"¿Qué te gustaría recordar?"})).labelled_by(label.id);
                let units=editor.note.encode_utf16().count();
                ui.label(RichText::new(format!("{units} / 4000")).size(12.0).color(p.secondary));
                if editor.stored().is_some() {
                    ui.add_space(8.0);
                    let label=ui.label(if english {"Review"} else {"Revisión"});
                    egui::ComboBox::from_id_salt("annotation-review").selected_text(match editor.review {AnnotationReviewState::Pending=>if english {"Pending"}else{"Pendiente"},AnnotationReviewState::Accepted=>if english {"Accepted"}else{"Aceptada"},AnnotationReviewState::Resolved=>if english {"Resolved"}else{"Resuelta"}}).show_ui(ui,|ui| {
                        for (state,en,es) in [(AnnotationReviewState::Pending,"Pending","Pendiente"),(AnnotationReviewState::Accepted,"Accepted","Aceptada"),(AnnotationReviewState::Resolved,"Resolved","Resuelta")] {ui.selectable_value(&mut editor.review,state,if english {en}else{es});}
                    }).response.labelled_by(label.id);
                }
                if fallback {
                    ui.add_space(12.0);
                    ui.label(if english {"This passage has incomplete geometry. Keep it as a page note with its text."} else {"Este pasaje no tiene geometría completa. Consérvalo como nota de página con su texto."});
                    ui.checkbox(&mut editor.confirmed,if english {"Save as a page note"}else{"Guardar como nota de página"});
                }
            });
            ui.add_space(12.0);
            let valid = self.annotations.editor.as_ref().is_some_and(|editor| {
                editor.note.encode_utf16().count() <= 4000 && (!fallback || editor.confirmed)
            });
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled_ui(!busy && valid, |ui| {
                        theme::control(
                            ui,
                            self.tr("Guardar nota", "Save note"),
                            Some(theme::Icon::Add),
                            false,
                            true,
                            p,
                        )
                    })
                    .inner
                    .clicked()
                {
                    match self.annotations.mutation(&self.governor) {
                        Ok(change) => self.annotation_command(Some(change), offset),
                        Err(error) => self.annotations.error = Some(error.to_string()),
                    }
                }
                if !busy && theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked()
                {
                    self.annotations.editor = None;
                }
            });
        } else {
            if ui
                .add_enabled_ui(!busy, |ui| {
                    theme::navigation(
                        ui,
                        self.tr("Anotar selección", "Annotate selection"),
                        false,
                        p,
                    )
                })
                .inner
                .clicked()
            {
                self.annotate_selection();
            }
            ui.label(
                RichText::new(self.tr(
                    "Selecciona un pasaje en el PDF para añadir una nota.",
                    "Select a passage in the PDF to add a note.",
                ))
                .color(p.secondary),
            );
        }
        ui.add_space(20.0);
        let Some(page) = self.annotations.page.clone() else {
            return;
        };
        if page.total == 0 {
            ui.label(
                RichText::new(
                    self.tr("Tus notas aparecerán aquí.", "Your notes will appear here."),
                )
                .color(p.secondary),
            );
        }
        for (index, record) in page.entries().iter().enumerate() {
            let data = record.data();
            ui.push_id(&data.id, |ui| {
                ui.separator();
                ui.add_space(8.0);
                ui.label(RichText::new(kind_label(data.values.kind, self.english)).color(p.ink));
                let passage = data
                    .values
                    .selected_text
                    .as_deref()
                    .or(data.values.note.as_deref())
                    .unwrap_or("");
                ui.add(egui::Label::new(RichText::new(passage).color(p.secondary)).truncate());
                ui.horizontal_wrapped(|ui| {
                    let caption = format!(
                        "{} {}",
                        self.tr("Página", "Page"),
                        u64::from(data.values.page_index) + 1
                    );
                    if ui
                        .add_enabled_ui(
                            (data.values.page_index as usize) < info.pages.len(),
                            |ui| theme::navigation(ui, &caption, false, p),
                        )
                        .inner
                        .clicked()
                    {
                        self.navigate(data.values.page_index);
                        if ui.ctx().content_rect().width() < 620.0 {
                            self.panel = false;
                        }
                    }
                    if !busy && theme::navigation(ui, self.tr("Editar", "Edit"), false, p).clicked()
                    {
                        self.annotations.editor = Some(AnnotationEditor {
                            source: EditorSource::Stored {
                                page: page.clone(),
                                index,
                            },
                            kind: data.values.kind,
                            color: data.values.color,
                            note: data.values.note.clone().unwrap_or_default(),
                            review: data.values.review_state,
                            confirmed: false,
                        });
                    }
                    if !busy
                        && theme::navigation(ui, self.tr("Eliminar", "Remove"), false, p).clicked()
                    {
                        self.annotation_command(
                            Some(AnnotationMutation::Remove {
                                id: data.id.clone(),
                            }),
                            offset,
                        );
                    }
                });
            });
        }
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled_ui(!busy && page.offset > 0, |ui| {
                    theme::navigation(ui, self.tr("Anteriores", "Previous"), false, p)
                })
                .inner
                .clicked()
            {
                self.annotation_command(None, page.offset.saturating_sub(80));
            }
            if ui
                .add_enabled_ui(!busy && (page.offset as u64 + 80) < page.total, |ui| {
                    theme::navigation(ui, self.tr("Siguientes", "Next"), false, p)
                })
                .inner
                .clicked()
            {
                self.annotation_command(None, page.offset.saturating_add(80));
            }
        });
    }
    pub(in crate::app) fn paint_annotations(
        &self,
        ui: &egui::Ui,
        page: u32,
        paper: egui::Rect,
        rotation: PageRotation,
    ) {
        let Some(records) = &self.annotations.page else {
            return;
        };
        for record in records.entries() {
            let values = &record.data().values;
            if values.page_index != page || values.status != AttachmentStatus::Attached {
                continue;
            }
            let color = mark_color(values.color);
            for rect in &values.rects {
                let region = reader_viewport::page_region(paper, rotation, rect.region());
                match values.kind {
                    AnnotationKind::Underline | AnnotationKind::Strikeout => {
                        let native = rect.region();
                        let y = if values.kind == AnnotationKind::Underline {
                            native.max()[1]
                        } else {
                            (native.min()[1] + native.max()[1]) * 0.5
                        };
                        let a = rotation.rotate_point([native.min()[0], y]);
                        let b = rotation.rotate_point([native.max()[0], y]);
                        let map = |point: [f64; 2]| {
                            paper.min
                                + egui::vec2(
                                    point[0] as f32 * paper.width(),
                                    point[1] as f32 * paper.height(),
                                )
                        };
                        ui.painter()
                            .line_segment([map(a), map(b)], egui::Stroke::new(1.5, color));
                    }
                    AnnotationKind::PageComment => {}
                    _ => {
                        ui.painter()
                            .rect_filled(region, 0, color.linear_multiply(0.2));
                    }
                }
            }
        }
    }
}
