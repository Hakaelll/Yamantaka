use super::*;
use yamantaka_domain::annotations::selection_note_draft::SelectionNoteDraft;

impl AnnotationState {
    pub(in crate::app) fn selection_note_draft(
        &self,
        reference: bool,
    ) -> Option<SelectionNoteDraft> {
        if let Some(RecoveredDraft::Selection(draft)) = &self.recovered {
            return Some(draft.clone());
        }
        let editor = self.editor.as_ref()?;
        let EditorSource::Selection {
            snapshot,
            fingerprint,
            rotation,
        } = &editor.source
        else {
            return None;
        };
        let range = snapshot.selection();
        Some(SelectionNoteDraft {
            document: self.document.clone()?,
            reference,
            fingerprint: fingerprint.clone(),
            rotation: rotation.degrees(),
            start: [range.start().page(), range.start().character() as u32],
            end: [range.end().page(), range.end().character() as u32],
            passage: snapshot.text().into(),
            kind: editor.kind,
            color: editor.color,
            note: editor.note.clone(),
            tags: editor.tags.clone(),
            confirmed: editor.confirmed,
        })
    }
    pub(in crate::app) fn install_selection_draft(
        &mut self,
        draft: SelectionNoteDraft,
        snapshot: Option<TextSelectionSnapshot>,
    ) {
        self.bind(&draft.document);
        self.saved = false;
        self.error = None;
        self.stale = false;
        if let Some(snapshot) = snapshot {
            self.editor = Some(AnnotationEditor {
                source: EditorSource::Selection {
                    snapshot,
                    fingerprint: draft.fingerprint,
                    rotation: PageRotation::from_degrees(draft.rotation.into())
                        .expect("validated recovery rotation"),
                },
                kind: draft.kind,
                color: draft.color,
                review: AnnotationReviewState::Pending,
                note: draft.note,
                tags: draft.tags,
                confirmed: draft.confirmed,
            });
        } else {
            self.recovered = Some(RecoveredDraft::Selection(draft));
        }
    }
    pub(in crate::app) fn selection_geometry_pages(&self) -> usize {
        match self.editor.as_ref().map(|editor| &editor.source) {
            Some(EditorSource::Selection { snapshot, .. }) => snapshot.pages().len(),
            _ => 0,
        }
    }
    pub(in crate::app) fn selection_recovery_conflict(&self) -> bool {
        matches!(self.recovered, Some(RecoveredDraft::Selection(_)))
    }
}
