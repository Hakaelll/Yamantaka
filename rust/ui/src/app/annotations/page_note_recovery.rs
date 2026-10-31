use super::*;
use yamantaka_domain::annotations::page_note_draft::PageNoteDraft;

impl YamantakaApp {
    pub(in crate::app) fn submit_page_note_recovery_preview(&mut self) -> bool {
        if self.reader.annotations.pending.is_some() {
            return false;
        }
        let change = self.reader.annotations.mutation(&self.governor).unwrap();
        self.annotation_command(Some(change), 0);
        assert!(self.reader.annotations.writing());
        true
    }
}

impl AnnotationState {
    pub(in crate::app) fn seed_page_note_recovery(&mut self, note: &str) {
        let editor = self.editor.as_mut().unwrap();
        editor.note = note.into();
        editor.tags = "Study, Recovery".into();
        editor.color = AnnotationColor::Rose;
    }
    pub(in crate::app) fn page_note_draft(&self, reference: bool) -> Option<PageNoteDraft> {
        let editor = self.editor.as_ref()?;
        let EditorSource::PageNote {
            page,
            fingerprint,
            rotation,
        } = &editor.source
        else {
            return None;
        };
        Some(PageNoteDraft {
            document: self.document.clone()?,
            fingerprint: fingerprint.clone(),
            reference,
            page: *page,
            rotation: rotation.degrees(),
            color: editor.color,
            note: editor.note.clone(),
            tags: editor.tags.clone(),
        })
    }
}

impl reader_pane::ReaderPane {
    pub(in crate::app) fn restore_page_note(
        &mut self,
        draft: &PageNoteDraft,
    ) -> yamantaka_domain::Result<bool> {
        let Some((document, info)) = &self.active else {
            return Ok(false);
        };
        if document.id != draft.document
            || self.loading
            || self.annotations.editing()
            || self.annotations.writing()
        {
            return Ok(false);
        }
        if document.fingerprint != draft.fingerprint || draft.page as usize >= info.pages.len() {
            return Err(Failure::Invalid("The saved page-note draft belongs to a different PDF revision or page. Its recovery record is retained.".into()));
        }
        draft.validate()?;
        self.annotations.bind(&draft.document);
        self.annotations.editor = Some(AnnotationEditor {
            source: EditorSource::PageNote {
                page: draft.page,
                fingerprint: draft.fingerprint.clone(),
                rotation: PageRotation::from_degrees(draft.rotation.into())?,
            },
            kind: AnnotationKind::PageComment,
            color: draft.color,
            note: draft.note.clone(),
            tags: draft.tags.clone(),
            review: AnnotationReviewState::Pending,
            confirmed: false,
        });
        self.panel = true;
        self.reader_panel_tab = outline::ReaderPanelTab::Annotations;
        Ok(true)
    }
}
