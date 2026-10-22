use super::*;
use yamantaka_domain::workspaces::WorkspaceComparison;

pub(super) struct RestoreComparison {
    saved: WorkspaceComparison,
    document: Option<Document>,
    started: bool,
}

impl YamantakaApp {
    pub(super) fn capture_workspace_comparison(&self) -> Option<WorkspaceComparison> {
        let (primary, _) = self.reader.active.as_ref()?;
        self.reference.as_ref()?.snapshot(&primary.id)
    }

    pub(super) fn schedule_workspace_comparison(
        &mut self,
        view: &yamantaka_domain::workspaces::WorkspaceView,
        document: Option<Document>,
        warning: Option<String>,
    ) {
        if let Some(warning) = warning {
            self.error = Some(format!(
                "{}: {warning}",
                self.tr(
                    "La comparación guardada no se pudo recuperar",
                    "The saved comparison could not be restored"
                )
            ));
        }
        self.workspaces.comparison_restore = view
            .parsed_query()
            .ok()
            .and_then(|query| query.comparison)
            .map(|saved| RestoreComparison {
                saved,
                document,
                started: false,
            });
    }

    pub(in crate::app) fn pump_workspace_comparison(&mut self) {
        let Some(mut restore) = self.workspaces.comparison_restore.take() else {
            return;
        };
        if self.tabs.busy() || self.opening.busy() || self.reader.reader_preferences_loading {
            self.workspaces.comparison_restore = Some(restore);
            return;
        }
        if self
            .reader
            .active
            .as_ref()
            .is_none_or(|(doc, _)| doc.id != restore.saved.primary)
        {
            self.finish_workspace_comparison(Some(
                "The saved reference belongs to another reading".into(),
            ));
            return;
        }
        if !restore.started {
            let Some(document) = restore.document.take().filter(|doc| !doc.missing) else {
                self.finish_workspace_comparison(Some(self.tr("No se encuentra el PDF de referencia. La lectura principal se ha recuperado.", "The reference PDF is missing. The main reading was restored.").into()));
                return;
            };
            if self.reference.is_some() {
                self.finish_workspace_comparison(Some(
                    "Close the current reference before restoring a comparison".into(),
                ));
                return;
            }
            match reference::ReferenceView::new(&self.reader, self.reading_direction()) {
                Ok(reference) => self.reference = Some(reference),
                Err(error) => {
                    self.finish_workspace_comparison(Some(error.to_string()));
                    return;
                }
            }
            self.open_reference(document);
            restore.started = true;
        }
        if self
            .reference
            .as_ref()
            .is_some_and(|reference| reference.busy())
        {
            self.workspaces.comparison_restore = Some(restore);
            return;
        }
        let direction = self.reading_direction();
        let result = self
            .reference
            .as_mut()
            .ok_or_else(|| {
                yamantaka_domain::Failure::Invalid("Reference restoration was cancelled".into())
            })
            .and_then(|reference| reference.restore(&restore.saved, direction));
        self.finish_workspace_comparison(result.err().map(|error| error.to_string()));
    }

    fn finish_workspace_comparison(&mut self, error: Option<String>) {
        let actual = self.capture_workspace_comparison();
        if let Some(binding) = &mut self.workspaces.binding {
            binding.comparison = actual;
        }
        if let Some(error) = error {
            self.error = Some(error);
        }
    }
}
