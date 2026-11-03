use super::preparation::Prepared;
use super::*;

impl YamantakaApp {
    pub(super) fn adopt_prepared_open(&mut self, prepared: Prepared) {
        let Prepared {
            document,
            info,
            cancel,
            preferences,
            context,
            previews,
        } = prepared;
        let editing =
            self.reader.annotations.editing() && self.opening.matches(&document.id, &cancel);
        if editing {
            self.error = Some(
                self.tr(
                    "Termina o cancela la nota antes de cambiar de lectura.",
                    "Finish or cancel the note before switching readings.",
                )
                .into(),
            );
        }
        if cancel.is_cancelled() || !self.opening.matches(&document.id, &cancel) || editing {
            if self.opening.fail(&document.id, &cancel, false) {
                self.reader.loading = false;
                self.tabs.fail(&document.id, &cancel);
                self.reader.restoration.clear();
            }
            cancel.cancel();
            self.open_confirmation = Some((document.id.clone(), cancel));
            return;
        }
        let (target_page, reveal_reader) = self.open_intent.finish_open(&document.id);
        let page = target_page
            .unwrap_or(document.last_page_index)
            .min(info.pages.len().saturating_sub(1) as u32);
        let count = u32::try_from(info.pages.len()).unwrap_or(0);
        let navigation = match self
            .open_intent
            .take_navigation(&document.id)
            .filter(|nav| nav.page_count() == count)
            .map(|mut nav| {
                nav.go_to_page(page);
                nav
            })
            .map(Ok)
            .unwrap_or_else(|| ReaderNavigation::new(count, page))
        {
            Ok(navigation) => navigation,
            Err(error) => {
                self.error = Some(error.to_string());
                self.opening.fail(&document.id, &cancel, false);
                self.tabs.fail(&document.id, &cancel);
                self.reader.restoration.clear();
                cancel.cancel();
                self.open_confirmation = Some((document.id.clone(), cancel));
                self.reader.loading = false;
                return;
            }
        };
        if let Err(error) = self.finish_reading_session() {
            self.error = Some(error.to_string());
            self.opening.fail(&document.id, &cancel, false);
            self.tabs.fail(&document.id, &cancel);
            self.reader.restoration.clear();
            cancel.cancel();
            self.open_confirmation = Some((document.id.clone(), cancel));
            self.reader.loading = false;
            return;
        }
        let Some(preferences_cancel) = self.opening.accept(&document.id, &cancel) else {
            cancel.cancel();
            self.open_confirmation = Some((document.id.clone(), cancel));
            return;
        };
        self.record_reader_health(
            &document,
            yamantaka_domain::quarantine::ReaderHealth::Opened,
        );
        self.sync_tab_context();
        self.tabs
            .accept_open(&document.id, &cancel, preferences_cancel.clone());
        self.forget_failed_retirement(&document.id);
        self.replace_reader_state(preferences_cancel);
        self.reader.page = page;
        self.reader.navigation = Some(navigation);
        self.open_confirmation = Some((document.id.clone(), cancel));
        if target_page.is_some() {
            self.send(Action::Navigate(document.id.clone(), self.reader.page));
        }
        self.reader.active = Some((*document, Arc::new(info)));
        let id = self.reader.active.as_ref().unwrap().0.id.clone();
        if let Some(context) = context {
            self.reader
                .restoration
                .arm(id, self.reader.reader_preferences_cancel.clone(), context);
        }
        self.load_reader_preferences(preferences);
        self.opening
            .first_raster_probe
            .begin_adoption(self.reader.viewport.paint_count);
        for (mut key, image) in previews {
            self.opening.first_raster_probe.record_prepared(key.page);
            key.generation = self.reader.viewport.generation;
            self.reader.viewport.pages.insert(
                key.page,
                ReaderPage {
                    requested: None,
                    cancel: Cancellation::default(),
                    image: Some(image),
                    image_key: Some(key),
                    plan: Default::default(),
                    refinement: Default::default(),
                },
            );
        }
        if reveal_reader {
            self.show_surface(Surface::Reader);
        }
        self.reader.loading = false;
    }
}
