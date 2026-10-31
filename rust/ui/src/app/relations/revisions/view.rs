use super::*;
impl YamantakaApp {
    pub(in crate::app) fn revision_capture_summary(&self) -> serde_json::Value {
        self.relations.revisions.summary()
    }
    pub(in crate::app) fn revision_groups_view(&mut self, ui: &mut egui::Ui, p: Palette) {
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let busy = self.relations.writing()
            || self.relations.revisions.busy()
            || self.relations.context_lookup.is_some();
        if let Some(error) = self.relations.revisions.error.clone() {
            ui.label(error);
            if theme::navigation(ui, tr("Recargar revisiones", "Reload revisions"), false, p)
                .clicked()
            {
                self.load_revision_groups();
            }
        }
        ui.horizontal_wrapped(|ui| {
            if theme::navigation(
                ui,
                tr("Nuevo grupo", "New group"),
                self.relations.revisions.creation_open,
                p,
            )
            .clicked()
            {
                self.relations.revisions.creation_open = !self.relations.revisions.creation_open;
            }
            if self.relations.revisions.saved {
                ui.label(
                    RichText::new(tr("Grupo actualizado.", "Group updated.")).color(p.secondary),
                );
            }
        });
        if self.relations.revisions.creation_open {
            self.revision_create_form(ui, p, busy);
        }
        ui.add_space(24.0);
        if ui.available_width() >= 860.0 {
            ui.columns(2, |columns| {
                self.revision_group_list(&mut columns[0], p);
                self.revision_member_list(&mut columns[1], p);
            });
        } else {
            self.revision_group_list(ui, p);
            ui.add_space(24.0);
            self.revision_member_list(ui, p);
        }
    }
    fn revision_create_form(&mut self, ui: &mut egui::Ui, p: Palette, busy: bool) {
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let Some(context) = self.relations.context.clone() else {
            return;
        };
        let compact = ui.available_width() < 620.0;
        let flow = || {
            if compact {
                egui::Layout::top_down(egui::Align::Min)
            } else {
                egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true)
            }
        };
        ui.add_space(16.0);
        let label = ui.label(tr("Nombre del grupo", "Group name"));
        ui.add_enabled_ui(!busy, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.relations.revisions.name)
                    .desired_width(f32::INFINITY)
                    .char_limit(120)
                    .hint_text(tr("Ediciones para este estudio", "Editions for this study")),
            )
            .labelled_by(label.id);
            ui.add_space(8.0);
            ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 0.0), flow(), |ui| {
                let choose = theme::navigation(
                    ui,
                    tr("Elegir otra edición", "Choose another edition"),
                    false,
                    p,
                );
                egui::Popup::menu(&choose)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        if let Some(target) = self.relations.chooser.show(
                            ui,
                            &self.runtime,
                            english,
                            p,
                            catalogue_chooser::ChooserPurpose::Relation(&context.id),
                        ) {
                            self.relations.target = Some(target);
                        }
                    });
                let valid = self.relations.target.is_some()
                    && !self.relations.revisions.name.trim().is_empty();
                if ui
                    .add_enabled_ui(valid, |ui| {
                        theme::control(
                            ui,
                            tr("Crear grupo", "Create group"),
                            Some(theme::Icon::Add),
                            false,
                            true,
                            p,
                        )
                    })
                    .inner
                    .clicked()
                {
                    self.change_revision_group(RevisionChange::Create {
                        name: self.relations.revisions.name.clone(),
                        members: vec![
                            context.id.clone(),
                            self.relations.target.as_ref().unwrap().id.clone(),
                        ],
                        current: Some(context.id.clone()),
                    });
                }
            });
        });
        if let Some(target) = &self.relations.target {
            ui.add_space(8.0);
            ui.label(RichText::new(target.display_title()).color(p.secondary));
        }
        ui.label(
            RichText::new(tr(
                "El documento de origen se marca como actual al crear el grupo.",
                "The source document is current when the group is created.",
            ))
            .size(12.0)
            .color(p.secondary),
        );
        ui.add_space(24.0);
        ui.separator();
        ui.add_space(16.0);
    }
    fn revision_group_list(&mut self, ui: &mut egui::Ui, p: Palette) {
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let busy = self.relations.writing()
            || self.relations.revisions.busy()
            || self.relations.context_lookup.is_some();
        let compact = ui.available_width() < 620.0;
        let flow = || {
            if compact {
                egui::Layout::top_down(egui::Align::Min)
            } else {
                egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true)
            }
        };
        ui.label(RichText::new(tr("Tus grupos", "Your groups")).color(p.secondary));
        ui.add_space(8.0);
        let mut select = None;
        if let Some(page) = self.relations.revisions.groups.clone() {
            if page.total == 0 {
                ui.label(tr(
                    "Todavía no hay grupos para este documento.",
                    "No revision groups for this document yet.",
                ));
            }
            for group in &page.entries {
                let selected = self
                    .relations
                    .revisions
                    .selected
                    .as_ref()
                    .is_some_and(|selected| selected.id == group.id);
                egui::Frame::new()
                    .fill(if selected { p.raised } else { p.surface })
                    .corner_radius(8)
                    .inner_margin(16)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(RichText::new(&group.name).size(20.0).color(p.ink));
                        ui.label(
                            RichText::new(if english {
                                format!("{} editions", group.member_count)
                            } else {
                                format!("{} ediciones", group.member_count)
                            })
                            .color(p.secondary),
                        );
                        if let Some(title) = &group.current_title {
                            ui.label(if english {
                                format!("Current: {title}")
                            } else {
                                format!("Actual: {title}")
                            });
                        } else {
                            ui.label(tr(
                                "Elige una edición actual entre los miembros.",
                                "Choose a current edition from the members.",
                            ));
                        }
                        if ui
                            .add_enabled_ui(!busy, |ui| {
                                theme::navigation(
                                    ui,
                                    tr("Ver ediciones", "View editions"),
                                    selected,
                                    p,
                                )
                            })
                            .inner
                            .clicked()
                        {
                            select = Some(group.clone());
                        }
                    });
                ui.add_space(8.0);
            }
            ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 0.0), flow(), |ui| {
                if ui
                    .add_enabled_ui(!busy && self.relations.revisions.offset > 0, |ui| {
                        theme::navigation(ui, tr("Grupos anteriores", "Previous groups"), false, p)
                    })
                    .inner
                    .clicked()
                {
                    self.relations.revisions.offset =
                        self.relations.revisions.offset.saturating_sub(80);
                    self.load_revision_groups();
                }
                if ui
                    .add_enabled_ui(
                        !busy && (self.relations.revisions.offset as u64 + 80) < page.total,
                        |ui| theme::navigation(ui, tr("Más grupos", "More groups"), false, p),
                    )
                    .inner
                    .clicked()
                {
                    self.relations.revisions.offset =
                        self.relations.revisions.offset.saturating_add(80);
                    self.load_revision_groups();
                }
            });
        }
        if let Some(group) = select {
            self.select_revision_group(group);
        }
    }
    fn revision_member_list(&mut self, ui: &mut egui::Ui, p: Palette) {
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let busy = self.relations.writing()
            || self.relations.revisions.busy()
            || self.relations.context_lookup.is_some();
        let compact = ui.available_width() < 620.0;
        let flow = || {
            if compact {
                egui::Layout::top_down(egui::Align::Min)
            } else {
                egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true)
            }
        };
        if self.relations.revisions.members.is_none() {
            ui.label(tr(
                "Selecciona un grupo para ver sus ediciones.",
                "Select a group to see its editions.",
            ));
        }
        let mut open = None;
        let mut current = None;
        if let Some(page) = self.relations.revisions.members.clone() {
            if let Some(group) = self
                .relations
                .revisions
                .groups
                .as_ref()
                .and_then(|groups| groups.entries.iter().find(|group| group.id == page.group))
            {
                ui.label(RichText::new(&group.name).size(22.0).color(p.ink));
            }
            for member in &page.entries {
                egui::Frame::new()
                    .fill(p.surface)
                    .corner_radius(8)
                    .inner_margin(16)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(RichText::new(&member.title).size(18.0).color(p.ink));
                        if page.current.as_deref() == Some(member.document.as_str()) {
                            ui.label(
                                RichText::new(tr("Edición actual", "Current edition"))
                                    .color(p.secondary),
                            );
                        }
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 0.0),
                            flow(),
                            |ui| {
                                if ui
                                    .add_enabled_ui(!member.missing, |ui| {
                                        theme::navigation(
                                            ui,
                                            tr("Abrir edición", "Open edition"),
                                            false,
                                            p,
                                        )
                                    })
                                    .inner
                                    .clicked()
                                {
                                    open = Some(member.document.clone());
                                }
                                if page.current.as_deref() != Some(member.document.as_str()) {
                                    let action = ui
                                        .add_enabled_ui(!busy, |ui| {
                                            theme::navigation(
                                                ui,
                                                tr("Marcar como actual", "Make current"),
                                                false,
                                                p,
                                            )
                                        })
                                        .inner;
                                    if action.clicked() {
                                        current = Some(member.document.clone());
                                    }
                                    if self.relations.revisions.preview == 10
                                        && self.relations.revisions.preview_mode == 2
                                    {
                                        action.scroll_to_me(Some(egui::Align::Center));
                                    }
                                }
                            },
                        );
                    });
                ui.add_space(8.0);
            }
            ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 0.0), flow(), |ui| {
                if ui
                    .add_enabled_ui(!busy && self.relations.revisions.member_offset > 0, |ui| {
                        theme::navigation(
                            ui,
                            tr("Ediciones anteriores", "Previous editions"),
                            false,
                            p,
                        )
                    })
                    .inner
                    .clicked()
                {
                    self.relations.revisions.member_offset =
                        self.relations.revisions.member_offset.saturating_sub(80);
                    self.load_revision_members();
                }
                if ui
                    .add_enabled_ui(
                        !busy && (self.relations.revisions.member_offset as u64 + 80) < page.total,
                        |ui| theme::navigation(ui, tr("Más ediciones", "More editions"), false, p),
                    )
                    .inner
                    .clicked()
                {
                    self.relations.revisions.member_offset =
                        self.relations.revisions.member_offset.saturating_add(80);
                    self.load_revision_members();
                }
            });
        }
        if let Some(document) = current
            && let Some(group) = &self.relations.revisions.selected
        {
            self.change_revision_group(RevisionChange::SetCurrent {
                expected: group.into(),
                document,
            });
        }
        if let Some(id) = open {
            let cancel = self.open_intent.lookup(id.clone(), None);
            self.send(Action::GetDocument(id, cancel));
        }
    }
}
