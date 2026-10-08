use super::*;

impl YamantakaApp {
    pub(super) fn navigation_menu(&mut self, ui: &mut egui::Ui, p: Palette) {
        let Some(navigation) = self.navigation.as_ref() else {
            return;
        };
        let back = navigation.can_back();
        let forward = navigation.can_forward();
        let mut changed = false;
        if ui
            .add_enabled_ui(back, |ui| {
                theme::control(
                    ui,
                    self.tr("Volver a la posición anterior", "Back to previous position"),
                    Some(theme::Icon::Left),
                    false,
                    false,
                    p,
                )
            })
            .inner
            .clicked()
        {
            changed = self.navigation.as_mut().is_some_and(ReaderNavigation::back);
        }
        if ui
            .add_enabled_ui(forward, |ui| {
                theme::control(
                    ui,
                    self.tr("Avanzar en el historial", "Forward in history"),
                    Some(theme::Icon::Right),
                    false,
                    false,
                    p,
                )
            })
            .inner
            .clicked()
        {
            changed = self
                .navigation
                .as_mut()
                .is_some_and(ReaderNavigation::forward);
        }
        ui.separator();
        for (slot, name) in [(MarkSlot::A, "A"), (MarkSlot::B, "B")] {
            let mark = self.navigation.as_ref().and_then(|nav| nav.mark(slot));
            let save = format!("{} {name}", self.tr("Guardar posición", "Save position"));
            if theme::navigation(ui, &save, false, p).clicked()
                && let Some(nav) = self.navigation.as_mut()
            {
                nav.capture_mark(slot);
                ui.close();
            }
            let jump = match mark {
                Some(position) => format!(
                    "{} {name} · {} {}",
                    self.tr("Ir a", "Go to"),
                    self.tr("página", "page"),
                    position.page() + 1
                ),
                None => format!("{} {name}", self.tr("Ir a", "Go to")),
            };
            if ui
                .add_enabled_ui(mark.is_some(), |ui| theme::navigation(ui, &jump, false, p))
                .inner
                .clicked()
            {
                changed = self
                    .navigation
                    .as_mut()
                    .is_some_and(|nav| nav.jump_to_mark(slot));
            }
        }
        if changed {
            self.apply_navigation();
            ui.close();
        }
    }
}
