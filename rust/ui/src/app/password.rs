use super::*;
use yamantaka_domain::PdfPassword;

enum PasswordTarget {
    Reader {
        document: Box<Document>,
        reference: bool,
    },
    Import(yamantaka_application::import_password::Challenge),
}

pub(super) struct PasswordChallenge {
    target: PasswordTarget,
    value: PdfPassword,
    rejected: bool,
    focus: bool,
}
impl PasswordChallenge {
    pub fn new(document: Document, rejected: bool) -> Self {
        Self {
            target: PasswordTarget::Reader {
                document: Box::new(document),
                reference: false,
            },
            value: PdfPassword::default(),
            rejected,
            focus: true,
        }
    }
    pub(super) fn reference(document: Document, rejected: bool) -> Self {
        Self {
            target: PasswordTarget::Reader {
                document: Box::new(document),
                reference: true,
            },
            value: PdfPassword::default(),
            rejected,
            focus: true,
        }
    }
    fn import(request: yamantaka_application::import_password::Challenge) -> Self {
        Self {
            rejected: request.rejected(),
            target: PasswordTarget::Import(request),
            value: PdfPassword::default(),
            focus: true,
        }
    }
    fn title(&self) -> std::borrow::Cow<'_, str> {
        match &self.target {
            PasswordTarget::Reader { document, .. } => {
                std::borrow::Cow::Borrowed(document.display_title())
            }
            PasswordTarget::Import(request) => request
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
        }
    }
    fn importing(&self) -> bool {
        matches!(&self.target, PasswordTarget::Import(_))
    }
    pub(super) fn reference_document(&self) -> Option<&Document> {
        match &self.target {
            PasswordTarget::Reader {
                document,
                reference: true,
            } => Some(document),
            _ => None,
        }
    }
}

impl YamantakaApp {
    pub(super) fn dismiss_password_challenge(&mut self) {
        if let Some(challenge) = self.password.take()
            && let Some(document) = challenge.reference_document()
        {
            self.cancel_reference_password(&document.id);
        }
    }

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
        self.reader.loading = true;
        if let Err(error) = self
            .runtime
            .submit(Action::Open(document, Some(value), cancel.clone()))
        {
            self.cancel_reader_opening();
            self.error = Some(error.to_string());
        }
    }

    pub(super) fn password_dialog(&mut self, ctx: &egui::Context) {
        while self.password.is_none() {
            let Some(request) = self.runtime.next_import_password() else {
                break;
            };
            if !request.cancelled() {
                if let Some(preview) = &mut self.import_password_preview {
                    let answer = match preview.mode {
                        "unlock" => Some(if preview.attempts == 0 {
                            "synthetic incorrect credential"
                        } else {
                            "fixture-password"
                        }),
                        "wrong" if preview.attempts == 0 => Some("synthetic incorrect credential"),
                        "cancel" => {
                            drop(request);
                            preview.attempts += 1;
                            break;
                        }
                        _ => None,
                    };
                    if let Some(answer) = answer {
                        preview.attempts += 1;
                        request.answer(answer.into()).unwrap();
                        continue;
                    }
                }
                self.password = Some(PasswordChallenge::import(request));
            }
        }
        let Some(mut challenge) = self.password.take() else {
            return;
        };
        if matches!(&challenge.target,PasswordTarget::Import(request) if request.cancelled()) {
            ctx.memory_mut(|memory| {
                memory.surrender_focus(egui::Id::new("ephemeral-pdf-password"))
            });
            ctx.data_mut(|data| {
                data.remove::<egui::text_edit::TextEditState>(egui::Id::new(
                    "ephemeral-pdf-password",
                ))
            });
            return;
        }
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
                if compact {
                    ui.spacing_mut().item_spacing.y = 6.0;
                }
                let header_start = ui.cursor().top();
                ui.label(
                    RichText::new(if compact && challenge.rejected {
                        self.tr("Contraseña incorrecta", "Incorrect password")
                    } else {
                        self.tr("Desbloquear PDF", "Unlock PDF")
                    })
                    .size(if compact { 20.0 } else { 24.0 })
                    .color(p.ink),
                );
                ui.add_space(if compact { 4.0 } else { 8.0 });
                let title = egui::Label::new(
                    RichText::new(challenge.title())
                        .size(if compact { 14.0 } else { 16.0 })
                        .color(p.ink),
                );
                ui.add(if compact {
                    title.truncate()
                } else {
                    title.wrap()
                });
                let header_height = ui.cursor().top() - header_start;
                let footer_height = if ui.available_width() < 360.0 {
                    104.0
                } else {
                    56.0
                };
                egui::ScrollArea::vertical()
                    .min_scrolled_height(0.0)
                    .max_height(
                        (ctx.content_rect().height()
                            - if compact { 48.0 } else { 72.0 }
                            - header_height
                            - footer_height)
                            .max(44.0),
                    )
                    .show(ui, |ui| {
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
                            if compact {
                                output.response.scroll_to_me(Some(egui::Align::Center));
                            }
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
                    });
                ui.add_space(if compact { 6.0 } else { 16.0 });
                ui.horizontal_wrapped(|ui| {
                    if theme::control(
                        ui,
                        if challenge.importing() {
                            self.tr("Añadir PDF", "Add PDF")
                        } else {
                            self.tr("Abrir PDF", "Open PDF")
                        },
                        Some(theme::Icon::Right),
                        true,
                        false,
                        p,
                    )
                    .clicked()
                    {
                        submit = true;
                    }
                    if theme::navigation(
                        ui,
                        if challenge.importing() {
                            self.tr("Cancelar importación", "Cancel import")
                        } else {
                            self.tr("Cancelar", "Cancel")
                        },
                        false,
                        p,
                    )
                    .clicked()
                    {
                        cancel = true;
                    }
                });
            });
        cancel |= modal.should_close();
        if cancel || submit {
            ctx.data_mut(|data| data.remove::<egui::text_edit::TextEditState>(field_id));
            ctx.memory_mut(|memory| memory.surrender_focus(field_id));
        }
        if cancel {
            match challenge.target {
                PasswordTarget::Reader {
                    reference: false, ..
                } => self.cancel_reader_opening(),
                PasswordTarget::Reader {
                    document,
                    reference: true,
                } => self.cancel_reference_password(&document.id),
                PasswordTarget::Import(request) => drop(request),
            }
        } else if submit {
            match challenge.target {
                PasswordTarget::Reader {
                    document,
                    reference: false,
                } => self.retry_password(*document, challenge.value),
                PasswordTarget::Reader {
                    document,
                    reference: true,
                } => self.retry_reference_password(*document, challenge.value),
                PasswordTarget::Import(request) => {
                    let _ = request.answer(challenge.value);
                }
            }
        } else {
            self.password = Some(challenge);
        }
    }
}

pub(super) struct ImportPreview {
    mode: &'static str,
    attempts: u8,
}
impl YamantakaApp {
    pub fn preview_import_password(&mut self, mode: &'static str) {
        if self.demo && self.capture.is_some() {
            self.import_password_preview = Some(ImportPreview { mode, attempts: 0 });
            self.import(vec![
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../build/rust-fixtures/protected.pdf"),
            ]);
        }
    }
    pub(super) fn import_password_waiting_capture(&self) -> bool {
        self.import_password_preview
            .as_ref()
            .is_some_and(|preview| matches!(preview.mode, "prompt" | "wrong"))
            && self
                .password
                .as_ref()
                .is_some_and(PasswordChallenge::importing)
    }
    pub(super) fn import_password_capture_ready(&self) -> bool {
        self.import_password_preview
            .as_ref()
            .is_none_or(|preview| match preview.mode {
                "prompt" => self.import_password_waiting_capture(),
                "wrong" => {
                    self.import_password_waiting_capture()
                        && self.password.as_ref().is_some_and(|p| p.rejected)
                }
                "unlock" => preview.attempts >= 2 && !self.importing.busy() && self.count == 11,
                "cancel" => preview.attempts == 1 && !self.importing.busy(),
                _ => false,
            })
    }
    pub(super) fn import_password_summary(&self) -> serde_json::Value {
        serde_json::json!({"ids":self.importing.last_batch.as_ref().map(|batch|batch.ids().collect::<Vec<_>>()).unwrap_or_default(),"waiting":self.password.as_ref().is_some_and(PasswordChallenge::importing),
            "rejected":self.password.as_ref().is_some_and(|p|p.importing()&&p.rejected),
            "busy":self.importing.busy(),"attempts":self.import_password_preview.as_ref().map_or(0,|p|p.attempts),
            "imported":self.importing.last_batch.as_ref().map_or(0,|batch|batch.len())})
    }
}
