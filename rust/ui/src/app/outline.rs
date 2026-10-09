use super::*;
use std::collections::HashSet;
use yamantaka_domain::outline::PdfOutline;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ReaderPanelTab {
    Outline,
    Bookmarks,
    Text,
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::{
        memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
        outline::OutlineEntry,
    };

    fn outline() -> Arc<PdfOutline> {
        let governor = MemoryGovernor::default();
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                4096,
            ))
            .unwrap();
        Arc::new(
            PdfOutline::from_reserved(
                vec![
                    OutlineEntry {
                        title: "Root".into(),
                        parent: None,
                        destination: None,
                    },
                    OutlineEntry {
                        title: "Child".into(),
                        parent: Some(0),
                        destination: None,
                    },
                    OutlineEntry {
                        title: "Nested".into(),
                        parent: Some(1),
                        destination: None,
                    },
                    OutlineEntry {
                        title: "Next".into(),
                        parent: None,
                        destination: None,
                    },
                ],
                lease,
            )
            .unwrap(),
        )
    }

    #[test]
    fn stale_and_cancelled_replies_cannot_replace_current_contents() {
        let current = Cancellation::default();
        let mut state = OutlineState {
            document: Some("current".into()),
            pending: Some(current.clone()),
            outline: None,
            error: None,
            collapsed: HashSet::new(),
            rows: Vec::new(),
        };
        state.accept("old", &current, Ok(outline()));
        state.accept("current", &Cancellation::default(), Ok(outline()));
        assert!(!state.ready());
        state.accept("current", &current, Ok(outline()));
        assert!(state.ready());
        assert_eq!(state.rows.len(), 4);
        let cancelled = Cancellation::default();
        cancelled.cancel();
        state.pending = Some(cancelled.clone());
        state.accept("current", &cancelled, Err("Cancelled".into()));
        assert!(state.error.is_none());
    }

    #[test]
    fn folding_preserves_nested_choices_and_other_roots() {
        let mut state = OutlineState {
            outline: Some(outline()),
            document: None,
            pending: None,
            error: None,
            collapsed: HashSet::new(),
            rows: Vec::new(),
        };
        state.rebuild_rows();
        assert_eq!(
            state.rows.iter().map(|row| row.depth).collect::<Vec<_>>(),
            [0, 1, 2, 0]
        );
        state.collapsed.insert(1);
        state.collapsed.insert(0);
        state.rebuild_rows();
        assert_eq!(
            state.rows.iter().map(|row| row.entry).collect::<Vec<_>>(),
            [0, 3]
        );
        state.collapsed.remove(&0);
        state.rebuild_rows();
        assert_eq!(
            state.rows.iter().map(|row| row.entry).collect::<Vec<_>>(),
            [0, 1, 3]
        );
        assert!(state.rows[0].children && state.rows[1].children);
    }

    #[test]
    fn dropping_contents_cancels_its_pending_read() {
        let cancel = Cancellation::default();
        drop(OutlineState {
            pending: Some(cancel.clone()),
            document: None,
            outline: None,
            error: None,
            collapsed: HashSet::new(),
            rows: Vec::new(),
        });
        assert!(cancel.is_cancelled());
    }
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
            self.panel = true;
            self.reader_panel_tab = ReaderPanelTab::Outline;
            self.outline_preview_target = target;
        }
    }
    pub(super) fn apply_outline_preview(&mut self) {
        if !self.outline.ready() {
            return;
        }
        if let Some(index) = self.outline_preview_target.take()
            && let Some(destination) = self
                .outline
                .outline
                .as_ref()
                .and_then(|outline| outline.entries().get(index))
                .and_then(|entry| entry.destination)
        {
            self.choose_outline_destination(destination, false);
        }
    }

    fn choose_outline_destination(
        &mut self,
        destination: yamantaka_domain::outline::OutlineDestination,
        compact: bool,
    ) {
        if let Some(document) = self.outline.document.clone() {
            self.outline_destination = Some((document, destination));
            if compact {
                self.panel = false;
            }
        }
    }

    pub(super) fn request_outline(&mut self, document: &str) {
        if self.outline.document.as_deref() != Some(document) {
            self.outline = OutlineState::default();
            self.outline.document = Some(document.into());
        }
        if self.outline.pending.is_some()
            || self.outline.outline.is_some()
            || self.outline.error.is_some()
        {
            return;
        }
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::Outline(document.into(), cancel.clone()))
        {
            Ok(()) => self.outline.pending = Some(cancel),
            Err(error) => self.outline.error = Some(error.to_string()),
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
        if let Some(error) = self.outline.error.clone() {
            ui.label(self.tr(
                "No se pudo cargar el índice.",
                "Contents could not be loaded.",
            ));
            ui.collapsing(self.tr("Detalles", "Details"), |ui| {
                ui.label(error);
            });
            if theme::navigation(ui, self.tr("Volver a intentar", "Try again"), false, p).clicked()
            {
                self.outline.error = None;
            }
            return;
        }
        let Some(outline) = self.outline.outline.clone() else {
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
            .show_rows(ui, row_height, self.outline.rows.len(), |ui, range| {
                for index in range {
                    let row = self.outline.rows[index];
                    let entry = &outline.entries()[row.entry];
                    ui.push_id(row.entry, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space((row.depth.min(6) * 12) as f32);
                            if row.children {
                                let collapsed = self.outline.collapsed.contains(&row.entry);
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
                                let selected = self.page == target.page;
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
            });
        if let Some(index) = toggle {
            if !self.outline.collapsed.remove(&index) {
                self.outline.collapsed.insert(index);
            }
            self.outline.rebuild_rows();
        }
        if let Some(destination) = destination {
            self.choose_outline_destination(destination, compact);
        }
    }

    pub(super) fn apply_outline_destination(
        &mut self,
        document: &Document,
        info: &PdfInfo,
        viewport: [f64; 2],
    ) {
        let Some((id, destination)) = self.outline_destination.take() else {
            return;
        };
        if id != document.id {
            return;
        }
        let result = (|| {
            let layout = self
                .reader_layout
                .as_ref()
                .ok_or_else(|| "Reader layout is unavailable".to_string())?;
            let navigation = self
                .navigation
                .as_ref()
                .ok_or_else(|| "Reader navigation is unavailable".to_string())?;
            let mut preferences = self
                .reader_preferences
                .clone()
                .ok_or_else(|| "Reader preferences are unavailable".to_string())?;
            let anchor = navigation.position();
            let space = info
                .pages
                .get(anchor.page() as usize)
                .ok_or_else(|| "Current page is unavailable".to_string())?
                .coordinates();
            let scroll = layout
                .scroll_for_anchor(
                    viewport,
                    anchor,
                    preferences.zoom_mode,
                    f64::from(self.zoom),
                )
                .map_err(|error| error.to_string())?;
            let frame = layout
                .frame(
                    viewport,
                    anchor,
                    preferences.zoom_mode,
                    f64::from(self.zoom),
                    scroll,
                    0.0,
                )
                .map_err(|error| error.to_string())?;
            let point = layout
                .pdf_point_at_viewport(&frame, anchor.page(), space)
                .map_err(|error| error.to_string())?;
            let resolved = yamantaka_domain::reader_destination::resolve(
                layout,
                destination,
                viewport,
                point,
                frame.zoom,
            )
            .map_err(|error| error.to_string())?;
            if !(resolved.zoom as f32).is_finite() || resolved.zoom as f32 <= 0.0 {
                return Err("Destination zoom cannot be represented by the reader".into());
            }
            preferences.zoom_mode = ZoomMode::Custom;
            preferences.zoom_level = resolved.zoom;
            preferences.updated_at = None;
            preferences.validate().map_err(|error| error.to_string())?;
            Ok((preferences, resolved))
        })();
        match result {
            Ok((preferences, resolved)) => {
                self.zoom = resolved.zoom as f32;
                self.reader_preferences = Some(preferences.clone());
                if let Some(navigation) = &mut self.navigation {
                    navigation.navigate_to(resolved.position);
                }
                self.apply_navigation();
                if !self.demo {
                    self.send(Action::SaveReaderPreferences(
                        document.id.clone(),
                        preferences,
                    ));
                }
            }
            Err(error) => self.error = Some(error),
        }
    }
}
