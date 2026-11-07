use super::*;
use outline::ReaderPanelTab;
use theme::Icon;

impl YamantakaApp {
    pub(super) fn reader_toolbar_contents(
        &mut self,
        ui: &mut egui::Ui,
        page_count: u32,
        p: Palette,
    ) -> Option<egui::Id> {
        let width = ui.available_width();
        let labeled = width >= 1040.0;
        let full_zoom = width >= 1200.0;
        let inline_steps = width >= 500.0;
        self.library_return(ui, true, p);
        if width >= 730.0 {
            self.reader_switcher(ui, if labeled { 248.0 } else { 176.0 }, p);
            reader_group_rule(ui, p);
        }
        if inline_steps {
            self.reader_page_step(ui, false, page_count, p);
        }
        let mut shown = self.reader.page + 1;
        let mut page = egui::DragValue::new(&mut shown).range(1..=page_count);
        if labeled {
            page = page.suffix(format!(" / {page_count}"));
        }
        let response = ui.add_sized([if labeled { 98.0 } else { 64.0 }, 40.0], page);
        let label = self.tr("Página", "Page");
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::DragValue, ui.is_enabled(), label)
        });
        theme::record_control_bounds(ui, label, response.rect);
        if response
            .clone()
            .on_hover_text(format!("{label} {shown} / {page_count}"))
            .changed()
        {
            self.navigate(shown.saturating_sub(1));
        }
        if inline_steps {
            self.reader_page_step(ui, true, page_count, p);
            reader_group_rule(ui, p);
            if full_zoom {
                self.reader_zoom_step(ui, false, p);
            }
            let zoom = if full_zoom {
                theme::navigation(ui, &self.reader_zoom_label(), false, p)
            } else {
                theme::icon_button(ui, "Zoom", Icon::Plus, p)
            };
            egui::Popup::menu(&zoom).show(|ui| self.reader_zoom_menu(ui, !full_zoom, p));
            if full_zoom {
                self.reader_zoom_step(ui, true, p);
            }
            reader_group_rule(ui, p);
        }

        let find_label = self.tr("Buscar", "Find");
        let find = self.reader.panel && self.reader.reader_panel_tab == ReaderPanelTab::Find;
        if study_control(ui, find_label, Icon::Search, find, labeled, p).clicked() {
            if find {
                self.close_study_tray();
            } else {
                self.show_find();
            }
        }
        self.reader_study_control(ui, ReaderPanelTab::Outline, labeled, p);
        self.reader_study_control(ui, ReaderPanelTab::Annotations, labeled, p);
        if width >= 430.0
            && study_control(
                ui,
                self.tr("Estudio", "Study"),
                Icon::Pencil,
                self.study.active,
                labeled,
                p,
            )
            .clicked()
        {
            self.toggle_study();
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let more =
                theme::icon_button(ui, self.tr("Más controles", "More controls"), Icon::Menu, p);
            self.reader_options_popup(&more)
                .show(|ui| self.reader_options_menu(ui, p));
        });
        Some(response.id)
    }

    fn reader_study_control(
        &mut self,
        ui: &mut egui::Ui,
        tab: ReaderPanelTab,
        labeled: bool,
        p: Palette,
    ) {
        let (label, icon) = if tab == ReaderPanelTab::Outline {
            (self.tr("Navegar", "Navigate"), Icon::Panel)
        } else {
            (self.tr("Notas", "Notes"), Icon::List)
        };
        let same_group = self.reader.reader_panel_tab.study_group() == tab.study_group();
        let selected = self.reader.panel && same_group;
        if study_control(ui, label, icon, selected, labeled, p).clicked() {
            if selected {
                self.close_study_tray();
            } else {
                self.reader.panel = true;
                if !same_group {
                    self.select_study_tab(tab);
                }
            }
        }
    }

    fn reader_page_step(&mut self, ui: &mut egui::Ui, forward: bool, count: u32, p: Palette) {
        let (enabled, label, icon) = if forward {
            (
                self.reader.page + 1 < count,
                self.tr("Página siguiente", "Next page"),
                Icon::Right,
            )
        } else {
            (
                self.reader.page > 0,
                self.tr("Página anterior", "Previous page"),
                Icon::Left,
            )
        };
        if ui
            .add_enabled_ui(enabled, |ui| theme::icon_button(ui, label, icon, p))
            .inner
            .clicked()
        {
            self.step_reader(forward);
        }
    }

    fn reader_zoom_step(&mut self, ui: &mut egui::Ui, inward: bool, p: Palette) {
        let (label, icon, factor) = if inward {
            (self.tr("Acercar", "Zoom in"), Icon::Plus, 1.2)
        } else {
            (self.tr("Alejar", "Zoom out"), Icon::Minus, 1.0 / 1.2)
        };
        if theme::icon_button(ui, label, icon, p).clicked() {
            self.set_reader_zoom(self.reader.viewport.scale * factor);
        }
    }

    fn reader_zoom_menu(&mut self, ui: &mut egui::Ui, show_steps: bool, p: Palette) {
        if show_steps {
            ui.horizontal(|ui| {
                self.reader_zoom_step(ui, false, p);
                self.reader_zoom_step(ui, true, p);
                ui.label(self.reader_zoom_label());
            });
        }
        for (label, mode) in [
            (self.tr("Ajustar página", "Fit page"), ZoomMode::FitPage),
            (self.tr("Ajustar ancho", "Fit width"), ZoomMode::FitWidth),
            (self.tr("Tamaño real", "Actual size"), ZoomMode::Custom),
        ] {
            if theme::navigation(ui, label, false, p).clicked() {
                if mode == ZoomMode::Custom {
                    self.set_reader_zoom(1.0);
                } else if let Some(mut preferences) = self.reader.reader_preferences.clone() {
                    preferences.zoom_mode = mode;
                    self.apply_reader_preferences(preferences);
                }
                ui.close();
            }
        }
    }

    pub(super) fn set_reader_zoom(&mut self, zoom: f32) {
        if let Some(mut preferences) = self.reader.reader_preferences.clone() {
            preferences.zoom_mode = ZoomMode::Custom;
            preferences.zoom_level = zoom.clamp(0.2, 8.0) as f64;
            self.apply_reader_preferences(preferences);
            return;
        }
        self.reader.zoom = zoom.clamp(0.2, 8.0);
        self.reader.viewport.generation += 1;
        self.reset_render_request();
    }
}

fn study_control(
    ui: &mut egui::Ui,
    label: &str,
    icon: Icon,
    selected: bool,
    labeled: bool,
    p: Palette,
) -> egui::Response {
    if labeled {
        theme::control(ui, label, Some(icon), selected, false, p)
    } else {
        theme::icon_toggle(ui, label, icon, selected, p)
    }
}

pub(super) fn reader_group_rule(ui: &mut egui::Ui, p: Palette) {
    let (_, rect) = ui.allocate_space(egui::vec2(8.0, 24.0));
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        egui::Stroke::new(1.0, p.line),
    );
}

pub(super) fn toolbar_slot(ui: &mut egui::Ui) -> egui::Rect {
    let spacing = ui.spacing().item_spacing.y;
    ui.spacing_mut().item_spacing.y = 0.0;
    let (_, rect) = ui.allocate_space(egui::vec2(ui.available_width(), 48.0));
    ui.spacing_mut().item_spacing.y = spacing;
    rect
}
