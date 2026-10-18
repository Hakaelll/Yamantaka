use super::view::Actions;
use super::*;
use yamantaka_domain::metadata_review::MetadataEvidenceKind;

impl YamantakaApp {
    pub(super) fn metadata_reference_panel(
        &self,
        ui: &mut egui::Ui,
        editor: &mut MetadataEditor,
        actions: &mut Actions,
        p: Palette,
    ) {
        let Some(snapshot) = editor.snapshot.as_ref() else {
            return;
        };
        let snapshot = snapshot.clone();
        let response = egui::Frame::new()
            .fill(p.surface)
            .stroke(egui::Stroke::new(1.0, p.line))
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(self.tr("Referencia del documento", "Document reference"))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(12.0);
                ui.horizontal_wrapped(|ui| {
                    for (field, name) in [
                        (MetadataField::Title, self.tr("Título", "Title")),
                        (MetadataField::Author, self.tr("Autor", "Author")),
                    ] {
                        if ui
                            .add_enabled_ui(snapshot.evidence(field).is_some(), |ui| {
                                theme::control(
                                    ui,
                                    name,
                                    None,
                                    editor.reference.field == field,
                                    false,
                                    p,
                                )
                            })
                            .inner
                            .clicked()
                        {
                            editor.reference.field = field;
                        }
                    }
                });
                ui.add_space(14.0);
                let width = ui.available_width();
                let height = 240.0;
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
                ui.painter().rect_filled(rect, 4.0, p.raised);
                if let Some(image) = &editor.reference.image {
                    let aspect = editor.reference.aspect.unwrap_or(0.75);
                    let size = egui::vec2((height - 24.0) * aspect, height - 24.0);
                    let size = size * ((width - 24.0) / size.x).min(1.0);
                    ui.painter().image(
                        image.id(),
                        egui::Rect::from_center_size(rect.center(), size),
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        self.tr("Página del PDF", "PDF page"),
                        egui::FontId::proportional(15.0),
                        p.secondary,
                    );
                }
                if let Some(page) = editor.reference.page {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("{} {}", self.tr("Página", "Page"), page + 1))
                            .size(13.0)
                            .color(p.secondary),
                    );
                }
                if let Some(evidence) = snapshot.evidence(editor.reference.field) {
                    ui.add_space(14.0);
                    if editor.reference.page == Some(evidence.page) {
                        ui.add(
                            egui::Label::new(RichText::new(&evidence.value).font(
                                egui::FontId::new(20.0, egui::FontFamily::Name("Editorial".into())),
                            ))
                            .wrap(),
                        );
                        ui.label(
                            RichText::new(match evidence.kind {
                                MetadataEvidenceKind::TitleWithByline => self.tr(
                                    "Título de portada acompañado de una firma",
                                    "Cover title accompanied by a byline",
                                ),
                                MetadataEvidenceKind::ProminentText => self
                                    .tr("Texto destacado del documento", "Prominent document text"),
                                MetadataEvidenceKind::RepeatedTitleWithByline => self.tr(
                                    "Título repetido junto a una firma",
                                    "Repeated title beside a byline",
                                ),
                                MetadataEvidenceKind::ExplicitByline => {
                                    self.tr("Firma de autor explícita", "Explicit author byline")
                                }
                                MetadataEvidenceKind::ConflictingBylines => self.tr(
                                    "El PDF contiene firmas diferentes",
                                    "The PDF contains different bylines",
                                ),
                                MetadataEvidenceKind::Corroborated => self.tr(
                                    "Coincide con los metadatos del PDF",
                                    "Matches the PDF metadata",
                                ),
                            })
                            .size(13.0)
                            .color(p.secondary),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "{} {}",
                                self.tr("Referencia: página", "Reference: page"),
                                evidence.page + 1
                            ))
                            .color(p.secondary),
                        );
                    }
                    ui.add_space(10.0);
                    if ui
                        .add_enabled_ui(
                            editor.ready()
                                && editor.clean_draft()
                                && !snapshot.file.missing
                                && editor.error.is_none(),
                            |ui| {
                                theme::control(
                                    ui,
                                    &format!(
                                        "{} {} {}",
                                        self.tr("Abrir página", "Open page"),
                                        evidence.page + 1,
                                        self.tr("en el lector", "in reader")
                                    ),
                                    Some(theme::Icon::Right),
                                    false,
                                    false,
                                    p,
                                )
                            },
                        )
                        .inner
                        .on_hover_text(self.tr(
                            "Guarda o descarta el borrador antes de salir.",
                            "Save or discard your draft before leaving.",
                        ))
                        .clicked()
                    {
                        actions.source = Some(editor.reference.field);
                    }
                }
                if editor.reference.error.is_some() {
                    ui.add_space(10.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(self.tr(
                                "No se pudo mostrar esta página.",
                                "This page could not be shown.",
                            ))
                            .color(p.secondary),
                        )
                        .wrap(),
                    );
                    if theme::navigation(
                        ui,
                        self.tr("Reintentar vista previa", "Retry preview"),
                        false,
                        p,
                    )
                    .clicked()
                    {
                        actions.retry_preview = true;
                    }
                }
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(12.0);
                ui.label(
                    RichText::new(self.tr("Datos incluidos en el PDF", "Embedded PDF metadata"))
                        .strong(),
                );
                for (label, value) in [
                    (self.tr("Título", "Title"), snapshot.source_title.as_deref()),
                    (
                        self.tr("Autor", "Author"),
                        snapshot.source_author.as_deref(),
                    ),
                ] {
                    ui.add_space(8.0);
                    ui.label(RichText::new(label).size(13.0).color(p.secondary));
                    ui.add(
                        egui::Label::new(
                            value
                                .filter(|s| !s.is_empty())
                                .unwrap_or(self.tr("Sin especificar", "Not specified")),
                        )
                        .wrap(),
                    );
                }
            })
            .response;
        if editor.reference.reveal {
            response.scroll_to_me(Some(egui::Align::Center));
            editor.reference.reveal = false;
        }
    }
}
