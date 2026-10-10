use super::*;
use yamantaka_application::annotation_export::{
    AnnotationExportFormat,
    worker::{AnnotationExportReply, AnnotationExportRequest, AnnotationExportScope},
};

struct PendingExport {
    scope: AnnotationExportScope,
    cancel: Cancellation,
}
#[derive(Default)]
pub(in crate::app) struct ExportState {
    pending: Option<PendingExport>,
    outcome: Option<(AnnotationExportScope, yamantaka_domain::Result<u64>)>,
    preview: Option<AnnotationExportFormat>,
    artifact: Option<String>,
}
impl ExportState {
    pub(in crate::app) fn cancel(&self) {
        if let Some(pending) = &self.pending {
            pending.cancel.cancel();
        }
    }
    pub(in crate::app) fn busy(&self) -> bool {
        self.pending.is_some()
    }
    pub(in crate::app) fn ready(&self) -> bool {
        self.preview.is_none() && !self.busy()
    }
    pub(in crate::app) fn capture_summary(&self) -> serde_json::Value {
        let (saved, count) = match &self.outcome {
            Some((_, Ok(count))) => (true, *count),
            _ => (false, 0),
        };
        serde_json::json!({"pending":self.busy(),"saved":saved,"count":count,"artifact":self.artifact})
    }
    pub(in crate::app) fn accept(&mut self, reply: AnnotationExportReply) {
        if !self.pending.as_ref().is_some_and(|pending| {
            reply.scope == pending.scope && pending.cancel.same_request(&reply.cancel)
        }) {
            return;
        }
        self.pending = None;
        self.outcome = Some((reply.scope, reply.result));
    }
    fn submitted(&mut self, scope: AnnotationExportScope, cancel: Cancellation) {
        self.pending = Some(PendingExport { scope, cancel });
        self.outcome = None;
    }
}
impl YamantakaApp {
    pub(in crate::app) fn preview_review_export(&mut self, markdown: bool) {
        if self.demo && self.capture.is_some() {
            self.annotation_export.preview = Some(if markdown {
                AnnotationExportFormat::Markdown
            } else {
                AnnotationExportFormat::Json
            });
        }
    }
    fn annotation_export_scope_label(&self, ui: &mut egui::Ui, scope: &AnnotationExportScope) {
        match scope {
            AnnotationExportScope::Review => {
                ui.label(self.tr("Exportación de la revisión global", "Global review export"));
            }
            AnnotationExportScope::Document(id)
                if matches!(self.surface, Surface::Notes)
                    || self
                        .active
                        .as_ref()
                        .is_some_and(|(active, _)| active.id != *id) =>
            {
                ui.label(self.tr(
                    "Exportación del documento anterior",
                    "Export for the previous document",
                ));
            }
            _ => {}
        }
    }
    pub fn preview_annotation_export(&mut self, markdown: bool) {
        if self.demo && self.capture.is_some() {
            self.preview_annotation_list();
            self.annotation_export.preview = Some(if markdown {
                AnnotationExportFormat::Markdown
            } else {
                AnnotationExportFormat::Json
            });
        }
    }
    pub(in crate::app) fn apply_annotation_export_preview(&mut self) {
        let ready = if matches!(self.surface, Surface::Notes) {
            self.global_notes.export_ready()
        } else {
            self.annotations.ready()
        };
        if self.annotation_preview != 0 || !ready || self.annotations.editing() {
            return;
        }
        let Some(format) = self.annotation_export.preview.take() else {
            return;
        };
        let Some(capture) = &self.capture else {
            return;
        };
        let path = capture.with_extension(format!("{}.{}", std::process::id(), format.extension()));
        self.annotation_export.artifact = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        self.start_annotation_export(path, format);
    }
    fn start_annotation_export(&mut self, path: PathBuf, format: AnnotationExportFormat) {
        if self.annotation_export.pending.is_some() {
            return;
        }
        let request = if matches!(self.surface, Surface::Notes) {
            let Some(query) = self.global_notes.export_query() else {
                return;
            };
            AnnotationExportRequest::review(query, path, format, self.governor.clone())
        } else {
            let Some((document, info)) = self.active.as_ref() else {
                return;
            };
            let progress = if info.pages.is_empty() {
                0.0
            } else {
                (u64::from(self.page) + 1) as f64 / info.pages.len() as f64
            };
            AnnotationExportRequest::new(document, progress, path, format, self.governor.clone())
        };
        let id = request.scope();
        let cancel = request.cancel.clone();
        match self
            .runtime
            .submit(Action::AnnotationExport(Box::new(request)))
        {
            Ok(()) => self.annotation_export.submitted(id, cancel),
            Err(error) => self.annotation_export.outcome = Some((id, Err(error))),
        }
    }
    pub(in crate::app) fn annotation_export_panel(&mut self, ui: &mut egui::Ui, p: Palette) {
        let review = matches!(self.surface, Surface::Notes);
        ui.separator();
        ui.add_space(8.0);
        if let Some(pending) = &self.annotation_export.pending {
            self.annotation_export_scope_label(ui, &pending.scope);
            let cancel = pending.cancel.clone();
            let cancelling = cancel.is_cancelled();
            ui.horizontal_wrapped(|ui| {
                ui.spinner();
                ui.label(self.tr(
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
                        theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p)
                    })
                    .inner
                    .clicked()
                {
                    cancel.cancel();
                }
            });
        } else {
            let can_export = if review {
                self.global_notes.export_ready() && !self.annotations.editing()
            } else {
                self.annotations.pending.is_none() && self.active.is_some()
            };
            let mut selected = None;
            ui.add_enabled_ui(can_export, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(self.tr("Exportar", "Export"));
                    for (format, label) in [
                        (AnnotationExportFormat::Markdown, "Markdown"),
                        (AnnotationExportFormat::Json, "JSON"),
                    ] {
                        if theme::navigation(ui, label, false, p)
                            .on_hover_text(self.tr(
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
                RichText::new(self.tr(
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
            if let Some(format) = selected {
                match yamantaka_windows::annotation_export::pick_annotation_export(
                    self.tr(
                        "Guardar notas en un archivo nuevo",
                        "Save notes to a new file",
                    ),
                    format.extension(),
                ) {
                    Ok(Some(path)) => self.start_annotation_export(path, format),
                    Ok(None) => {}
                    Err(error) => {
                        let scope = if review {
                            Some(AnnotationExportScope::Review)
                        } else {
                            self.active.as_ref().map(|(document, _)| {
                                AnnotationExportScope::Document(document.id.clone())
                            })
                        };
                        if let Some(scope) = scope {
                            self.annotation_export.outcome = Some((scope, Err(error)));
                        }
                    }
                }
            }
        }
        if let Some((document, result)) = &self.annotation_export.outcome {
            ui.add_space(8.0);
            self.annotation_export_scope_label(ui, document);
            match result {
                Ok(count) => {
                    ui.label(
                        RichText::new(if self.english {
                            format!("Saved {count} notes.")
                        } else {
                            format!("Se guardaron {count} notas.")
                        })
                        .color(p.ink),
                    );
                }
                Err(Failure::Cancelled) => {
                    ui.label(self.tr(
                        "Exportación cancelada. No se guardó ningún archivo.",
                        "Export cancelled. No file was saved.",
                    ));
                }
                Err(Failure::DestinationOccupied) => {
                    ui.label(self.tr(
                        "Ese archivo ya existe. Elige otro nombre para conservarlo.",
                        "That file already exists. Choose another name to keep it intact.",
                    ));
                }
                Err(error) => {
                    ui.label(self.tr(
                        "No se pudieron exportar las notas. Vuelve a intentarlo con otro destino.",
                        "Could not export your notes. Try another destination.",
                    ));
                    ui.collapsing(
                        self.tr("Detalles de la exportación", "Export details"),
                        |ui| {
                            ui.label(error.to_string());
                        },
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn review_exports_accept_only_their_scope_and_token_and_keep_published_success() {
        let mut state = ExportState::default();
        let token = Cancellation::default();
        state.submitted(AnnotationExportScope::Review, token.clone());
        for (scope, cancel) in [
            (AnnotationExportScope::Document("a".into()), token.clone()),
            (AnnotationExportScope::Review, Cancellation::default()),
        ] {
            state.accept(AnnotationExportReply {
                scope,
                result: Ok(181),
                cancel,
            });
            assert!(state.busy());
            assert!(state.outcome.is_none());
        }
        token.cancel();
        state.accept(AnnotationExportReply {
            scope: AnnotationExportScope::Review,
            result: Ok(80),
            cancel: token,
        });
        assert!(!state.busy());
        assert!(matches!(
            state.outcome,
            Some((AnnotationExportScope::Review, Ok(80)))
        ));
    }
    #[test]
    fn stale_replies_do_not_finish_a_different_request_or_document() {
        let mut state = ExportState::default();
        let token = Cancellation::default();
        state.submitted(AnnotationExportScope::Document("a".into()), token.clone());
        state.accept(AnnotationExportReply {
            scope: AnnotationExportScope::Review,
            result: Ok(80),
            cancel: token.clone(),
        });
        assert!(state.pending.is_some());
        assert!(state.outcome.is_none());
        for (document, cancel) in [("a", Cancellation::default()), ("b", token.clone())] {
            state.accept(AnnotationExportReply {
                scope: AnnotationExportScope::Document(document.into()),
                result: Ok(181),
                cancel,
            });
            assert!(state.pending.is_some());
            assert!(state.outcome.is_none());
        }
        state.accept(AnnotationExportReply {
            scope: AnnotationExportScope::Document("a".into()),
            result: Ok(181),
            cancel: token,
        });
        assert!(state.pending.is_none());
        assert!(matches!(state.outcome, Some((_, Ok(181)))));
    }
    #[test]
    fn a_published_success_is_kept_even_if_cancellation_arrives_before_its_reply() {
        let mut state = ExportState::default();
        let token = Cancellation::default();
        state.submitted(AnnotationExportScope::Document("a".into()), token.clone());
        token.cancel();
        state.accept(AnnotationExportReply {
            scope: AnnotationExportScope::Document("a".into()),
            result: Ok(2),
            cancel: token,
        });
        assert!(matches!(state.outcome, Some((_, Ok(2)))));
        let retry = Cancellation::default();
        state.submitted(AnnotationExportScope::Document("b".into()), retry.clone());
        assert!(state.outcome.is_none());
        state.accept(AnnotationExportReply {
            scope: AnnotationExportScope::Document("b".into()),
            result: Err(Failure::Cancelled),
            cancel: retry,
        });
        assert!(matches!(state.outcome, Some((_, Err(Failure::Cancelled)))));
    }
}
