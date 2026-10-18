use super::*;
use yamantaka_domain::PdfPassword;

pub(super) struct PasswordChallenge {
    document: Document,
    value: PdfPassword,
    rejected: bool,
    focus: bool,
}

impl PasswordChallenge {
    pub fn new(document: Document, rejected: bool) -> Self {
        Self {
            document,
            value: PdfPassword::default(),
            rejected,
            focus: true,
        }
    }
}

impl YamantakaApp {
    pub(crate) fn preview_password(&mut self, action: &'static str) {
        if self.demo && self.capture.is_some() {
            self.preview_password_target = Some(action);
            self.preview_password_action = Some(action);
        }
    }
    pub(super) fn retry_password(&mut self, document: Document, value: PdfPassword) {
        let id = document.id.clone();
        let cancel = self.opening.retry(id.clone());
        self.tabs.bind_open(cancel.clone());
        self.password_attempted = true;
        self.loading = true;
        if let Err(error) = self
            .runtime
            .submit(Action::Open(document, Some(value), cancel.clone()))
        {
            self.cancel_reader_opening();
            self.error = Some(error.to_string());
        }
    }

    pub(super) fn password_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut challenge) = self.password.take() else {
            return;
        };
        let p = Palette::new(self.dark);
        let compact = ctx.content_rect().height() < 420.0;
        let gap = if compact { 4.0 } else { 12.0 };
        let field_id = egui::Id::new("ephemeral-pdf-password");
        let mut submit = false;
        let mut cancel = false;
        let modal = egui::Modal::new(egui::Id::new("unlock-pdf"))
            .frame(
                egui::Frame::new()
                    .fill(p.surface)
                    .stroke(egui::Stroke::new(1.0, p.line))
                    .corner_radius(12)
                    .inner_margin(if compact { 12 } else { 24 }),
            )
            .show(ctx, |ui| {
                ui.set_width(
                    (ctx.content_rect().width() - if compact { 48.0 } else { 64.0 })
                        .clamp(160.0, 440.0),
                );
                egui::ScrollArea::vertical()
                    .max_height(
                        (ctx.content_rect().height() - if compact { 48.0 } else { 64.0 }).max(80.0),
                    )
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(self.tr("Desbloquear PDF", "Unlock PDF"))
                                .size(if compact { 20.0 } else { 24.0 })
                                .color(p.ink),
                        );
                        ui.add_space(if compact { 4.0 } else { 8.0 });
                        let title = egui::Label::new(
                            RichText::new(challenge.document.display_title())
                                .size(if compact { 14.0 } else { 16.0 })
                                .color(p.ink),
                        );
                        ui.add(if compact {
                            title.truncate()
                        } else {
                            title.wrap()
                        });
                        ui.add_space(gap);
                        ui.label(
                            RichText::new(if challenge.rejected {
                                self.tr(
                                    "La contraseña no es correcta. Inténtalo de nuevo.",
                                    "The password is incorrect. Try again.",
                                )
                            } else {
                                self.tr(
                                    "Este PDF necesita una contraseña para abrirse.",
                                    "This PDF needs a password to open.",
                                )
                            })
                            .size(if compact { 13.0 } else { 16.0 })
                            .color(p.secondary),
                        );
                        ui.add_space(gap);
                        let label = self.tr("Contraseña del PDF", "PDF password");
                        if !compact {
                            ui.label(label);
                        }
                        let mut output = egui::TextEdit::singleline(challenge.value.edit())
                            .hint_text(label)
                            .id(field_id)
                            .password(true)
                            .char_limit(1024)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, 40.0))
                            .show(ui);
                        // egui's undo history otherwise retains plaintext between
                        // frames, despite password masking and clipboard protection.
                        output.state.clear_undoer();
                        output.state.store(ctx, field_id);
                        output.response.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::TextEdit,
                                ui.is_enabled(),
                                label,
                            )
                        });
                        if challenge.focus {
                            output.response.request_focus();
                            challenge.focus = false;
                        }
                        submit = output.response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        ui.add_space(if compact { 4.0 } else { 8.0 });
                        ui.label(
                            RichText::new(self.tr(
                                "Se usa solo para esta apertura y no se guarda.",
                                "Used only for this opening and never saved.",
                            ))
                            .size(if compact { 12.0 } else { 13.0 })
                            .color(p.secondary),
                        );
                        ui.add_space(if compact { 6.0 } else { 16.0 });
                        ui.horizontal_wrapped(|ui| {
                            if theme::control(
                                ui,
                                self.tr("Abrir PDF", "Open PDF"),
                                Some(theme::Icon::Right),
                                true,
                                false,
                                p,
                            )
                            .clicked()
                            {
                                submit = true;
                            }
                            if theme::navigation(ui, self.tr("Cancelar", "Cancel"), false, p)
                                .clicked()
                            {
                                cancel = true;
                            }
                        });
                    });
            });
        cancel |= modal.should_close();
        if cancel || submit {
            ctx.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(field_id));
            ctx.memory_mut(|memory| memory.surrender_focus(field_id));
        }
        if cancel {
            self.cancel_reader_opening();
        } else if submit {
            self.retry_password(challenge.document, challenge.value);
        } else {
            self.password = Some(challenge);
        }
    }
}
