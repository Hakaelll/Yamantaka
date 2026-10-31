use super::*;
use yamantaka_domain::metadata_review::draft::MetadataDraft;

impl MetadataEditor {
    pub(in crate::app) fn is_document(&self, document: &str) -> bool {
        self.document.id == document
    }
    pub(in crate::app) fn accept_shutdown_reply(&mut self, reply: MetadataReply) {
        self.accept(reply);
    }
    pub(in crate::app) fn recovery_draft(&self) -> Option<MetadataDraft> {
        if let Some(draft) = &self.recovery {
            return Some(draft.clone());
        }
        let snapshot = self.snapshot.as_ref()?;
        self.unsaved().then(|| MetadataDraft {
            document: self.document.id.clone(),
            revision: snapshot.revision.clone(),
            title: self.title.clone(),
            author: self.author.clone(),
        })
    }
    pub(in crate::app) fn restore_draft(&mut self, draft: MetadataDraft) {
        if self
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.revision == draft.revision)
        {
            self.title = draft.title;
            self.author = draft.author;
        } else {
            self.recovery = Some(draft);
        }
    }
    pub(super) fn recovery_panel(&mut self, ui: &mut egui::Ui, p: Palette, english: bool) {
        let Some(draft) = &self.recovery else {
            return;
        };
        let tr = |es, en| if english { en } else { es };
        ui.label(
            RichText::new(tr(
                "Borrador de una revisión anterior",
                "Draft from an earlier revision",
            ))
            .strong()
            .color(p.ink),
        );
        ui.label(tr(
            "Los metadatos han cambiado. Revisa el texto recuperado antes de usarlo.",
            "Metadata has changed. Review the recovered text before using it.",
        ));
        egui::ScrollArea::vertical()
            .id_salt("metadata-recovery-text")
            .max_height(120.0)
            .show(ui, |ui| {
                ui.label(&draft.title);
                ui.label(&draft.author);
            });
        let mut use_text = false;
        let mut keep_current = false;
        ui.horizontal_wrapped(|ui| {
            use_text = theme::control(
                ui,
                tr("Usar texto recuperado", "Use recovered text"),
                None,
                false,
                false,
                p,
            )
            .clicked();
            keep_current = theme::navigation(
                ui,
                tr("Conservar valores actuales", "Keep current values"),
                false,
                p,
            )
            .clicked();
        });
        if use_text && let Some(draft) = self.recovery.take() {
            self.title = draft.title;
            self.author = draft.author;
        } else if keep_current {
            self.recovery = None;
        }
        ui.add_space(16.0);
    }
}
