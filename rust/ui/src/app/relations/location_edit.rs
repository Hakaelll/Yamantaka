use super::*;
pub(super) struct Editor {
    expected: Relation,
    page: Option<u32>,
    pub(super) replace: bool,
}
impl YamantakaApp {
    pub(super) fn edit_relation_location(&mut self, relation: Relation, document: &str) {
        let (kind, id) = if relation.source == document {
            (&relation.source_anchor_kind, &relation.source_anchor_id)
        } else {
            (&relation.target_anchor_kind, &relation.target_anchor_id)
        };
        let page = if kind.as_deref() == Some("page") {
            id.as_deref().and_then(|id| id.parse().ok())
        } else {
            None
        };
        self.relations.removing = None;
        self.relations.editing = Some(Editor {
            expected: relation,
            page,
            replace: false,
        });
    }
    pub(super) fn relation_location_editor(
        &mut self,
        ui: &mut egui::Ui,
        relation: &Relation,
        context: &Document,
        p: Palette,
    ) -> Option<RelationChange> {
        if !self
            .relations
            .editing
            .as_ref()
            .is_some_and(|edit| edit.expected.id == relation.id)
        {
            return None;
        }
        let mut edit = self.relations.editing.take().unwrap();
        let busy = self.relations.writing();
        let mut save = false;
        let mut cancel = false;
        ui.add_space(12.0);
        ui.label(
            RichText::new(format!(
                "{} {}",
                self.tr("Ubicación en:", "Location in:"),
                context.display_title()
            ))
            .strong(),
        );
        let (kind, id) = if edit.expected.source == context.id {
            (
                &edit.expected.source_anchor_kind,
                &edit.expected.source_anchor_id,
            )
        } else {
            (
                &edit.expected.target_anchor_kind,
                &edit.expected.target_anchor_id,
            )
        };
        let current = match kind.as_deref() {
            Some("page") => id
                .as_deref()
                .and_then(|id| id.parse::<u32>().ok())
                .map(|page| {
                    format!(
                        "{} {}",
                        self.tr("Página guardada:", "Saved page:"),
                        page.saturating_add(1)
                    )
                })
                .unwrap_or_else(|| {
                    self.tr(
                        "Ubicación guardada no disponible",
                        "Saved location unavailable",
                    )
                    .into()
                }),
            Some("bookmark") => self.tr("Marcador guardado", "Saved bookmark").into(),
            Some("annotation") => self.tr("Anotación guardada", "Saved annotation").into(),
            None if id.is_none() => self.tr("Documento completo", "Whole document").into(),
            _ => self
                .tr("Ubicación guardada conservada", "Saved location preserved")
                .into(),
        };
        ui.label(RichText::new(current).color(p.secondary));
        ui.label(self.tr("El otro extremo se conserva. Guardar reemplaza solo la ubicación de este libro; desactiva Página para vincular el documento completo.","The other end is preserved. Saving replaces only this book's location; turn off Page to link the whole document."));
        ui.add_enabled_ui(!busy, |ui| {
            ui.checkbox(
                &mut edit.replace,
                self.tr(
                    "Reemplazar la ubicación guardada",
                    "Replace the saved location",
                ),
            );
            ui.add_enabled_ui(edit.replace, |ui| {
                super::view::page_choice(
                    ui,
                    self.tr("Página", "Page"),
                    &mut edit.page,
                    context.page_count,
                )
            });
            let actions = ui.horizontal_wrapped(|ui| {
                if theme::navigation(ui, self.tr("Cancelar edición", "Cancel edit"), false, p)
                    .clicked()
                {
                    cancel = true;
                }
                save = ui
                    .add_enabled_ui(edit.replace, |ui| {
                        theme::navigation(
                            ui,
                            self.tr("Guardar ubicación", "Save location"),
                            false,
                            p,
                        )
                    })
                    .inner
                    .clicked();
            });
            if self.relations.preview_locations >= 3 && self.relations.preview == 10 {
                actions.response.scroll_to_me(Some(egui::Align::Center));
            }
        });
        let change = save.then(|| RelationChange::UpdateLocation {
            document: context.id.clone(),
            expected: edit.expected.clone(),
            page: edit.page,
        });
        if !cancel {
            self.relations.editing = Some(edit);
        }
        change
    }
}
