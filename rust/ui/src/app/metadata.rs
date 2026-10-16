use super::*;
use yamantaka_application::metadata_review::{MetadataChange, MetadataReply, MetadataRequest};
use yamantaka_domain::metadata_review::{ManualMetadata, MetadataMutation, MetadataReview};
#[cfg(test)]
#[path = "metadata/tests.rs"]
mod tests;

pub(super) struct MetadataEditor {
    document: Document,
    snapshot: Option<MetadataReview>,
    pending: Option<(Cancellation, bool)>,
    title: String,
    author: String,
    error: Option<String>,
    confirm_reset: bool,
    saved: bool,
}
impl Drop for MetadataEditor {
    fn drop(&mut self) {
        if let Some((cancel, false)) = &self.pending {
            cancel.cancel();
        }
    }
}
impl MetadataEditor {
    pub(super) fn capture_ready(&self) -> bool {
        self.snapshot.is_some() && self.pending.is_none()
    }
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"document":self.document.id,"title":self.title,"author":self.author,"snapshot":self.snapshot.as_ref().map(|s|serde_json::json!({"title":s.title,"author":s.author,"titleOverride":s.title_override,"authorOverride":s.author_override,"suggestedTitle":s.suggested_title,"suggestedAuthor":s.suggested_author})),"saved":self.saved,"error":self.error,"pending":self.pending.is_some()})
    }
    fn read(&mut self) -> MetadataRequest {
        if let Some((cancel, false)) = self.pending.take() {
            cancel.cancel();
        }
        let cancel = Cancellation::default();
        self.pending = Some((cancel.clone(), false));
        self.error = None;
        self.saved = false;
        MetadataRequest {
            document: self.document.id.clone(),
            change: MetadataChange::Read,
            cancel,
        }
    }
    fn apply(&mut self, mutation: MetadataMutation) -> Option<MetadataRequest> {
        if self.pending.is_some() {
            return None;
        }
        let expected = self.snapshot.clone()?;
        let cancel = Cancellation::default();
        self.pending = Some((cancel.clone(), true));
        self.error = None;
        self.saved = false;
        Some(MetadataRequest {
            document: self.document.id.clone(),
            change: MetadataChange::Apply {
                expected: Box::new(expected),
                mutation,
            },
            cancel,
        })
    }
    fn accept(&mut self, reply: MetadataReply) -> Option<MetadataReview> {
        if self.document.id != reply.document
            || self.pending.as_ref().is_none_or(|(cancel, writing)| {
                !cancel.same_request(&reply.cancel) || *writing != reply.writing
            })
        {
            return None;
        }
        self.pending = None;
        match reply.result {
            Ok(snapshot) => {
                self.title = snapshot.title.clone();
                self.author = snapshot.author.clone();
                self.snapshot = Some(snapshot.clone());
                self.confirm_reset = false;
                self.error = None;
                self.saved = reply.writing;
                reply.writing.then_some(snapshot)
            }
            Err(error) => {
                self.error = Some(error.to_string());
                None
            }
        }
    }
}
impl YamantakaApp {
    pub fn preview_metadata(&mut self, action: &'static str) {
        if self.demo && self.capture.is_some() {
            self.metadata_preview = Some((action, 0));
        }
    }
    pub(super) fn advance_metadata_preview(&mut self) {
        let Some((action, stage)) = self.metadata_preview else {
            return;
        };
        if stage == 0 {
            if let Some(document) = self.documents.iter().find(|d| d.id == "demo-0").cloned() {
                self.begin_metadata_editor(document);
                self.metadata_preview = Some((action, 1));
            }
        } else if stage == 1
            && self
                .metadata
                .as_ref()
                .is_some_and(MetadataEditor::capture_ready)
        {
            if action == "open" {
                self.metadata_preview = Some((action, 3));
                return;
            }
            let mutation = match action {
                "dismiss" => MetadataMutation::DismissSuggestions,
                "reset" => MetadataMutation::ResetOverrides,
                _ => MetadataMutation::Save(
                    ManualMetadata::new("Geometry of Light", "Ada Lovelace").unwrap(),
                ),
            };
            if matches!(mutation, MetadataMutation::Save(_))
                && let Some(editor) = self.metadata.as_mut()
            {
                editor.title = "Geometry of Light".into();
                editor.author = "Ada Lovelace".into();
            }
            if let Some(request) = self
                .metadata
                .as_mut()
                .and_then(|editor| editor.apply(mutation))
            {
                self.metadata_preview = Some((action, 2));
                self.send_metadata(request);
            }
        } else if stage == 2
            && self
                .metadata
                .as_ref()
                .is_some_and(MetadataEditor::capture_ready)
        {
            self.metadata_preview = Some((action, 3));
        }
    }
    pub(super) fn begin_metadata_editor(&mut self, document: Document) {
        let mut editor = MetadataEditor {
            title: document.display_title().into(),
            author: document.author.clone().unwrap_or_default(),
            document,
            snapshot: None,
            pending: None,
            error: None,
            confirm_reset: false,
            saved: false,
        };
        let request = editor.read();
        self.metadata = Some(editor);
        self.send_metadata(request);
    }
    fn send_metadata(&mut self, request: MetadataRequest) {
        let document = request.document.clone();
        let writing = matches!(request.change, MetadataChange::Apply { .. });
        let cancel = request.cancel.clone();
        if let Err(error) = self.runtime.submit(Action::Metadata(Box::new(request))) {
            self.accept_metadata(MetadataReply {
                document,
                writing,
                cancel,
                result: Err(error),
            });
        }
    }
    pub(super) fn accept_metadata(&mut self, reply: MetadataReply) {
        if let Some(saved) = self
            .metadata
            .as_mut()
            .and_then(|editor| editor.accept(reply))
        {
            for document in self
                .documents
                .iter_mut()
                .chain(self.active.iter_mut().map(|(document, _)| document))
            {
                if document.id == saved.document {
                    document.title = Some(saved.title.clone());
                    document.author = (!saved.author.is_empty()).then(|| saved.author.clone());
                }
            }
            self.refresh();
        }
    }
    pub(super) fn metadata_view(&mut self, ui: &mut egui::Ui) {
        let Some(mut editor) = self.metadata.take() else {
            return;
        };
        let p = Palette::new(self.dark);
        let compact = ui.ctx().content_rect().height() < 500.0;
        let writing = editor.pending.as_ref().is_some_and(|(_, writing)| *writing);
        let ready = editor.snapshot.is_some() && editor.pending.is_none();
        let mut close = false;
        let mut request = None;
        egui::ScrollArea::vertical().id_salt("metadata-review").auto_shrink([false,false]).show(ui,|ui| {
            egui::Frame::new().inner_margin(egui::Margin::symmetric(if ui.available_width()>850.0 {42}else if compact {12}else{18},if compact {10}else{20})).show(ui,|ui| {
                ui.set_width(ui.available_width().min(880.0));
                ui.horizontal_top(|ui| {
                    close=ui.add_enabled_ui(!writing,|ui|theme::icon_button(ui,self.tr("Volver a la biblioteca","Back to library"),theme::Icon::Left,p)).inner.clicked();
                    ui.vertical(|ui| {
                        ui.label(RichText::new(self.tr("Título y autor","Title and author")).font(egui::FontId::new(if compact {20.0}else{26.0},egui::FontFamily::Name("Editorial".into()))));
                        ui.add(egui::Label::new(RichText::new(&editor.document.file_name).size(13.0).color(p.secondary)).truncate());
                    });
                });
                ui.add_space(if compact {8.0}else{18.0});
                ui.add(egui::Label::new(RichText::new(if compact {self.tr("Edita tu biblioteca sin modificar el PDF.","Edit your library without changing the PDF.")}else{self.tr("Tus correcciones tienen prioridad sobre el análisis automático. El PDF original se conserva intacto.","Your corrections take priority over automatic analysis. The original PDF stays unchanged.")}).color(p.secondary)).wrap());
                ui.add_space(if compact {8.0}else{18.0});
                ui.add_enabled_ui(ready,|ui| {
                    let title_label=ui.label(self.tr("Título","Title"));
                    if ui.add(egui::TextEdit::singleline(&mut editor.title).id_salt("metadata-title").desired_width(f32::INFINITY).char_limit(512)).labelled_by(title_label.id).changed() {editor.saved=false;}
                    if let Some(proposal)=editor.snapshot.as_ref().and_then(|s|s.suggested_title.as_ref()) {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(format!("{}: {proposal}",self.tr("Propuesta","Suggestion"))).color(p.accent));
                            if theme::navigation(ui,self.tr("Usar este título","Use this title"),false,p).clicked() {editor.title=proposal.clone();editor.saved=false;}
                        });
                    }
                    ui.add_space(16.0);
                    let author_label=ui.label(self.tr("Autor","Author"));
                    if ui.add(egui::TextEdit::singleline(&mut editor.author).id_salt("metadata-author").desired_width(f32::INFINITY).char_limit(512)).labelled_by(author_label.id).changed() {editor.saved=false;}
                    if let Some(proposal)=editor.snapshot.as_ref().and_then(|s|s.suggested_author.as_ref()) {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(format!("{}: {proposal}",self.tr("Propuesta","Suggestion"))).color(p.accent));
                            if theme::navigation(ui,self.tr("Usar este autor","Use this author"),false,p).clicked() {editor.author=proposal.clone();editor.saved=false;}
                        });
                    }
                });
                ui.add_space(22.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.add_enabled_ui(ready,|ui|theme::control(ui,self.tr("Guardar cambios","Save changes"),None,false,true,p)).inner.clicked() {
                        match ManualMetadata::new(&editor.title,&editor.author) {
                            Ok(value)=>request=editor.apply(MetadataMutation::Save(value)),
                            Err(error)=>editor.error=Some(error.to_string()),
                        }
                    }
                    if editor.snapshot.as_ref().is_some_and(|s|s.suggested_title.is_some() || s.suggested_author.is_some()) && ui.add_enabled_ui(ready,|ui|theme::navigation(ui,self.tr("Descartar propuestas","Dismiss suggestions"),false,p)).inner.clicked() {request=editor.apply(MetadataMutation::DismissSuggestions);}
                });
                if editor.saved {ui.label(RichText::new(self.tr("Cambios guardados en la biblioteca.","Changes saved to the library.")).color(p.secondary));}
                if let Some(error)=editor.error.as_ref() {
                    ui.add_space(12.0);
                    ui.label(RichText::new(error).color(p.ink));
                    if editor.pending.is_none() && theme::navigation(ui,self.tr("Volver a consultar","Reload metadata"),false,p).clicked() {request=Some(editor.read());}
                }
                if let Some(snapshot)=editor.snapshot.clone() {
                    ui.add_space(28.0);
                    ui.separator();
                    ui.add_space(12.0);
                    ui.label(RichText::new(self.tr("Datos incluidos en el PDF","Embedded PDF metadata")).size(14.0));
                    for (label,value) in [(self.tr("Título","Title"),snapshot.source_title.as_deref()),(self.tr("Autor","Author"),snapshot.source_author.as_deref())] {
                        ui.label(RichText::new(format!("{label}: {}",value.filter(|s|!s.is_empty()).unwrap_or(self.tr("Sin especificar","Not specified")))).color(p.secondary));
                    }
                    if snapshot.title_override.is_some() || snapshot.author_override.is_some() {
                        ui.add_space(12.0);
                        ui.add_enabled_ui(ready,|ui| {
                            if editor.confirm_reset {
                                ui.label(self.tr("Se eliminarán tus correcciones manuales de título y autor.","Your manual title and author corrections will be removed."));
                                ui.horizontal_wrapped(|ui| {
                                    if theme::navigation(ui,self.tr("Restablecer valores automáticos","Restore automatic values"),false,p).clicked() {request=editor.apply(MetadataMutation::ResetOverrides);}
                                    if theme::navigation(ui,self.tr("Cancelar","Cancel"),false,p).clicked() {editor.confirm_reset=false;}
                                });
                            } else if theme::navigation(ui,self.tr("Restablecer valores automáticos…","Restore automatic values…"),false,p).clicked() {editor.confirm_reset=true;}
                        });
                    }
                }
            });
        });
        if !close {
            self.metadata = Some(editor);
        }
        if let Some(request) = request {
            self.send_metadata(request);
        }
    }
}
