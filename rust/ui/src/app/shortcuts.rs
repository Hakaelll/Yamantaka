use eframe::egui;
mod focus;
pub(super) use focus::{ReadingFocus, ReadingTarget};
mod preview;
mod routing;
pub(super) use preview::ShortcutPreview;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Shortcut {
    GlobalSearch,
    DocumentFind,
    PreviousMatch,
    NextMatch,
    PreviousPage,
    NextPage,
    SwitchPane,
    HistoryBack,
    HistoryForward,
    ZoomIn,
    ZoomOut,
    FitPage,
    Fullscreen,
    Focus,
    NextTab,
    PreviousTab,
    CloseTab,
    ReopenTab,
}

pub(super) fn read(
    ctx: &egui::Context,
    modal_open: bool,
    reader_active: bool,
    document_focused: bool,
) -> Vec<Shortcut> {
    let text_has_focus = ctx.egui_wants_keyboard_input() && !document_focused;
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
            if input.consume_key(egui::Modifiers::NONE, egui::Key::F6) {
                commands.push(Shortcut::SwitchPane);
            }
            if input.consume_key(egui::Modifiers::CTRL, egui::Key::Plus)
                || input.consume_key(egui::Modifiers::CTRL, egui::Key::Equals)
                || input.consume_key(
                    egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                    egui::Key::Plus,
                )
                || input.consume_key(
                    egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                    egui::Key::Equals,
                )
            {
                commands.push(Shortcut::ZoomIn);
            }
            if input.consume_key(egui::Modifiers::CTRL, egui::Key::Minus) {
                commands.push(Shortcut::ZoomOut);
            }
            if input.consume_key(egui::Modifiers::CTRL, egui::Key::Num0) {
                commands.push(Shortcut::FitPage);
            }
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
                if input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowLeft) {
                    commands.push(Shortcut::HistoryBack);
                }
                if input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowRight) {
                    commands.push(Shortcut::HistoryForward);
                }
                if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)
                    || input.consume_key(egui::Modifiers::NONE, egui::Key::PageDown)
                {
                    commands.push(Shortcut::NextPage);
                }
                if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)
                    || input.consume_key(egui::Modifiers::NONE, egui::Key::PageUp)
                {
                    commands.push(Shortcut::PreviousPage);
                }
            }
        }
        commands
    })
}
