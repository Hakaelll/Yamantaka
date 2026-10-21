use super::*;

impl YamantakaApp {
    pub(in crate::app) fn tag_management_dialogs(&mut self, ctx: &egui::Context) {
        if self.library_tags.writing() {
            return;
        }
        if self.library_tags.review.is_some() {
            self.tag_review_dialog(ctx);
        } else {
            self.tag_merge_dialog(ctx);
        }
    }
    fn tag_review_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut review) = self.library_tags.review.take() else {
            return;
        };
        let p = Palette::new(self.dark);
        let compact = ctx.content_rect().height() < 420.0;
        let merging = review.target.is_some();
        let mut approved = false;
        let mut close = false;
        let modal=egui::Modal::new(egui::Id::new("tag-change-review"))
            .frame(egui::Frame::new().fill(p.surface).stroke(egui::Stroke::new(1.0,p.line)).corner_radius(12).inner_margin(if compact {12} else {24}))
            .show(ctx,|ui| {
                ui.set_width((ctx.content_rect().width()-56.0).clamp(160.0,420.0));
                ui.spacing_mut().item_spacing.y=if compact {8.0} else {16.0};
                theme::display(ui,if merging {self.tr("Fusionar etiquetas","Merge tags")} else {self.tr("Eliminar etiqueta","Delete tag")},if compact {20.0} else {24.0});
                egui::ScrollArea::vertical().id_salt("tag-review-body").max_height((ctx.content_rect().height()-144.0).max(40.0)).show(ui,|ui| {
                    let names=match &review.target {Some(target)=>format!("{} → {}",review.source.name,target.name),None=>review.source.name.clone()};
                    ui.label(RichText::new(names).strong());
                    ui.label(if merging {self.tr("Combina sus asociaciones. Conserva libros y notas. No se puede deshacer.","Combines their associations. Keeps books and notes. Cannot be undone.")} else {self.tr("Elimina esta etiqueta y sus asociaciones. Conserva libros y notas. No se puede deshacer.","Removes this tag and its associations. Keeps books and notes. Cannot be undone.")});
                    if !compact {ui.label(RichText::new(format!("{} {} · {} {}",review.source.document_count,self.tr("libros","books"),review.source.annotation_count,self.tr("anotaciones con esta etiqueta","annotations using this tag"))).color(p.secondary));}
                });
                ui.horizontal_wrapped(|ui| {
                    let cancel=theme::navigation(ui,self.tr("Cancelar","Cancel"),false,p);
                    if review.focus_cancel {cancel.request_focus();review.focus_cancel=false;}
                    close=cancel.clicked();
                    approved=theme::control(ui,if merging {self.tr("Fusionar","Merge")} else {self.tr("Eliminar","Delete")},None,true,false,p).clicked();
                });
            });
        close |= modal.should_close();
        if approved {
            self.library_tags.review = Some(review);
            self.approve_tag_review();
        } else if !close {
            self.library_tags.review = Some(review);
        }
    }
    fn tag_merge_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut picker) = self.library_tags.merge.take() else {
            return;
        };
        let p = Palette::new(self.dark);
        let compact = ctx.content_rect().height() < 420.0;
        let mut close = false;
        let mut target = None;
        let modal = egui::Modal::new(egui::Id::new("tag-merge-target"))
            .frame(
                egui::Frame::new()
                    .fill(p.surface)
                    .stroke(egui::Stroke::new(1.0, p.line))
                    .corner_radius(12)
                    .inner_margin(16),
            )
            .show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width() - 64.0).clamp(160.0, 480.0));
                ui.set_height((ctx.content_rect().height() - 64.0).clamp(140.0, 520.0));
                ui.spacing_mut().item_spacing.y = if compact { 4.0 } else { 8.0 };
                theme::display(
                    ui,
                    self.tr("Elegir destino", "Choose destination"),
                    if compact { 20.0 } else { 22.0 },
                );
                ui.label(
                    RichText::new(format!(
                        "{}: {}",
                        self.tr("Fusionar", "Merge"),
                        picker.source.name
                    ))
                    .size(if compact { 12.0 } else { 14.0 }),
                );
                let search = self.tr("Buscar etiqueta destino", "Search destination tag");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut picker.query.text)
                        .hint_text(search)
                        .char_limit(256)
                        .desired_width(ui.available_width()),
                );
                theme::record_control_bounds(ui, search, response.rect);
                if response.changed() {
                    picker.query.offset = 0;
                    self.load_tag_merge_targets(&mut picker);
                }
                let height = (ui.available_height() - 44.0).max(40.0);
                egui::ScrollArea::vertical()
                    .id_salt("tag-merge-rows")
                    .max_height(height)
                    .min_scrolled_height(40.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if picker.failed {
                            ui.label(self.tr(
                                "No se pudieron consultar las etiquetas.",
                                "Could not read the tags.",
                            ));
                            if theme::navigation(ui, self.tr("Reintentar", "Retry"), false, p)
                                .clicked()
                            {
                                self.load_tag_merge_targets(&mut picker);
                            }
                        } else if picker.loaded
                            && picker
                                .entries
                                .iter()
                                .all(|entry| entry.id == picker.source.id)
                        {
                            ui.label(self.tr(
                                "No hay otras etiquetas que coincidan.",
                                "No other matching tags.",
                            ));
                        }
                        for entry in picker
                            .entries
                            .iter()
                            .filter(|entry| entry.id != picker.source.id)
                        {
                            let label = format!(
                                "{} · {} {}",
                                entry.name,
                                entry.document_count,
                                if entry.document_count == 1 {
                                    self.tr("libro", "book")
                                } else {
                                    self.tr("libros", "books")
                                }
                            );
                            if ui
                                .add_enabled_ui(
                                    !self.library_tags.writing()
                                        && picker.pending.is_none()
                                        && !picker.failed
                                        && entry.id != picker.source.id,
                                    |ui| theme::bounded_navigation(ui, &label, p),
                                )
                                .inner
                                .clicked()
                            {
                                target = Some(entry.clone());
                            }
                        }
                    });
                ui.horizontal(|ui| {
                    let busy = picker.pending.is_some() || self.library_tags.writing();
                    if ui
                        .add_enabled_ui(!busy && picker.query.offset > 0, |ui| {
                            theme::icon_button(
                                ui,
                                self.tr("Anterior", "Previous"),
                                theme::Icon::Left,
                                p,
                            )
                        })
                        .inner
                        .clicked()
                    {
                        picker.query.offset = picker.query.offset.saturating_sub(80);
                        self.load_tag_merge_targets(&mut picker);
                    }
                    if ui
                        .add_enabled_ui(
                            !busy
                                && ((picker.query.offset + picker.entries.len()) as u64)
                                    < picker.total,
                            |ui| {
                                theme::icon_button(
                                    ui,
                                    self.tr("Siguiente", "Next"),
                                    theme::Icon::Right,
                                    p,
                                )
                            },
                        )
                        .inner
                        .clicked()
                    {
                        picker.query.offset += 80;
                        self.load_tag_merge_targets(&mut picker);
                    }
                    if ui.available_width() > 120.0 {
                        ui.label(format!(
                            "{}/{}",
                            picker.query.offset / 80 + 1,
                            picker.total.div_ceil(80).max(1)
                        ));
                    }
                    if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked() {
                        close = true;
                    }
                });
            });
        close |= modal.should_close();
        if !close {
            self.library_tags.merge = Some(picker);
            if let Some(target) = target {
                self.review_tag_merge(target);
            }
        }
    }
}
