use super::*;
use yamantaka_application::quarantine::{Operation, Reply, Request};
use yamantaka_domain::quarantine::{
    QuarantinePage, QuarantineReason, QuarantineRevision, ReaderHealth,
};
mod preview;

#[derive(Default)]
pub(super) struct QuarantineState {
    pub(super) open: bool,
    page: Option<QuarantinePage>,
    pending: Option<(Cancellation, bool)>,
    release: Option<QuarantineRevision>,
    error: bool,
    health_pending: usize,
    preview: u8,
    preview_mode: u8,
    close_after: bool,
}
impl QuarantineState {
    pub(super) fn closing(&self) -> bool {
        self.close_after
    }
    fn writing(&self) -> bool {
        self.health_pending > 0 || self.pending.as_ref().is_some_and(|(_, writing)| *writing)
    }
    pub fn ready(&self) -> bool {
        self.pending.is_none() && self.health_pending == 0 && matches!(self.preview, 0 | 90)
    }
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({"preview":self.preview,"healthPending":self.health_pending,"open":self.open,"pending":self.pending.is_some(),"error":self.error,"review":self.release.is_some(),"total":self.page.as_ref().map(|page|page.total),"offset":self.page.as_ref().map(|page|page.offset),"entries":self.page.as_ref().map(|page|page.entries.iter().map(|entry|serde_json::json!({"id":entry.document.id,"title":entry.document.display_title(),"native":entry.reason==QuarantineReason::PdfOpenFailed})).collect::<Vec<_>>())})
    }
    fn close(&mut self) {
        self.open = false;
        self.release = None;
        if self.pending.as_ref().is_some_and(|(_, writing)| !writing)
            && let Some((token, _)) = self.pending.take()
        {
            token.cancel();
        }
    }
}
impl Drop for QuarantineState {
    fn drop(&mut self) {
        self.close();
    }
}
impl YamantakaApp {
    pub(super) fn guard_quarantine_close(&mut self, ctx: &egui::Context) {
        if ctx.input(|input| input.viewport().close_requested()) && self.quarantine.writing() {
            self.quarantine.close_after = true;
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.runtime.wake();
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        if self.quarantine.close_after && !self.quarantine.writing() {
            self.quarantine.close_after = false;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
    pub(super) fn show_quarantine(&mut self) {
        self.quarantine.open = true;
        self.query_quarantine(Operation::Read, 0);
    }
    fn retry_quarantined(&mut self, document: Document) {
        self.quarantine.close();
        self.tabs.cancel();
        self.open_candidate(document);
    }
    fn query_quarantine(&mut self, operation: Operation, offset: usize) {
        if self
            .quarantine
            .pending
            .as_ref()
            .is_some_and(|(_, writing)| *writing)
        {
            return;
        }
        if let Some((token, _)) = self.quarantine.pending.take() {
            token.cancel();
        }
        let writing = matches!(operation, Operation::Release(_));
        let cancel = Cancellation::default();
        self.quarantine.error = false;
        match self.runtime.submit(Action::Quarantine(Box::new(Request {
            operation,
            offset,
            cancel: cancel.clone(),
        }))) {
            Ok(()) => self.quarantine.pending = Some((cancel, writing)),
            Err(_) => self.quarantine.error = true,
        }
    }
    pub(super) fn record_reader_health(&mut self, document: &Document, health: ReaderHealth) {
        if let Err(error) = self.runtime.submit(Action::Quarantine(Box::new(Request {
            operation: Operation::Health {
                document: Box::new(document.clone()),
                health,
            },
            offset: 0,
            cancel: Cancellation::default(),
        }))) {
            self.error = Some(error.to_string());
        } else {
            self.quarantine.health_pending += 1;
        }
    }
    pub(super) fn record_reader_failure(
        &mut self,
        document: &Document,
        error: &yamantaka_domain::Failure,
    ) {
        if let Some(reason) = QuarantineReason::from_failure(error) {
            self.record_reader_health(document, ReaderHealth::Failed(reason));
        }
    }
    pub(super) fn accept_quarantine(&mut self, ctx: &egui::Context, reply: Reply) {
        if reply.result.is_ok()
            && let Some((document, blocked)) = &reply.change
        {
            if *blocked {
                self.library_covers.pause(std::slice::from_ref(document));
            } else {
                self.library_covers.resume(std::slice::from_ref(document));
                self.retry_failed_library_cover(ctx, document);
            }
        }
        if reply.health {
            self.quarantine.health_pending = self.quarantine.health_pending.saturating_sub(1);
            if let Err(error) = reply.result {
                self.quarantine.close_after = false;
                self.error = Some(error.to_string());
            } else if self.quarantine.open
                && !self.quarantine.close_after
                && self.quarantine.pending.is_none()
            {
                let offset = self.quarantine.page.as_ref().map_or(0, |page| page.offset);
                self.query_quarantine(Operation::Read, offset);
            }
            return;
        }
        if !self
            .quarantine
            .pending
            .as_ref()
            .is_some_and(|(token, _)| token.same_request(&reply.cancel))
        {
            return;
        }
        let (_, writing) = self.quarantine.pending.take().unwrap();
        self.quarantine.release = None;
        match reply.result {
            Ok(Some(page)) => {
                self.quarantine.page = Some(page);
                self.quarantine.error = false;
            }
            Err(error) if !matches!(error, yamantaka_domain::Failure::Cancelled) => {
                self.quarantine.close_after = false;
                self.quarantine.error = true;
                if writing && !self.quarantine.open {
                    self.error = Some(error.to_string());
                }
            }
            _ => {}
        }
    }
    pub(super) fn quarantine_dialog(&mut self, ctx: &egui::Context) {
        if !self.quarantine.open {
            return;
        }
        let p = Palette::new(self.dark);
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let mut close = false;
        let mut refresh = None;
        let mut release = None;
        let mut retry = None;
        let busy = self.quarantine.pending.is_some();
        let modal = egui::Modal::new(egui::Id::new("quarantined-documents")).show(ctx, |ui| {
            ui.set_width((ctx.content_rect().width()-48.0).clamp(120.0,720.0));
            ui.set_max_height((ctx.content_rect().height()-48.0).max(80.0));
            egui::ScrollArea::vertical().id_salt("quarantine-review").max_height((ctx.content_rect().height()-48.0).max(80.0)).animated(!self.reduced_motion).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    theme::display(ui,tr("Recuperación de documentos","Document recovery"),24.0);
                    close = theme::navigation(ui,tr("Cerrar","Close"),false,p).clicked();
                });
                ui.add_space(10.0);
                ui.label(tr("Estos libros no se restauran automáticamente después de un fallo. Sus PDF y anotaciones se conservan.","These books are not restored automatically after a failure. Their PDFs and annotations are preserved."));
                if self.quarantine.error { ui.label(tr("La lista o el documento han cambiado. Actualiza antes de volver a intentarlo.","The list or document changed. Refresh before retrying.")); }
                if busy { ui.label(tr("Actualizando…","Updating…")); }
                let offset = self.quarantine.page.as_ref().map_or(0,|page|page.offset);
                ui.add_enabled_ui(!busy,|ui| {
                    if let Some(expected) = self.quarantine.release.clone() {
                        ui.add_space(12.0);
                        if let Some(entry) = self.quarantine.page.as_ref().and_then(|page|page.entries.iter().find(|entry|entry.document.id==expected.document)) {
                            ui.label(RichText::new(entry.document.display_title().chars().take(160).collect::<String>()).size(18.0).strong());
                        }
                        ui.label(tr("¿Permitir de nuevo la restauración automática? Esto no repara el PDF; podría volver a fallar.","Allow automatic restoration again? This does not repair the PDF; it could fail again."));
                        ui.horizontal_wrapped(|ui| {
                            if theme::control(ui,tr("Quitar bloqueo","Remove block"),Some(theme::Icon::Right),false,true,p).clicked() { release = Some(expected.clone()); }
                            if theme::navigation(ui,tr("Cancelar","Cancel"),false,p).clicked() { self.quarantine.release = None; }
                        });
                    } else if let Some(page) = &self.quarantine.page {
                        ui.label(format!("{} {}",page.total,tr("documentos bloqueados","blocked documents")));
                        if page.total == 0 { ui.label(tr("No hay lecturas pendientes de recuperación.","No readings need recovery.")); }
                        for entry in &page.entries {
                            ui.add_space(14.0);
                            ui.separator();
                            ui.label(RichText::new(entry.document.display_title().chars().take(160).collect::<String>()).size(17.0).strong());
                            ui.label(if entry.document.missing {tr("Archivo ausente. Localízalo desde los detalles del libro.","File missing. Locate it from the book's details.")} else if entry.reason == QuarantineReason::PdfOpenFailed {tr("La apertura del PDF falló.","Opening the PDF failed.")} else {tr("Bloqueo conservado de una lectura anterior.","Block preserved from an earlier reading.")});
                            ui.horizontal_wrapped(|ui| {
                                ui.add_enabled_ui(!entry.document.missing,|ui| { if theme::navigation(ui,tr("Reintentar lectura","Retry reading"),false,p).clicked() { retry = Some(entry.document.clone()); } });
                                if theme::navigation(ui,tr("Revisar bloqueo","Review block"),false,p).clicked() { self.quarantine.release = Some(entry.revision.clone()); }
                            });
                        }
                        ui.add_space(14.0);
                        ui.horizontal_wrapped(|ui| {
                            if page.offset > 0 && theme::navigation(ui,tr("Anterior","Previous"),false,p).clicked() {refresh=Some(page.offset.saturating_sub(80));}
                            if ((page.offset+page.entries.len()) as u64) < page.total && theme::navigation(ui,tr("Siguiente","Next"),false,p).clicked() {refresh=Some(page.offset+80);}
                        });
                    }
                    ui.add_space(12.0);
                    if theme::navigation(ui,tr("Actualizar","Refresh"),false,p).clicked() { refresh=Some(offset); }
                });
            });
        });
        if close || modal.should_close() {
            self.quarantine.close();
        } else if let Some(revision) = release {
            let offset = self.quarantine.page.as_ref().map_or(0, |page| page.offset);
            self.query_quarantine(Operation::Release(revision), offset);
        } else if let Some(document) = retry {
            self.retry_quarantined(document);
        } else if let Some(offset) = refresh {
            self.quarantine.release = None;
            self.query_quarantine(Operation::Read, offset);
        }
    }
}
