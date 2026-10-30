use super::ReadingTarget;
#[path = "history_preview.rs"]
mod history;
use crate::app::*;

#[derive(Default)]
pub(in crate::app) struct ShortcutPreview {
    phase: u8,
    close: bool,
    history: bool,
    primary_positions: bool,
    pending: Vec<egui::Event>,
    reference_zoom: f32,
    primary_zoom: f32,
    selected: usize,
    reference_page: u32,
}

impl ShortcutPreview {
    pub fn ready(&self) -> bool {
        matches!(self.phase, 0 | 14 | 16 | 34)
    }
    pub fn phase(&self) -> u8 {
        self.phase
    }
    pub fn input(&mut self, input: &mut egui::RawInput) {
        input.events.append(&mut self.pending);
    }
    fn key(&mut self, phase: u8, key: egui::Key, modifiers: egui::Modifiers) {
        self.phase = phase;
        for pressed in [true, false] {
            self.pending.push(egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed,
                repeat: false,
                modifiers,
            });
        }
    }
}

impl YamantakaApp {
    pub fn preview_reference_shortcuts(
        &mut self,
        close: bool,
        history: bool,
        primary_positions: bool,
    ) {
        if self.demo {
            self.reference_shortcuts_preview.phase = 1;
            self.reference_shortcuts_preview.close = close;
            self.reference_shortcuts_preview.history = history;
            self.reference_shortcuts_preview.primary_positions = primary_positions;
        }
    }

    pub(in crate::app) fn advance_reference_shortcuts(&mut self, ctx: &egui::Context) {
        if self.reference_shortcuts_preview.ready() {
            return;
        }
        let reference_ready = self.reference.as_ref().is_some_and(|reference| {
            !reference.busy()
                && page_ready(&reference.pane)
                && reference
                    .pane
                    .active
                    .as_ref()
                    .is_some_and(|(doc, _)| doc.id == "demo-1")
        });
        if self.reference_shortcuts_preview.phase >= 20 {
            self.advance_history_preview(ctx, reference_ready);
            return;
        }
        let preview = &mut self.reference_shortcuts_preview;
        match preview.phase {
            1 if reference_ready && matches!(self.reference_demo, 3 | 9) => {
                assert_eq!(self.reader.page, 0);
                if ctx.content_rect().width() < 620.0 {
                    self.reader.panel = true;
                    self.reader.reader_panel_tab = outline::ReaderPanelTab::Text;
                }
                preview.primary_zoom = self.reader.zoom;
                preview.key(2, egui::Key::F6, egui::Modifiers::NONE);
            }
            2 if self.reading_focus.reference() && self.reading_focus.canvas_focused(ctx) => {
                preview.key(3, egui::Key::ArrowRight, egui::Modifiers::NONE);
            }
            3 if reference_ready && self.reference.as_ref().unwrap().pane.page == 1 => {
                assert_eq!(self.reader.page, 0);
                preview.reference_zoom = self.reference.as_ref().unwrap().pane.viewport.scale;
                preview.key(4, egui::Key::Equals, egui::Modifiers::CTRL);
            }
            4 if reference_ready
                && (self.reference.as_ref().unwrap().pane.zoom
                    - (preview.reference_zoom * 1.2).clamp(0.2, 8.0))
                .abs()
                    < 0.0001 =>
            {
                assert_eq!(self.reader.zoom, preview.primary_zoom);
                preview.key(5, egui::Key::Num0, egui::Modifiers::CTRL);
            }
            5 if reference_ready
                && self
                    .reference
                    .as_ref()
                    .unwrap()
                    .pane
                    .reader_preferences
                    .as_ref()
                    .unwrap()
                    .zoom_mode
                    == ZoomMode::FitPage =>
            {
                preview.key(6, egui::Key::F, egui::Modifiers::CTRL);
            }
            6 if ctx.text_edit_focused()
                && self.reference.as_ref().is_some_and(|reference| {
                    reference.pane.panel
                        && reference.pane.reader_panel_tab == outline::ReaderPanelTab::Find
                }) =>
            {
                preview.pending.push(egui::Event::Text("e".into()));
                preview.key(7, egui::Key::ArrowLeft, egui::Modifiers::NONE);
            }
            7 if preview.pending.is_empty() => {
                assert_eq!(self.reference.as_ref().unwrap().pane.page, 1);
                assert_eq!(self.reader.page, 0);
                preview.key(8, egui::Key::Enter, egui::Modifiers::NONE);
            }
            8 if reference_ready && self.reference.as_ref().unwrap().pane.find.ready() => {
                let summary = self.reference.as_ref().unwrap().pane.find.capture_summary();
                assert!(summary["matches"].as_u64().unwrap() > 1);
                preview.selected = summary["selected"].as_u64().unwrap() as usize;
                preview.key(9, egui::Key::F3, egui::Modifiers::NONE);
            }
            9 if reference_ready
                && self.reference.as_ref().unwrap().pane.find.capture_summary()["selected"]
                    .as_u64()
                    .is_some_and(|index| index as usize != preview.selected) =>
            {
                assert!(self.reader.find.capture_summary().is_null());
                preview.key(10, egui::Key::F3, egui::Modifiers::SHIFT);
            }
            10 if reference_ready
                && self.reference.as_ref().unwrap().pane.find.capture_summary()["selected"]
                    .as_u64()
                    == Some(preview.selected as u64) =>
            {
                preview.reference_page = self.reference.as_ref().unwrap().pane.page;
                if ctx.content_rect().width() < 620.0 {
                    preview.key(11, egui::Key::F6, egui::Modifiers::NONE);
                } else {
                    let position = self
                        .reading_focus
                        .canvas_rect(ctx, ReadingTarget::Primary)
                        .unwrap()
                        .center();
                    preview.pending.push(egui::Event::PointerMoved(position));
                    for pressed in [true, false] {
                        preview.pending.push(egui::Event::PointerButton {
                            pos: position,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                    preview.phase = 11;
                }
            }
            11 if !self.reading_focus.reference() && !ctx.text_edit_focused() => {
                preview.key(12, egui::Key::ArrowRight, egui::Modifiers::NONE);
            }
            12 if self.reader.page == 1 && page_ready(&self.reader) => {
                assert_eq!(
                    self.reference.as_ref().unwrap().pane.page,
                    preview.reference_page
                );
                preview.primary_zoom = self.reader.viewport.scale;
                preview.reference_zoom = self.reference.as_ref().unwrap().pane.zoom;
                preview.key(13, egui::Key::Minus, egui::Modifiers::CTRL);
            }
            13 if page_ready(&self.reader)
                && (self.reader.zoom - (preview.primary_zoom / 1.2).clamp(0.2, 8.0)).abs()
                    < 0.0001 =>
            {
                assert_eq!(
                    self.reference.as_ref().unwrap().pane.zoom,
                    preview.reference_zoom
                );
                preview.key(15, egui::Key::F6, egui::Modifiers::NONE);
            }
            15 if self.reading_focus.reference() && self.reading_focus.canvas_focused(ctx) => {
                if preview.history {
                    preview.key(20, egui::Key::ArrowRight, egui::Modifiers::NONE);
                } else if preview.close {
                    preview.key(17, egui::Key::W, egui::Modifiers::CTRL);
                } else {
                    preview.phase = 14;
                }
            }
            17 if self.reference.is_none() && !self.reading_focus.reference() => {
                assert_eq!(self.reader.active.as_ref().unwrap().0.id, "demo-0");
                assert_eq!(self.reader.page, 1);
                preview.phase = 16;
            }
            _ => {}
        }
    }
}

fn page_ready(pane: &reader_pane::ReaderPane) -> bool {
    !pane.loading
        && pane.viewport.pages.get(&pane.page).is_some_and(|slot| {
            slot.image_key.as_ref().is_some_and(|key| {
                key.generation == pane.viewport.generation
                    && slot.requested.as_ref() == Some(key)
                    && slot.plan.ready(Some(key))
            })
        })
}
