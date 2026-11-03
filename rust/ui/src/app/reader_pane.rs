use super::*;

pub(super) struct ReaderPane {
    pub visual_profile: reader_profile::VisualProfile,
    pub restoration: restoration::RestorationState,
    pub viewport: reader_surface::ReaderSurface,
    pub active: Option<(Document, Arc<PdfInfo>)>,
    pub navigation: Option<ReaderNavigation>,
    pub reader_preferences: Option<ReaderPreferences>,
    pub reader_preferences_cancel: Cancellation,
    pub reader_preferences_loading: bool,
    pub page: u32,
    pub zoom: f32,
    pub panel: bool,
    pub reader_panel_tab: outline::ReaderPanelTab,
    pub find: find::FindState,
    pub links: links::LinkState,
    pub outline: outline::OutlineState,
    pub thumbnails: thumbnails::ThumbnailState,
    pub reader_destination: Option<(
        String,
        yamantaka_domain::pdf_destination::NavigationDestination,
    )>,
    pub bookmarks: bookmarks::BookmarkState,
    pub annotations: annotations::AnnotationState,
    pub annotation_paint: annotations::paint::AnnotationPaint,
    pub text: text::TextState,
    pub selection: selection::SelectionState,
    pub loading: bool,
}

impl Default for ReaderPane {
    fn default() -> Self {
        Self {
            visual_profile: Default::default(),
            restoration: Default::default(),
            viewport: Default::default(),
            active: None,
            navigation: None,
            reader_preferences: None,
            reader_preferences_cancel: Cancellation::default(),
            reader_preferences_loading: false,
            page: 0,
            zoom: 1.0,
            panel: false,
            reader_panel_tab: outline::ReaderPanelTab::Bookmarks,
            find: Default::default(),
            links: Default::default(),
            outline: Default::default(),
            thumbnails: Default::default(),
            reader_destination: None,
            bookmarks: bookmarks::BookmarkState::default(),
            annotations: Default::default(),
            annotation_paint: Default::default(),
            text: Default::default(),
            selection: Default::default(),
            loading: false,
        }
    }
}

impl ReaderPane {
    pub fn toggle_selection(&mut self) {
        if let Some((document, _)) = &self.active {
            self.selection
                .toggle(&document.id, self.page, &mut self.text);
        }
    }
    pub fn accept_text(
        &mut self,
        document: &str,
        page: u32,
        cancel: &Cancellation,
        result: std::result::Result<Arc<yamantaka_domain::pdf_text::PdfPageText>, String>,
    ) -> bool {
        if self.selection.owns(document, page, cancel) {
            self.selection.accept(document, page, cancel, result);
        } else if self.text.owns(document, page, cancel) {
            self.text.accept(document, page, cancel, result);
        } else {
            return false;
        }
        true
    }
    pub fn navigate_find(&mut self, target: find::FindTarget) -> bool {
        let changed = self
            .navigation
            .as_mut()
            .is_some_and(|navigation| match target.position {
                Some(position) => navigation.navigate_to(position),
                None => navigation.go_to_page(target.page),
            });
        if changed {
            self.viewport.scroll_anchor = true;
        }
        changed
    }
    pub fn prepare_replacement(&mut self, preferences: Cancellation) {
        self.links.clear();
        self.annotation_paint.invalidate();
        self.text.clear();
        self.selection.clear();
        self.thumbnails.clear();
        let preview = self.find.take_preview();
        self.find = Default::default();
        self.find.restore_preview(preview);
        self.reader_destination = None;
        self.outline = Default::default();
        self.reader_preferences_cancel.cancel();
        self.reader_preferences_cancel = preferences;
        self.reader_preferences = None;
        self.reader_preferences_loading = true;
        self.viewport.clear();
        self.viewport.generation += 1;
    }

    pub fn rebuild_layout(&mut self, direction: ReadingDirection) -> yamantaka_domain::Result<()> {
        let (Some((_, info)), Some(preferences)) = (&self.active, self.effective_preferences())
        else {
            return Ok(());
        };
        match ReaderLayout::new(
            info.pages.iter().map(|page| page.size()),
            &preferences,
            direction,
            24.0,
        ) {
            Ok(layout) => {
                if let Some(navigation) = &mut self.navigation {
                    navigation.apply_preferences(&preferences, direction);
                }
                self.viewport.layout = Some(Arc::new(layout));
                self.viewport.flat_pages =
                    self.visual_profile == reader_profile::VisualProfile::ElectronicInk;
                self.viewport.scroll_anchor = true;
                Ok(())
            }
            Err(error) => {
                self.viewport.layout = None;
                Err(error)
            }
        }
    }

    pub fn navigation_changed(&mut self) -> Option<(String, u32)> {
        self.restoration.clear();
        let (Some((document, _)), Some(navigation)) = (&self.active, &self.navigation) else {
            return None;
        };
        self.page = navigation.position().page();
        self.viewport.generation += 1;
        self.viewport.cancel.cancel();
        self.viewport.cancel = Cancellation::default();
        self.viewport.reset_requests();
        Some((document.id.clone(), self.page))
    }
}

impl Drop for ReaderPane {
    fn drop(&mut self) {
        self.reader_preferences_cancel.cancel();
        self.links.clear();
        self.annotation_paint.invalidate();
        self.text.clear();
        self.selection.clear();
    }
}
