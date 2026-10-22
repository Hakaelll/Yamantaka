use super::*;
use std::collections::HashSet;
use yamantaka_domain::outline::PdfOutline;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ReaderPanelTab {
    Outline,
    Thumbnails,
    Find,
    Bookmarks,
    Annotations,
    Text,
}

#[derive(Clone, Copy)]
struct Row {
    entry: usize,
    depth: usize,
    children: bool,
}

#[derive(Default)]
pub(super) struct OutlineState {
    document: Option<String>,
    pending: Option<Cancellation>,
    outline: Option<Arc<PdfOutline>>,
    error: Option<String>,
    collapsed: HashSet<usize>,
    rows: Vec<Row>,
}

impl Drop for OutlineState {
    fn drop(&mut self) {
        if let Some(cancel) = &self.pending {
            cancel.cancel();
        }
    }
}

impl OutlineState {
    pub(super) fn ready(&self) -> bool {
        self.outline.is_some() && self.pending.is_none()
    }

    pub(super) fn accept(
        &mut self,
        document: &str,
        cancel: &Cancellation,
        result: Result<Arc<PdfOutline>, String>,
    ) {
        if cancel.is_cancelled()
            || self.document.as_deref() != Some(document)
            || !self
                .pending
                .as_ref()
                .is_some_and(|pending| pending.same_request(cancel))
        {
            return;
        }
        self.pending = None;
        match result {
            Ok(outline) => {
                self.outline = Some(outline);
                self.error = None;
                self.collapsed.clear();
                self.rebuild_rows();
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn rebuild_rows(&mut self) {
        self.rows.clear();
        let Some(outline) = &self.outline else { return };
        let entries = outline.entries();
        let mut depths = vec![0usize; entries.len()];
        let mut hidden = vec![false; entries.len()];
        let mut children = vec![false; entries.len()];
        for entry in entries {
            if let Some(parent) = entry.parent {
                children[parent] = true;
            }
        }
        for (index, entry) in entries.iter().enumerate() {
            if let Some(parent) = entry.parent {
                depths[index] = depths[parent] + 1;
                hidden[index] = hidden[parent] || self.collapsed.contains(&parent);
            }
            if !hidden[index] {
                self.rows.push(Row {
                    entry: index,
                    depth: depths[index],
                    children: children[index],
                });
            }
        }
    }
}

impl YamantakaApp {
    pub fn preview_outline(&mut self, target: Option<usize>) {
        if self.demo && self.capture.is_some() {
            self.reader.panel = true;
            self.reader.reader_panel_tab = ReaderPanelTab::Outline;
            self.outline_preview_target = target;
        }
    }
    pub(super) fn apply_outline_preview(&mut self) {
        if !self.reader.outline.ready() {
            return;
        }
        if let Some(index) = self.outline_preview_target.take()
            && let Some(destination) = self
                .reader
                .outline
                .outline
                .as_ref()
                .and_then(|outline| outline.entries().get(index))
                .and_then(|entry| entry.destination)
        {
            self.choose_reader_destination(destination, false);
        }
    }

    fn choose_reader_destination(
        &mut self,
        destination: yamantaka_domain::pdf_destination::NavigationDestination,
        compact: bool,
    ) {
        if let Some(document) = self.reader.outline.document.clone() {
            self.reader.reader_destination = Some((document, destination));
            if compact {
                self.reader.panel = false;
            }
        }
    }

    pub(super) fn request_outline(&mut self, document: &str) {
        if self.reader.outline.document.as_deref() != Some(document) {
            self.reader.outline = OutlineState::default();
            self.reader.outline.document = Some(document.into());
        }
        if self.reader.outline.pending.is_some()
            || self.reader.outline.outline.is_some()
            || self.reader.outline.error.is_some()
        {
            return;
        }
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::Outline(document.into(), cancel.clone()))
        {
            Ok(()) => self.reader.outline.pending = Some(cancel),
            Err(error) => self.reader.outline.error = Some(error.to_string()),
        }
    }

    pub(super) fn outline_panel_content(&mut self, ui: &mut egui::Ui, p: Palette) {
        let compact = ui.ctx().content_rect().width() < 620.0;
        if !compact {
            ui.label(
                RichText::new(self.tr("Explora el documento", "Explore this document")).size(22.0),
            );
            ui.add_space(6.0);
        }
        if let Some(error) = self.reader.outline.error.clone() {
            ui.label(self.tr(
                "No se pudo cargar el índice.",
                "Contents could not be loaded.",
            ));
            ui.collapsing(self.tr("Detalles", "Details"), |ui| {
                ui.label(error);
            });
            if theme::navigation(ui, self.tr("Volver a intentar", "Try again"), false, p).clicked()
            {
                self.reader.outline.error = None;
            }
            return;
        }
        let Some(outline) = self.reader.outline.outline.clone() else {
            ui.label(
                RichText::new(self.tr("Cargando capítulos…", "Loading chapters…"))
                    .color(p.secondary),
            );
            return;
        };
        if outline.entries().is_empty() {
            ui.label(
                RichText::new(self.tr(
                    "Este documento no tiene índice.",
                    "This document has no contents.",
                ))
                .color(p.secondary),
            );
            return;
        }
        if !compact {
            ui.label(
                RichText::new(self.tr(
                    "Elige un capítulo para continuar leyendo.",
                    "Choose a chapter to continue reading.",
                ))
                .color(p.secondary),
            );
            ui.add_space(14.0);
        }
        let row_height = 40.0;
        let mut toggle = None;
        let mut destination = None;
        egui::ScrollArea::vertical()
            .id_salt("outline-chapters")
            .show_rows(
                ui,
                row_height,
                self.reader.outline.rows.len(),
                |ui, range| {
                    for index in range {
                        let row = self.reader.outline.rows[index];
                        let entry = &outline.entries()[row.entry];
                        ui.push_id(row.entry, |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space((row.depth.min(6) * 12) as f32);
                                if row.children {
                                    let collapsed =
                                        self.reader.outline.collapsed.contains(&row.entry);
                                    let label = format!(
                                        "{} {}",
                                        self.tr(
                                            if collapsed { "Desplegar" } else { "Plegar" },
                                            if collapsed { "Expand" } else { "Collapse" }
                                        ),
                                        entry.title
                                    );
                                    if theme::icon_button(
                                        ui,
                                        &label,
                                        if collapsed {
                                            theme::Icon::Plus
                                        } else {
                                            theme::Icon::Minus
                                        },
                                        p,
                                    )
                                    .clicked()
                                    {
                                        toggle = Some(row.entry);
                                    }
                                }
                                if !row.children {
                                    ui.add_space(40.0 + ui.spacing().item_spacing.x);
                                }
                                let mut label = entry.title.clone();
                                if let Some(target) = entry.destination {
                                    label.push_str(&format!(" · {}", u64::from(target.page) + 1));
                                    let selected = self.reader.page == target.page;
                                    if ui
                                        .add_sized(
                                            [ui.available_width().max(40.0), row_height],
                                            egui::Button::new(
                                                RichText::new(&label).color(if selected {
                                                    p.ink
                                                } else {
                                                    p.secondary
                                                }),
                                            )
                                            .selected(selected)
                                            .stroke(egui::Stroke::NONE)
                                            .fill(if selected {
                                                p.raised
                                            } else {
                                                Color32::TRANSPARENT
                                            })
                                            .corner_radius(8)
                                            .wrap_mode(egui::TextWrapMode::Truncate),
                                        )
                                        .on_hover_text(&entry.title)
                                        .clicked()
                                    {
                                        destination = Some(target);
                                    }
                                } else {
                                    ui.add_sized(
                                        [ui.available_width().max(40.0), row_height],
                                        egui::Label::new(RichText::new(&label).color(p.secondary))
                                            .truncate(),
                                    )
                                    .on_hover_text(&entry.title);
                                }
                            });
                        });
                    }
                },
            );
        if let Some(index) = toggle {
            if !self.reader.outline.collapsed.remove(&index) {
                self.reader.outline.collapsed.insert(index);
            }
            self.reader.outline.rebuild_rows();
        }
        if let Some(destination) = destination {
            self.choose_reader_destination(destination, compact);
        }
    }
}
