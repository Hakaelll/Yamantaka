use eframe::egui;
pub(super) mod bindings;
mod focus;
mod settings;
mod verification;
pub(super) use focus::{ReadingFocus, ReadingTarget};
mod preview;
mod routing;
pub(super) use preview::ShortcutPreview;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Shortcut {
    OpenDocument,
    ActualSize,
    FitWidth,
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
    bindings: &bindings::Bindings,
) -> Vec<Shortcut> {
    let text_focus = ctx.egui_wants_keyboard_input() && !document_focused;
    ctx.input_mut(|input| {
        let mut commands = Vec::new();
        input.events.retain(|event| {
            let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                repeat,
                ..
            } = event
            else {
                return true;
            };
            if text_focus
                && matches!(
                    key,
                    egui::Key::ArrowLeft
                        | egui::Key::ArrowRight
                        | egui::Key::PageUp
                        | egui::Key::PageDown
                )
            {
                return true;
            }
            for definition in bindings::DEFINITIONS {
                let command = definition.command;
                if modal_open && command != Shortcut::Fullscreen {
                    continue;
                }
                if !reader_active && !definition.global {
                    continue;
                }
                if text_focus && definition.canvas {
                    continue;
                }
                if *repeat && !definition.repeats {
                    continue;
                }
                if bindings::matches(bindings.token(command), *key, *modifiers) {
                    if !commands.contains(&command) {
                        commands.push(command);
                    }
                    return false;
                }
            }
            if !modal_open
                && reader_active
                && !text_focus
                && !modifiers.ctrl
                && !modifiers.alt
                && !modifiers.shift
                && !modifiers.mac_cmd
            {
                let command = match key {
                    egui::Key::PageDown => Some(Shortcut::NextPage),
                    egui::Key::PageUp => Some(Shortcut::PreviousPage),
                    _ => None,
                };
                if let Some(command) = command {
                    commands.push(command);
                    return false;
                }
            }
            true
        });
        commands
    })
}
