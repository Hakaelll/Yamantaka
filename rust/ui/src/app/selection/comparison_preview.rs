use super::*;

#[derive(Default)]
pub(in crate::app) struct ComparisonSelectionPreview {
    phase: u8,
    shared: bool,
    close: bool,
    primary: serde_json::Value,
    pending: Vec<egui::Event>,
}
impl ComparisonSelectionPreview {
    pub fn ready(&self) -> bool {
        matches!(self.phase, 0 | 10 | 14)
    }
    pub fn phase(&self) -> u8 {
        self.phase
    }
    pub fn input(&mut self, input: &mut egui::RawInput) {
        input.events.append(&mut self.pending);
    }
    fn key(&mut self, key: egui::Key, modifiers: egui::Modifiers) {
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
    pub fn preview_reference_selection(&mut self, shared: bool, close: bool) {
        if self.demo && self.capture.is_some() {
            self.comparison_selection_preview.phase = 1;
            self.comparison_selection_preview.shared = shared;
            self.comparison_selection_preview.close = close;
        }
    }
    pub(in crate::app) fn advance_comparison_selection(&mut self, ctx: &egui::Context) {
        if self.comparison_selection_preview.ready() {
            return;
        }
        let ready = self.reference.as_ref().is_some_and(|reference| {
            !reference.busy()
                && !reference.pane.loading
                && reference.pane.active.as_ref().is_some_and(|(doc, _)| {
                    doc.id
                        == if self.comparison_selection_preview.shared {
                            "demo-0"
                        } else {
                            "demo-1"
                        }
                })
        });
        if self.comparison_selection_preview.phase == 1
            && ready
            && (self.comparison_selection_preview.shared || matches!(self.reference_demo, 3 | 9))
        {
            self.preview_selection(true);
            self.comparison_selection_preview.phase = 15;
        }
        #[cfg(debug_assertions)]
        {
            let trace = format!(
                "selection phase={} main={:?}/{} reference={:?}/{} target={:?} panes={:?}/{:?} scale={}/{}",
                self.comparison_selection_preview.phase,
                self.selection_preview,
                self.reader.selection.capture_summary(),
                self.reference
                    .as_ref()
                    .map(|reference| reference.selection_preview),
                self.reference
                    .as_ref()
                    .map(|reference| reference.pane.selection.capture_summary())
                    .unwrap_or_default(),
                self.reading_focus.target,
                self.reading_focus
                    .canvas_rect(ctx, shortcuts::ReadingTarget::Primary),
                self.reading_focus
                    .canvas_rect(ctx, shortcuts::ReadingTarget::Reference),
                self.reader.viewport.scale,
                self.reference
                    .as_ref()
                    .map_or(0.0, |reference| reference.pane.viewport.scale)
            );
            let changed = ctx.data_mut(|data| {
                let id = egui::Id::new("comparison-selection-progress");
                let changed = data.get_temp::<String>(id).as_ref() != Some(&trace);
                data.insert_temp(id, trace.clone());
                changed
            });
            if changed {
                eprintln!("{trace}");
            }
        }
        let preview = &mut self.comparison_selection_preview;
        match preview.phase {
            15 if ready
                && self.selection_preview == SelectionPreview::Complete
                && self.reader.selection.preview_ready() =>
            {
                assert_eq!(
                    self.reader.selection.copy(&self.governor).unwrap().text(),
                    "Y"
                );
                preview.primary = self.reader.selection.capture_summary();
                let reference = self.reference.as_mut().unwrap();
                reference.set_zoom(ZoomMode::FitPage, 1.0);
                reference.set_page(if preview.shared { 0 } else { 1 });
                preview.phase = 2;
            }
            2 if ready => {
                let reference = self.reference.as_mut().unwrap();
                let document = &reference.pane.active.as_ref().unwrap().0.id;
                reference
                    .pane
                    .text
                    .request(&self.runtime, document, reference.pane.page);
                assert!(reference.pane.text.pending());
                reference.pane.toggle_selection();
                assert!(!reference.pane.text.pending());
                reference.selection_preview = SelectionPreview::Waiting;
                reference.selection_keyboard_preview = true;
                preview.phase = 3;
            }
            3 if ready
                && self.reference.as_ref().unwrap().selection_preview
                    == SelectionPreview::Complete
                && self
                    .reference
                    .as_ref()
                    .unwrap()
                    .pane
                    .selection
                    .preview_ready() =>
            {
                let reference = self.reference.as_ref().unwrap();
                assert!(self.reading_focus.reference());
                assert_eq!(self.reader.selection.capture_summary(), preview.primary);
                if preview.shared {
                    assert_ne!(
                        self.reader.selection.focus_ids[&0],
                        reference.pane.selection.focus_ids[&0]
                    );
                }
                preview.pending.push(egui::Event::Copy);
                preview.phase = 4;
            }
            4 if self
                .selection_copy_preview
                .as_ref()
                .is_some_and(|(_, text)| text == if preview.shared { "Y" } else { "M" }) =>
            {
                assert_eq!(
                    self.selection_copy_preview.as_ref().unwrap().0,
                    if preview.shared { "demo-0" } else { "demo-1" }
                );
                assert_eq!(self.reader.selection.capture_summary(), preview.primary);
                preview.key(egui::Key::Escape, egui::Modifiers::NONE);
                preview.phase = 5;
            }
            5 if self
                .reference
                .as_ref()
                .unwrap()
                .pane
                .selection
                .range()
                .is_none() =>
            {
                assert_eq!(self.reader.selection.capture_summary(), preview.primary);
                preview.key(egui::Key::A, egui::Modifiers::CTRL);
                preview.phase = 6;
            }
            6 if self
                .reference
                .as_ref()
                .unwrap()
                .pane
                .selection
                .range()
                .is_some_and(|range| !range.is_empty()) =>
            {
                preview.pending.push(egui::Event::Copy);
                preview.phase = 7;
            }
            7 if self
                .selection_copy_preview
                .as_ref()
                .is_some_and(|(_, text)| {
                    text.trim()
                        == if preview.shared {
                            "Yamantaka Rust: text, links and vector geometry"
                        } else {
                            "Mixed landscape page"
                        }
                }) =>
            {
                assert_eq!(self.reader.selection.capture_summary(), preview.primary);
                preview.key(egui::Key::ArrowLeft, egui::Modifiers::ALT);
                preview.phase = 8;
            }
            8 if ready
                && self.reference.as_ref().unwrap().pane.page
                    == if preview.shared { 1 } else { 0 } =>
            {
                assert_eq!(self.reader.selection.capture_summary(), preview.primary);
                preview.key(egui::Key::ArrowRight, egui::Modifiers::ALT);
                preview.phase = 9;
            }
            9 if ready
                && self.reference.as_ref().unwrap().pane.page
                    == if preview.shared { 0 } else { 1 } =>
            {
                assert_eq!(self.reader.selection.capture_summary(), preview.primary);
                if preview.close {
                    self.request_reference_close();
                    self.comparison_selection_preview.phase = 11;
                } else {
                    self.reference.as_mut().unwrap().selection_tools_preview = true;
                    preview.phase = 10;
                }
            }
            11 if self.reference.is_none()
                && !self.reading_focus.reference()
                && !self.reader.loading =>
            {
                assert_eq!(self.reader.selection.capture_summary(), preview.primary);
                preview.phase = 12;
            }
            12 if !self.reader.loading => {
                let id = self.reader.selection.focus_ids[&0];
                if ctx.read_response(id).is_some() {
                    ctx.memory_mut(|memory| memory.request_focus(id));
                    preview.pending.push(egui::Event::Copy);
                    preview.phase = 13;
                }
            }
            13 if self
                .selection_copy_preview
                .as_ref()
                .is_some_and(|(doc, text)| doc == "demo-0" && text == "Y") =>
            {
                assert_eq!(self.reader.selection.capture_summary(), preview.primary);
                preview.phase = 14;
            }
            _ => {}
        }
    }
}
