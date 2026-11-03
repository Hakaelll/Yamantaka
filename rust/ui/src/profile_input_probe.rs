use eframe::egui::{self, Event, Modifiers, PointerButton, RawInput, Vec2};

fn wheel(immediate: bool, modifiers: Modifiers) -> Vec<(Vec2, f32)> {
    let ctx = egui::Context::default();
    let mut samples = Vec::new();
    for frame in 0..3 {
        let input = RawInput {
            time: Some(f64::from(frame) / 60.0),
            events: if frame == 0 {
                vec![Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: egui::vec2(0.0, -1.0),
                    phase: egui::TouchPhase::Move,
                    modifiers,
                }]
            } else {
                vec![]
            },
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            if immediate {
                ui.ctx().input_mut(|input| input.finish_scroll_smoothing());
            }
            samples.push(ui.input(|input| (input.smooth_scroll_delta(), input.zoom_delta())));
        });
        output.textures_delta.clear();
    }
    samples
}

fn drag(kinetic: bool) -> Vec<f32> {
    let ctx = egui::Context::default();
    let mut offsets = Vec::new();
    for frame in 0..7 {
        let pos = egui::pos2(
            50.0,
            if frame < 3 {
                150.0
            } else if frame == 3 {
                100.0
            } else {
                50.0
            },
        );
        let mut events = vec![Event::PointerMoved(pos)];
        if frame == 2 || frame == 5 {
            events.push(Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: frame == 2,
                modifiers: Modifiers::NONE,
            });
        }
        let mut output = ctx.run_ui(
            RawInput {
                time: Some(f64::from(frame) / 60.0),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(320.0, 240.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                let output = egui::ScrollArea::vertical()
                    .scroll_source(egui::containers::scroll_area::ScrollSource::ALL)
                    .kinetic_scrolling(kinetic)
                    .show(ui, |ui| {
                        ui.allocate_space(egui::vec2(280.0, 2000.0));
                    });
                offsets.push(output.state.offset.y);
            },
        );
        output.textures_delta.clear();
    }
    offsets
}

pub fn verify(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let standard = wheel(false, Modifiers::NONE);
    let ink = wheel(true, Modifiers::NONE);
    assert!(standard[0].0.y.abs() < 40.0 && standard[1].0.y.abs() > 0.0);
    assert_eq!(ink[0].0, egui::vec2(0.0, -40.0));
    assert!(
        ink[1..]
            .iter()
            .all(|(scroll, zoom)| *scroll == Vec2::ZERO && *zoom == 1.0)
    );
    let shift = wheel(true, Modifiers::SHIFT);
    assert_eq!(shift[0].0, egui::vec2(-40.0, 0.0));
    let zoom = wheel(
        true,
        Modifiers {
            ctrl: true,
            command: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(zoom[0].0, Vec2::ZERO);
    assert!((zoom[0].1 - (-0.2_f32).exp()).abs() < 0.00001);
    assert!(
        zoom[1..]
            .iter()
            .all(|(scroll, zoom)| *scroll == Vec2::ZERO && *zoom == 1.0)
    );
    let normal_drag = drag(true);
    let ink_drag = drag(false);
    assert!(
        normal_drag[6] > normal_drag[5],
        "Standard drag offsets: {normal_drag:?}"
    );
    assert!(ink_drag[4] > 0.0);
    assert_eq!(ink_drag[4], ink_drag[5]);
    assert_eq!(ink_drag[5], ink_drag[6]);
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "synthetic":true,"standardWheel":standard.iter().map(|(delta,zoom)|serde_json::json!([delta.x,delta.y,zoom])).collect::<Vec<_>>(),
            "immediateWheel":ink.iter().map(|(delta,zoom)|serde_json::json!([delta.x,delta.y,zoom])).collect::<Vec<_>>(),
            "shiftWheelImmediate":true,"controlZoomImmediate":true,
            "standardDragOffsets":normal_drag,"nonKineticDragOffsets":ink_drag
        }))?,
    )?;
    Ok(())
}
