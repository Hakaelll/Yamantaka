use super::*;
use yamantaka_domain::LibrarySmartView;
impl YamantakaApp {
    pub(super) fn library_has_filters(&self) -> bool {
        !self.query.trim().is_empty()
            || self.favorites
            || self.reading_state != LibraryReadingState::All
            || self.missing_only
            || self.modified_only
            || self.collections.filter.is_some()
            || self.smart_view.is_some()
            || !self.library_tags.filters.is_empty()
    }
    pub(super) fn reset_library_filters(&mut self) {
        self.query.clear();
        self.smart_view = None;
        self.clear_tag_filters();
        self.clear_collection_filter();
        self.reading_state = LibraryReadingState::All;
        self.missing_only = false;
        self.modified_only = false;
        self.favorites = false;
        self.offset = 0;
        self.refresh();
        self.persist_library_state();
    }
    pub fn preview_reset_library_filters(&mut self) {
        if !self.demo || self.capture.is_none() {
            return;
        }
        self.query = "Synthetic filter".into();
        self.favorites = true;
        self.reading_state = LibraryReadingState::Completed;
        self.missing_only = true;
        self.modified_only = true;
        self.smart_view = Some(LibrarySmartView::Missing);
        self.collections.filter = Some(("synthetic-empty".into(), "Empty collection".into()));
        self.library_tags.filters = vec![("synthetic-empty".into(), "Empty tag".into())];
        self.offset = 80;
        let sort = (self.sort, self.sort_direction);
        assert!(self.library_has_filters());
        self.reset_library_filters();
        assert!(!self.library_has_filters());
        assert_eq!(self.offset, 0);
        assert_eq!((self.sort, self.sort_direction), sort);
    }
    pub fn preview_smart_view(&mut self, key: &str, menu: bool) {
        if self.demo && self.capture.is_some() {
            self.smart_view = LibrarySmartView::from_key(key);
            self.smart_menu_preview = menu;
            self.offset = 0;
            self.refresh();
        }
    }
    fn smart_view_label(&self, view: LibrarySmartView) -> &'static str {
        let (es, en) = match view {
            LibrarySmartView::InProgress => ("En curso", "In progress"),
            LibrarySmartView::NotStarted => ("Sin empezar", "Not started"),
            LibrarySmartView::Finished => ("Finalizados", "Completed"),
            LibrarySmartView::Annotated => ("Con anotaciones", "With annotations"),
            LibrarySmartView::Bookmarked => ("Con marcadores", "With bookmarks"),
            LibrarySmartView::Missing => ("Archivos ausentes", "Missing files"),
            LibrarySmartView::RecentlyOpened => ("Abiertos recientemente", "Recently opened"),
            LibrarySmartView::RecentlyAdded => ("Añadidos recientemente", "Recently added"),
        };
        self.tr(es, en)
    }
    pub(super) fn library_state_filter(&mut self, ui: &mut egui::Ui, p: Palette) {
        let label = self
            .smart_view
            .map(|view| self.smart_view_label(view))
            .unwrap_or_else(|| {
                self.tr(
                    match self.reading_state {
                        LibraryReadingState::All => "Estado",
                        LibraryReadingState::InProgress => "En lectura",
                        LibraryReadingState::NotStarted => "Por leer",
                        LibraryReadingState::Completed => "Terminados",
                    },
                    match self.reading_state {
                        LibraryReadingState::All => "State",
                        LibraryReadingState::InProgress => "Reading",
                        LibraryReadingState::NotStarted => "Unread",
                        LibraryReadingState::Completed => "Finished",
                    },
                )
            });
        let response = theme::navigation(
            ui,
            label,
            self.smart_view.is_some()
                || self.reading_state != LibraryReadingState::All
                || self.missing_only
                || self.modified_only,
            p,
        );
        let mut popup = egui::Popup::menu(&response);
        if self.smart_menu_preview {
            popup = popup.open(true);
        }
        popup.show(|ui| {
            let height = (ui.ctx().content_rect().bottom() - ui.cursor().top() - 12.0).max(40.0);
            ui.set_max_height(height);
            egui::ScrollArea::vertical()
                .id_salt("library-state-views")
                .max_height(height)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(self.tr("Estado de lectura", "Reading state"))
                            .size(12.0)
                            .color(p.secondary),
                    );
                    for (state, es, en) in [
                        (
                            LibraryReadingState::All,
                            "Cualquier estado",
                            "Any reading state",
                        ),
                        (LibraryReadingState::InProgress, "En lectura", "Reading"),
                        (LibraryReadingState::NotStarted, "Por leer", "Unread"),
                        (LibraryReadingState::Completed, "Terminados", "Finished"),
                    ] {
                        if theme::navigation(ui, self.tr(es, en), self.reading_state == state, p)
                            .clicked()
                        {
                            self.reading_state = state;
                            self.offset = 0;
                            self.refresh();
                            ui.close();
                        }
                    }
                    ui.separator();
                    ui.label(
                        RichText::new(self.tr("Vistas inteligentes", "Smart views"))
                            .size(12.0)
                            .color(p.secondary),
                    );
                    if theme::navigation(
                        ui,
                        self.tr("Sin vista inteligente", "No smart view"),
                        self.smart_view.is_none(),
                        p,
                    )
                    .clicked()
                    {
                        self.smart_view = None;
                        self.offset = 0;
                        self.refresh();
                        ui.close();
                    }
                    for view in LibrarySmartView::ALL {
                        let selected = self.smart_view == Some(view);
                        let mut choice =
                            theme::navigation(ui, self.smart_view_label(view), selected, p);
                        if matches!(
                            view,
                            LibrarySmartView::RecentlyOpened | LibrarySmartView::RecentlyAdded
                        ) {
                            choice = choice
                                .on_hover_text(self.tr("Últimos 14 días.", "The last 14 days."));
                        }
                        if choice.clicked() {
                            self.smart_view = Some(view);
                            self.offset = 0;
                            self.refresh();
                            ui.close();
                        }
                        if selected && self.smart_menu_preview && self.text_scale >= 2.0 {
                            ui.scroll_to_rect(choice.rect, Some(egui::Align::Center));
                        }
                    }
                    ui.separator();
                    let label = self.tr("Solo archivos ausentes", "Only missing files");
                    if ui.checkbox(&mut self.missing_only, label).changed() {
                        self.offset = 0;
                        self.refresh();
                    }
                    let label = self.tr("Solo archivos modificados", "Modified files only");
                    if ui.checkbox(&mut self.modified_only, label).changed() {
                        self.offset = 0;
                        self.refresh();
                    }
                });
        });
    }
}
