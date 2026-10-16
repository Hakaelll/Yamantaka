use super::*;

#[derive(Default)]
pub(super) struct BackgroundImport {
    pending: Option<Cancellation>,
    catalogue_revision: u64,
}

impl BackgroundImport {
    pub(super) fn mark_refreshed(&mut self, revision: u64) {
        self.catalogue_revision = revision;
    }
    pub(super) fn busy(&self) -> bool {
        self.pending.is_some()
    }

    pub(super) fn begin(&mut self) -> Cancellation {
        self.cancel();
        let token = Cancellation::default();
        self.pending = Some(token.clone());
        token
    }

    pub(super) fn finish(&mut self, token: &Cancellation) -> bool {
        if !self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.same_request(token))
        {
            return false;
        }
        self.pending = None;
        true
    }

    fn cancel(&mut self) {
        if let Some(token) = self.pending.take() {
            token.cancel();
        }
    }
}

impl Drop for BackgroundImport {
    fn drop(&mut self) {
        self.cancel();
    }
}

impl YamantakaApp {
    pub(super) fn poll_import_progress(&mut self, ctx: &egui::Context) {
        if self.runtime.import_revision() == self.importing.catalogue_revision {
            return;
        }
        match self.try_refresh() {
            Ok(()) => {}
            Err(yamantaka_domain::Failure::QueueFull) => {
                ctx.request_repaint_after(Duration::from_millis(100))
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }
    pub(super) fn operation_actions(&mut self, ui: &mut egui::Ui, p: Palette) {
        if self.importing.pending.is_some()
            && theme::navigation(
                ui,
                self.tr("Cancelar importación", "Cancel import"),
                false,
                p,
            )
            .clicked()
        {
            self.importing.cancel();
            self.refresh();
            ui.close();
        }
        if !self.tabs.closing.busy()
            && (self.opening.busy() || self.tabs.busy())
            && theme::navigation(ui, self.tr("Cancelar apertura", "Cancel opening"), false, p)
                .clicked()
        {
            self.cancel_reader_opening();
            ui.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_do_not_cancel_reader_work_or_accept_superseded_completions() {
        let reader = Cancellation::default();
        let mut import = BackgroundImport::default();
        let first = import.begin();
        let next = import.begin();
        assert!(first.is_cancelled());
        assert!(!next.is_cancelled());
        assert!(!reader.is_cancelled());
        assert!(!import.finish(&first));
        assert!(!import.finish(&reader));
        assert!(import.finish(&next));
        assert!(!import.finish(&next));
        let cancelled = import.begin();
        import.cancel();
        assert!(cancelled.is_cancelled());
        assert!(!import.finish(&cancelled));
        let shutdown = import.begin();
        drop(import);
        assert!(shutdown.is_cancelled());
        assert!(!reader.is_cancelled());
        assert!(!next.is_cancelled());
    }
}
