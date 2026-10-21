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
    Focus,
    NextTab,
    PreviousTab,
    CloseTab,
    ReopenTab,
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
        if input.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::T) {
            commands.push(Shortcut::ReopenTab);
        }
        if reader_active {
            if input.consume_key(egui::Modifiers::NONE, egui::Key::F8) {
                commands.push(Shortcut::Focus);
            }
            if input.consume_key(egui::Modifiers::CTRL, egui::Key::W) {
                commands.push(Shortcut::CloseTab);
            }
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
