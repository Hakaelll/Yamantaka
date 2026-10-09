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
        self.cancel.cancel();
        self.cancel = Cancellation::default();
        self.password_attempted = true;
        self.loading = true;
        self.reader_preferences_loading = true;
        self.send(Action::Open(document, Some(value), self.cancel.clone()));
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
            self.cancel.cancel();
            self.open_intent.leave_destination();
            self.password_attempted = false;
            self.loading = false;
        } else if submit {
            self.retry_password(challenge.document, challenge.value);
        } else {
            self.password = Some(challenge);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::{Failure, LibraryRepository};
    use yamantaka_pdf::{MemoryLimits, PdfWorker};
    use yamantaka_storage::Database;

    #[test]
    fn native_actor_reports_password_challenges_and_unlocks_without_changing_original() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = root
            .join("build/rust-fixtures/protected.pdf")
            .canonicalize()
            .unwrap();
        let original = std::fs::read(&path).unwrap();
        let db = Database::in_memory_at(11).unwrap();
        let escaped = path.to_string_lossy().replace('\'', "''");
        db.execute_batch(&format!("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES ('protected','{escaped}','synthetic','protected.pdf',1,1,'now','now','now','synthetic');")).unwrap();
        let document = db.query(&LibraryQuery::default()).unwrap().remove(0);
        let worker = PdfWorker::start(
            &root.join("build/native/pdfium-7881/bin/pdfium.dll"),
            MemoryLimits::default(),
        )
        .unwrap();
        let runtime = Runtime::start(db, worker, || {}).unwrap();
        for password in [
            None,
            Some(PdfPassword::from("synthetic incorrect credential")),
        ] {
            runtime
                .submit(Action::Open(
                    document.clone(),
                    password,
                    Cancellation::default(),
                ))
                .unwrap();
            assert!(
                matches!(runtime.updates.recv_timeout(Duration::from_secs(5)).unwrap(),
                Update::OpenFailed(doc, Failure::PasswordRequired, token) if doc.id == "protected" && !token.is_cancelled())
            );
        }
        runtime
            .submit(Action::Open(
                document,
                Some(PdfPassword::from("fixture-password")),
                Cancellation::default(),
            ))
            .unwrap();
        assert!(
            matches!(runtime.updates.recv_timeout(Duration::from_secs(5)).unwrap(),
            Update::Opened(doc, info, _) if doc.id == "protected" && !info.pages.is_empty())
        );
        runtime
            .submit(Action::Text("protected".into(), 0, Cancellation::default()))
            .unwrap();
        assert!(
            matches!(runtime.updates.recv_timeout(Duration::from_secs(5)).unwrap(), Update::Text(id, 0, text, _) if id == "protected" && !text.is_empty())
        );
        runtime.submit(Action::Close("protected".into())).unwrap();
        assert!(matches!(
            runtime
                .updates
                .recv_timeout(Duration::from_secs(5))
                .unwrap(),
            Update::Committed(_, yamantaka_application::runtime::WriteKind::Close)
        ));
        drop(runtime);
        assert_eq!(std::fs::read(path).unwrap(), original);
    }
}
