use super::*;
use yamantaka_domain::annotations::stored_note_draft::StoredNoteDraft;

fn excerpt(value: &str, characters: usize) -> &str {
    &value[..value
        .char_indices()
        .nth(characters)
        .map_or(value.len(), |(index, _)| index)]
}

impl AnnotationState {
    pub(in crate::app) fn stored_note_draft(&self, reference: bool) -> Option<StoredNoteDraft> {
        if let Some(RecoveredDraft::Stored(draft)) = &self.recovered {
            return Some(draft.clone());
        }
        let editor = self.editor.as_ref()?;
        if !self.unsaved_editor() {
            return None;
        }
        let record = editor.stored_record()?;
        let data = record.data();
        Some(StoredNoteDraft {
            document: data.values.document_id.clone(),
            annotation: data.id.clone(),
            reference,
            revision: record.revision().clone(),
            fingerprint: data.values.document_fingerprint.clone(),
            page: data.values.page_index,
            rotation: data.values.page_rotation,
            passage: data.values.selected_text.clone().unwrap_or_default(),
            kind: editor.kind,
            color: editor.color,
            review: editor.review,
            note: editor.note.clone(),
            tags: editor.tags.clone(),
        })
    }
    pub(in crate::app) fn install_stored_draft(
        &mut self,
        draft: StoredNoteDraft,
        record: Option<AnnotationRecord>,
    ) {
        self.bind(&draft.document);
        self.saved = false;
        self.error = None;
        self.stale = false;
        if let Some(record) = record {
            self.editor = Some(AnnotationEditor {
                source: EditorSource::RecoveredStored {
                    record: Arc::new(record),
                },
                kind: draft.kind,
                color: draft.color,
                review: draft.review,
                note: draft.note,
                tags: draft.tags,
                confirmed: false,
            });
        } else {
            self.recovered = Some(RecoveredDraft::Stored(draft));
        }
    }
    pub(super) fn stored_recovery_panel(
        &mut self,
        ui: &mut egui::Ui,
        p: Palette,
        english: bool,
    ) -> Option<NotesAction> {
        let draft = RecoveryView::new(self.recovered.as_ref()?);
        let tr = |es, en| if english { en } else { es };
        ui.label(
            RichText::new(tr("Borrador recuperado", "Recovered draft"))
                .strong()
                .color(p.ink),
        );
        ui.label(if draft.selection {
            tr("El pasaje ha cambiado o no está disponible. Este borrador se conserva por separado.", "The passage changed or is unavailable. This draft is kept separately.")
        } else {
            tr("La anotación ha cambiado o no está disponible. Este borrador se conserva por separado.", "The annotation changed or is unavailable. This draft is kept separately.")
        });
        ui.add_space(12.0);
        ui.label(format!(
            "{} · {} · {}",
            panel::kind_label(draft.kind, english),
            panel::color_label(draft.color, english),
            panel::review_label(draft.review, english)
        ));
        egui::ScrollArea::vertical()
            .id_salt("stored-note-recovery-text")
            .max_height(180.0)
            .show(ui, |ui| {
                if !draft.passage.is_empty() {
                    ui.label(excerpt(draft.passage, 2000));
                    ui.add_space(8.0);
                }
                ui.label(excerpt(draft.note, 4000));
                ui.label(excerpt(draft.tags, 160));
            });
        if excerpt(draft.passage, 2000).len() < draft.passage.len()
            || excerpt(draft.note, 4000).len() < draft.note.len()
            || excerpt(draft.tags, 160).len() < draft.tags.len()
        {
            ui.label(
                RichText::new(tr(
                    "Vista previa. Copiar borrador incluye todo el contenido.",
                    "Preview. Copy draft includes the complete content.",
                ))
                .color(p.secondary),
            );
        }
        ui.add_space(12.0);
        let mut retry = false;
        let mut discard = false;
        ui.horizontal_wrapped(|ui| {
            if theme::control(
                ui,
                tr("Copiar borrador", "Copy draft"),
                None,
                true,
                false,
                p,
            )
            .clicked()
            {
                ui.ctx().copy_text(format!(
                    "{}\n{}\n{}\n{}\n\n{}\n\n{}",
                    panel::kind_label(draft.kind, english),
                    panel::color_label(draft.color, english),
                    panel::review_label(draft.review, english),
                    draft.tags,
                    draft.passage,
                    draft.note
                ));
            }
            retry = theme::navigation(
                ui,
                tr("Reintentar recuperación", "Retry recovery"),
                false,
                p,
            )
            .clicked();
            discard = theme::navigation(ui, tr("Descartar borrador", "Discard draft"), false, p)
                .clicked();
        });
        if discard {
            self.recovered = None;
            self.stale = true;
        }
        retry.then_some(NotesAction::RetryDraft)
    }
    pub(in crate::app) fn recovery_conflict(&self) -> bool {
        matches!(self.recovered, Some(RecoveredDraft::Stored(_)))
    }
    pub(in crate::app) fn retry_draft(&self) -> Option<StoredNoteDraft> {
        match &self.recovered {
            Some(RecoveredDraft::Stored(draft)) => Some(draft.clone()),
            _ => None,
        }
    }
    pub(in crate::app) fn clear_recovered_draft(&mut self) {
        self.recovered = None;
    }
    pub(in crate::app) fn seed_stored_note_recovery(&mut self, text: &str) {
        self.edit_entry(0);
        let editor = self.editor.as_mut().unwrap();
        editor.note = text.into();
        editor.tags = "Study, Recovered edit".into();
        editor.color = AnnotationColor::Rose;
        editor.review = AnnotationReviewState::Accepted;
    }
}

struct RecoveryView<'a> {
    kind: AnnotationKind,
    color: AnnotationColor,
    review: AnnotationReviewState,
    passage: &'a str,
    note: &'a str,
    tags: &'a str,
    selection: bool,
}
impl<'a> RecoveryView<'a> {
    fn new(draft: &'a RecoveredDraft) -> Self {
        match draft {
            RecoveredDraft::Stored(d) => Self {
                kind: d.kind,
                color: d.color,
                review: d.review,
                passage: &d.passage,
                note: &d.note,
                tags: &d.tags,
                selection: false,
            },
            RecoveredDraft::Selection(d) => Self {
                kind: d.kind,
                color: d.color,
                review: AnnotationReviewState::Pending,
                passage: &d.passage,
                note: &d.note,
                tags: &d.tags,
                selection: true,
            },
        }
    }
}
