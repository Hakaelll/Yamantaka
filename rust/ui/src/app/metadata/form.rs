use super::view::Actions;
use super::*;
use crate::app::settings::quiet;

impl YamantakaApp {
    pub(super) fn metadata_form(
        &self,
        ui: &mut egui::Ui,
        editor: &mut MetadataEditor,
        actions: &mut Actions,
        p: Palette,
        compact: bool,
    ) {
        let description = self.tr(
            "Las correcciones se guardan en tu biblioteca.",
            "Your corrections are saved to your library.",
        );
        egui::Frame::new()
            .fill(p.surface)
            .stroke(egui::Stroke::new(1.0, p.line))
            .corner_radius(12)
            .inner_margin(if compact { 12 } else { 24 })
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if !compact {
                    ui.label(
                        RichText::new(self.tr("Detalles de tu biblioteca", "Library details"))
                            .size(20.0)
                            .strong(),
                    );
                    ui.add_space(6.0);
                    quiet(ui, description, p);
                    ui.add_space(22.0);
                }
                self.metadata_field(ui, editor, MetadataField::Title, p);
                ui.add_space(if compact { 12.0 } else { 26.0 });
                self.metadata_field(ui, editor, MetadataField::Author, p);
                ui.add_space(if compact { 12.0 } else { 26.0 });
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled_ui(editor.ready(), |ui| {
                            theme::control(
                                ui,
                                self.tr("Guardar cambios", "Save changes"),
                                None,
                                false,
                                true,
                                p,
                            )
                        })
                        .inner
                        .clicked()
                    {
                        match ManualMetadata::new(&editor.title, &editor.author) {
                            Ok(value) => actions.save = editor.apply(MetadataMutation::Save(value)),
                            Err(error) => editor.error = Some(error.to_string()),
                        }
                    }
                    if editor.snapshot.as_ref().is_some_and(|s| {
                        s.suggested_title.is_some() || s.suggested_author.is_some()
                    }) && ui
                        .add_enabled_ui(editor.ready(), |ui| {
                            theme::navigation(
                                ui,
                                self.tr("Descartar propuestas", "Dismiss suggestions"),
                                false,
                                p,
                            )
                        })
                        .inner
                        .clicked()
                    {
                        actions.save = editor.apply(MetadataMutation::DismissSuggestions);
                    }
                });
                if compact {
                    quiet(ui, description, p);
                }
                if editor.snapshot.is_some() && !editor.clean_draft() {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(
                            self.tr("Tienes cambios sin guardar.", "You have unsaved changes."),
                        )
                        .color(p.secondary),
                    );
                } else if editor.saved {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(self.tr(
                            "Cambios guardados en la biblioteca.",
                            "Changes saved to the library.",
                        ))
                        .color(p.secondary),
                    );
                } else if editor.analyzed {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(self.tr(
                            "Análisis actualizado. Tus correcciones manuales se han conservado.",
                            "Analysis updated. Your manual corrections have been preserved.",
                        ))
                        .color(p.secondary),
                    );
                }
                if let Some(error) = &editor.error {
                    ui.add_space(16.0);
                    ui.add(egui::Label::new(RichText::new(error).color(p.ink)).wrap());
                    if editor.pending.is_none()
                        && theme::navigation(
                            ui,
                            self.tr("Volver a consultar", "Reload metadata"),
                            false,
                            p,
                        )
                        .clicked()
                    {
                        actions.save = Some(editor.read());
                    }
                }
            });
        ui.add_space(18.0);
        ui.horizontal_wrapped(|ui|{
            if editor.analyzing {
                if theme::navigation(ui,self.tr("Cancelar análisis","Cancel analysis"),false,p).clicked()&&let Some((cancel,_))=&editor.pending{cancel.cancel();self.runtime.wake();}
            }else if ui.add_enabled_ui(editor.ready()&&editor.clean_draft()&&!editor.document.missing,|ui|theme::navigation(ui,self.tr("Volver a analizar el PDF","Analyze PDF again"),false,p)).inner.on_hover_text(self.tr("Guarda o descarta el borrador antes de analizar. Tus correcciones se conservan.","Save or discard the draft before analysis. Your corrections are preserved.")).clicked(){actions.reanalyze=editor.reanalyze();}
        });
        if editor
            .snapshot
            .as_ref()
            .is_some_and(|s| s.title_override.is_some() || s.author_override.is_some())
        {
            ui.add_space(10.0);
            ui.add_enabled_ui(editor.ready(), |ui| {
                if editor.confirm_reset {
                    ui.label(self.tr(
                        "Se eliminarán tus correcciones manuales de título y autor.",
                        "Your manual title and author corrections will be removed.",
                    ));
                    ui.horizontal_wrapped(|ui| {
                        if theme::navigation(
                            ui,
                            self.tr(
                                "Restablecer valores automáticos",
                                "Restore automatic values",
                            ),
                            false,
                            p,
                        )
                        .clicked()
                        {
                            actions.save = editor.apply(MetadataMutation::ResetOverrides);
                        }
                        if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked()
                        {
                            editor.confirm_reset = false;
                        }
                    });
                } else if theme::navigation(
                    ui,
                    self.tr(
                        "Restablecer valores automáticos…",
                        "Restore automatic values…",
                    ),
                    false,
                    p,
                )
                .clicked()
                {
                    editor.confirm_reset = true;
                }
            });
        }
    }
    fn metadata_field(
        &self,
        ui: &mut egui::Ui,
        editor: &mut MetadataEditor,
        field: MetadataField,
        p: Palette,
    ) {
        let caption = match field {
            MetadataField::Title => self.tr("Título", "Title"),
            MetadataField::Author => self.tr("Autor", "Author"),
        };
        let label = ui.label(RichText::new(caption).strong());
        ui.add_space(6.0);
        let enabled = editor.pending.is_none() && editor.source_pending.is_none();
        let value = match field {
            MetadataField::Title => &mut editor.title,
            MetadataField::Author => &mut editor.author,
        };
        if ui
            .add_enabled_ui(enabled, |ui| {
                ui.visuals_mut().widgets.inactive.bg_stroke =
                    egui::Stroke::new(1.0, p.secondary.gamma_multiply(0.7));
                let response = ui
                    .add(
                        egui::TextEdit::singleline(value)
                            .id_salt((
                                "metadata-field",
                                &editor.document.id,
                                field == MetadataField::Title,
                            ))
                            .desired_width(f32::INFINITY)
                            .margin(egui::vec2(12.0, 10.0))
                            .char_limit(512),
                    )
                    .labelled_by(label.id);
                theme::record_control_bounds(ui, caption, response.rect);
                response
            })
            .inner
            .changed()
        {
            editor.saved = false;
        }
        let snapshot = editor.snapshot.clone();
        let proposal = snapshot.as_ref().and_then(|s| match field {
            MetadataField::Title => s.suggested_title.as_ref(),
            MetadataField::Author => s.suggested_author.as_ref(),
        });
        if let Some(proposal) = proposal {
            ui.add_space(12.0);
            ui.add(
                egui::Label::new(
                    RichText::new(format!(
                        "{} {proposal}",
                        self.tr("Propuesta:", "Suggestion:")
                    ))
                    .color(p.accent),
                )
                .wrap(),
            );
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled_ui(editor.ready(), |ui| {
                        theme::navigation(
                            ui,
                            match field {
                                MetadataField::Title => {
                                    self.tr("Usar este título", "Use this title")
                                }
                                MetadataField::Author => {
                                    self.tr("Usar este autor", "Use this author")
                                }
                            },
                            false,
                            p,
                        )
                    })
                    .inner
                    .clicked()
                {
                    match field {
                        MetadataField::Title => editor.title = proposal.clone(),
                        MetadataField::Author => editor.author = proposal.clone(),
                    };
                    editor.saved = false;
                }
                if let Some(evidence) = snapshot.as_ref().and_then(|s| s.evidence(field)) {
                    let page = format!(
                        "{} {}",
                        self.tr("Comparar página", "Compare page"),
                        evidence.page + 1
                    );
                    if theme::navigation(ui, &page, editor.reference.field == field, p).clicked() {
                        editor.reference.field = field;
                        editor.reference.reveal = true;
                    }
                }
            });
        }
    }
}
