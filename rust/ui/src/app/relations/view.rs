use super::*;

fn label(kind: RelationKind, backlink: bool, english: bool) -> &'static str {
    match (kind, backlink, english) {
        (RelationKind::Related, _, true) => "Related",
        (RelationKind::Related, _, false) => "Relacionado",
        (RelationKind::Replaces, false, true) => "Replaces",
        (RelationKind::Replaces, true, true) => "Replaced by",
        (RelationKind::Replaces, false, false) => "Reemplaza",
        (RelationKind::Replaces, true, false) => "Reemplazado por",
        (RelationKind::SourceOf, false, true) => "Source of",
        (RelationKind::SourceOf, true, true) => "Source",
        (RelationKind::SourceOf, false, false) => "Fuente de",
        (RelationKind::SourceOf, true, false) => "Fuente",
        (RelationKind::AttachmentOf, false, true) => "Attachment of",
        (RelationKind::AttachmentOf, true, true) => "Has attachment",
        (RelationKind::AttachmentOf, false, false) => "Adjunto de",
        (RelationKind::AttachmentOf, true, false) => "Tiene adjunto",
    }
}
impl YamantakaApp {
    pub(in crate::app) fn relations_view(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);

        let english = self.english;

        let tr = |es, en| if english { en } else { es };

        let Some(context) = self.relations.context.clone() else {
            return;
        };

        let busy = self.relations.pending.is_some()
            || self.relations.writing()
            || self.relations.context_lookup.is_some();

        let width = ui.available_width();
        let margin = if width < 700.0 { 16 } else { 40 };
        let content_width = (width - 2.0 * margin as f32).clamp(1.0, 1100.0);
        let flow = || {
            if content_width < 620.0 {
                egui::Layout::top_down(egui::Align::Min)
            } else {
                egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true)
            }
        };
        egui::ScrollArea::vertical()
            .id_salt("document-relations")
            .auto_shrink([false, false])
            .animated(!self.reduced_motion)
            .show(ui, |ui| {
                egui::Frame::new().inner_margin(margin).show(ui, |ui| {
                    ui.set_width(content_width);
                    ui.set_max_width(content_width);
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                    ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 0.0), flow(), |ui| {
                        let size = if ui.available_width() < 600.0 { 28.0 } else { 38.0 };
                        theme::display(ui, tr("Relaciones", "Connections"), size);
                        if theme::navigation(ui, tr("Volver a la biblioteca", "Back to library"), false, p).clicked() {
                            self.show_surface(Surface::Library);
                        }
                    });
                    ui.add_space(8.0);
                    ui.label(RichText::new(context.display_title()).color(p.secondary));
                    ui.add_space(8.0);
                    ui.label(tr("Conecta fuentes, versiones y lecturas complementarias.", "Connect sources, editions, and companion readings."));
                    ui.add_space(16.0);
                    let previous=self.relations.section;
                    ui.add_enabled_ui(!self.relations.writing(), |ui| {
                        egui::ComboBox::from_id_salt("connection-section")
                            .selected_text(match self.relations.section {RelationSection::Connections=>tr("Relaciones", "Connections"),RelationSection::Revisions=>tr("Grupos de revisiones", "Revision groups")})
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.relations.section,RelationSection::Connections,tr("Relaciones", "Connections"));
                                ui.selectable_value(&mut self.relations.section,RelationSection::Revisions,tr("Grupos de revisiones", "Revision groups"));
                            });
                    });
                    if previous!=self.relations.section {
                        if let Some(cancel)=self.relations.pending.take() {cancel.cancel();}
                        self.relations.revisions.leave();
                        match self.relations.section {RelationSection::Connections=>self.load_relations(),RelationSection::Revisions=>self.load_revision_groups()}
                    }
                    ui.add_space(24.0);
                    if self.relations.section==RelationSection::Revisions {
                        self.revision_groups_view(ui,p);
                        return;
                    }
                    if let Some(error) = self.relations.error.clone() {
                        ui.label(error);
                        if theme::navigation(ui, tr("Recargar", "Reload"), false, p).clicked() {
                            self.load_relations();
                        }
                    }
                    if self.relations.writing() {
                        ui.label(tr("Guardando relación…", "Saving connection…"));
                    } else if self.relations.saved {
                        ui.label(tr("Relaciones actualizadas.", "Connections updated."));
                    }
                    ui.add_enabled_ui(!busy, |ui| {
                        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 0.0), flow(), |ui| {
                            let caption = tr("Elegir documento", "Choose document");
                            let button = theme::navigation(ui, caption, false, p);
                            egui::Popup::menu(&button)
                                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                                .show(|ui| {
                                    if let Some(target) = self.relations.chooser.show(ui, &self.runtime, english, p, catalogue_chooser::ChooserPurpose::Relation(&context.id)) {
                                        self.relations.target = Some(target);
                                    }
                                });
                            egui::ComboBox::from_id_salt("relation-kind")
                                .selected_text(label(self.relations.kind, false, english))
                                .show_ui(ui, |ui| {
                                    for kind in RelationKind::ALL {
                                        ui.selectable_value(&mut self.relations.kind, kind, label(kind, false, english));
                                    }
                                });
                            let add = ui.add_enabled_ui(self.relations.target.is_some(), |ui| {
                                theme::control(ui, tr("Añadir relación", "Add connection"), Some(theme::Icon::Add), false, true, p)
                            }).inner;
                            if add.clicked() {
                                self.change_relation(RelationChange::Create {
                                    source: context.id.clone(),
                                    target: self.relations.target.as_ref().unwrap().id.clone(),
                                    kind: self.relations.kind,
                                });
                            }
                        });
                    });
                    if let Some(target) = &self.relations.target {
                        ui.add_space(8.0);
                        ui.label(RichText::new(target.display_title()).color(p.secondary)).on_hover_text(tr("Documento elegido", "Selected document"));
                    }
                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(16.0);
                    let mut open = None;
                    let mut view = None;
                    let mut remove = None;
                    if let Some(page) = self.relations.page.clone() {
                        if page.total == 0 {
                            ui.label(tr("Sin relaciones todavía. Elige otro documento para crear la primera.", "No connections yet. Choose another document to create the first."));
                        }
                        for entry in &page.entries {
                            egui::Frame::new().fill(p.surface).corner_radius(8).inner_margin(16).show(ui, |ui| {
                                ui.label(RichText::new(label(entry.relation.kind, entry.backlink, english)).color(p.secondary).size(12.0));
                                ui.label(RichText::new(&entry.title).size(19.0).color(p.ink));
                                ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 0.0), flow(), |ui| {
                                    if theme::navigation(ui, tr("Abrir documento", "Open document"), false, p).clicked() {
                                        open = Some(entry.other_document.clone());
                                    }
                                    if theme::navigation(ui, tr("Ver relaciones", "View connections"), false, p).clicked() {
                                        view = Some(entry.other_document.clone());
                                    }
                                    if ui.add_enabled_ui(!busy, |ui| theme::navigation(ui, tr("Quitar relación", "Remove connection"), false, p)).inner.clicked() {
                                        self.relations.removing = Some(entry.relation.clone());
                                    }
                                });
                                if self.relations.removing.as_ref().is_some_and(|relation| relation.id == entry.relation.id) {
                                    ui.label(tr("Solo se elimina este vínculo. Los documentos se conservan.", "Only this connection is removed. Both documents are kept."));
                                    let actions = ui.add_enabled_ui(!busy, |ui| {
                                        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 0.0), flow(), |ui| {
                                            if theme::navigation(ui, tr("Confirmar eliminación", "Confirm removal"), false, p).clicked() {
                                                remove = Some(entry.relation.clone());
                                            }
                                            if theme::navigation(ui, tr("Cancelar", "Cancel"), false, p).clicked() {
                                                self.relations.removing = None;
                                            }
                                        });
                                    }).response;
                                    if self.relations.preview_review && self.relations.preview == 10 {
                                        actions.scroll_to_me(Some(egui::Align::Center));
                                    }
                                }
                            });
                            ui.add_space(8.0);
                        }
                        let total = page.total;
                        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 0.0), flow(), |ui| {
                            if ui.add_enabled_ui(!busy && self.relations.offset > 0, |ui| theme::navigation(ui, tr("Anteriores", "Previous"), false, p)).inner.clicked() {
                                self.relations.offset = self.relations.offset.saturating_sub(80);
                                self.load_relations();
                            }
                            ui.label(format!("{} / {total}", self.relations.offset as u64 + page.entries.len() as u64));
                            if ui.add_enabled_ui(!busy && (self.relations.offset as u64 + 80) < total, |ui| theme::navigation(ui, tr("Siguientes", "Next"), false, p)).inner.clicked() {
                                self.relations.offset = self.relations.offset.saturating_add(80);
                                self.load_relations();
                            }
                        });
                    }
                    if let Some(expected) = remove {
                        self.change_relation(RelationChange::Remove { document: context.id.clone(), expected });
                    }
                    if let Some(id) = open {
                        let cancel = self.open_intent.lookup(id.clone(), None);
                        self.send(Action::GetDocument(id, cancel));
                    }
                    if let Some(id) = view {
                        self.load_relation_context(id);
                    }
                });
            });
    }
}
