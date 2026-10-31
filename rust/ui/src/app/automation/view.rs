use super::*;
fn trigger_label(kind: TriggerKind, english: bool) -> &'static str {
    match (kind, english) {
        (TriggerKind::DocumentImported, true) => "Document imported",
        (TriggerKind::DocumentImported, false) => "Documento importado",
        (TriggerKind::SourceFolder, true) => "Source folder",
        (TriggerKind::SourceFolder, false) => "Carpeta de origen",
        (TriggerKind::Extension, true) => "File extension",
        (TriggerKind::Extension, false) => "Extensión de archivo",
        (TriggerKind::Author, true) => "Author",
        (TriggerKind::Author, false) => "Autor",
        (TriggerKind::Tag, true) => "Tag",
        (TriggerKind::Tag, false) => "Etiqueta",
        (TriggerKind::Collection, true) => "Collection",
        (TriggerKind::Collection, false) => "Colección",
        (TriggerKind::ExactDuplicate, true) => "Exact duplicate",
        (TriggerKind::ExactDuplicate, false) => "Duplicado exacto",
        (TriggerKind::Inaccessible, true) => "Unavailable file",
        (TriggerKind::Inaccessible, false) => "Archivo no disponible",
    }
}
fn action_label(kind: ActionKind, english: bool) -> &'static str {
    match (kind, english) {
        (ActionKind::AddTag, true) => "Add tag",
        (ActionKind::AddTag, false) => "Añadir etiqueta",
        (ActionKind::AddCollection, true) => "Add to collection",
        (ActionKind::AddCollection, false) => "Añadir a colección",
        (ActionKind::AddWorkspace, true) => "Add to workspace",
        (ActionKind::AddWorkspace, false) => "Añadir a espacio",
        (ActionKind::SetReadingState, true) => "Reading state",
        (ActionKind::SetReadingState, false) => "Estado de lectura",
        (ActionKind::SetPriority, true) => "Favorite",
        (ActionKind::SetPriority, false) => "Favorito",
        (ActionKind::AddReadingQueue, true) => "Add to reading queue",
        (ActionKind::AddReadingQueue, false) => "Añadir a cola de lectura",
    }
}
impl YamantakaApp {
    pub(in crate::app) fn automation_rules_view(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let width = ui.available_width();
        let margin = if width < 700.0 { 16 } else { 40 };
        let content = (width - 2.0 * margin as f32).clamp(1.0, 1100.0);
        let flow = || {
            if content < 620.0 {
                egui::Layout::top_down(egui::Align::Min)
            } else {
                egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true)
            }
        };
        egui::ScrollArea::vertical()
            .id_salt("saved-automation-rules")
            .auto_shrink([false, false])
            .animated(!self.reduced_motion)
            .show(ui, |ui| {
                egui::Frame::new().inner_margin(margin).show(ui, |ui| {
                    ui.set_width(content);
                    ui.set_max_width(content);
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), 0.0),
                        flow(),
                        |ui| {
                            theme::display(
                                ui,
                                tr("Reglas guardadas", "Saved rules"),
                                if content < 620.0 { 28.0 } else { 38.0 },
                            );
                            if theme::navigation(
                                ui,
                                tr("Volver a la biblioteca", "Back to library"),
                                false,
                                p,
                            )
                            .clicked()
                            {
                                self.show_surface(Surface::Library);
                            }
                        },
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(tr(
                            "Condiciones y acciones para organizar tu biblioteca local.",
                            "Conditions and actions for organizing your local library.",
                        ))
                        .color(p.secondary),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(tr(
                            "Simula, revisa las acciones y confirma antes de cambiar tu biblioteca.",
                            "Simulate, review the actions, and confirm before changing your library.",
                        ))
                        .size(12.0)
                        .color(p.secondary),
                    );
                    ui.add_space(24.0);
                    if let Some(batch)=&self.importing.last_batch {
                        let count=batch.len();
                        if count>0 {
                            ui.label(format!("{}: {count}",tr("Documentos del último lote disponibles para simular", "Documents from the last batch available for simulation")));
                            if theme::navigation(ui,tr("Olvidar lote", "Forget batch"),false,p).clicked() {self.importing.last_batch=None;}
                        }
                    }
                    self.rule_execution_view(ui,p);
                    self.rule_journal_view(ui,p);
                    if !self.legacy_recovery_view(ui,p) && !self.rule_review_view(ui, p) {
                        self.rule_simulation_view(ui, p);
                    }
                    let busy = self.automation.busy();
                    if let Some(error) = self.automation.error.clone() {
                        ui.label(error);
                        if theme::navigation(ui, tr("Recargar reglas", "Reload rules"), false, p)
                            .clicked()
                        {
                            self.load_automation_rules();
                        }
                    }
                    if self.automation.saved {
                        ui.label(
                            RichText::new(tr("Reglas actualizadas.", "Rules updated."))
                                .color(p.secondary),
                        );
                    }
                    if self.automation.editor.is_none()
                        && ui
                            .add_enabled_ui(!busy, |ui| {
                                theme::control(
                                    ui,
                                    tr("Nueva regla", "New rule"),
                                    Some(theme::Icon::Add),
                                    false,
                                    true,
                                    p,
                                )
                            })
                            .inner
                            .clicked()
                    {
                        self.automation.editor = Some(RuleEditor::default());
                    }
                    self.automation_editor(ui, p, busy);
                    ui.add_space(24.0);
                    if let Some(page) = self.automation.page.clone() {
                        if page.total == 0 {
                            ui.label(tr(
                                "Todavía no hay reglas guardadas.",
                                "No saved rules yet.",
                            ));
                        }
                        let mut change = None;
                        let mut simulate = None;
                        for rule in &page.entries {
                            egui::Frame::new()
                                .fill(p.surface)
                                .corner_radius(8)
                                .inner_margin(16)
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(RichText::new(&rule.name).size(21.0).color(p.ink));
                                    ui.label(
                                        RichText::new(if rule.enabled {
                                            tr("Habilitada", "Enabled")
                                        } else {
                                            tr("Deshabilitada", "Disabled")
                                        })
                                        .size(12.0)
                                        .color(p.secondary),
                                    );
                                    if let Some(definition) = rule.definition() {
                                        ui.label(
                                            RichText::new(trigger_label(
                                                definition.trigger.kind,
                                                english,
                                            ))
                                            .color(p.secondary),
                                        );
                                        if let Some(value) = &definition.trigger.value {
                                            ui.label(value);
                                        }
                                        for action in &definition.actions {
                                            ui.label(format!(
                                                "{}: {}",
                                                action_label(action.kind, english),
                                                action.value
                                            ));
                                        }
                                    } else {
                                        ui.label(tr(
                                            "Este formato de regla requiere revisión.",
                                            "This saved rule format needs review.",
                                        ));
                                    }
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(ui.available_width(), 0.0),
                                        flow(),
                                        |ui| {
                                            if ui
                                                .add_enabled_ui(
                                                    !busy && rule.definition().is_some(),
                                                    |ui| {
                                                        theme::navigation(
                                                            ui,
                                                            tr("Simular", "Simulate"),
                                                            false,
                                                            p,
                                                        )
                                                    },
                                                )
                                                .inner
                                                .clicked()
                                            {
                                                simulate = Some(rule.clone());
                                            }
                                            if ui
                                                .add_enabled_ui(
                                                    !busy
                                                        && (rule.enabled
                                                            || rule.definition().is_some()),
                                                    |ui| {
                                                        theme::navigation(
                                                            ui,
                                                            if rule.enabled {
                                                                tr("Deshabilitar", "Disable")
                                                            } else {
                                                                tr("Habilitar", "Enable")
                                                            },
                                                            false,
                                                            p,
                                                        )
                                                    },
                                                )
                                                .inner
                                                .clicked()
                                            {
                                                change = Some(RuleChange::SetEnabled {
                                                    expected: rule.clone(),
                                                    enabled: !rule.enabled,
                                                });
                                            }
                                            if ui
                                                .add_enabled_ui(!busy, |ui| {
                                                    theme::navigation(
                                                        ui,
                                                        tr("Eliminar regla", "Delete rule"),
                                                        false,
                                                        p,
                                                    )
                                                })
                                                .inner
                                                .clicked()
                                            {
                                                self.automation.deleting = Some(rule.clone());
                                            }
                                        },
                                    );
                                    if self.automation.deleting.as_ref().is_some_and(|selected| {
                                        selected.revision.id == rule.revision.id
                                    }) {
                                        ui.label(tr(
                                            "Los documentos y el historial se conservan.",
                                            "Documents and history are kept.",
                                        ));
                                        let actions = ui
                                            .add_enabled_ui(!busy, |ui| {
                                                ui.allocate_ui_with_layout(
                                                    egui::vec2(ui.available_width(), 0.0),
                                                    flow(),
                                                    |ui| {
                                                        if theme::navigation(
                                                            ui,
                                                            tr(
                                                                "Confirmar eliminación",
                                                                "Confirm deletion",
                                                            ),
                                                            false,
                                                            p,
                                                        )
                                                        .clicked()
                                                        {
                                                            change = Some(RuleChange::Delete {
                                                                expected: rule.clone(),
                                                            });
                                                        }
                                                        if theme::navigation(
                                                            ui,
                                                            tr("Cancelar", "Cancel"),
                                                            false,
                                                            p,
                                                        )
                                                        .clicked()
                                                        {
                                                            self.automation.deleting = None;
                                                        }
                                                    },
                                                );
                                            })
                                            .response;
                                        if self.automation.preview == 10
                                            && self.automation.preview_mode == 2
                                        {
                                            actions.scroll_to_me(Some(egui::Align::Center));
                                        }
                                    }
                                });
                            ui.add_space(10.0);
                        }
                        if let Some(rule) = simulate {
                            self.simulate_automation_rule(rule);
                        }
                        if let Some(change) = change {
                            self.change_automation_rule(change);
                        }
                        if page.total > 80 || self.automation.offset > 0 {
                            ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), 0.0),
                                flow(),
                                |ui| {
                                    if ui
                                        .add_enabled_ui(!busy && self.automation.offset > 0, |ui| {
                                            theme::navigation(
                                                ui,
                                                tr("Reglas anteriores", "Previous rules"),
                                                false,
                                                p,
                                            )
                                        })
                                        .inner
                                        .clicked()
                                    {
                                        self.automation.offset =
                                            self.automation.offset.saturating_sub(80);
                                        self.load_automation_rules();
                                    }
                                    if ui
                                        .add_enabled_ui(
                                            !busy
                                                && (self.automation.offset as u64 + 80)
                                                    < page.total,
                                            |ui| {
                                                theme::navigation(
                                                    ui,
                                                    tr("Más reglas", "More rules"),
                                                    false,
                                                    p,
                                                )
                                            },
                                        )
                                        .inner
                                        .clicked()
                                    {
                                        self.automation.offset =
                                            self.automation.offset.saturating_add(80);
                                        self.load_automation_rules();
                                    }
                                },
                            );
                        }
                    }
                });
            });
    }
    fn automation_editor(&mut self, ui: &mut egui::Ui, p: Palette, busy: bool) {
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let Some(editor) = &mut self.automation.editor else {
            return;
        };
        let mut save = false;
        let mut cancel = false;
        egui::Frame::new()
            .fill(p.raised)
            .corner_radius(8)
            .inner_margin(16)
            .show(ui, |ui| {
                ui.add_enabled_ui(!busy, |ui| {
                    let label = ui.label(tr("Nombre de la regla", "Rule name"));
                    ui.add(
                        egui::TextEdit::singleline(&mut editor.name)
                            .desired_width(f32::INFINITY)
                            .char_limit(120),
                    )
                    .labelled_by(label.id);
                    let label = ui.label(tr("Condición", "Condition"));
                    egui::ComboBox::from_id_salt("automation-trigger")
                        .selected_text(trigger_label(editor.trigger, english))
                        .show_ui(ui, |ui| {
                            for kind in TriggerKind::ALL {
                                ui.selectable_value(
                                    &mut editor.trigger,
                                    kind,
                                    trigger_label(kind, english),
                                );
                            }
                        })
                        .response
                        .labelled_by(label.id);
                    if editor.trigger.needs_value() {
                        let label = ui.label(tr("Valor de la condición", "Condition value"));
                        ui.add(
                            egui::TextEdit::singleline(&mut editor.trigger_value)
                                .desired_width(f32::INFINITY)
                                .char_limit(32767),
                        )
                        .labelled_by(label.id);
                    }
                    ui.add_space(12.0);
                    let label = ui.label(tr("Acción", "Action"));
                    egui::ComboBox::from_id_salt("automation-action")
                        .selected_text(action_label(editor.action, english))
                        .show_ui(ui, |ui| {
                            for kind in ActionKind::ALL {
                                ui.selectable_value(
                                    &mut editor.action,
                                    kind,
                                    action_label(kind, english),
                                );
                            }
                        })
                        .response
                        .labelled_by(label.id);
                    let label = ui.label(tr("Valor de la acción", "Action value"));
                    ui.add(
                        egui::TextEdit::singleline(&mut editor.action_value)
                            .desired_width(f32::INFINITY)
                            .char_limit(32767),
                    )
                    .labelled_by(label.id);
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        if ui
                            .add_enabled_ui(editor.draft().validate().is_ok(), |ui| {
                                theme::control(
                                    ui,
                                    tr("Guardar regla", "Save rule"),
                                    Some(theme::Icon::Add),
                                    false,
                                    true,
                                    p,
                                )
                            })
                            .inner
                            .clicked()
                        {
                            save = true;
                        }
                        if theme::navigation(ui, tr("Cancelar edición", "Cancel editing"), false, p)
                            .clicked()
                        {
                            cancel = true;
                        }
                    });
                });
            });
        if save {
            let draft = self.automation.editor.as_ref().unwrap().draft();
            self.change_automation_rule(RuleChange::Create(draft));
        } else if cancel {
            self.automation.editor = None;
        }
    }
}
