use super::{Shortcut, bindings::Bindings, read};
use crate::app::*;
pub(in crate::app) fn verify_annotation_text_focus() {
    let ctx = egui::Context::default();
    let mut text = String::from("Draft note");
    let mut output = ctx.run_ui(Default::default(), |ui| {
        ui.add(egui::TextEdit::singleline(&mut text))
            .request_focus();
    });
    output.textures_delta.clear();
    let global_undo = Bindings::load(None)
        .changed(Shortcut::UndoAnnotation, "ctrl-shift-k")
        .unwrap()
        .changed(Shortcut::OpenDocument, "ctrl-z")
        .unwrap();
    for modifiers in [
        egui::Modifiers::CTRL,
        egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
    ] {
        let mut output = ctx.run_ui(events(egui::Key::Z, modifiers), |ui| {
            assert!(
                read(ui.ctx(), false, true, false, &Bindings::load(None)).is_empty(),
                "Annotation history must not steal text editing undo/redo"
            );
            assert!(
                read(ui.ctx(), false, true, false, &global_undo).is_empty(),
                "A rebound global action must also preserve text undo"
            );
            ui.add(egui::TextEdit::singleline(&mut text));
        });
        output.textures_delta.clear();
    }
}
fn events(key: egui::Key, modifiers: egui::Modifiers) -> egui::RawInput {
    egui::RawInput {
        events: vec![egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers,
        }],
        ..Default::default()
    }
}
fn routed(
    bindings: &Bindings,
    key: egui::Key,
    modifiers: egui::Modifiers,
    modal: bool,
    reader: bool,
) -> Vec<Shortcut> {
    let ctx = egui::Context::default();
    let mut commands = Vec::new();
    let mut output = ctx.run_ui(events(key, modifiers), |ui| {
        commands = read(ui.ctx(), modal, reader, true, bindings)
    });
    output.textures_delta.clear();
    commands
}
impl YamantakaApp {
    pub fn preview_binding_settings(&mut self, ctx: &egui::Context, mode: u8) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        let initial =
            serde_json::json!({"nextPage":"ctrl-right","futureAction":{"keep":7}}).to_string();
        let bindings = Bindings::load(Some(&initial.into()));
        assert!(!bindings.warning);
        let text_ctx = egui::Context::default();
        let mut text = String::from("Study note");
        let mut output = text_ctx.run_ui(Default::default(), |ui| {
            ui.add(egui::TextEdit::singleline(&mut text))
                .request_focus();
        });
        output.textures_delta.clear();
        let mut text_commands = Vec::new();
        let mut output =
            text_ctx.run_ui(events(egui::Key::ArrowRight, egui::Modifiers::CTRL), |ui| {
                text_commands = read(ui.ctx(), false, true, false, &bindings);
                ui.add(egui::TextEdit::singleline(&mut text));
            });
        output.textures_delta.clear();
        assert!(
            text_commands.is_empty(),
            "Page navigation must not consume text-editing keys"
        );
        let global_arrow = bindings
            .changed(Shortcut::GlobalSearch, "arrow-right")
            .unwrap_or_else(|_| panic!("The old page key is free"));
        let mut output =
            text_ctx.run_ui(events(egui::Key::ArrowRight, egui::Modifiers::NONE), |ui| {
                text_commands = read(ui.ctx(), false, true, false, &global_arrow);
                ui.add(egui::TextEdit::singleline(&mut text));
            });
        output.textures_delta.clear();
        assert!(
            text_commands.is_empty(),
            "Rebinding a global action must not steal text navigation"
        );

        assert_eq!(
            routed(
                &bindings,
                egui::Key::ArrowRight,
                egui::Modifiers::CTRL,
                false,
                true
            ),
            [Shortcut::NextPage]
        );
        assert!(
            routed(
                &bindings,
                egui::Key::ArrowRight,
                egui::Modifiers::NONE,
                false,
                true
            )
            .is_empty()
        );
        assert!(
            routed(
                &bindings,
                egui::Key::ArrowRight,
                egui::Modifiers::CTRL,
                true,
                true
            )
            .is_empty()
        );
        assert!(
            routed(
                &bindings,
                egui::Key::ArrowRight,
                egui::Modifiers::CTRL,
                false,
                false
            )
            .is_empty()
        );
        assert_eq!(
            routed(
                &bindings,
                egui::Key::PageDown,
                egui::Modifiers::NONE,
                false,
                true
            ),
            [Shortcut::NextPage]
        );
        assert_eq!(
            routed(
                &bindings,
                egui::Key::F,
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                false,
                true
            ),
            [Shortcut::GlobalSearch]
        );
        assert_eq!(
            routed(&bindings, egui::Key::F11, egui::Modifiers::NONE, true, true),
            [Shortcut::Fullscreen]
        );
        assert!(matches!(
            bindings.changed(Shortcut::NextPage, "ctrl-f"),
            Err(Shortcut::DocumentFind)
        ));
        assert!(matches!(
            bindings.changed(Shortcut::NextPage, "ctrl-shift-plus"),
            Err(Shortcut::ZoomIn)
        ));
        assert!(bindings.changed(Shortcut::NextPage, "page-down").is_err());
        let bad = Bindings::load(Some(&serde_json::json!("{\"nextPage\":\"ctrl-f\"}")));
        assert!(bad.warning);
        assert_eq!(bad.token(Shortcut::NextPage), "arrow-right");
        assert!(Bindings::load(Some(&serde_json::json!("malformed"))).warning);
        let encoded = bindings.reset().encoded();
        let restored: serde_json::Value = serde_json::from_str(encoded.as_str().unwrap()).unwrap();
        assert_eq!(restored["futureAction"]["keep"], 7);
        assert!(restored.get("nextPage").is_none());
        self.shortcut_bindings = bindings;
        self.assign_shortcut(Shortcut::SwitchPane, "f7", ctx);
        self.assign_shortcut(Shortcut::ZoomIn, "ctrl-d", ctx);
        let path = self.store.join("shortcut-verification.json");
        self.preferences.save(&path).unwrap();
        let loaded = Preferences::load(&path).unwrap();
        let bindings = Bindings::load(loaded.get("shortcutOverrides"));
        assert_eq!(bindings.token(Shortcut::NextPage), "ctrl-right");
        assert_eq!(bindings.token(Shortcut::ZoomIn), "ctrl-d");
        assert_eq!(bindings.token(Shortcut::SwitchPane), "f7");
        let raw: serde_json::Value =
            serde_json::from_str(loaded.get("shortcutOverrides").unwrap().as_str().unwrap())
                .unwrap();
        assert_eq!(raw["futureAction"]["keep"], 7);
        self.shortcut_bindings = bindings;
        if mode == 2 {
            self.reference_shortcuts_preview.custom = true;
            return;
        }
        if mode == 1 {
            self.assign_shortcut(Shortcut::NextPage, "ctrl-f", ctx);
            assert!(self.shortcut_error.is_some());
        }
        ctx.data_mut(|data| data.insert_temp(egui::Id::new("settings-category"), 5_usize));
        self.show_surface(Surface::Settings);
    }
}
