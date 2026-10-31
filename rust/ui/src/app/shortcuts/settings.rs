use super::{
    Shortcut,
    bindings::{Bindings, CHOICES, DEFINITIONS, caption},
};
use crate::app::*;
impl YamantakaApp {
    pub(in crate::app) fn shortcut_settings(&mut self, ui: &mut egui::Ui, p: Palette) {
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        theme::display(ui, tr("Atajos", "Shortcuts"), 28.0);
        ui.label(tr("Personaliza las acciones del lector y la biblioteca. Re Pág y Av Pág mantienen la navegación.","Customize reader and library actions. Page Up and Page Down keep their navigation role."));
        if self.shortcut_bindings.warning {
            ui.label(tr("Las asignaciones guardadas tienen conflictos o un formato no válido. Se usan los valores predeterminados; restaura los atajos para editarlos.","Stored assignments conflict or use an invalid format. Defaults are active; restore shortcuts before editing."));
        }
        let unavailable = self.shortcut_bindings.inherited_unavailable();
        if unavailable > 0 {
            ui.label(format!(
                "{}: {unavailable}",
                tr(
                    "Asignaciones conservadas para comandos aún no conectados",
                    "Preserved assignments for commands not yet connected"
                )
            ));
        }
        if let Some(error) = &self.shortcut_error {
            let response = ui.label(error);
            if self.demo && self.capture.is_some() {
                response.scroll_to_me(Some(egui::Align::Center));
            }
        }
        ui.add_space(16.0);
        let mut proposed = None;
        ui.add_enabled_ui(!self.shortcut_bindings.warning, |ui| {
            for item in DEFINITIONS {
                let label = if english { item.en } else { item.es };
                let horizontal = ui.available_width() >= 550.0;
                let layout = if horizontal {
                    egui::Layout::left_to_right(egui::Align::Center)
                } else {
                    egui::Layout::top_down(egui::Align::Min)
                };
                ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 0.0), layout, |ui| {
                    let label_response = if horizontal {
                        ui.add_sized(
                            [ui.available_width() - 180.0, 40.0],
                            egui::Label::new(RichText::new(label).color(p.ink))
                                .halign(egui::Align::Min)
                                .wrap(),
                        )
                    } else {
                        ui.label(RichText::new(label).color(p.ink))
                    };
                    let choice = egui::ComboBox::from_id_salt(("shortcut", item.id))
                        .selected_text(caption(self.shortcut_bindings.token(item.command)))
                        .show_ui(ui, |ui| {
                            for token in CHOICES {
                                if ui
                                    .selectable_label(
                                        self.shortcut_bindings.token(item.command) == *token,
                                        caption(token),
                                    )
                                    .clicked()
                                {
                                    proposed = Some((item.command, *token));
                                }
                            }
                        });
                    let response = choice.response.labelled_by(label_response.id);
                    theme::record_control_bounds(ui, label, response.rect);
                });
                ui.add_space(12.0);
            }
        });
        if let Some((command, token)) = proposed {
            self.assign_shortcut(command, token, ui.ctx());
        }
        if theme::navigation(
            ui,
            tr(
                "Restaurar atajos predeterminados",
                "Restore default shortcuts",
            ),
            false,
            p,
        )
        .clicked()
        {
            self.save_shortcut_bindings(self.shortcut_bindings.reset(), ui.ctx());
        }
    }
    pub(in crate::app) fn assign_shortcut(
        &mut self,
        command: Shortcut,
        token: &str,
        ctx: &egui::Context,
    ) {
        match self.shortcut_bindings.changed(command, token) {
            Ok(bindings) => self.save_shortcut_bindings(bindings, ctx),
            Err(conflict) => {
                let item = DEFINITIONS
                    .iter()
                    .find(|item| item.command == conflict)
                    .unwrap();
                self.shortcut_error = Some(format!(
                    "{}: {}",
                    self.tr(
                        "Esta combinación ya está asignada a",
                        "This combination is already assigned to"
                    ),
                    if self.english { item.en } else { item.es }
                ));
            }
        }
    }
    fn save_shortcut_bindings(&mut self, bindings: Bindings, ctx: &egui::Context) {
        let mut preferences = self.preferences.clone();
        preferences
            .rust
            .insert("shortcutOverrides".into(), bindings.encoded());
        if !self.demo
            && let Err(error) = preferences.save(&self.store.join("preferences.json"))
        {
            self.shortcut_error = Some(error.to_string());
            return;
        }
        self.preferences = preferences;
        self.shortcut_bindings = bindings;
        self.shortcut_error = None;
        ctx.request_repaint();
    }
}
