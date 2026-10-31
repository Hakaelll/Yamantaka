use super::*;
use yamantaka_application::annotation_export::{
    AnnotationExportFormat,
    worker::{AnnotationExportReply, AnnotationExportRequest, AnnotationExportScope},
};

mod view;
#[derive(Clone, Copy)]
pub(in crate::app) enum ExportTarget {
    Primary,
    Reference,
    Review,
}

struct PendingExport {
    scope: AnnotationExportScope,
    cancel: Cancellation,
}
#[derive(Default)]
pub(in crate::app) struct ExportState {
    pending: Option<PendingExport>,
    outcome: Option<(AnnotationExportScope, yamantaka_domain::Result<u64>)>,
    preview: Option<AnnotationExportFormat>,
    preview_target: Option<ExportTarget>,
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
        let scope = self
            .pending
            .as_ref()
            .map(|pending| &pending.scope)
            .or_else(|| self.outcome.as_ref().map(|(scope, _)| scope));
        let document = match scope {
            Some(AnnotationExportScope::Document(id)) => Some(id),
            _ => None,
        };
        serde_json::json!({"pending":self.busy(),"saved":saved,"count":count,"artifact":self.artifact,"document":document})
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
    pub fn preview_reference_annotation_export(&mut self, markdown: bool) {
        if self.demo && self.capture.is_some() {
            self.annotation_export.preview = Some(if markdown {
                AnnotationExportFormat::Markdown
            } else {
                AnnotationExportFormat::Json
            });
            self.annotation_export.preview_target = Some(ExportTarget::Reference);
        }
    }
    pub(in crate::app) fn apply_annotation_export_preview(&mut self) {
        let target = self
            .annotation_export
            .preview_target
            .unwrap_or_else(|| self.current_export_target());
        let ready = match target {
            ExportTarget::Review => self.global_notes.export_ready(),
            ExportTarget::Primary => self.reader.annotations.ready(),
            ExportTarget::Reference => {
                self.reference_notes_preview.ready()
                    && self
                        .reference
                        .as_ref()
                        .is_some_and(|view| view.pane.annotations.ready())
            }
        };
        if self.annotation_preview != 0 || !ready || self.reader.annotations.editing() {
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
        self.start_notes_export(target, path, format);
    }
    fn start_notes_export(
        &mut self,
        target: ExportTarget,
        path: PathBuf,
        format: AnnotationExportFormat,
    ) {
        if self.annotation_export.pending.is_some() || !self.can_export_notes(target) {
            return;
        }
        let request = if matches!(target, ExportTarget::Review) {
            let Some(query) = self.global_notes.export_query() else {
                return;
            };
            AnnotationExportRequest::review(query, path, format, self.governor.clone())
        } else {
            let pane = match target {
                ExportTarget::Reference => match &self.reference {
                    Some(view) => &view.pane,
                    None => return,
                },
                _ => &self.reader,
            };
            let Some((document, info)) = pane.active.as_ref() else {
                return;
            };
            let progress = if info.pages.is_empty() {
                0.0
            } else {
                (u64::from(pane.page) + 1) as f64 / info.pages.len() as f64
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
    fn current_export_target(&self) -> ExportTarget {
        if matches!(self.surface, Surface::Notes) {
            ExportTarget::Review
        } else {
            ExportTarget::Primary
        }
    }
    fn export_scope(&self, target: ExportTarget) -> Option<AnnotationExportScope> {
        match target {
            ExportTarget::Review => Some(AnnotationExportScope::Review),
            ExportTarget::Primary => self
                .reader
                .active
                .as_ref()
                .map(|(doc, _)| AnnotationExportScope::Document(doc.id.clone())),
            ExportTarget::Reference => self
                .reference
                .as_ref()?
                .pane
                .active
                .as_ref()
                .map(|(doc, _)| AnnotationExportScope::Document(doc.id.clone())),
        }
    }
    fn can_export_notes(&self, target: ExportTarget) -> bool {
        match target {
            ExportTarget::Review => {
                self.global_notes.export_ready()
                    && !self.reader.annotations.editing()
                    && !self
                        .reference
                        .as_ref()
                        .is_some_and(|view| view.pane.annotations.editing())
            }
            ExportTarget::Primary => {
                self.reader.active.is_some()
                    && self.reader.annotations.pending.is_none()
                    && !self.reader.annotations.editing()
            }
            ExportTarget::Reference => self.reference.as_ref().is_some_and(|view| {
                view.pane.active.is_some()
                    && view.pane.annotations.pending.is_none()
                    && !view.pane.annotations.editing()
            }),
        }
    }
    pub(in crate::app) fn pick_annotation_export(
        &mut self,
        target: ExportTarget,
        format: AnnotationExportFormat,
    ) {
        if self.annotation_export.busy() || !self.can_export_notes(target) {
            return;
        }
        let Some(scope) = self.export_scope(target) else {
            return;
        };
        match yamantaka_windows::annotation_export::pick_annotation_export(
            self.tr(
                "Guardar notas en un archivo nuevo",
                "Save notes to a new file",
            ),
            format.extension(),
        ) {
            Ok(Some(path)) => self.start_notes_export(target, path, format),
            Ok(None) => {}
            Err(error) => self.annotation_export.outcome = Some((scope, Err(error))),
        }
    }
    pub(in crate::app) fn annotation_export_panel(&mut self, ui: &mut egui::Ui, p: Palette) {
        let target = self.current_export_target();
        let Some(scope) = self.export_scope(target) else {
            return;
        };
        let can_export = self.can_export_notes(target);
        if let Some(format) = self
            .annotation_export
            .show(ui, self.english, &scope, can_export, p)
        {
            self.pick_annotation_export(target, format);
        }
    }
}
