use super::cover_picker::CoverPicker;
use super::*;

#[derive(Default)]
struct PickerActions {
    close: bool,
    save: bool,
    changed: bool,
    retry: bool,
    restore: bool,
}

impl YamantakaApp {
    pub(super) fn cover_picker_view(&mut self, ui: &mut egui::Ui) {
        let Some(mut picker) = self.cover_picker.take() else {
            return;
        };
        let p = Palette::new(self.dark);
        let compact = ui.ctx().content_rect().height() < 650.0;
        let mut actions = PickerActions::default();
        let can_close = picker.can_close();
        egui::ScrollArea::vertical()
            .id_salt("cover-picker-view")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let horizontal = if compact {
                    14
                } else if ui.available_width() > 850.0 {
                    42
                } else {
                    22
                };
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(
                        horizontal,
                        if compact { 8 } else { 20 },
                    ))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = if compact { 4.0 } else { 10.0 };
                        if compact {
                            ui.horizontal_top(|ui| {
                                actions.close |= ui
                                    .add_enabled_ui(picker.can_close(), |ui| {
                                        theme::icon_button(
                                            ui,
                                            self.tr("Volver a la biblioteca", "Back to library"),
                                            theme::Icon::Left,
                                            p,
                                        )
                                    })
                                    .inner
                                    .clicked();
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 2.0;
                                    ui.label(
                                        RichText::new(
                                            self.tr("Elige una portada", "Choose a cover"),
                                        )
                                        .font(
                                            egui::FontId::new(
                                                22.0,
                                                egui::FontFamily::Name("Editorial".into()),
                                            ),
                                        ),
                                    );
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(picker.document.display_title())
                                                .size(12.0)
                                                .color(p.secondary),
                                        )
                                        .truncate(),
                                    );
                                });
                            });
                        } else {
                            actions.close |= ui
                                .add_enabled_ui(picker.can_close(), |ui| {
                                    theme::control(
                                        ui,
                                        self.tr("Volver a la biblioteca", "Back to library"),
                                        Some(theme::Icon::Left),
                                        false,
                                        false,
                                        p,
                                    )
                                })
                                .inner
                                .clicked();
                            ui.add_space(18.0);
                            ui.label(
                                RichText::new(self.tr("Elige una portada", "Choose a cover")).font(
                                    egui::FontId::new(
                                        34.0,
                                        egui::FontFamily::Name("Editorial".into()),
                                    ),
                                ),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(picker.document.display_title())
                                        .size(16.0)
                                        .color(p.secondary),
                                )
                                .wrap(),
                            );
                        }
                        ui.add_space(if compact { 8.0 } else { 22.0 });
                        if ui.available_width() >= 820.0 && !compact {
                            ui.horizontal_top(|ui| {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(300.0, 0.0),
                                    egui::Layout::top_down(egui::Align::Center),
                                    |ui| {
                                        self.cover_picker_preview_image(
                                            ui, &picker, p, 300.0, 340.0,
                                        )
                                    },
                                );
                                ui.add_space(36.0);
                                let width = (ui.available_width() - 10.0).min(520.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(width, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.set_width(width);
                                        self.cover_picker_form(
                                            ui,
                                            &mut picker,
                                            &mut actions,
                                            p,
                                            false,
                                        );
                                    },
                                );
                            });
                        } else {
                            self.cover_picker_form(ui, &mut picker, &mut actions, p, compact);
                            ui.add_space(16.0);
                            self.cover_picker_preview_image(
                                ui,
                                &picker,
                                p,
                                ui.available_width().min(300.0),
                                if compact { 120.0 } else { 340.0 },
                            );
                        }
                    });
            });
        actions.close |= can_close && ui.input(|input| input.key_pressed(egui::Key::Escape));
        if actions.close {
            if picker.restoring {
                self.invalidate_library_cover(ui.ctx(), &picker.document.id);
            }
            drop(picker);
            self.runtime.wake();
            return;
        }
        let retry_read = actions.retry && !picker.loaded;
        let retry_save = actions.retry && picker.save_failed;
        let retry_restore = actions.retry && picker.restore_failed;
        let reload_committed = retry_restore && picker.restore_committed;
        let document = picker.document.clone();
        if retry_save || retry_restore {
            picker.error = None;
        }
        self.cover_picker = Some(picker);
        if retry_read || reload_committed {
            self.begin_cover_picker(document);
        } else if retry_save {
            self.save_cover_picker();
        } else if retry_restore {
            self.restore_cover_picker();
        } else if actions.changed || actions.retry {
            self.preview_cover_picker();
        }
        if actions.save {
            self.save_cover_picker();
        }
        if actions.restore {
            self.restore_cover_picker();
        }
    }

    fn cover_picker_preview_image(
        &self,
        ui: &mut egui::Ui,
        picker: &CoverPicker,
        p: Palette,
        max_width: f32,
        max_height: f32,
    ) {
        let aspect = picker.aspect.unwrap_or(0.75) as f32;
        let width = max_width.min(max_height * aspect);
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, width / aspect), egui::Sense::hover());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Image,
                true,
                self.tr("Vista previa de la portada PDF", "PDF cover preview"),
            )
        });
        ui.painter().add(
            egui::epaint::Shadow {
                offset: [0, 5],
                blur: 16,
                spread: 0,
                color: Color32::from_black_alpha(if self.dark { 70 } else { 24 }),
            }
            .as_shape(rect, 1),
        );
        if let Some(image) = &picker.image {
            ui.painter().image(
                image.id(),
                rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            ui.painter().rect_filled(rect, 1.0, p.surface);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.tr("Preparando vista previa…", "Preparing preview…"),
                egui::FontId::proportional(14.0),
                p.secondary,
            );
        }
        if let Some(page) = picker.image_page {
            ui.label(
                RichText::new(format!(
                    "{} · {} {}",
                    self.tr("Vista previa", "Preview"),
                    self.tr("Página", "Page"),
                    page.saturating_add(1)
                ))
                .size(13.0)
                .color(p.secondary),
            );
        }
    }

    fn cover_picker_form(
        &self,
        ui: &mut egui::Ui,
        picker: &mut CoverPicker,
        actions: &mut PickerActions,
        p: Palette,
        compact: bool,
    ) {
        let busy = picker.saving.is_some();
        if !compact {
            ui.label(RichText::new(self.tr("Página del PDF", "PDF source page")).size(18.0));
            ui.label(
                RichText::new(self.tr(
                    "Busca la página que mejor identifique este documento.",
                    "Find the page that best identifies this document.",
                ))
                .color(p.secondary),
            );
            ui.add_space(14.0);
        }
        ui.add_enabled_ui(!busy && picker.loaded, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled_ui(picker.page > 0, |ui| {
                        theme::icon_button(
                            ui,
                            self.tr("Página anterior de portada", "Previous cover page"),
                            theme::Icon::Left,
                            p,
                        )
                    })
                    .inner
                    .clicked()
                {
                    picker.page -= 1;
                    actions.changed = true;
                }
                let label = ui.label(self.tr("Página", "Page"));
                let mut shown = picker.page.saturating_add(1);
                let response = ui
                    .add(
                        egui::DragValue::new(&mut shown)
                            .range(1..=picker.document.page_count.max(1)),
                    )
                    .labelled_by(label.id);
                if response.changed() {
                    picker.page = shown.saturating_sub(1);
                    actions.changed = true;
                }
                ui.label(
                    RichText::new(format!("/ {}", picker.document.page_count)).color(p.secondary),
                );
                if ui
                    .add_enabled_ui(
                        picker.page.saturating_add(1) < picker.document.page_count,
                        |ui| {
                            theme::icon_button(
                                ui,
                                self.tr("Página siguiente de portada", "Next cover page"),
                                theme::Icon::Right,
                                p,
                            )
                        },
                    )
                    .inner
                    .clicked()
                {
                    picker.page += 1;
                    actions.changed = true;
                }
            });
        });
        if picker.loading.is_some() || picker.preview.is_some() {
            ui.label(
                RichText::new(self.tr("Cargando la página…", "Loading the page…"))
                    .color(p.secondary),
            );
        }
        if let Some(error) = &picker.error {
            let message = if matches!(error, yamantaka_domain::Failure::CoverSourceChanged) {
                self.tr("El PDF ha cambiado o no está disponible. Actualiza la vista previa antes de guardar.",
                    "The PDF changed or is unavailable. Refresh the preview before saving.")
            } else if matches!(error, yamantaka_domain::Failure::PasswordRequired) {
                self.tr(
                    "Abre y desbloquea el PDF en el lector antes de elegir su portada.",
                    "Open and unlock the PDF in the reader before choosing its cover.",
                )
            } else if picker.restore_committed {
                self.tr("La portada automática se guardó, pero no se pudo mostrar. Reintenta la vista previa.","The automatic cover was saved, but could not be displayed. Retry the preview.")
            } else {
                self.tr(
                    "No se pudo completar el cambio. La portada guardada se conserva.",
                    "The change could not be completed. The saved cover is preserved.",
                )
            };
            ui.label(message);
            let retry_label = if matches!(error, yamantaka_domain::Failure::CoverSourceChanged) {
                self.tr("Actualizar vista previa", "Refresh preview")
            } else {
                self.tr("Reintentar", "Retry")
            };
            actions.retry |= theme::control(ui, retry_label, None, false, false, p).clicked();
        }
        if !compact {
            ui.add_space(14.0);
            ui.label(
                RichText::new(self.tr(
                    "La elección se guarda al confirmar. El PDF original se conserva.",
                    "Your choice is saved when you confirm. The original PDF is preserved.",
                ))
                .color(p.secondary)
                .size(13.0),
            );
        }
        ui.add_space(if compact { 8.0 } else { 18.0 });
        ui.horizontal_wrapped(|ui| {
            actions.save |= ui
                .add_enabled_ui(picker.ready() && !actions.changed, |ui| {
                    theme::control(
                        ui,
                        if busy {
                            if picker.restoring {
                                self.tr("Restaurando…", "Restoring…")
                            } else {
                                self.tr("Guardando…", "Saving…")
                            }
                        } else {
                            self.tr("Usar como portada", "Use as cover")
                        },
                        None,
                        false,
                        true,
                        p,
                    )
                })
                .inner
                .clicked();
            actions.close |= ui
                .add_enabled_ui(picker.can_close(), |ui| {
                    theme::control(ui, self.tr("Cancelar", "Cancel"), None, false, false, p)
                })
                .inner
                .clicked();
        });
        ui.add_space(8.0);
        actions.restore |= ui
            .add_enabled_ui(picker.ready() && !actions.changed, |ui| {
                theme::control(
                    ui,
                    self.tr("Restaurar portada automática", "Restore automatic cover"),
                    None,
                    false,
                    false,
                    p,
                )
            })
            .inner
            .clicked();
    }
}
