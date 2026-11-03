use super::*;
use yamantaka_domain::reader_preferences::{CoverPolicy, InitialSide, PageRotation, ReaderMode};
use yamantaka_domain::{PageLayout, reader::ReaderPosition};

impl YamantakaApp {
    pub fn preview_reader_options(&mut self, section: usize) {
        self.preview_reader_options = self.demo && self.capture.is_some();
        if self.preview_reader_options {
            self.reader_options_section = section.min(2);
        }
    }

    pub(super) fn reader_options_popup(&self, response: &egui::Response) -> egui::Popup<'static> {
        let popup = egui::Popup::menu(response)
            .align(egui::RectAlign::BOTTOM_END)
            .align_alternatives(&[])
            .gap(8.0)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        if self.preview_reader_options {
            popup.open(true)
        } else {
            popup
        }
    }

    pub(super) fn reader_canvas(&self, palette: Palette) -> Color32 {
        if self.electronic_ink() {
            return Color32::WHITE;
        }
        self.reader
            .reader_preferences
            .as_ref()
            .map_or(palette.canvas, |prefs| {
                Self::reading_paper(prefs.reading_theme)
            })
    }

    pub(super) fn reading_paper(environment: ReaderTheme) -> Color32 {
        match environment {
            ReaderTheme::Light => Color32::from_rgb(243, 243, 241),
            ReaderTheme::Ivory => Color32::from_rgb(234, 229, 216),
            ReaderTheme::Sepia => Color32::from_rgb(216, 202, 178),
            ReaderTheme::Gray => Color32::from_rgb(185, 186, 182),
            ReaderTheme::Dark => Color32::from_rgb(34, 35, 32),
            ReaderTheme::Black => Color32::BLACK,
        }
    }

    pub(super) fn reader_zoom_label(&self) -> String {
        match self
            .reader
            .reader_preferences
            .as_ref()
            .map(|prefs| prefs.zoom_mode)
        {
            Some(ZoomMode::FitPage) => self.tr("Ajustar página", "Fit page").into(),
            Some(ZoomMode::FitWidth) => self.tr("Ajustar ancho", "Fit width").into(),
            _ => format!("{:.0}%", self.reader.zoom * 100.0),
        }
    }

    pub(super) fn load_reader_preferences(&mut self, preferences: ReaderPreferences) {
        self.reader.reader_preferences_loading = false;
        let preferences = self.restore_loaded_preferences(preferences);
        let preferences = self.restore_tab_preferences(preferences);
        let zoom = preferences.zoom_level as f32;
        if !zoom.is_finite() || zoom <= 0.0 {
            self.error = Some("Reader zoom cannot be represented".into());
            return;
        }
        self.reader.zoom = zoom;
        self.reader.viewport.scale = zoom;
        let start = preferences.page_rotation.unrotate_point([0.5, 0.0]);
        self.reader.reader_preferences = Some(preferences);
        self.rebuild_reader_layout();
        if let Some(navigation) = &mut self.reader.navigation {
            navigation.update_anchor(
                ReaderPosition::at(self.reader.page, start)
                    .expect("Quarter-turn page starts are normalized"),
            );
        }
        self.reader.viewport.generation += 1;
        self.reader.viewport.cancel.cancel();
        self.reader.viewport.cancel = Cancellation::default();
        self.reset_render_request();
        self.sync_tab_context();
    }

    pub(super) fn apply_reader_preferences(&mut self, mut preferences: ReaderPreferences) {
        preferences.updated_at = None;
        if let Err(error) = preferences.validate() {
            self.error = Some(error.to_string());
            return;
        }
        let zoom = preferences.zoom_level as f32;
        if !zoom.is_finite() || zoom <= 0.0 {
            self.error = Some("Reader zoom cannot be represented".into());
            return;
        }
        self.reader.zoom = zoom;
        self.reader.viewport.generation += 1;
        self.reader.viewport.cancel.cancel();
        self.reader.viewport.cancel = Cancellation::default();
        self.reset_render_request();
        self.reader.reader_preferences = Some(preferences.clone());
        self.rebuild_reader_layout();
        if !self.demo {
            self.send(Action::SaveReaderPreferences(
                preferences.document_id.clone(),
                preferences,
            ));
        }
    }

    pub(super) fn reader_options_menu(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.set_min_width(264.0);
        ui.menu_button(
            format!(
                "{} ({})",
                self.tr("Lecturas abiertas", "Open readings"),
                self.tabs.model.tabs().len()
            ),
            |ui| self.reader_documents_menu(ui),
        );
        ui.separator();
        ui.horizontal(|ui| {
            for (index, es, en) in [
                (0, "Lectura", "Reading"),
                (1, "Páginas", "Layout"),
                (2, "Posiciones", "Positions"),
            ] {
                if theme::navigation(ui, self.tr(es, en), self.reader_options_section == index, p)
                    .clicked()
                {
                    self.reader_options_section = index;
                }
            }
        });
        ui.separator();
        // Use the space below the popup's actual cursor, including the compact
        // zoom row. The menu must never move over the toolbar at enlarged text.
        let height =
            (ui.ctx().content_rect().bottom() - ui.cursor().top() - 28.0).clamp(48.0, 760.0);
        ui.set_min_width(264.0);
        egui::ScrollArea::vertical()
            .max_height(height)
            .min_scrolled_height(height)
            .show(ui, |ui| self.reader_options_content(ui, p));
    }

    fn reader_options_content(&mut self, ui: &mut egui::Ui, p: Palette) {
        ui.set_max_width(280.0);
        self.operation_actions(ui, p);
        if ui
            .add_enabled_ui(self.reader.reader_preferences.is_some(), |ui| {
                theme::navigation(
                    ui,
                    self.tr("Comparar páginas", "Compare pages"),
                    self.reference.is_some() || self.reference_mount.is_some(),
                    p,
                )
            })
            .inner
            .clicked()
        {
            self.toggle_reference(ui.ctx());
            ui.close();
        }
        if theme::navigation(
            ui,
            &format!(
                "{} · {}",
                self.tr("Modo concentración", "Focus mode"),
                shortcuts::bindings::caption(
                    self.shortcut_bindings.token(shortcuts::Shortcut::Focus)
                )
            ),
            self.reader_focus,
            p,
        )
        .clicked()
        {
            self.toggle_immersive_reader(ui.ctx());
            ui.close();
        }
        if theme::navigation(
            ui,
            self.tr("Seleccionar texto", "Select text"),
            self.reader.selection.enabled,
            p,
        )
        .clicked()
        {
            self.toggle_text_selection();
            ui.close();
        }
        if self.reference.is_some() && self.reader.selection.enabled {
            self.selection_actions(ui, p);
        }
        ui.separator();
        let Some(mut preferences) = self.reader.reader_preferences.clone() else {
            ui.label(
                RichText::new(if self.reader.reader_preferences_loading {
                    self.tr("Cargando preferencias…", "Loading preferences…")
                } else {
                    self.tr(
                        "Las preferencias guardadas necesitan revisión.",
                        "Saved preferences need review.",
                    )
                })
                .color(p.secondary),
            );
            return;
        };
        let before = preferences.clone();
        if self.reader_options_section == 2 {
            self.navigation_menu(ui, p);
            return;
        }
        if self.reader_options_section == 0 {
            ui.label(
                RichText::new(self.tr("Ajuste", "Page fit"))
                    .size(13.0)
                    .color(p.secondary),
            );
            for (mode, es, en) in [
                (ZoomMode::FitPage, "Página completa", "Whole page"),
                (ZoomMode::FitWidth, "Ancho de página", "Page width"),
            ] {
                if theme::navigation(ui, self.tr(es, en), preferences.zoom_mode == mode, p)
                    .clicked()
                {
                    preferences.zoom_mode = mode;
                }
            }
            ui.separator();
            ui.label(
                RichText::new(self.tr("Entorno de lectura", "Reading environment"))
                    .size(13.0)
                    .color(p.secondary),
            );
            ui.horizontal(|ui| {
                if theme::icon_button(ui, self.tr("Alejar", "Zoom out"), theme::Icon::Minus, p)
                    .clicked()
                {
                    self.set_reader_zoom(self.reader.viewport.scale / 1.2);
                }
                ui.label(self.reader_zoom_label());
                if theme::icon_button(ui, self.tr("Acercar", "Zoom in"), theme::Icon::Plus, p)
                    .clicked()
                {
                    self.set_reader_zoom(self.reader.viewport.scale * 1.2);
                }
            });
            let environments = [
                (ReaderTheme::Light, "Claro", "Light"),
                (ReaderTheme::Ivory, "Marfil", "Ivory"),
                (ReaderTheme::Sepia, "Sepia", "Sepia"),
                (ReaderTheme::Gray, "Gris", "Gray"),
                (ReaderTheme::Dark, "Oscuro", "Dark"),
                (ReaderTheme::Black, "Negro", "Black"),
            ];
            for row in environments.chunks(2) {
                ui.horizontal(|ui| {
                    for &(value, es, en) in row {
                        if theme::reading_environment(
                            ui,
                            self.tr(es, en),
                            Self::reading_paper(value),
                            preferences.reading_theme == value,
                            p,
                        )
                        .clicked()
                        {
                            preferences.reading_theme = value;
                        }
                    }
                });
            }
            ui.separator();
        } else {
            ui.label(
                RichText::new(self.tr("Modo de lectura", "Reader mode"))
                    .size(13.0)
                    .color(p.secondary),
            );
            ui.horizontal(|ui| {
                for (mode, es, en) in [
                    (ReaderMode::Document, "Documento", "Document"),
                    (ReaderMode::Book, "Libro", "Book"),
                ] {
                    if theme::navigation(ui, self.tr(es, en), preferences.view_mode == mode, p)
                        .clicked()
                    {
                        preferences.view_mode = mode;
                    }
                }
            });
            ui.label(
                RichText::new(self.tr("Composición", "Page layout"))
                    .size(13.0)
                    .color(p.secondary),
            );
            for (layout, es, en) in [
                (PageLayout::Single, "Una página", "Single page"),
                (PageLayout::Spread, "Doble página", "Facing pages"),
                (PageLayout::Continuous, "Continuo", "Continuous"),
            ] {
                if theme::navigation(
                    ui,
                    self.tr(es, en),
                    preferences.effective_layout() == layout,
                    p,
                )
                .clicked()
                {
                    preferences.page_layout = layout;
                    if layout != PageLayout::Spread {
                        preferences.view_mode = ReaderMode::Document;
                    }
                }
            }
            ui.separator();
            ui.label(
                RichText::new(self.tr("Dirección", "Reading direction"))
                    .size(13.0)
                    .color(p.secondary),
            );
            ui.horizontal(|ui| {
                for (direction, label, stored) in [
                    (ReadingDirection::Ltr, "LTR", "leftToRight"),
                    (ReadingDirection::Rtl, "RTL", "rightToLeft"),
                ] {
                    if theme::navigation(ui, label, self.reading_direction() == direction, p)
                        .clicked()
                    {
                        self.preferences
                            .rust
                            .insert("readingDirection".into(), stored.into());
                        self.settings_changed(ui.ctx());
                        self.rebuild_reader_layout();
                        self.reset_render_request();
                    }
                }
            });
            ui.separator();
            ui.label(
                RichText::new(self.tr("Primera página", "First page"))
                    .size(13.0)
                    .color(p.secondary),
            );
            for (policy, es, en) in [
                (
                    CoverPolicy::FirstPageIsCover,
                    "Portada independiente",
                    "Separate cover",
                ),
                (CoverPolicy::NoCover, "Sin portada", "No cover"),
            ] {
                if theme::navigation(ui, self.tr(es, en), preferences.cover_policy == policy, p)
                    .clicked()
                {
                    preferences.cover_policy = policy;
                }
            }
            ui.horizontal(|ui| {
                for (side, es, en) in [
                    (InitialSide::Left, "Izquierda", "Left"),
                    (InitialSide::Right, "Derecha", "Right"),
                ] {
                    if theme::navigation(ui, self.tr(es, en), preferences.initial_side == side, p)
                        .clicked()
                    {
                        preferences.initial_side = side;
                    }
                }
            });
            ui.separator();
            ui.label(
                RichText::new(self.tr("Rotación", "Rotation"))
                    .size(13.0)
                    .color(p.secondary),
            );
            ui.horizontal_wrapped(|ui| {
                for rotation in [
                    PageRotation::Deg0,
                    PageRotation::Deg90,
                    PageRotation::Deg180,
                    PageRotation::Deg270,
                ] {
                    if theme::navigation(
                        ui,
                        &format!("{}°", rotation.degrees()),
                        preferences.page_rotation == rotation,
                        p,
                    )
                    .clicked()
                    {
                        preferences.page_rotation = rotation;
                    }
                }
            });
            ui.separator();
            ui.label(
                RichText::new(self.tr("Márgenes", "Margins"))
                    .size(13.0)
                    .color(p.secondary),
            );
            for (label, value) in [
                (
                    self.tr("Horizontal", "Horizontal"),
                    &mut preferences.horizontal_margin,
                ),
                (
                    self.tr("Vertical", "Vertical"),
                    &mut preferences.vertical_margin,
                ),
            ] {
                ui.horizontal(|ui| {
                    let label = ui.label(label);
                    ui.add(
                        egui::DragValue::new(value)
                            .range(0.0..=16_384.0)
                            .speed(1.0)
                            .suffix(" pt"),
                    )
                    .labelled_by(label.id);
                });
            }
        }
        if preferences != before {
            self.apply_reader_preferences(preferences);
        }
    }
}
