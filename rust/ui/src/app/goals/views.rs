use super::*;

impl YamantakaApp {
    pub fn preview_goal(&mut self, action: u8) {
        if self.demo && self.capture.is_some() {
            self.inspector = true;
            self.goals.preview = action;
        }
    }

    fn send_goal(&mut self, request: ReadingGoalRequest) {
        let document = request.document.clone();
        let change = request.change;
        let cancel = request.cancel.clone();
        if let Err(error) = self.runtime.submit(Action::ReadingGoal(Box::new(request))) {
            self.accept_goal(ReadingGoalReply {
                document,
                change,
                cancel,
                result: Err(error),
            });
        }
    }
    pub(in crate::app) fn accept_goal(&mut self, reply: ReadingGoalReply) {
        if let Some(error) = self.goals.accept(reply) {
            self.error = Some(error);
        }
    }
    pub(in crate::app) fn reading_goal(&mut self, ui: &mut egui::Ui, document: &Document) {
        if let Some(request) = self.goals.select(&document.id) {
            self.send_goal(request);
        }
        if self.goals.loaded
            && self.goals.reading.is_none()
            && self.goals.writing.is_none()
            && self.goals.preview != 0
        {
            let action = std::mem::take(&mut self.goals.preview);
            self.goals.edit();
            if action == 2 {
                self.goals.pages = "25".into();
                self.goals.minutes = "45".into();
                if let Ok(goal) = self.goals.draft()
                    && let Some(request) = self.goals.save(goal)
                {
                    self.send_goal(request);
                }
            } else if action == 3
                && let Some(request) = self.goals.save(None)
            {
                self.send_goal(request);
            }
        }
        let p = Palette::new(self.dark);
        let compact = ui.available_width() < 380.0;
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(self.tr("Objetivo de lectura", "Reading goal"))
                    .color(p.secondary)
                    .size(12.0),
            );
            if let Some(goal) = self.goals.goal {
                if let Some(pages) = goal.pages() {
                    ui.label(format!(
                        "{}/{} {}",
                        document.last_page_index.saturating_add(1).min(pages),
                        pages,
                        self.tr("páginas", "pages")
                    ));
                }
                if let Some(minutes) = goal.minutes() {
                    ui.label(format!("{minutes} {}", self.tr("minutos", "minutes")));
                }
            } else if self.goals.loaded {
                ui.label(self.tr("A tu ritmo", "At your own pace"));
            }
            if self.goals.reading.is_some() || self.goals.writing.is_some() {
                ui.spinner();
            }
            if compact && !self.goals.editing {
                ui.end_row();
            }
            ui.add_enabled_ui(self.goals.loaded && self.goals.writing.is_none(), |ui| {
                if !self.goals.editing
                    && theme::navigation(ui, self.tr("Editar objetivo", "Edit goal"), false, p)
                        .clicked()
                {
                    self.goals.edit();
                }
            });
        });
        if let Some(error) = &self.goals.error {
            ui.label(RichText::new(error).color(p.secondary));
        }
        if !self.goals.loaded
            && self.goals.reading.is_none()
            && theme::navigation(ui, self.tr("Reintentar", "Retry"), false, p).clicked()
            && let Some(request) = self.goals.read()
        {
            self.send_goal(request);
        }
        if !self.goals.editing {
            return;
        }
        let context = ui.ctx().clone();
        let width = (context.content_rect().width() - 48.0).clamp(180.0, 420.0);
        let compact = context.content_rect().height() < 500.0 || width < 320.0;
        egui::Window::new(self.tr("Objetivo de lectura", "Reading goal"))
            .id(egui::Id::new("reading-goal-editor"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(width)
            .show(&context, |ui| {
                ui.set_max_width(width);
                egui::ScrollArea::vertical()
                    .id_salt("goal-fields")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .max_height((context.content_rect().height() * 0.33).max(80.0))
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(RichText::new(document.display_title()).font(
                                egui::FontId::new(
                                    if compact { 14.0 } else { 20.0 },
                                    egui::FontFamily::Name("Editorial".into()),
                                ),
                            ))
                            .wrap(),
                        );
                        ui.add_space(if compact { 4.0 } else { 8.0 });
                        if !compact {
                            ui.label(
                                RichText::new(self.tr(
                                    "Opcional. Deja un campo vacío para no fijar ese objetivo.",
                                    "Optional. Leave a field empty to skip that target.",
                                ))
                                .color(p.secondary)
                                .size(12.0),
                            );
                            ui.add_space(10.0);
                        }
                        ui.add_enabled_ui(self.goals.writing.is_none(), |ui| {
                            ui.horizontal_wrapped(|ui| {
                                let label = ui.label(self.tr("Páginas", "Pages"));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.goals.pages)
                                        .desired_width(90.0)
                                        .char_limit(5)
                                        .hint_text("1–10000"),
                                )
                                .labelled_by(label.id);
                            });
                            ui.horizontal_wrapped(|ui| {
                                let label = ui.label(self.tr("Minutos", "Minutes"));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.goals.minutes)
                                        .desired_width(90.0)
                                        .char_limit(4)
                                        .hint_text("1–1440"),
                                )
                                .labelled_by(label.id);
                            });
                        });
                        if let Some(error) = &self.goals.error {
                            ui.label(error);
                        }
                    });
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                ui.add_enabled_ui(self.goals.writing.is_none(), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if theme::control(
                            ui,
                            self.tr("Guardar objetivo", "Save goal"),
                            None,
                            false,
                            true,
                            p,
                        )
                        .clicked()
                        {
                            match self.goals.draft() {
                                Ok(goal) => {
                                    if let Some(request) = self.goals.save(goal) {
                                        self.send_goal(request);
                                    }
                                }
                                Err(_) => self.goals.error = Some(
                                    self.tr(
                                        "Usa números enteros: 1–10000 páginas y 1–1440 minutos.",
                                        "Use whole numbers: 1–10000 pages and 1–1440 minutes.",
                                    )
                                    .to_owned(),
                                ),
                            }
                        }
                        if theme::navigation(ui, self.tr("Quitar objetivo", "Clear goal"), false, p)
                            .clicked()
                            && let Some(request) = self.goals.save(None)
                        {
                            self.send_goal(request);
                        }
                        if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p).clicked()
                        {
                            self.goals.editing = false;
                            self.goals.error = None;
                        }
                    });
                });
                if self.goals.writing.is_some() {
                    ui.spinner();
                    ui.label(self.tr("Guardando objetivo…", "Saving goal…"));
                }
            });
    }
}
