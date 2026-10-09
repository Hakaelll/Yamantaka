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

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::{
        memory::{Cost, MemoryCategory, MemoryRepresentation},
        pdf_text::TextCharacter,
    };

    fn page(governor: &MemoryGovernor) -> Arc<PdfPageText> {
        let text = "e\u{301} 👩\u{200d}🔬\r\nsecond line";
        let characters = text
            .char_indices()
            .map(|(start, character)| TextCharacter {
                bytes: start as u32..(start + character.len_utf8()) as u32,
                region: None,
            })
            .collect();
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                4096,
            ))
            .unwrap();
        Arc::new(PdfPageText::from_reserved(0, text.into(), characters, lease).unwrap())
    }

    #[test]
    fn keyboard_ranges_preserve_graphemes_words_lines_and_reverse_anchor() {
        let governor = MemoryGovernor::default();
        let page = page(&governor);
        let inside = SelectionPosition::new(0, 1).unwrap();
        let mut gesture = SelectionGesture::Selected(inside, inside);
        apply_key(
            &page,
            &mut gesture,
            egui::Key::ArrowRight,
            egui::Modifiers::SHIFT,
        );
        assert_eq!(
            gesture
                .range()
                .unwrap()
                .page(&page)
                .unwrap()
                .unwrap()
                .text(),
            "e\u{301}"
        );
        gesture = SelectionGesture::Empty;
        assert!(apply_key(
            &page,
            &mut gesture,
            egui::Key::ArrowRight,
            egui::Modifiers::SHIFT
        ));
        assert_eq!(
            gesture
                .range()
                .unwrap()
                .page(&page)
                .unwrap()
                .unwrap()
                .text(),
            "e\u{301}"
        );
        apply_key(
            &page,
            &mut gesture,
            egui::Key::ArrowRight,
            egui::Modifiers::SHIFT,
        );
        apply_key(
            &page,
            &mut gesture,
            egui::Key::ArrowRight,
            egui::Modifiers::SHIFT,
        );
        assert_eq!(
            gesture
                .range()
                .unwrap()
                .page(&page)
                .unwrap()
                .unwrap()
                .text(),
            "e\u{301} 👩\u{200d}🔬"
        );
        apply_key(
            &page,
            &mut gesture,
            egui::Key::ArrowLeft,
            egui::Modifiers::SHIFT,
        );
        assert_eq!(gesture.range().unwrap().end().character(), 3);
        apply_key(&page, &mut gesture, egui::Key::End, egui::Modifiers::SHIFT);
        assert_eq!(
            gesture
                .range()
                .unwrap()
                .page(&page)
                .unwrap()
                .unwrap()
                .text(),
            "e\u{301} 👩\u{200d}🔬"
        );
        apply_key(&page, &mut gesture, egui::Key::End, egui::Modifiers::CTRL);
        apply_key(&page, &mut gesture, egui::Key::Home, egui::Modifiers::SHIFT);
        assert_eq!(
            gesture
                .range()
                .unwrap()
                .page(&page)
                .unwrap()
                .unwrap()
                .text(),
            "second line"
        );
        apply_key(&page, &mut gesture, egui::Key::Home, egui::Modifiers::CTRL);
        apply_key(
            &page,
            &mut gesture,
            egui::Key::ArrowRight,
            egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
        );
        assert_eq!(
            gesture
                .range()
                .unwrap()
                .page(&page)
                .unwrap()
                .unwrap()
                .text(),
            "e\u{301}"
        );
        apply_key(&page, &mut gesture, egui::Key::A, egui::Modifiers::CTRL);
        assert_eq!(
            gesture
                .range()
                .unwrap()
                .page(&page)
                .unwrap()
                .unwrap()
                .text(),
            page.text()
        );
    }

    #[test]
    fn focused_page_consumes_selection_keys_before_page_navigation_but_modal_and_other_focus_keep_events()
     {
        let governor = MemoryGovernor::default();
        let payload = page(&governor);
        let mut state = SelectionState {
            enabled: true,
            document: Some("study".into()),
            ..Default::default()
        };
        let cancel = state.pages.entry(0).or_default().begin("study", 0).unwrap();
        state.accept("study", 0, &cancel, Ok(payload));
        let ctx = egui::Context::default();
        let id = egui::Id::new(("select-pdf-text", "study", 0u32));
        let event = egui::Event::Key {
            key: egui::Key::ArrowRight,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::SHIFT,
        };
        for (focus, modal, consumed) in [
            (true, false, true),
            (true, true, false),
            (false, false, false),
        ] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events: vec![event.clone()],
                    ..Default::default()
                },
                |ui| {
                    ui.memory_mut(|memory| {
                        if focus {
                            memory.request_focus(id)
                        } else {
                            memory.surrender_focus(id)
                        }
                    });
                    state.keyboard_input(&ctx, modal);
                    assert_eq!(ui.input(|input| input.events.is_empty()), consumed);
                    if consumed {
                        assert!(crate::app::shortcuts::read(&ctx, false, true).is_empty());
                    }
                },
            );
            output.textures_delta.clear();
        }
        assert_eq!(state.range().unwrap().end().character(), 2);
    }

    #[test]
    fn copy_and_escape_work_with_page_focus_and_preserve_text_edit_and_modal_input() {
        let governor = MemoryGovernor::default();
        let mut state = SelectionState {
            enabled: true,
            document: Some("study".into()),
            ..Default::default()
        };
        let cancel = state.pages.entry(0).or_default().begin("study", 0).unwrap();
        state.accept("study", 0, &cancel, Ok(page(&governor)));
        let ctx = egui::Context::default();
        let id = egui::Id::new(("select-pdf-text", "study", 0u32));
        for modal in [true, false] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events: vec![egui::Event::Copy],
                    ..Default::default()
                },
                |ui| {
                    ui.memory_mut(|memory| memory.request_focus(id));
                    assert!(ctx.egui_wants_keyboard_input());
                    assert_eq!(state.edit_commands(&ctx, modal), !modal);
                    assert_eq!(ui.input(|input| input.events.is_empty()), !modal);
                },
            );
            output.textures_delta.clear();
        }
        let mut text = String::from("note");
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Copy],
                ..Default::default()
            },
            |ui| {
                ui.text_edit_singleline(&mut text).request_focus();
                assert!(!state.edit_commands(&ctx, false));
                assert!(ui.input(|input| {
                    input
                        .events
                        .iter()
                        .any(|event| matches!(event, egui::Event::Copy))
                }));
            },
        );
        output.textures_delta.clear();
        let start = SelectionPosition::new(0, 0).unwrap();
        let end = SelectionPosition::new(0, 2).unwrap();
        state.gesture = SelectionGesture::Selected(start, end);
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            },
            |ui| {
                ui.memory_mut(|memory| memory.request_focus(id));
                assert!(!state.edit_commands(&ctx, false));
                assert!(state.range().is_none());
                assert!(ui.input(|input| input.events.is_empty()));
            },
        );
        output.textures_delta.clear();
    }
}
