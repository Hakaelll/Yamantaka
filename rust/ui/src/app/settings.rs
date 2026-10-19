use super::*;

const CATEGORY_KEY: &str = "settings-category";

impl YamantakaApp {
    pub(super) fn preview_storage(&mut self, ctx: &egui::Context) {
        ctx.data_mut(|data| data.insert_temp(egui::Id::new(CATEGORY_KEY), 4_usize));
        self.show_surface(Surface::Settings);
    }
    pub fn preview_privacy(&mut self, ctx: &egui::Context) {
        if self.demo && self.capture.is_some() {
            ctx.data_mut(|data| data.insert_temp(egui::Id::new(CATEGORY_KEY), 3_usize));
            self.show_surface(Surface::Settings);
        }
    }
    pub(super) fn settings(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let cramped = ui.available_width() < 420.0;
        let inset = if ui.available_width() < 720.0 { 16 } else { 44 };
        let mut category = ui
            .ctx()
            .data_mut(|data| data.get_temp::<usize>(egui::Id::new(CATEGORY_KEY)))
            .unwrap_or(0)
            .min(4);
        let mut changed = false;
        egui::ScrollArea::vertical()
            .id_salt("settings-page")
            .animated(!self.reduced_motion)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(
                        inset,
                        if cramped { 12 } else { 24 },
                    ))
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width().min(1040.0));
                        if !cramped {
                            theme::display(
                                ui,
                                self.tr("Ajustes", "Settings"),
                                if ui.available_width() < 420.0 {
                                    26.0
                                } else {
                                    38.0
                                },
                            );
                            ui.add_space(4.0);
                            quiet(
                                ui,
                                self.tr(
                                    "Un espacio de lectura a tu medida.",
                                    "A reading space that feels like yours.",
                                ),
                                p,
                            );
                            ui.add_space(if cramped { 8.0 } else { 24.0 });
                            ui.separator();
                            ui.add_space(if cramped { 8.0 } else { 24.0 });
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                theme::display(ui, self.tr("Ajustes", "Settings"), 20.0);
                                let label = self.tr(
                                    [
                                        "Apariencia",
                                        "Accesibilidad",
                                        "Idioma",
                                        "Privacidad",
                                        "Almacenamiento",
                                    ][category],
                                    [
                                        "Appearance",
                                        "Accessibility",
                                        "Language",
                                        "Privacy",
                                        "Storage",
                                    ][category],
                                );
                                let response = theme::navigation(ui, label, true, p);
                                egui::Popup::menu(&response).show(|ui| {
                                    self.settings_navigation(ui, &mut category, false, p)
                                });
                            });
                            ui.add_space(8.0);
                        }
                        if ui.available_width() >= 850.0 {
                            let content_width = (ui.available_width() - 232.0).min(700.0);
                            ui.horizontal_top(|ui| {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(180.0, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.set_width(180.0);
                                        self.settings_navigation(ui, &mut category, false, p);
                                    },
                                );
                                ui.add_space(22.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(content_width, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.set_width(content_width);
                                        changed |= self.settings_section(ui, category, p);
                                    },
                                );
                            });
                        } else {
                            if !cramped {
                                self.settings_navigation(ui, &mut category, true, p);
                                ui.add_space(22.0);
                            }
                            changed |= self.settings_section(ui, category, p);
                        }
                        ui.add_space(32.0);
                    });
            });
        ui.ctx().data_mut(|data| {
            data.insert_temp(egui::Id::new(CATEGORY_KEY), category);
        });
        if changed {
            self.settings_changed(ui.ctx());
        }
    }

    fn settings_navigation(
        &self,
        ui: &mut egui::Ui,
        category: &mut usize,
        compact: bool,
        p: Palette,
    ) {
        let labels = [
            self.tr("Apariencia", "Appearance"),
            self.tr("Accesibilidad", "Accessibility"),
            self.tr("Idioma", "Language"),
            self.tr("Privacidad", "Privacy"),
            self.tr("Almacenamiento", "Storage"),
        ];
        let mut choices = |ui: &mut egui::Ui| {
            for (index, label) in labels.iter().enumerate() {
                let selected = *category == index;
                let response = theme::navigation(ui, label, selected, p);
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::WidgetType::SelectableLabel,
                        ui.is_enabled(),
                        selected,
                        label,
                    )
                });
                if response.clicked() {
                    *category = index;
                }
            }
        };
        if compact {
            ui.horizontal_wrapped(choices);
        } else {
            choices(ui);
        }
    }

    fn settings_section(&mut self, ui: &mut egui::Ui, category: usize, p: Palette) -> bool {
        let mut changed = false;
        let cramped = ui.available_width() < 420.0;
        match category {
            0 => {
                if !cramped {
                    section_title(ui, self.tr("Apariencia", "Appearance"));
                    quiet(
                        ui,
                        self.tr(
                            "Elige la luz que acompaña a tu colección.",
                            "Choose the light around your collection.",
                        ),
                        p,
                    );
                }
                ui.add_space(if cramped { 4.0 } else { 24.0 });
                let available = ui.available_width();
                let width = if available >= 460.0 {
                    ((available - ui.spacing().item_spacing.x) / 2.0).min(310.0)
                } else {
                    available.min(360.0)
                };
                ui.horizontal_wrapped(|ui| {
                    for dark in [false, true] {
                        let label = if dark {
                            self.tr("Grafito oscuro", "Dark graphite")
                        } else {
                            self.tr("Papel claro", "Warm paper")
                        };
                        let description = if dark {
                            self.tr("Una estancia en calma.", "A quiet study room.")
                        } else {
                            self.tr("La claridad del papel.", "The clarity of paper.")
                        };
                        let response = theme_choice(
                            ui,
                            width,
                            dark,
                            self.dark == dark,
                            self.reduced_motion,
                            cramped,
                            label,
                            description,
                            p,
                        );
                        if response.clicked() && self.dark != dark {
                            self.dark = dark;
                            changed = true;
                        }
                    }
                });
                ui.add_space(14.0);
                quiet(
                    ui,
                    self.tr(
                        "Las miniaturas muestran la biblioteca y el lector.",
                        "The thumbnails show the library and reader.",
                    ),
                    p,
                );
            }
            1 => {
                if !cramped {
                    section_title(ui, self.tr("Accesibilidad", "Accessibility"));
                    quiet(
                        ui,
                        self.tr(
                            "Más comodidad para leer y moverte por tu biblioteca.",
                            "More comfort as you read and explore your library.",
                        ),
                        p,
                    );
                }
                ui.add_space(26.0);
                let scale_heading =
                    ui.label(RichText::new(self.tr("Tamaño del texto", "Text size")).strong());
                ui.add_space(4.0);
                quiet(
                    ui,
                    self.tr(
                        "Amplía el texto y los controles de la interfaz.",
                        "Enlarge the interface text and controls.",
                    ),
                    p,
                );
                ui.add_space(12.0);
                let scale_label = self.tr("Tamaño del texto", "Text size").to_owned();
                ui.horizontal_wrapped(|ui| {
                    ui.scope(|ui| {
                        ui.spacing_mut().slider_width =
                            (ui.available_width() - 90.0).clamp(100.0, 340.0);
                        ui.visuals_mut().widgets.inactive.bg_fill = p.secondary;
                        ui.visuals_mut().selection.bg_fill = p.accent;
                        let response = ui
                            .add(
                                egui::Slider::new(&mut self.text_scale, 1.0..=2.0)
                                    .show_value(false)
                                    .trailing_fill(true),
                            )
                            .on_hover_text(&scale_label)
                            .labelled_by(scale_heading.id);
                        changed |= response.changed();
                    });
                    let response = ui
                        .add(
                            egui::DragValue::new(&mut self.text_scale)
                                .range(1.0..=2.0)
                                .speed(0.05)
                                .fixed_decimals(2)
                                .suffix("×"),
                        )
                        .on_hover_text(&scale_label)
                        .labelled_by(scale_heading.id);
                    changed |= response.changed();
                });
                ui.add_space(12.0);
                ui.horizontal_wrapped(|ui| {
                    for (value, label) in [(1.0, "100%"), (1.5, "150%"), (2.0, "200%")] {
                        if ui
                            .selectable_label((self.text_scale - value).abs() < 0.01, label)
                            .clicked()
                            && (self.text_scale - value).abs() > f32::EPSILON
                        {
                            self.text_scale = value;
                            changed = true;
                        }
                    }
                });
                ui.add_space(24.0);
                ui.separator();
                ui.add_space(20.0);
                let motion_label = self.tr("Reducir el movimiento", "Reduce motion").to_owned();
                changed |= ui
                    .checkbox(&mut self.reduced_motion, motion_label)
                    .changed();
                quiet(ui, self.tr(
                    "Mantiene los libros y las transiciones quietos, con la selección y el foco visibles.",
                    "Keeps books and transitions still, with selection and focus visible.",
                ), p);
            }
            2 => {
                if !cramped {
                    section_title(ui, self.tr("Idioma", "Language"));
                    quiet(
                        ui,
                        self.tr(
                            "El idioma de los menús y los controles.",
                            "The language used for menus and controls.",
                        ),
                        p,
                    );
                }
                ui.add_space(24.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.radio(!self.english, "Español").clicked() && self.english {
                        self.english = false;
                        changed = true;
                    }
                    if ui.radio(self.english, "English").clicked() && !self.english {
                        self.english = true;
                        changed = true;
                    }
                });
                ui.add_space(24.0);
                ui.separator();
                ui.add_space(16.0);
                quiet(ui, self.tr(
                    "Los títulos y el contenido de tus documentos conservan su idioma original.",
                    "Document titles and content keep their original language.",
                ), p);
            }
            4 => self.cache_maintenance_settings(ui, p),
            _ => {
                if !cramped {
                    section_title(ui, self.tr("Privacidad", "Privacy"));
                    ui.add_space(4.0);
                    ui.label(self.tr(
                        "Tus documentos y tu actividad de lectura permanecen en este dispositivo.",
                        "Your documents and reading activity stay on this device.",
                    ));
                    ui.add_space(24.0);
                }

                egui::Frame::new().fill(p.surface).corner_radius(12)
                    .inner_margin(egui::Margin::same(if cramped { 12 } else { 18 })).show(ui, |ui| {
                        if !cramped {
                        ui.label(RichText::new(self.tr("Tu tiempo de lectura", "Your reading time")).size(18.0).strong());
                        ui.add_space(8.0);
                        }
                        let mut enabled = self.preferences.record_reading_sessions();
                        if ui.checkbox(&mut enabled, self.tr("Guardar sesiones de lectura", "Save reading sessions")).changed() {
                            let path = self.store.join("preferences.json");
                            let demo = self.demo;
                            if let Err(error) = sessions::change_preference(&mut self.preferences, "recordReadingSessions", enabled.into(), |preferences| {
                                if demo { Ok(()) } else { preferences.save(&path) }
                            }) { self.error = Some(error.to_string()); }
                        }
                        ui.add_space(6.0);
                        quiet(ui, self.tr("Conserva el tiempo activo y el punto donde dejaste cada lectura. Al desactivarlo, se cierra la sesión actual y se mantienen las anteriores.", "Keep your active reading time and where you left each reading. Turning this off ends the current session and keeps earlier sessions."), p);
                        ui.add_space(16.0);
                        let limit_heading = ui.label(RichText::new(self.tr("Sesiones completadas que conservar", "Completed sessions to keep")).strong());
                        let saved_limit = self.preferences.reading_history_limit();
                        let limit_id = egui::Id::new("reading-history-limit-draft");
                        let mut limit = ui.ctx().data_mut(|data| data.get_temp::<usize>(limit_id)).unwrap_or(saved_limit);
                        ui.horizontal_wrapped(|ui| {
                            ui.add(egui::DragValue::new(&mut limit).range(25..=1000).speed(1.0)).labelled_by(limit_heading.id);
                            let apply = theme::control(ui, self.tr("Aplicar", "Apply"), Some(theme::Icon::Right), false, true, p);
                            if apply.clicked() {
                                let path = self.store.join("preferences.json");
                                let demo = self.demo;
                                match sessions::change_preference(&mut self.preferences, "readingHistoryLimit", limit.into(), |preferences| {
                                    if demo { Ok(()) } else { preferences.save(&path) }
                                }) {
                                    Ok(()) => self.send(Action::TrimReadingHistory(limit, Cancellation::default())),
                                    Err(error) => self.error = Some(error.to_string()),
                                }
                            }
                        });
                        ui.ctx().data_mut(|data| data.insert_temp(limit_id, limit));
                        quiet(ui, self.tr("Al aplicar un límite, se eliminan las sesiones completadas más antiguas. Las lecturas abiertas se mantienen.", "Applying a limit removes the oldest completed sessions. Open readings are kept."), p);
                    });
                ui.add_space(20.0);

                if theme::control(
                    ui,
                    self.tr("Gestionar búsqueda local", "Manage local search"),
                    Some(theme::Icon::Search),
                    false,
                    false,
                    p,
                )
                .clicked()
                {
                    self.show_search();
                }
                ui.add_space(16.0);
                for (title, description) in [
                    (
                        self.tr("Sin cuentas", "No accounts"),
                        self.tr(
                            "Tu biblioteca no necesita un perfil ni un inicio de sesión.",
                            "Your library needs no profile or sign-in.",
                        ),
                    ),
                    (
                        self.tr("Sin telemetría", "No telemetry"),
                        self.tr(
                            "Yamantaka no envía datos de uso durante la lectura.",
                            "Yamantaka sends no usage data as you read.",
                        ),
                    ),
                    (
                        self.tr("Lectura local", "Local reading"),
                        self.tr(
                            "La lectura y la organización de tu colección funcionan sin conexión.",
                            "Reading and organizing your collection work offline.",
                        ),
                    ),
                ] {
                    ui.label(RichText::new(title).strong());
                    quiet(ui, description, p);
                    ui.add_space(16.0);
                }
            }
        }
        if category < 3 {
            ui.add_space(28.0);
            ui.separator();
            ui.add_space(14.0);
            quiet(
                ui,
                self.tr(
                    "Los cambios se aplican y se guardan automáticamente.",
                    "Changes are applied and saved automatically.",
                ),
                p,
            );
        }
        changed
    }
}

pub(super) fn section_title(ui: &mut egui::Ui, title: &str) {
    theme::display(ui, title, 28.0);
    ui.add_space(6.0);
}

pub(super) fn quiet(ui: &mut egui::Ui, text: &str, p: Palette) {
    ui.add(egui::Label::new(RichText::new(text).color(p.secondary)).wrap());
}

#[allow(clippy::too_many_arguments)]
fn theme_choice(
    ui: &mut egui::Ui,
    width: f32,
    dark: bool,
    selected: bool,
    reduced_motion: bool,
    cramped: bool,
    label: &str,
    description: &str,
    p: Palette,
) -> egui::Response {
    let text_width = (width - 58.0).max(40.0);
    let title = ui.painter().layout(
        label.to_owned(),
        egui::FontId::proportional(16.0),
        p.ink,
        text_width,
    );
    let detail = ui.painter().layout(
        description.to_owned(),
        egui::FontId::proportional(13.0),
        p.secondary,
        text_width,
    );
    let height = if cramped { 106.0 } else { 144.0 } + title.size().y + detail.size().y;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::RadioButton,
            ui.is_enabled(),
            selected,
            label,
        )
    });
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let selection = ui.ctx().animate_bool_with_time_and_easing(
            response.id.with("selection"),
            selected,
            if reduced_motion || response.has_focus() {
                0.0
            } else {
                0.18
            },
            |t| {
                if t >= 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * t)
                }
            },
        );
        painter.rect_filled(
            rect,
            10.0,
            if response.hovered() {
                p.raised
            } else {
                p.surface
            },
        );
        let boundary = p.line.lerp_to_gamma(p.accent, selection);
        painter.rect_stroke(
            rect,
            10.0,
            egui::Stroke::new(1.0 + selection, boundary),
            egui::StrokeKind::Inside,
        );
        let preview = egui::Rect::from_min_size(
            rect.min
                + egui::vec2(
                    12.0,
                    if cramped {
                        30.0 + title.size().y + detail.size().y
                    } else {
                        12.0
                    },
                ),
            egui::vec2(width - 24.0, if cramped { 68.0 } else { 104.0 }),
        );
        paint_theme_preview(painter, preview, dark);
        let text_pos = egui::pos2(
            rect.left() + 16.0,
            rect.top() + if cramped { 12.0 } else { 130.0 },
        );
        painter.galley(text_pos, title.clone(), p.ink);
        painter.galley(
            text_pos + egui::vec2(0.0, title.size().y + 4.0),
            detail,
            p.secondary,
        );
        let indicator = egui::pos2(rect.right() - 25.0, text_pos.y + 9.0);
        painter.circle_stroke(
            indicator,
            7.0,
            egui::Stroke::new(1.5, if selected { p.accent } else { p.secondary }),
        );
        if selected {
            painter.circle_filled(indicator, 3.5, p.accent);
        }
        if response.has_focus() {
            painter.rect_stroke(
                rect.expand(3.0),
                12.0,
                egui::Stroke::new(2.0, p.accent),
                egui::StrokeKind::Outside,
            );
        }
    }
    response
}

/// A diagram of the two real surfaces, with no pretend controls or sample text.
fn paint_theme_preview(painter: &egui::Painter, rect: egui::Rect, dark: bool) {
    let p = Palette::new(dark);
    let painter = painter.with_clip_rect(rect);
    painter.rect_filled(rect, 5.0, p.canvas);
    let y = rect.top() + 16.0;
    painter.line_segment(
        [
            egui::pos2(rect.left() + 10.0, y),
            egui::pos2(rect.right() - 10.0, y),
        ],
        egui::Stroke::new(1.0, p.line),
    );
    let left_width = rect.width() * 0.55;
    let cover_width = (left_width - 32.0) / 3.0;
    for (index, color) in [
        Color32::from_rgb(43, 61, 81),
        Color32::from_rgb(83, 46, 57),
        Color32::from_rgb(44, 71, 58),
    ]
    .iter()
    .enumerate()
    {
        let x = rect.left() + 10.0 + index as f32 * (cover_width + 6.0);
        let cover =
            egui::Rect::from_min_size(egui::pos2(x, y + 12.0), egui::vec2(cover_width, 47.0));
        painter.rect_filled(cover, 1.0, *color);
        let artwork = cover.shrink(cover_width * 0.21);
        painter.rect_stroke(
            artwork,
            0.0,
            egui::Stroke::new(0.8, Color32::from_rgb(198, 187, 165)),
            egui::StrokeKind::Inside,
        );
        painter.line_segment(
            [
                egui::pos2(x, cover.bottom() + 8.0),
                egui::pos2(x + cover_width * 0.8, cover.bottom() + 8.0),
            ],
            egui::Stroke::new(2.0, p.secondary),
        );
    }
    let reader = egui::Rect::from_min_max(
        egui::pos2(rect.left() + left_width + 6.0, y + 8.0),
        egui::pos2(rect.right() - 10.0, rect.bottom() - 9.0),
    );
    painter.rect_filled(reader, 2.0, p.raised);
    let page = reader.shrink2(egui::vec2(9.0, 6.0));
    painter.rect_filled(page, 0.0, Color32::from_rgb(244, 239, 226));
    for line in 0..7 {
        let yy = page.top() + 10.0 + line as f32 * 6.0;
        let end = page.right() - if line == 6 { 15.0 } else { 6.0 };
        painter.line_segment(
            [egui::pos2(page.left() + 6.0, yy), egui::pos2(end, yy)],
            egui::Stroke::new(0.7, Color32::from_rgb(158, 149, 135)),
        );
    }
}
