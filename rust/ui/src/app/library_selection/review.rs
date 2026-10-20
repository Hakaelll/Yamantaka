use super::*;
impl YamantakaApp {
    pub(in crate::app) fn selection_review_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut review) = self.library_selection.review.take() else {
            return;
        };
        let p = Palette::new(self.dark);
        let compact = ctx.content_rect().height() < 420.0;
        let mut approved = false;
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::new("selection-review"))
            .frame(egui::Frame::new().fill(p.surface).stroke(egui::Stroke::new(1.0,p.line))
                .corner_radius(12).inner_margin(if compact {12} else {24}))
            .show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width()-56.0).clamp(160.0,420.0));
                ui.spacing_mut().item_spacing.y = if compact {8.0} else {16.0};
                theme::display(ui,match review.action {ReviewAction::Covers=>self.tr("Portadas automáticas","Automatic covers"),ReviewAction::Remove=>self.tr("Retirar libros","Remove books")},if compact {20.0} else {24.0});
                let count = format!("{} {}",review.ids.len(),if review.ids.len()==1 {
                    self.tr("libro seleccionado.","selected book.")
                } else {self.tr("libros seleccionados.","selected books.")});
                match review.action {
                    ReviewAction::Covers=> {
                        ui.label(format!("{count} {}",self.tr("Sustituye sus portadas manuales. Los PDF no cambian.","Replaces their manual covers. PDF files stay unchanged.")))
                            .on_hover_text(self.tr("Cualquier elección posterior se conserva.","Later choices are kept."));
                        if !compact { ui.label(RichText::new(self.tr(
                            "Las elecciones posteriores se conservan. Cada portada sigue visible hasta tener su sustitución. Puedes cancelar los libros pendientes.",
                            "Later choices are kept. Each cover stays visible until its replacement is ready. You can cancel the remaining books."
                        )).color(p.secondary)); }
                    }
                    ReviewAction::Remove=> {
                        ui.label(format!("{count} {}",self.tr("Elimina sus datos locales de lectura. Conserva los PDF. No se puede deshacer desde aquí.","Deletes their local reading data. Keeps the PDFs. No undo is available here.")))
                            .on_hover_text(self.tr("Se retiran también sus anotaciones, marcadores, historial y pertenencias. No se puede deshacer desde aquí.","Their annotations, bookmarks, history and memberships are also removed. This action cannot be undone here."));
                        if !compact {ui.label(RichText::new(self.tr("Incluye notas, anotaciones, marcadores e historial. También se retiran estos libros de sus etiquetas y colecciones.","Includes notes, annotations, bookmarks and history. Removes these books from their tags and collections.")).color(p.secondary));}
                    }
                }
                ui.horizontal_wrapped(|ui| {
                    let cancel = theme::navigation(ui,self.tr("Cancelar","Cancel"),false,p);
                    if review.focus_cancel {cancel.request_focus();review.focus_cancel=false;}
                    close = cancel.clicked();
                    approved = theme::control(ui,match review.action {ReviewAction::Covers=>self.tr("Regenerar","Regenerate"),ReviewAction::Remove=>self.tr("Retirar","Remove")},None,true,false,p).clicked();
                });
            });
        close |= modal.should_close();
        if approved {
            match review.action {
                ReviewAction::Covers => {
                    self.submit_cover_batch(review.ids);
                }
                ReviewAction::Remove => {
                    self.prepare_library_removal(review.ids);
                }
            }
        } else if !close {
            self.library_selection.review = Some(review);
        }
    }
}
