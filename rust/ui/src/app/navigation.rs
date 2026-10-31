use super::*;

#[derive(Clone, Copy)]
pub(in crate::app) enum PositionAction {
    Swap,
    Back,
    Forward,
    Save(MarkSlot),
    Jump(MarkSlot),
}

impl PositionAction {
    pub fn apply(self, navigation: &mut ReaderNavigation) -> bool {
        match self {
            Self::Swap => {
                navigation.swap_marks();
                false
            }
            Self::Back => navigation.back(),
            Self::Forward => navigation.forward(),
            Self::Save(slot) => {
                navigation.capture_mark(slot);
                false
            }
            Self::Jump(slot) => navigation.jump_to_mark(slot),
        }
    }
}

pub(in crate::app) fn positions(
    ui: &mut egui::Ui,
    navigation: &ReaderNavigation,
    english: bool,
    p: Palette,
) -> Option<PositionAction> {
    let tr = |es, en| if english { en } else { es };
    let mut action = None;
    ui.horizontal(|ui| {
        for (enabled, label, icon, command) in [
            (
                navigation.can_back(),
                tr("Volver a la posición anterior", "Back to previous position"),
                theme::Icon::Left,
                PositionAction::Back,
            ),
            (
                navigation.can_forward(),
                tr("Avanzar en el historial", "Forward in history"),
                theme::Icon::Right,
                PositionAction::Forward,
            ),
        ] {
            if ui
                .add_enabled_ui(enabled, |ui| theme::icon_button(ui, label, icon, p))
                .inner
                .clicked()
            {
                action = Some(command);
            }
        }
        ui.label(
            egui::RichText::new(tr("Historial · Alt + ← / →", "History · Alt + ← / →"))
                .color(p.secondary)
                .size(12.0),
        );
    });
    ui.separator();
    for (slot, name) in [(MarkSlot::A, "A"), (MarkSlot::B, "B")] {
        let mark = navigation.mark(slot);
        ui.horizontal(|ui| {
            let save = format!("{} {name}", tr("Guardar posición", "Save position"));
            if theme::icon_button(ui, &save, theme::Icon::Bookmark, p).clicked() {
                action = Some(PositionAction::Save(slot));
            }
            let jump = match mark {
                Some(position) => format!(
                    "{} {name} · {} {}",
                    tr("Ir a", "Go to"),
                    tr("página", "page"),
                    position.page() + 1
                ),
                None => format!("{} {name}", tr("Ir a", "Go to")),
            };
            if ui
                .add_enabled_ui(mark.is_some(), |ui| theme::navigation(ui, &jump, false, p))
                .inner
                .clicked()
            {
                action = Some(PositionAction::Jump(slot));
            }
        });
    }
    action
}

impl YamantakaApp {
    pub(super) fn navigation_menu(&mut self, ui: &mut egui::Ui, p: Palette) {
        let Some(nav) = &self.reader.navigation else {
            return;
        };
        if let Some(action) = positions(ui, nav, self.english, p) {
            if action.apply(self.reader.navigation.as_mut().unwrap()) {
                self.apply_navigation();
            }
            ui.close();
        }
    }
}

pub(in crate::app) fn summary(navigation: Option<&ReaderNavigation>) -> serde_json::Value {
    navigation
        .map(|nav| {
            serde_json::json!({
                "page": nav.position().page(),
                "point": nav.position().point(),
                "back": nav.can_back(), "forward": nav.can_forward(), "entries": nav.history_len(),
                "markA": nav.mark(MarkSlot::A).map(|position| position.page()),
                "markB": nav.mark(MarkSlot::B).map(|position| position.page()),
            })
        })
        .unwrap_or_default()
}
