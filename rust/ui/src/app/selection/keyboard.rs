use super::*;
use unicode_segmentation::UnicodeSegmentation;

impl SelectionState {
    pub(super) fn focused_page(&self, ctx: &egui::Context) -> Option<u32> {
        let document = self.document.as_deref()?;
        let focused = ctx.memory(|memory| memory.focused());
        self.pages
            .keys()
            .copied()
            .find(|page| focused == Some(egui::Id::new(("select-pdf-text", document, *page))))
    }
    pub(super) fn edit_commands(&mut self, ctx: &egui::Context, modal: bool) -> bool {
        if !self.enabled
            || modal
            || ctx.egui_wants_keyboard_input() && self.focused_page(ctx).is_none()
        {
            return false;
        }
        ctx.input_mut(|input| {
            let copy = input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Copy));
            if copy {
                input
                    .events
                    .retain(|event| !matches!(event, egui::Event::Copy));
            }
            if input.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                self.gesture = SelectionGesture::Empty;
            }
            copy
        })
    }
    pub(in crate::app) fn keyboard_input(&mut self, ctx: &egui::Context, modal: bool) {
        if !self.enabled || modal {
            return;
        }
        let Some(payload) = self
            .focused_page(ctx)
            .and_then(|page| self.pages.get(&page))
            .and_then(text::TextState::payload)
        else {
            return;
        };
        ctx.input_mut(|input| {
            input.events.retain(|event| {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    !apply_key(payload, &mut self.gesture, *key, *modifiers)
                } else {
                    true
                }
            });
        });
    }
}

fn apply_key(
    page: &PdfPageText,
    gesture: &mut SelectionGesture,
    key: egui::Key,
    modifiers: egui::Modifiers,
) -> bool {
    if modifiers.alt || modifiers.mac_cmd {
        return false;
    }
    if key == egui::Key::A && modifiers.ctrl && !modifiers.shift {
        let (Ok(start), Ok(end)) = (
            SelectionPosition::new(page.page(), 0),
            SelectionPosition::new(page.page(), page.characters().len()),
        ) else {
            return false;
        };
        *gesture = SelectionGesture::Selected(start, end);
        return true;
    }
    if !matches!(
        key,
        egui::Key::ArrowLeft | egui::Key::ArrowRight | egui::Key::Home | egui::Key::End
    ) {
        return false;
    }
    let zero = SelectionPosition::new(page.page(), 0).unwrap();
    let (anchor, extent) = match *gesture {
        SelectionGesture::Selected(anchor, extent) | SelectionGesture::Dragging(anchor, extent)
            if extent.page() == page.page() =>
        {
            (anchor, extent)
        }
        _ => (zero, zero),
    };
    let index = extent.character();
    if index > page.characters().len() {
        return false;
    }
    let byte = page
        .characters()
        .get(index)
        .map_or(page.text().len(), |character| {
            character.bytes.start as usize
        });
    let text = page.text();
    let Some(anchor) = snap_position(page, anchor, false) else {
        return false;
    };
    let next_byte = match key {
        egui::Key::Home if modifiers.ctrl => 0,
        egui::Key::End if modifiers.ctrl => text.len(),
        egui::Key::Home => text[..byte].rfind('\n').map_or(0, |index| index + 1),
        egui::Key::End => text[byte..].find('\n').map_or(text.len(), |index| {
            let end = byte + index;
            if text[..end].ends_with('\r') {
                end - 1
            } else {
                end
            }
        }),
        egui::Key::ArrowLeft if modifiers.ctrl => text
            .unicode_word_indices()
            .map(|(offset, _)| offset)
            .take_while(|offset| *offset < byte)
            .last()
            .unwrap_or(0),
        egui::Key::ArrowRight if modifiers.ctrl => text
            .unicode_word_indices()
            .map(|(offset, word)| offset + word.len())
            .find(|offset| *offset > byte)
            .unwrap_or(text.len()),
        egui::Key::ArrowLeft => text
            .grapheme_indices(true)
            .map(|(offset, _)| offset)
            .take_while(|offset| *offset < byte)
            .last()
            .unwrap_or(0),
        egui::Key::ArrowRight => text
            .grapheme_indices(true)
            .map(|(offset, _)| offset)
            .find(|offset| *offset > byte)
            .unwrap_or(text.len()),
        _ => return false,
    };
    let next = if next_byte == text.len() {
        page.characters().len()
    } else {
        match page
            .characters()
            .binary_search_by_key(&next_byte, |character| character.bytes.start as usize)
        {
            Ok(index) => index,
            Err(_) => return false,
        }
    };
    let Ok(next) = SelectionPosition::new(page.page(), next) else {
        return false;
    };
    if !modifiers.shift
        && !modifiers.ctrl
        && matches!(key, egui::Key::ArrowLeft | egui::Key::ArrowRight)
        && let Some(range) = gesture.range().filter(|range| !range.is_empty())
    {
        let edge = if key == egui::Key::ArrowLeft {
            range.start()
        } else {
            range.end()
        };
        let Some(edge) = snap_position(page, edge, key == egui::Key::ArrowRight) else {
            return false;
        };
        *gesture = SelectionGesture::Selected(edge, edge);
    } else {
        *gesture = SelectionGesture::Selected(if modifiers.shift { anchor } else { next }, next);
    }
    true
}

fn snap_position(
    page: &PdfPageText,
    position: SelectionPosition,
    forward: bool,
) -> Option<SelectionPosition> {
    if position.page() != page.page() {
        return Some(position);
    }
    if position.character() == page.characters().len() {
        return Some(position);
    }
    let byte = page.characters().get(position.character())?.bytes.start as usize;
    let boundary = if forward {
        page.text()
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .find(|index| *index >= byte)
            .unwrap_or(page.text().len())
    } else {
        page.text()
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .take_while(|index| *index <= byte)
            .last()
            .unwrap_or(0)
    };
    let character = if boundary == page.text().len() {
        page.characters().len()
    } else {
        page.characters()
            .binary_search_by_key(&boundary, |character| character.bytes.start as usize)
            .ok()?
    };
    SelectionPosition::new(page.page(), character).ok()
}
