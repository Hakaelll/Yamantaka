use super::*;

impl YamantakaApp {
    pub fn preview_workspace_layout(&mut self, columns: bool, panel: bool) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        if columns {
            self.list = true;
            self.library_columns = ["document", "status"].map(str::to_owned).into();
        }
        if panel {
            self.reader_panel_width = 420.0;
            self.reader_panel_restore = true;
            self.reader.panel = true;
        }
    }

    pub(super) fn library_column(&self, key: &str) -> bool {
        self.library_columns.iter().any(|column| column == key)
    }

    pub(super) fn library_column_options(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.menu_button(self.tr("Columnas de la lista", "List columns"), |ui| {
            ui.label(self.tr(
                "El título y las acciones siempre se muestran.",
                "The title and actions are always shown.",
            ));
            for (key, es, en) in [
                ("author", "Autor", "Author"),
                ("status", "Estado de lectura", "Reading state"),
                ("progress", "Progreso", "Progress"),
                ("pages", "Páginas", "Pages"),
            ] {
                let visible = self.library_column(key);
                if ui
                    .add_enabled_ui(visible || self.library_columns.len() < 12, |ui| {
                        theme::navigation(ui, self.tr(es, en), visible, p)
                    })
                    .inner
                    .clicked()
                {
                    if visible {
                        self.library_columns.retain(|column| column != key);
                    } else if self.library_columns.len() < 12 {
                        self.library_columns.push(key.into());
                    }
                    self.persist_library_state();
                }
            }
        });
        ui.separator();
    }
}
