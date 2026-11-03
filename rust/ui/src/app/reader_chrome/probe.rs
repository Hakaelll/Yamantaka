use super::*;

fn key(key: egui::Key, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

pub(crate) fn verify(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = egui::Context::default();
    let mut state = State::default();
    state.bind("synthetic-document");
    let mut first = None;
    let mut stable = true;
    let mut draw = |state: &mut State, events: Vec<egui::Event>, auto_hide: bool, blocked: bool| {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                let before = ui.available_rect_before_wrap();
                state.show(
                    ui,
                    "synthetic-controls",
                    Policy {
                        auto_hide,
                        blocked,
                        manual_hidden: false,
                    },
                    "Show reading controls",
                    Palette::new(true),
                    |ui| {
                        let response = ui.button("Next page");
                        first = Some(response.id);
                        Some(response.id)
                    },
                );
                stable &= before == ui.available_rect_before_wrap();
            },
        );
        output.textures_delta.clear();
    };
    for _ in 0..3 {
        draw(&mut state, vec![], true, false);
    }
    state.last_activity = Some(Instant::now() - Duration::from_secs(4));
    draw(&mut state, vec![], true, false);
    assert!(state.hidden);
    draw(&mut state, vec![], true, false);
    draw(&mut state, vec![key(egui::Key::Tab, true)], true, false);
    draw(
        &mut state,
        vec![key(egui::Key::Tab, false), key(egui::Key::Enter, true)],
        true,
        false,
    );
    assert!(!state.hidden, "Keyboard reveal failed");
    draw(&mut state, vec![key(egui::Key::Enter, false)], true, false);
    state.last_activity = Some(Instant::now() - Duration::from_secs(4));
    draw(&mut state, vec![], true, false);
    assert!(!state.hidden, "Focused controls were hidden");
    assert_eq!(ctx.memory(|memory| memory.focused()), first);
    if let Some(first) = first {
        ctx.memory_mut(|memory| memory.surrender_focus(first));
    }
    state.last_activity = Some(Instant::now() - Duration::from_secs(4));
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        state.advance(
            ui.ctx(),
            ui.available_rect_before_wrap(),
            &Policy {
                auto_hide: true,
                blocked: false,
                manual_hidden: false,
            },
        );
    });
    output.textures_delta.clear();
    assert!(state.hidden);
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: vec![egui::Event::PointerMoved(egui::pos2(30.0, 20.0))],
            ..Default::default()
        },
        |ui| {
            state.advance(
                ui.ctx(),
                ui.available_rect_before_wrap(),
                &Policy {
                    auto_hide: true,
                    blocked: false,
                    manual_hidden: false,
                },
            );
        },
    );
    output.textures_delta.clear();
    assert!(!state.hidden);
    for (auto_hide, blocked) in [(false, false), (true, true)] {
        state.last_activity = Some(Instant::now() - Duration::from_secs(4));
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::PointerGone],
                ..Default::default()
            },
            |ui| {
                state.advance(
                    ui.ctx(),
                    ui.available_rect_before_wrap(),
                    &Policy {
                        auto_hide,
                        blocked,
                        manual_hidden: false,
                    },
                );
            },
        );
        output.textures_delta.clear();
        assert!(!state.hidden);
    }
    state.hidden = true;
    state.bind("another-synthetic-document");
    assert!(!state.hidden);
    assert!(stable);
    std::fs::write(
        path,
        serde_json::to_vec_pretty(
            &serde_json::json!({"synthetic":true,"timeoutHides":true,"keyboardReveal":true,
        "focusRetained":true,"edgeReveal":true,"disabledKeepsVisible":true,"blockedKeepsVisible":true,"newDocumentReveals":true,"overlayPreservesParentRect":stable}),
        )?,
    )?;
    Ok(())
}
