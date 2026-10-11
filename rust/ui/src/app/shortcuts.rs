use eframe::egui;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Shortcut {
    GlobalSearch,
    DocumentFind,
    PreviousMatch,
    NextMatch,
    PreviousPage,
    NextPage,
    Fullscreen,
    NextTab,
    PreviousTab,
}

pub(super) fn read(ctx: &egui::Context, modal_open: bool, reader_active: bool) -> Vec<Shortcut> {
    let text_has_focus = ctx.egui_wants_keyboard_input();
    ctx.input_mut(|input| {
        let mut commands = Vec::new();
        if input.key_pressed(egui::Key::F11) {
            commands.push(Shortcut::Fullscreen);
        }
        // Modal controls keep their events, including Escape and focus navigation.
        if modal_open {
            return commands;
        }
        if input.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::F) {
            commands.push(Shortcut::GlobalSearch);
            return commands;
        }
        if reader_active {
            if input.consume_key(
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::Tab,
            ) {
                commands.push(Shortcut::PreviousTab);
            } else if input.consume_key(egui::Modifiers::CTRL, egui::Key::Tab) {
                commands.push(Shortcut::NextTab);
            }
            if input.consume_key(egui::Modifiers::CTRL, egui::Key::F) {
                commands.push(Shortcut::DocumentFind);
            }
            if input.consume_key(egui::Modifiers::SHIFT, egui::Key::F3) {
                commands.push(Shortcut::PreviousMatch);
            } else if input.consume_key(egui::Modifiers::NONE, egui::Key::F3) {
                commands.push(Shortcut::NextMatch);
            }
            if !text_has_focus {
                if input.key_pressed(egui::Key::ArrowRight) {
                    commands.push(Shortcut::NextPage);
                }
                if input.key_pressed(egui::Key::ArrowLeft) {
                    commands.push(Shortcut::PreviousPage);
                }
            }
        }
        commands
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    fn route(events: Vec<egui::Event>, modal: bool, reader: bool) -> Vec<Shortcut> {
        let ctx = egui::Context::default();
        let mut result = Vec::new();
        let mut output = ctx.run_ui(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |_| result = read(&ctx, modal, reader),
        );
        output.textures_delta.clear();
        result
    }

    #[test]
    fn tab_shortcuts_cycle_only_in_the_reader_and_leave_modal_focus_alone() {
        for (modifiers, expected) in [
            (egui::Modifiers::CTRL, Shortcut::NextTab),
            (
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                Shortcut::PreviousTab,
            ),
        ] {
            assert_eq!(
                route(vec![key(egui::Key::Tab, modifiers)], false, true),
                vec![expected]
            );
            assert!(route(vec![key(egui::Key::Tab, modifiers)], false, false).is_empty());
            assert!(route(vec![key(egui::Key::Tab, modifiers)], true, true).is_empty());
        }
    }

    #[test]
    fn modal_preserves_its_input_and_blocks_reader_and_search_commands() {
        let ctx = egui::Context::default();
        let events = vec![
            key(egui::Key::F, egui::Modifiers::CTRL | egui::Modifiers::SHIFT),
            key(egui::Key::F, egui::Modifiers::CTRL),
            key(egui::Key::F3, egui::Modifiers::SHIFT),
            key(egui::Key::F3, egui::Modifiers::NONE),
            key(egui::Key::ArrowRight, egui::Modifiers::NONE),
            key(egui::Key::ArrowLeft, egui::Modifiers::NONE),
            key(egui::Key::Escape, egui::Modifiers::NONE),
            key(egui::Key::Tab, egui::Modifiers::NONE),
        ];
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: events.clone(),
                ..Default::default()
            },
            |_| {
                let before = ctx.input(|input| input.events.clone());
                assert!(read(&ctx, true, true).is_empty());
                assert_eq!(ctx.input(|input| input.events.clone()), before);
            },
        );
        output.textures_delta.clear();
        assert_eq!(
            route(vec![key(egui::Key::F11, egui::Modifiers::NONE)], true, true),
            vec![Shortcut::Fullscreen]
        );
    }

    #[test]
    fn document_shortcuts_require_reader_and_global_search_takes_precedence() {
        for (key_code, modifiers, expected) in [
            (egui::Key::F, egui::Modifiers::CTRL, Shortcut::DocumentFind),
            (
                egui::Key::F3,
                egui::Modifiers::SHIFT,
                Shortcut::PreviousMatch,
            ),
            (egui::Key::F3, egui::Modifiers::NONE, Shortcut::NextMatch),
            (
                egui::Key::ArrowLeft,
                egui::Modifiers::NONE,
                Shortcut::PreviousPage,
            ),
            (
                egui::Key::ArrowRight,
                egui::Modifiers::NONE,
                Shortcut::NextPage,
            ),
        ] {
            assert!(route(vec![key(key_code, modifiers)], false, false).is_empty());
            assert_eq!(
                route(vec![key(key_code, modifiers)], false, true),
                vec![expected]
            );
        }
        for reader in [false, true] {
            assert_eq!(
                route(
                    vec![key(
                        egui::Key::F,
                        egui::Modifiers::CTRL | egui::Modifiers::SHIFT
                    )],
                    false,
                    reader
                ),
                vec![Shortcut::GlobalSearch]
            );
        }
    }

    #[test]
    fn focused_text_input_keeps_arrow_keys_without_disabling_find() {
        let ctx = egui::Context::default();
        let mut text = String::from("study");
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.text_edit_singleline(&mut text).request_focus();
        });
        output.textures_delta.clear();
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![
                    key(egui::Key::ArrowLeft, egui::Modifiers::NONE),
                    key(egui::Key::F, egui::Modifiers::CTRL),
                ],
                ..Default::default()
            },
            |ui| {
                assert_eq!(read(&ctx, false, true), vec![Shortcut::DocumentFind]);
                ui.text_edit_singleline(&mut text);
            },
        );
        output.textures_delta.clear();
    }
}
