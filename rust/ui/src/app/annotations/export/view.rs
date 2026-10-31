use super::*;

impl ExportState {
    fn scope_label(
        ui: &mut egui::Ui,
        english: bool,
        expected: &AnnotationExportScope,
        actual: &AnnotationExportScope,
    ) {
        match actual {
            AnnotationExportScope::Review => {
                ui.label(if english {
                    "Global review export"
                } else {
                    "Exportación de la revisión global"
                });
            }
            AnnotationExportScope::Document(_) if actual != expected => {
                ui.label(if english {
                    "Export for another document"
                } else {
                    "Exportación de otro documento"
                });
            }
            _ => {}
        }
    }
    pub(in crate::app) fn show(
        &mut self,
        ui: &mut egui::Ui,
        english: bool,
        scope: &AnnotationExportScope,
        can_export: bool,
        p: Palette,
    ) -> Option<AnnotationExportFormat> {
        let tr = |es, en| if english { en } else { es };
        let review = matches!(scope, AnnotationExportScope::Review);
        let mut selected = None;
        ui.separator();
        ui.add_space(8.0);
        if let Some(pending) = &self.pending {
            Self::scope_label(ui, english, scope, &pending.scope);
            let cancel = pending.cancel.clone();
            let cancelling = cancel.is_cancelled();
            ui.horizontal_wrapped(|ui| {
                ui.spinner();
                ui.label(tr(
                    if cancelling {
                        "Cancelando exportación…"
                    } else {
                        "Guardando tus notas…"
                    },
                    if cancelling {
                        "Cancelling export…"
                    } else {
                        "Saving your notes…"
                    },
                ));
                if ui
                    .add_enabled_ui(!cancelling, |ui| {
                        theme::navigation(ui, tr("Cancelar", "Cancel"), false, p)
                    })
                    .inner
                    .clicked()
                {
                    cancel.cancel();
                }
            });
        } else {
            ui.add_enabled_ui(can_export, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(tr("Exportar", "Export"));
                    for (format, label) in [
                        (AnnotationExportFormat::Markdown, "Markdown"),
                        (AnnotationExportFormat::Json, "JSON"),
                    ] {
                        if theme::navigation(ui, label, false, p)
                            .on_hover_text(tr(
                                if review {
                                    "Exportar esta página filtrada, hasta 80 notas"
                                } else {
                                    "Exportar todas las notas del documento"
                                },
                                if review {
                                    "Export this filtered page, up to 80 notes"
                                } else {
                                    "Export every note in this document"
                                },
                            ))
                            .clicked()
                        {
                            selected = Some(format);
                        }
                    }
                });
            });
            ui.label(
                RichText::new(tr(
                    if review {
                        "Incluye esta página filtrada, hasta 80 notas. Elige un archivo nuevo."
                    } else {
                        "Incluye todas las páginas. Elige un archivo nuevo."
                    },
                    if review {
                        "Includes this filtered page, up to 80 notes. Choose a new file."
                    } else {
                        "Includes every page. Choose a new file."
                    },
                ))
                .size(12.0)
                .color(p.secondary),
            );
        }
        if let Some((document, result)) = &self.outcome {
            ui.add_space(8.0);
            Self::scope_label(ui, english, scope, document);
            match result {
                Ok(count) => {
                    ui.label(
                        RichText::new(if english {
                            format!("Saved {count} notes.")
                        } else {
                            format!("Se guardaron {count} notas.")
                        })
                        .color(p.ink),
                    );
                }
                Err(Failure::Cancelled) => {
                    ui.label(tr(
                        "Exportación cancelada. No se guardó ningún archivo.",
                        "Export cancelled. No file was saved.",
                    ));
                }
                Err(Failure::DestinationOccupied) => {
                    ui.label(tr(
                        "Ese archivo ya existe. Elige otro nombre para conservarlo.",
                        "That file already exists. Choose another name to keep it intact.",
                    ));
                }
                Err(error) => {
                    ui.label(tr(
                        "No se pudieron exportar las notas. Vuelve a intentarlo con otro destino.",
                        "Could not export your notes. Try another destination.",
                    ));
                    ui.collapsing(tr("Detalles de la exportación", "Export details"), |ui| {
                        ui.label(error.to_string());
                    });
                }
            }
        }
        selected
    }
}
