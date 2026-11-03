use eframe::egui::{self, Color32, FontId, Key, Modifiers, RawInput};
use egui::{epaint::text::cursor::CCursor, text::CCursorRange};
use serde_json::json;

fn key(key: Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

pub fn verify(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = egui::Context::default();
    crate::theme::load_fonts(&ctx);
    let mut records = Vec::new();
    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        let ctx = ui.ctx();
        let latin = "Hello, world! fi ffi 123 (x) - ok\tdone";
        let galley = ctx.fonts_mut(|fonts|fonts.layout_no_wrap(latin.into(),FontId::proportional(26.0),Color32::WHITE));
        records.push(json!({"text":latin,"glyphs":galley.rows[0].row.glyphs.iter().map(|g|json!([g.chr,g.pos.x,g.advance_width])).collect::<Vec<_>>(),"size":[galley.size().x,galley.size().y]}));
        for text in ["A\u{30a}ngstro\u{308}m", "ǻ fi ffi", "مرحبا\u{64e}"] {
            let galley = ctx.fonts_mut(|fonts|fonts.layout_no_wrap(text.into(),FontId::proportional(26.0),Color32::WHITE));
            assert_eq!(galley.rows[0].row.text(),text);
            assert_eq!(galley.rows[0].row.glyphs.len(),text.chars().count());
            records.push(json!({"text":text,"characters":text.chars().count(),"glyphs":galley.rows[0].row.glyphs.len()}));
        }
        for (text, expected) in [
            ("שלום 42 עולם", vec![11,10,9,8,7,5,6,4,3,2,1,0]),
            ("Hello שלום world", vec![0,1,2,3,4,5,9,8,7,6,10,11,12,13,14,15]),
            ("abc אבג def", vec![0,1,2,3,6,5,4,7,8,9,10]),
            ("عدد 123 مرحبا", vec![12,11,10,9,8,7,4,5,6,3,2,1,0]),
        ] {
            let galley = ctx.fonts_mut(|fonts| fonts.layout_no_wrap(text.into(),FontId::proportional(26.0),Color32::WHITE));
            assert_eq!(galley.rows.len(),1);
            let row = &galley.rows[0].row;
            assert_eq!(row.text(),text);
            assert_eq!(row.glyphs.len(),text.chars().count());
            let mut order: Vec<_> = (0..row.glyphs.len()).collect();
            order.sort_by(|&a,&b|row.glyphs[a].pos.x.total_cmp(&row.glyphs[b].pos.x));
            assert_eq!(order,expected,"Visual order differs for {text}");
            assert!(ctx.fonts_mut(|fonts|fonts.has_glyphs(&FontId::proportional(26.0),text)),"Missing native font glyphs for {text}");
            for (i,glyph) in row.glyphs.iter().enumerate().filter(|(_,g)|g.advance_width>0.0) {
                let rect = glyph.logical_rect();
                let left = galley.cursor_from_pos(egui::vec2(rect.lerp_inside([0.25,0.5]).x,row.size.y/2.0));
                let right = galley.cursor_from_pos(egui::vec2(rect.lerp_inside([0.75,0.5]).x,row.size.y/2.0));
                let (before,after) = if glyph.is_rtl() {(right,left)} else {(left,right)};
                assert_eq!(before.index.0,i,"Hit test before {text} character {i}");
                assert_eq!(after.index.0,i+1,"Hit test after {text} character {i}");
            }
            if text == "abc אבג def" {
                use egui::epaint::text::CharIndex;
                assert_eq!(row.x_ranges(CharIndex(0)..CharIndex(5)).len(),2);
            }
            records.push(json!({"text":text,"characters":text.chars().count(),"glyphs":row.glyphs.len(),"visualOrder":order,"hitTesting":true}));
        }
    });
    output.textures_delta.clear();
    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        let ctx = ui.ctx();
        let word = ctx.fonts_mut(|fonts| {
            fonts.layout_no_wrap("مرحبا".into(), FontId::proportional(26.0), Color32::WHITE)
        });
        let mixed = ctx.fonts_mut(|fonts| {
            fonts.layout_no_wrap(
                "שלום مرحبا".into(),
                FontId::proportional(26.0),
                Color32::WHITE,
            )
        });
        let advances = |glyphs: &[egui::epaint::text::Glyph]| {
            glyphs.iter().map(|g| g.advance_width).collect::<Vec<_>>()
        };
        assert_eq!(
            advances(&word.rows[0].row.glyphs),
            advances(&mixed.rows[0].row.glyphs[5..])
        );
        records.push(json!({"mixedArabicHebrewPreservesArabicJoining":true}));
        for text in ["abc אבג   xyz", "אבג abc   xyz"] {
            let width = ctx
                .fonts_mut(|fonts| {
                    fonts.layout_no_wrap(
                        text[..text.len() - 3].into(),
                        FontId::proportional(26.0),
                        Color32::WHITE,
                    )
                })
                .size()
                .x;
            let job = egui::epaint::text::LayoutJob::simple(
                text.into(),
                FontId::proportional(26.0),
                Color32::WHITE,
                width + 1.0,
            );
            let galley = ctx.fonts_mut(|fonts| fonts.layout_job(job));
            assert!(galley.rows.len() > 1);
            let first = &galley.rows[0].row;
            let expected_rtl = text.starts_with('א');
            let trailing_count = first
                .glyphs
                .iter()
                .rev()
                .take_while(|glyph| glyph.chr.is_whitespace())
                .count();
            assert!(
                trailing_count > 0,
                "The wrapping probe must include trailing whitespace"
            );
            for glyph in first
                .glyphs
                .iter()
                .rev()
                .take_while(|glyph| glyph.chr.is_whitespace())
            {
                assert_eq!(glyph.is_rtl(), expected_rtl);
            }
            records
                .push(json!({"wrappedText":text,"trailingWhitespaceUsesParagraphDirection":true}));
        }
    });
    output.textures_delta.clear();
    let id = egui::Id::new("bidi-edit-probe");
    let mut text = "عدد 123 مرحبا".to_owned();
    let mut state = egui::text_edit::TextEditState::default();
    state
        .cursor
        .set_char_range(Some(CCursorRange::one(CCursor::new(text.chars().count()))));
    state.store(&ctx, id);
    ctx.memory_mut(|memory| memory.request_focus(id));
    let mut output = ctx.run_ui(
        RawInput {
            events: vec![
                key(Key::End),
                key(Key::ArrowLeft),
                key(Key::ArrowLeft),
                key(Key::ArrowLeft),
                egui::Event::Text("x".into()),
            ],
            ..Default::default()
        },
        |ui| {
            egui::TextEdit::singleline(&mut text).id(id).show(ui);
        },
    );
    output.textures_delta.clear();
    assert_eq!(text, "عدد 123 مرxحبا");
    let mut state = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
    state.cursor.set_char_range(Some(CCursorRange::two(
        CCursor::new(0),
        CCursor::new(text.chars().count()),
    )));
    state.store(&ctx, id);
    let mut output = ctx.run_ui(
        RawInput {
            events: vec![egui::Event::Copy],
            ..Default::default()
        },
        |ui| {
            egui::TextEdit::singleline(&mut text).id(id).show(ui);
        },
    );
    assert!(output.platform_output.commands.iter().any(|command| matches!(command,egui::OutputCommand::CopyText(copied) if copied=="عدد 123 مرxحبا")));
    output.textures_delta.clear();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(
        path,
        serde_json::to_vec_pretty(
            &json!({"samples":records,"editorAfterLeftThreeAndInsert":text,"copyPreservesLogicalOrder":true}),
        )?,
    )?;
    Ok(())
}
