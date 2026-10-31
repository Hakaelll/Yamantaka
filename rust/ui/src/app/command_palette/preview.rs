use super::*;
#[derive(Default)]
pub(in crate::app) struct PalettePreview {
    phase: u8,
    mode: u8,
    events: Vec<egui::Event>,
    wanted: Option<String>,
    checked: bool,
}
impl PalettePreview {
    pub fn ready(&self) -> bool {
        matches!(self.phase, 0 | 90)
    }
    pub fn input(&mut self, input: &mut egui::RawInput) {
        input.events.append(&mut self.events);
    }
    fn key(&mut self, phase: u8, key: egui::Key, mut modifiers: egui::Modifiers) {
        self.phase = phase;
        eprintln!("Palette preview: phase {phase}");
        modifiers.command = modifiers.ctrl;
        for pressed in [true, false] {
            self.events.push(egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed,
                repeat: false,
                modifiers,
            });
        }
    }
    fn text(&mut self, phase: u8, value: &str) {
        self.phase = phase;
        eprintln!("Palette preview: phase {phase}");
        self.events.push(egui::Event::Text(value.into()));
    }
}
impl YamantakaApp {
    pub fn preview_command_palette(&mut self, mode: u8) {
        if self.demo && self.capture.is_some() {
            self.command_palette_preview.phase = 1;
            self.command_palette_preview.mode = mode;
        }
    }
    pub(in crate::app) fn advance_palette_preview(&mut self, ctx: &egui::Context) {
        let phase = self.command_palette_preview.phase;
        let mode = self.command_palette_preview.mode;
        if self.command_palette_preview.ready() {
            return;
        }
        let reference_ready = self.reference.as_ref().is_some_and(|reference| {
            !reference.busy()
                && !reference.pane.loading
                && reference
                    .pane
                    .active
                    .as_ref()
                    .is_some_and(|(document, _)| document.id == "demo-1")
        });
        if phase == 1 && self.library_loaded {
            if mode == 1 {
                if !reference_ready || self.reference_demo != 3 {
                    return;
                }
                self.command_palette_preview
                    .key(3, egui::Key::F6, egui::Modifiers::NONE);
            } else if mode == 2 {
                self.query = "no_such_catalogue_title_941".into();
                self.refresh();
                self.command_palette_preview.phase = 2;
            } else if self.reader.active.is_some() && !self.reader.loading {
                self.command_palette_preview
                    .key(4, egui::Key::K, egui::Modifiers::CTRL);
            }
            return;
        }
        if phase == 2 && self.library_loaded {
            assert!(self.documents.is_empty());
            self.command_palette_preview
                .key(4, egui::Key::K, egui::Modifiers::CTRL);
            return;
        }
        if phase == 3 && self.reading_focus.reference() && self.reading_focus.canvas_focused(ctx) {
            self.command_palette_preview
                .key(4, egui::Key::K, egui::Modifiers::CTRL);
            return;
        }
        let open_ready = self.command_palette.open
            && !self.command_palette.focus
            && self.command_palette.pending.is_none();
        if phase == 4 && open_ready {
            assert!(!self.command_palette.error);
            assert_eq!(self.command_palette.recents.len(), 8);
            let query = match mode {
                0 => "pagina".to_owned(),
                1 => "ultima pagina".to_owned(),
                2 => {
                    let document = &self.command_palette.recents[0];
                    self.command_palette_preview.wanted = Some(document.id.clone());
                    document.display_title().to_owned()
                }
                _ => "zqx_no_matching_action_846".to_owned(),
            };
            self.command_palette_preview.text(5, &query);
            return;
        }
        if phase == 5 && !self.command_palette.query.is_empty() {
            let entries = self.palette_entries();
            if mode == 0 {
                assert!(entries.iter().any(|entry| entry.id == "lastPage"));
                assert!(entries.len() > 3);
                self.command_palette_preview.phase = 90;
            } else if mode == 3 {
                assert!(entries.is_empty());
                if self.command_palette_preview.checked {
                    self.command_palette_preview.phase = 90;
                } else {
                    self.command_palette_preview.key(
                        7,
                        egui::Key::ArrowDown,
                        egui::Modifiers::NONE,
                    );
                    self.command_palette_preview
                        .key(7, egui::Key::Enter, egui::Modifiers::NONE);
                }
            } else {
                assert!(!entries.is_empty());
                self.command_palette_preview
                    .key(6, egui::Key::Enter, egui::Modifiers::NONE);
            }
            return;
        }
        if phase == 7 {
            assert!(self.command_palette.open);
            assert_eq!(self.reader.page, 0);
            self.command_palette_preview
                .key(8, egui::Key::Escape, egui::Modifiers::NONE);
            return;
        }
        if phase == 8 && !self.command_palette.open {
            self.command_palette_preview.checked = true;
            self.command_palette_preview
                .key(4, egui::Key::K, egui::Modifiers::CTRL);
            return;
        }
        if phase == 6 && !self.command_palette.open {
            if mode == 2 {
                if self.reader.loading
                    || !self.reader.active.as_ref().is_some_and(|(document, _)| {
                        Some(&document.id) == self.command_palette_preview.wanted.as_ref()
                    })
                {
                    return;
                }
                assert!(matches!(self.surface, Surface::Reader));
                self.command_palette_preview.phase = 90;
            } else if reference_ready && self.reference.as_ref().unwrap().pane.page == 1 {
                assert_eq!(self.reader.page, 0);
                self.command_palette_preview
                    .key(10, egui::Key::K, egui::Modifiers::CTRL);
            }
            return;
        }
        for (waiting, typed, query) in [
            (10, 11, "guardar posicion a"),
            (13, 14, "primera pagina"),
            (16, 17, "ir a posicion a"),
            (19, 20, "intercambiar posiciones"),
            (22, 23, "documentinfo"),
        ] {
            if phase == waiting && open_ready {
                self.command_palette_preview.text(typed, query);
                return;
            }
            if phase == typed && catalog::fold(&self.command_palette.query) == query {
                assert_eq!(self.palette_entries().len(), 1);
                self.command_palette_preview.key(
                    typed + 1,
                    egui::Key::Enter,
                    egui::Modifiers::NONE,
                );
                return;
            }
        }
        if phase == 12 && !self.command_palette.open {
            assert_eq!(
                self.reference
                    .as_ref()
                    .unwrap()
                    .pane
                    .navigation
                    .as_ref()
                    .unwrap()
                    .mark(MarkSlot::A)
                    .unwrap()
                    .page(),
                1
            );
            self.command_palette_preview
                .key(13, egui::Key::K, egui::Modifiers::CTRL);
        }
        if phase == 15
            && !self.command_palette.open
            && reference_ready
            && self.reference.as_ref().unwrap().pane.page == 0
        {
            assert_eq!(self.reader.page, 0);
            self.command_palette_preview
                .key(16, egui::Key::K, egui::Modifiers::CTRL);
        }
        if phase == 18
            && !self.command_palette.open
            && reference_ready
            && self.reference.as_ref().unwrap().pane.page == 1
        {
            self.command_palette_preview
                .key(19, egui::Key::K, egui::Modifiers::CTRL);
        }
        if phase == 21 && !self.command_palette.open {
            let nav = self
                .reference
                .as_ref()
                .unwrap()
                .pane
                .navigation
                .as_ref()
                .unwrap();
            assert!(nav.mark(MarkSlot::A).is_none());
            assert_eq!(nav.mark(MarkSlot::B).unwrap().page(), 1);
            self.command_palette_preview
                .key(22, egui::Key::K, egui::Modifiers::CTRL);
        }
        if phase == 24 && self.command_palette.info {
            assert_eq!(self.palette_document().unwrap().id, "demo-1");
            assert_eq!(self.reader.page, 0);
            assert_eq!(self.reference.as_ref().unwrap().pane.page, 1);
            self.command_palette_preview.phase = 90;
        }
    }
}
