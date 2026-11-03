use super::*;
use yamantaka_domain::workspaces::WorkspaceComparison;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Origin {
    Workspace,
    Continuity,
}

pub(super) struct RestoreComparison {
    origin: Origin,
    saved: WorkspaceComparison,
    document: Option<Document>,
    started: bool,
}

impl YamantakaApp {
    pub(in crate::app) fn schedule_continuity_comparison(
        &mut self,
        saved: WorkspaceComparison,
        document: Option<Document>,
    ) {
        self.workspaces.comparison_restore = Some(RestoreComparison {
            origin: Origin::Continuity,
            saved,
            document,
            started: false,
        });
    }
    pub(in crate::app) fn cancel_continuity_comparison_restore(&mut self) {
        if self
            .workspaces
            .comparison_restore
            .as_ref()
            .is_some_and(|restore| restore.origin == Origin::Continuity)
        {
            self.workspaces.comparison_restore = None;
        }
    }

    pub(super) fn schedule_workspace_comparison(
        &mut self,
        view: &yamantaka_domain::workspaces::WorkspaceView,
        document: Option<Document>,
        warning: Option<String>,
    ) {
        self.discard_retained_comparison();
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
                origin: Origin::Workspace,
                saved,
                document,
                started: false,
            });
    }

    pub(in crate::app) fn pump_workspace_comparison(&mut self) {
        let Some(mut restore) = self.workspaces.comparison_restore.take() else {
            return;
        };
        if self.tabs.busy()
            || self.opening.busy()
            || self.open_confirmation.is_some()
            || self.reader.reader_preferences_loading
        {
            self.workspaces.comparison_restore = Some(restore);
            return;
        }
        if self
            .reader
            .active
            .as_ref()
            .is_none_or(|(doc, _)| doc.id != restore.saved.primary)
        {
            self.finish_workspace_comparison(
                restore.origin,
                Some("The saved reference belongs to another reading".into()),
            );
            return;
        }
        if !restore.started {
            let Some(document) = restore.document.take().filter(|doc| !doc.missing) else {
                self.finish_workspace_comparison(restore.origin,Some(self.tr("No se encuentra el PDF de referencia. La lectura principal se ha recuperado.", "The reference PDF is missing. The main reading was restored.").into()));
                return;
            };
            if self.reference.is_some() {
                self.finish_workspace_comparison(
                    restore.origin,
                    Some("Close the current reference before restoring a comparison".into()),
                );
                return;
            }
            match reference::ReferenceView::new(&self.reader, self.reading_direction()) {
                Ok(mut reference) => {
                    reference.deferred = true;
                    self.reference = Some(reference);
                }
                Err(error) => {
                    self.finish_workspace_comparison(restore.origin, Some(error.to_string()));
                    return;
                }
            }
            self.open_reference_with_context(document, Some(restore.saved.clone()));
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
            .and_then(|reference| {
                reference.restore(&restore.saved, direction)?;
                reference.release_deferred()
            });
        self.finish_workspace_comparison(
            restore.origin,
            result.err().map(|error| error.to_string()),
        );
    }

    fn finish_workspace_comparison(&mut self, origin: Origin, error: Option<String>) {
        if origin == Origin::Continuity {
            self.finish_continuity_comparison(error.is_none());
            if error.is_some() {
                self.close_reference_preserving_continuity();
            }
        } else if error.is_some()
            && self
                .reference
                .as_ref()
                .is_some_and(|reference| reference.deferred)
        {
            self.close_reference_preserving_continuity();
        }
        let actual = self.capture_comparison();
        if origin == Origin::Workspace
            && let Some(binding) = &mut self.workspaces.binding
        {
            binding.comparison = actual;
        }
        if let Some(error) = error {
            self.error = Some(error);
        }
    }
}
