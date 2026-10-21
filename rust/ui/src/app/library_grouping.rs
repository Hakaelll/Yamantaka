use super::*;
use std::collections::HashSet;
use yamantaka_domain::LibraryMembership;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Grouping {
    #[default]
    None,
    Author,
    Collection,
    Tag,
    Reading,
}
impl Grouping {
    pub(super) fn key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Author => "author",
            Self::Collection => "collection",
            Self::Tag => "tag",
            Self::Reading => "readingState",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum GroupKey {
    Named(String),
    Missing,
    Reading(u8),
}
pub(super) struct GroupingState {
    pub mode: Grouping,
    pub(super) collapsed: HashSet<GroupKey>,
    pub(super) preview: Option<bool>,
    pub(super) verified: bool,
    pub memberships: Vec<LibraryMembership>,
}
impl GroupingState {
    pub(super) fn from_key(key: Option<&str>) -> Self {
        Self {
            mode: match key {
                Some("author") => Grouping::Author,
                Some("collection") => Grouping::Collection,
                Some("tag") => Grouping::Tag,
                Some("readingState") => Grouping::Reading,
                _ => Grouping::None,
            },
            collapsed: HashSet::new(),
            preview: None,
            verified: false,
            memberships: Vec::new(),
        }
    }
    pub(super) fn reset(&mut self) {
        self.mode = Grouping::None;
        self.collapsed.clear();
    }
}
pub(super) struct Group {
    pub(super) key: GroupKey,
    pub(super) label: String,
    pub(super) documents: Vec<Document>,
}
impl YamantakaApp {
    pub(super) fn library_groups(&self) -> Vec<Group> {
        let mut groups: Vec<Group> = Vec::new();
        for document in &self.documents {
            let member = self
                .grouping
                .memberships
                .iter()
                .find(|member| member.document_id == document.id);
            let (key, label) = match self.grouping.mode {
                Grouping::None => (GroupKey::Missing, String::new()),
                Grouping::Author => match document
                    .author
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    Some(author) => (GroupKey::Named(author.into()), author.into()),
                    None => (
                        GroupKey::Missing,
                        self.tr("Autor desconocido", "Unknown author").into(),
                    ),
                },
                Grouping::Collection | Grouping::Tag => {
                    let label = member.and_then(|member| {
                        if self.grouping.mode == Grouping::Collection {
                            member.collection.as_ref()
                        } else {
                            member.tag.as_ref()
                        }
                    });
                    match label {
                        Some(label) => (GroupKey::Named(label.clone()), label.clone()),
                        None => (
                            GroupKey::Missing,
                            if self.grouping.mode == Grouping::Collection {
                                self.tr("Sin colección", "No collection")
                            } else {
                                self.tr("Sin etiquetas", "Untagged")
                            }
                            .into(),
                        ),
                    }
                }
                Grouping::Reading => {
                    let (state, label) = if document.reading_progress >= 1.0 {
                        (2, self.tr("Terminados", "Completed"))
                    } else if document.reading_progress > 0.0 {
                        (1, self.tr("En lectura", "In progress"))
                    } else {
                        (0, self.tr("Por leer", "Not started"))
                    };
                    (GroupKey::Reading(state), label.into())
                }
            };
            if let Some(group) = groups.iter_mut().find(|group| group.key == key) {
                group.documents.push(document.clone());
            } else {
                groups.push(Group {
                    key,
                    label,
                    documents: vec![document.clone()],
                });
            }
        }
        groups
    }
    pub(super) fn library_visible_ids(&self) -> Vec<String> {
        self.library_groups()
            .into_iter()
            .filter(|group| {
                self.grouping.mode == Grouping::None
                    || !self.grouping.collapsed.contains(&group.key)
            })
            .flat_map(|group| group.documents.into_iter().map(|doc| doc.id))
            .collect()
    }
    pub(super) fn reconcile_group_selection(&mut self) {
        let visible = self.library_visible_ids();
        self.library_selection.model.retain(&visible);
        if self
            .selected
            .as_ref()
            .is_some_and(|id| !visible.contains(id))
        {
            self.selected = None;
        }
        if self
            .library_selection
            .focus
            .as_ref()
            .is_some_and(|id| !visible.contains(id))
        {
            self.library_selection.focus = None;
        }
    }
    pub(super) fn library_vertical_index(
        &self,
        page: &[String],
        index: usize,
        down: bool,
    ) -> usize {
        let columns = self.library_selection.columns.max(1);
        let mut rows: Vec<Vec<String>> = Vec::new();
        for group in self.library_groups().into_iter().filter(|group| {
            self.grouping.mode == Grouping::None || !self.grouping.collapsed.contains(&group.key)
        }) {
            for row in group.documents.chunks(columns) {
                rows.push(row.iter().map(|doc| doc.id.clone()).collect());
            }
        }
        for (row, ids) in rows.iter().enumerate() {
            if let Some(column) = ids.iter().position(|id| id == &page[index]) {
                let target = if down {
                    (row + 1).min(rows.len() - 1)
                } else {
                    row.saturating_sub(1)
                };
                let id = &rows[target][column.min(rows[target].len() - 1)];
                return page.iter().position(|entry| entry == id).unwrap_or(index);
            }
        }
        index
    }
    pub(super) fn library_grouping_options(&mut self, ui: &mut egui::Ui, p: Palette) {
        self.workspace_density_options(ui, p);
        ui.label(
            egui::RichText::new(self.tr("Agrupar por", "Group by"))
                .color(p.secondary)
                .size(12.0),
        );
        for (mode, es, en) in [
            (Grouping::None, "Sin agrupar", "No grouping"),
            (Grouping::Author, "Autor", "Author"),
            (Grouping::Collection, "Colección", "Collection"),
            (Grouping::Tag, "Etiqueta", "Tag"),
            (Grouping::Reading, "Estado de lectura", "Reading state"),
        ] {
            if theme::navigation(ui, self.tr(es, en), self.grouping.mode == mode, p).clicked() {
                self.grouping.mode = mode;
                self.grouping.collapsed.clear();
                self.persist_library_state();
                ui.close();
            }
        }
    }
    pub(super) fn grouped_library(&mut self, ui: &mut egui::Ui, compact: bool, p: Palette) {
        let groups = self.library_groups();
        self.grouping
            .collapsed
            .retain(|key| groups.iter().any(|group| &group.key == key));
        if self.grouping.mode == Grouping::None {
            if !self.list {
                ui.add_space(24.0);
            }
            self.library_document_page(ui, self.documents.clone(), compact, p);
            return;
        }
        let miniature = ui.ctx().content_rect().width() < 420.0;
        for group in groups {
            let mut collapsed = self.grouping.collapsed.contains(&group.key);
            ui.push_id(("library-group", &group.key), |ui| {
                ui.add_space(if miniature { 4.0 } else { 12.0 });
                let label = format!("{} · {}", group.label, group.documents.len());
                let galley = ui.painter().layout(
                    label,
                    egui::FontId::proportional(if miniature { 16.0 } else { 18.0 }),
                    p.ink,
                    (ui.available_width() - 38.0).max(1.0),
                );
                let (rect, response) =
                    ui.allocate_exact_size(
                        egui::vec2(
                            ui.available_width(),
                            (galley.size().y + if miniature { 8.0 } else { 16.0 })
                                .max(if miniature { 32.0 } else { 40.0 }),
                        ),
                        egui::Sense::click(),
                    );
                let action = self.tr(
                    if collapsed { "Desplegar" } else { "Plegar" },
                    if collapsed { "Expand" } else { "Collapse" },
                );
                let accessible = format!("{action} {} ({})", group.label, group.documents.len());
                theme::record_control_bounds(ui, &accessible, rect);
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::WidgetType::Button,
                        ui.is_enabled(),
                        !collapsed,
                        &accessible,
                    )
                });
                if response.hovered() {
                    ui.painter().rect_filled(rect, 6.0, p.surface);
                }
                if response.has_focus() {
                    ui.painter().rect_stroke(
                        rect,
                        6.0,
                        egui::Stroke::new(1.5, p.accent),
                        egui::StrokeKind::Inside,
                    );
                }
                let center = egui::pos2(rect.left() + 13.0, rect.center().y);
                let points = if collapsed {
                    vec![
                        center + egui::vec2(-2.0, -4.0),
                        center + egui::vec2(2.0, 0.0),
                        center + egui::vec2(-2.0, 4.0),
                    ]
                } else {
                    vec![
                        center + egui::vec2(-4.0, -2.0),
                        center + egui::vec2(0.0, 2.0),
                        center + egui::vec2(4.0, -2.0),
                    ]
                };
                ui.painter().add(egui::Shape::line(
                    points,
                    egui::Stroke::new(1.5, p.secondary),
                ));
                ui.painter().galley(
                    egui::pos2(rect.left() + 30.0, rect.center().y - galley.size().y / 2.0),
                    galley,
                    p.ink,
                );
                if response.on_hover_text(accessible).clicked() {
                    collapsed = !collapsed;
                    if collapsed {
                        self.grouping.collapsed.insert(group.key.clone());
                    } else {
                        self.grouping.collapsed.remove(&group.key);
                    }
                    self.reconcile_group_selection();
                }
                if !collapsed {
                    if !self.list {
                        ui.add_space(if miniature { 4.0 } else { 24.0 });
                    }
                    self.library_document_page(ui, group.documents, compact, p);
                }
                ui.add_space(8.0);
                ui.separator();
            });
        }
        ui.add_space(12.0);
    }
}
