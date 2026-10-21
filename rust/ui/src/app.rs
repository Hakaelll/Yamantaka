mod annotations;
mod backup_restore;
mod backups;
pub use backup_restore::RestoreHandover;
mod bookmarks;
mod bulk_picker;
mod cache_maintenance;
mod collections;
mod compact;
mod cover_picker;
mod cover_picker_preview;
mod cover_picker_view;
mod cover_restore;
mod details;
mod file_check;
mod file_comparison;
mod find;
mod global_notes;
mod goals;
mod history;
mod legacy_import;
mod library_check;
mod library_covers;
mod library_grouping;
mod library_grouping_grid;
mod library_grouping_preview;
mod library_intro;
mod library_selection;
mod links;
mod list;
mod metadata;
mod navigation;
mod open_intent;
mod opening;
mod operations;
mod outline;
mod password;
mod reader_controls;
mod reader_destination;
mod reader_options;
mod reader_viewport;
mod reading_queue;
mod relocation;
mod render_plan;
mod render_updates;
mod restoration;
mod search;
mod selection;
mod sessions;
mod settings;
mod shortcuts;
mod smart_views;
mod tabs;
mod tags;
mod text;
mod thumbnails;
mod workspaces;

use crate::{
    book,
    page_presenter::{PagePresenter, PageTexture},
    theme::{self, Palette},
};
use eframe::{
    App, Frame,
    egui::{self, Color32, RichText, TextureHandle, TextureOptions, ViewportCommand},
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, mpsc::Receiver},
    time::{Duration, Instant},
};
use yamantaka_application::runtime::{Action, Runtime, Update};
use yamantaka_domain::memory::MemoryGovernor;
use yamantaka_domain::reader::{MarkSlot, ReaderNavigation};
use yamantaka_domain::reader_layout::ReaderLayout;
use yamantaka_domain::reader_preferences::{ReaderPreferences, ReaderTheme, ZoomMode};
use yamantaka_domain::{
    Cancellation, Document, LibraryQuery, LibraryReadingState, LibrarySort, PdfInfo,
    ReadingDirection, RenderKey, SortDirection,
};
use yamantaka_storage::preferences::Preferences;
use yamantaka_windows::{ActivationWatch, Instance};

enum Surface {
    Search,
    Notes,
    History,
    Library,
    Reader,
    Settings,
    Collections,
    Tags,
}
enum PageImage {
    Native(PageTexture),
    Managed(TextureHandle),
}
struct ReaderPage {
    requested: Option<RenderKey>,
    cancel: Cancellation,
    image: Option<PageImage>,
    image_key: Option<RenderKey>,
    plan: render_plan::RenderPlan,
}
impl PageImage {
    fn id(&self) -> egui::TextureId {
        match self {
            Self::Native(texture) => texture.id(),
            Self::Managed(texture) => texture.id(),
        }
    }
}
pub struct YamantakaApp {
    search: search::SearchState,
    global_notes: global_notes::GlobalNotes,
    history: history::HistoryState,
    goals: goals::GoalState,
    metadata: Option<metadata::MetadataEditor>,
    metadata_preview: Option<(&'static str, u8)>,
    open_intent: open_intent::OpenIntent,
    opening: opening::ReaderOpening,
    tabs: tabs::TabState,
    reading_queue: reading_queue::QueueState,
    workspaces: workspaces::WorkspaceState,
    restoration: restoration::RestorationState,
    open_confirmation: Option<(String, Cancellation)>,
    password: Option<password::PasswordChallenge>,
    password_attempted: bool,
    preview_password_action: Option<&'static str>,
    preview_password_target: Option<&'static str>,
    runtime: Runtime,
    preferences: Preferences,
    sessions: sessions::SessionState,
    store: PathBuf,
    dark: bool,
    english: bool,
    demo: bool,
    reduced_motion: bool,
    text_scale: f32,
    surface: Surface,
    documents: Vec<Document>,
    count: u64,
    grouping: library_grouping::GroupingState,
    query: String,
    favorites: bool,
    reading_state: LibraryReadingState,
    sort: LibrarySort,
    sort_direction: SortDirection,
    missing_only: bool,
    modified_only: bool,
    list: bool,
    offset: usize,
    selected: Option<String>,
    inspector: bool,
    active: Option<(Document, Arc<PdfInfo>)>,
    navigation: Option<ReaderNavigation>,
    reader_preferences: Option<ReaderPreferences>,
    reader_preferences_cancel: Cancellation,
    reader_preferences_loading: bool,
    page: u32,
    zoom: f32,
    reader_scale: f32,
    panel: bool,
    reader_panel_tab: outline::ReaderPanelTab,
    find: find::FindState,
    links: links::LinkState,
    link_preview: links::LinkPreview,
    find_preview_step: Option<bool>,
    pub find_preview_closed: bool,
    outline: outline::OutlineState,
    thumbnails: thumbnails::ThumbnailState,
    library_covers: library_covers::LibraryCovers,
    cover_picker: Option<cover_picker::CoverPicker>,
    cover_picker_demo: cover_picker_preview::CoverPickerDemo,
    thumbnail_preview_page: Option<u32>,
    outline_preview_target: Option<usize>,
    reader_destination: Option<(
        String,
        yamantaka_domain::pdf_destination::NavigationDestination,
    )>,
    bookmarks: bookmarks::BookmarkState,
    annotations: annotations::AnnotationState,
    annotation_export: annotations::export::ExportState,
    file_comparison: file_comparison::ComparisonState,
    file_check: file_check::CheckState,
    cache_maintenance: cache_maintenance::CacheState,
    legacy_import: legacy_import::ImportState,
    backups: backups::BackupState,
    backup_restore: backup_restore::RestoreState,
    collections: collections::CollectionState,
    library_tags: tags::TagState,
    smart_view: Option<yamantaka_domain::LibrarySmartView>,
    smart_menu_preview: bool,
    library_check: library_check::CheckState,
    relocation: relocation::RelocationState,
    annotation_paint: annotations::paint::AnnotationPaint,
    annotation_preview: u8,
    annotation_multi_preview: bool,
    text: text::TextState,
    selection: selection::SelectionState,
    selection_preview: selection::SelectionPreview,
    selection_keyboard_preview: bool,
    reader_pages: HashMap<u32, ReaderPage>,
    reader_layout: Option<Arc<ReaderLayout>>,
    reader_scroll_anchor: bool,
    presenter: Option<PagePresenter>,
    governor: MemoryGovernor,
    generation: u64,
    requested: Option<RenderKey>,
    render_retry: bool,
    cancel: Cancellation,
    bulk_organization_preview: Option<bulk_picker::OrganizationPreview>,
    bulk_organization_verified: Option<serde_json::Value>,
    bulk_picker: Option<bulk_picker::BulkPicker>,
    library_selection: library_selection::SelectionState,
    query_cancel: Cancellation,
    library_loaded: bool,
    error: Option<String>,
    loading: bool,
    importing: operations::BackgroundImport,
    preview_tools_menu: bool,
    activation: Receiver<Vec<PathBuf>>,
    _instance: std::rc::Rc<Instance>,
    _watch: ActivationWatch,
    capture: Option<PathBuf>,
    capture_requested: bool,
    capture_ready_since: Option<Instant>,
    capture_render_events: Vec<serde_json::Value>,
    preview_reader: bool,
    logo: Option<TextureHandle>,
    preview_tilt: bool,
    preview_list_focus: bool,
    preview_reader_options: bool,
    reader_options_section: usize,
}
impl YamantakaApp {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        runtime: Runtime,
        preferences: Preferences,
        store: PathBuf,
        dark: bool,
        english: bool,
        demo: bool,
        settings: bool,
        capture: Option<PathBuf>,
        activation: Receiver<Vec<PathBuf>>,
        instance: std::rc::Rc<Instance>,
        watch: ActivationWatch,
        governor: MemoryGovernor,
        presenter: Option<PagePresenter>,
    ) -> Self {
        let reduced_motion = preferences
            .get("reduceMotion")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let text_scale = preferences
            .get("textScale")
            .and_then(serde_json::Value::as_f64)
            .filter(|scale| scale.is_finite())
            .unwrap_or(1.0)
            .clamp(1.0, 2.0) as f32;
        let grouping = library_grouping::GroupingState::from_key(
            preferences
                .get("libraryGrouping")
                .and_then(serde_json::Value::as_str),
        );
        let sort = match preferences
            .get("librarySort")
            .and_then(serde_json::Value::as_str)
        {
            Some("title") => LibrarySort::Title,
            Some("author") => LibrarySort::Author,
            Some("added") => LibrarySort::Added,
            Some("progress") => LibrarySort::Progress,
            Some("pageCount") => LibrarySort::PageCount,
            _ => LibrarySort::LastOpened,
        };
        let reading_state = match preferences
            .get("libraryReadingState")
            .and_then(serde_json::Value::as_str)
        {
            Some("reading") => LibraryReadingState::InProgress,
            Some("unread") => LibraryReadingState::NotStarted,
            Some("completed") => LibraryReadingState::Completed,
            _ => LibraryReadingState::All,
        };
        let sort_direction = match preferences
            .get("librarySortDirection")
            .and_then(serde_json::Value::as_str)
        {
            Some("ascending") => SortDirection::Ascending,
            Some("descending") => SortDirection::Descending,
            _ => SortDirection::Default,
        };
        let list = preferences
            .get("libraryView")
            .and_then(serde_json::Value::as_str)
            == Some("list");
        let inspector = preferences
            .get("libraryInspector")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let missing_only = preferences
            .get("libraryMissingOnly")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let modified_only = preferences
            .get("libraryModifiedOnly")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let mut app = Self {
            search: search::SearchState::new(
                preferences
                    .get("globalIndexEnabled")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
            ),
            open_intent: open_intent::OpenIntent::default(),
            opening: Default::default(),
            tabs: Default::default(),
            reading_queue: Default::default(),
            workspaces: Default::default(),
            open_confirmation: None,
            password: None,
            password_attempted: false,
            preview_password_action: None,
            preview_password_target: None,
            smart_view: preferences
                .get("librarySmartView")
                .and_then(serde_json::Value::as_str)
                .and_then(yamantaka_domain::LibrarySmartView::from_key),
            smart_menu_preview: false,
            library_tags: tags::TagState::new(&preferences),
            collections: collections::CollectionState::new(&preferences),
            runtime,
            preferences,
            sessions: Default::default(),
            store,
            dark,
            english,
            demo,
            reduced_motion,
            text_scale,
            surface: if settings {
                Surface::Settings
            } else {
                Surface::Library
            },
            documents: Vec::new(),
            count: 0,
            query: String::new(),
            favorites: false,
            reading_state,
            sort,
            sort_direction,
            missing_only,
            modified_only,
            list,
            offset: 0,
            selected: None,
            inspector,
            active: None,
            navigation: None,
            reader_preferences: None,
            reader_preferences_cancel: Cancellation::default(),
            reader_preferences_loading: false,
            page: 0,
            zoom: 1.0,
            reader_scale: 1.0,
            panel: false,
            reader_panel_tab: outline::ReaderPanelTab::Bookmarks,
            find: Default::default(),
            links: Default::default(),
            link_preview: Default::default(),
            find_preview_step: None,
            find_preview_closed: false,
            outline: Default::default(),
            thumbnails: Default::default(),
            library_covers: Default::default(),
            cover_picker: None,
            cover_picker_demo: Default::default(),
            thumbnail_preview_page: None,
            outline_preview_target: None,
            reader_destination: None,
            bookmarks: bookmarks::BookmarkState::default(),
            restoration: Default::default(),
            global_notes: Default::default(),
            history: Default::default(),
            goals: Default::default(),
            metadata: None,
            metadata_preview: None,
            annotations: Default::default(),
            annotation_export: Default::default(),
            file_comparison: Default::default(),
            file_check: Default::default(),
            cache_maintenance: Default::default(),
            legacy_import: Default::default(),
            backups: Default::default(),
            backup_restore: Default::default(),
            library_check: Default::default(),
            relocation: Default::default(),
            annotation_paint: Default::default(),
            annotation_preview: 0,
            annotation_multi_preview: false,
            text: Default::default(),
            selection: Default::default(),
            selection_preview: Default::default(),
            selection_keyboard_preview: false,
            reader_pages: HashMap::new(),
            reader_layout: None,
            reader_scroll_anchor: true,
            presenter,
            governor,
            generation: 0,
            requested: None,
            render_retry: false,
            cancel: Cancellation::default(),
            bulk_organization_preview: None,
            bulk_organization_verified: None,
            bulk_picker: None,
            library_selection: Default::default(),
            grouping,
            query_cancel: Cancellation::default(),
            library_loaded: false,
            error: None,
            loading: false,
            importing: Default::default(),
            preview_tools_menu: false,
            activation,
            _instance: instance,
            _watch: watch,
            capture,
            capture_requested: false,
            capture_ready_since: None,
            capture_render_events: Vec::new(),
            preview_reader: false,
            logo: None,
            preview_tilt: false,
            preview_list_focus: false,
            preview_reader_options: false,
            reader_options_section: 0,
        };
        app.check_tag_filters();
        app.check_collection_filter();
        app.refresh();
        app.send(Action::PendingImports);
        app.begin_reader_continuity();
        if app.search.enabled {
            app.send(Action::Index(
                yamantaka_application::search_index::IndexCommand::Enable(true),
            ));
        }
        app
    }
    fn tr<'a>(&self, es: &'a str, en: &'a str) -> &'a str {
        if self.english { en } else { es }
    }
    fn send(&mut self, action: Action) {
        if let Err(error) = self.runtime.submit(action) {
            self.error = Some(error.to_string());
        }
    }
    fn refresh(&mut self) {
        if let Err(error) = self.try_refresh() {
            self.error = Some(error.to_string());
        }
    }
    fn try_refresh(&mut self) -> yamantaka_domain::Result<()> {
        let import_revision = self.runtime.import_revision();
        let cancel = Cancellation::default();
        self.runtime.submit(Action::Query(
            LibraryQuery {
                text: self.query.clone(),
                smart_view: self.smart_view,
                tag_ids: self
                    .library_tags
                    .filters
                    .iter()
                    .map(|(id, _)| id.clone())
                    .collect(),
                favorites_only: self.favorites,
                reading_state: self.reading_state,
                sort: self.sort,
                sort_direction: self.sort_direction,
                missing_only: self.missing_only,
                modified_only: self.modified_only,
                collection_id: self.collections.filter.as_ref().map(|(id, _)| id.clone()),
                offset: self.offset,
                ..Default::default()
            },
            cancel.clone(),
        ))?;
        self.query_cancel.cancel();
        self.query_cancel = cancel;
        self.library_loaded = false;
        self.importing.mark_refreshed(import_revision);
        Ok(())
    }
    fn reset_render_request(&mut self) {
        self.requested = None;
        self.render_retry = false;
        self.reader_scroll_anchor = true;
        for page in self.reader_pages.values_mut() {
            page.cancel.cancel();
            page.requested = None;
        }
    }
    fn persist_library_state(&mut self) {
        if self.demo {
            return;
        }
        let state = [
            ("libraryGrouping", self.grouping.mode.key().into()),
            (
                "librarySort",
                serde_json::Value::from(match self.sort {
                    LibrarySort::LastOpened => "recent",
                    LibrarySort::Title => "title",
                    LibrarySort::Author => "author",
                    LibrarySort::Added => "added",
                    LibrarySort::Progress => "progress",
                    LibrarySort::PageCount => "pageCount",
                }),
            ),
            (
                "librarySortDirection",
                serde_json::Value::from(match self.sort_direction {
                    SortDirection::Default => "default",
                    SortDirection::Ascending => "ascending",
                    SortDirection::Descending => "descending",
                }),
            ),
            (
                "libraryReadingState",
                serde_json::Value::from(match self.reading_state {
                    LibraryReadingState::All => "all",
                    LibraryReadingState::InProgress => "reading",
                    LibraryReadingState::NotStarted => "unread",
                    LibraryReadingState::Completed => "completed",
                }),
            ),
            (
                "libraryView",
                serde_json::Value::from(if self.list { "list" } else { "covers" }),
            ),
            ("libraryInspector", self.inspector.into()),
            ("libraryMissingOnly", self.missing_only.into()),
            ("libraryModifiedOnly", self.modified_only.into()),
            (
                "librarySmartView",
                self.smart_view
                    .map(|view| serde_json::Value::from(view.key()))
                    .unwrap_or(serde_json::Value::Null),
            ),
            (
                "libraryCollection",
                self.collections
                    .filter
                    .as_ref()
                    .map(|(id, name)| serde_json::json!({"id":id,"name":name}))
                    .unwrap_or(serde_json::Value::Null),
            ),
        ];
        let mut changed = false;
        for (key, value) in state {
            if self.preferences.rust.get(key) != Some(&value) {
                self.preferences.rust.insert(key.to_owned(), value);
                changed = true;
            }
        }
        if changed && let Err(error) = self.preferences.save(&self.store.join("preferences.json")) {
            self.error = Some(error.to_string());
        }
    }
    pub fn import(&mut self, paths: Vec<PathBuf>) {
        self.tabs.continuity.overridden = true;
        self.open_intent.leave_destination();
        self.password = None;
        let cancel = self.importing.begin();
        if let Err(error) = self.runtime.submit(Action::Import(paths, cancel.clone())) {
            self.importing.submission_failed(&cancel);
            self.error = Some(error.to_string());
        }
    }
    pub fn import_folder(&mut self, root: PathBuf) {
        self.tabs.continuity.overridden = true;
        self.open_intent.leave_destination();
        self.password = None;
        let cancel = self.importing.begin();
        if let Err(error) = self
            .runtime
            .submit(Action::ImportFolder(root, cancel.clone()))
        {
            self.importing.submission_failed(&cancel);
            self.error = Some(error.to_string());
        }
    }
    pub fn preview_tools_menu(&mut self) {
        self.preview_tools_menu = self.demo && self.capture.is_some();
    }
    pub fn preview_reader(&mut self) {
        self.preview_reader = true;
    }
    pub fn text_scale(&self) -> f32 {
        self.text_scale
    }
    /// Capture-only synthetic pointer, using the same spring and rigid transform.
    pub fn preview_tilt(&mut self) {
        self.preview_tilt = self.demo && self.capture.is_some();
    }
    pub fn preview_layout(&mut self, list: bool, details: bool, panel: bool) {
        if self.demo && self.capture.is_some() {
            self.list = list;
            self.inspector = details;
            self.panel = panel;
            self.reader_panel_tab = outline::ReaderPanelTab::Text;
        }
    }
    pub fn preview_bookmarks(&mut self) {
        if self.demo && self.capture.is_some() {
            self.panel = true;
            self.reader_panel_tab = outline::ReaderPanelTab::Bookmarks;
        }
    }
    /// Exercise the real selected and keyboard-focus treatments on different rows.
    pub fn preview_list_states(&mut self) {
        if self.demo && self.capture.is_some() {
            self.list = true;
            self.selected = Some("demo-0".into());
            self.library_selection.model.all(&["demo-0".into()]);
            self.library_selection.model.mode = false;
            self.preview_list_focus = true;
        }
    }
    fn show_surface(&mut self, surface: Surface) {
        if matches!(surface, Surface::Library)
            && matches!(self.surface, Surface::Reader)
            && self.metadata_source_context()
            && !self.metadata_return_available()
        {
            self.metadata = None;
        }

        if self
            .cover_picker
            .as_ref()
            .is_some_and(|picker| picker.saving.is_some())
        {
            return;
        }
        self.cover_picker = None;
        self.runtime.wake();
        if !matches!(surface, Surface::Reader) {
            self.tabs.leave_destination();
            self.restoration.clear();
        }
        if matches!(self.surface, Surface::Tags) {
            self.library_tags.leave();
        }
        if matches!(self.surface, Surface::Collections) {
            self.collections.leave();
        }
        if matches!(self.surface, Surface::Notes) {
            self.global_notes.leave();
        }
        if matches!(self.surface, Surface::History) {
            self.history.leave();
        }
        self.links.clear();
        self.annotation_paint.invalidate();
        self.text.clear();
        self.selection.clear();
        self.find.stop();
        self.thumbnails.clear();
        self.open_intent.leave_destination();
        self.password = None;
        self.surface = surface;
    }
    fn open(&mut self, document: Document) -> bool {
        if self.library_selection.removing() {
            return false;
        }
        self.sync_tab_context();
        if let Some(id) = self
            .tabs
            .model
            .tabs()
            .iter()
            .find(|tab| tab.document() == document.id)
            .map(|tab| tab.id().to_owned())
        {
            return self.select_reader_tab(&id);
        }
        self.tabs.cancel();
        self.open_candidate(document)
    }
    fn open_candidate(&mut self, document: Document) -> bool {
        if self.library_selection.removing() {
            return false;
        }
        if self.tabs.closing.busy() {
            return false;
        }
        if self.annotations.editing() {
            self.error = Some(
                self.tr(
                    "Termina o cancela la nota antes de cambiar de lectura.",
                    "Finish or cancel the note before switching readings.",
                )
                .into(),
            );
            return false;
        }
        if !self.demo
            && self.preferences.record_reading_sessions()
            && !self.sessions.lifecycle.has_capacity()
        {
            self.error = Some(self.tr("Hay sesiones pendientes de guardar. Reintenta su guardado antes de abrir otro documento.", "Reading sessions are awaiting saving. Retry saving them before opening another document.").into());
            return false;
        }
        self.cache_reading_context();
        let next_cancel = Cancellation::default();
        let id = document.id.clone();
        if let Err(error) = self
            .runtime
            .submit(Action::Open(document, None, next_cancel.clone()))
        {
            self.error = Some(error.to_string());
            return false;
        }
        self.opening.bind(id, next_cancel);
        self.restoration.clear();
        self.password = None;
        self.password_attempted = false;
        self.open_intent.begin_open();
        self.loading = true;
        true
    }

    fn navigate(&mut self, page: u32) {
        if self
            .navigation
            .as_mut()
            .is_some_and(|nav| nav.go_to_page(page))
        {
            self.apply_navigation();
        }
    }
    fn apply_navigation(&mut self) {
        self.restoration.clear();
        if let (Some((doc, _)), Some(navigation)) = (&self.active, &self.navigation) {
            self.page = navigation.position().page();
            self.generation += 1;
            self.cancel.cancel();
            self.cancel = Cancellation::default();
            self.send(Action::Navigate(doc.id.clone(), self.page));
            self.reset_render_request();
        }
    }
    fn settings_changed(&mut self, ctx: &egui::Context) {
        theme::install(ctx, self.dark);
        ctx.set_zoom_factor(self.text_scale);
        self.preferences
            .rust
            .insert("darkMode".into(), self.dark.into());
        self.preferences
            .rust
            .insert("languageCode".into(), self.tr("es", "en").into());
        self.preferences
            .rust
            .insert("reduceMotion".into(), self.reduced_motion.into());
        self.preferences
            .rust
            .insert("textScale".into(), self.text_scale.into());
        if !self.demo
            && let Err(error) = self.preferences.save(&self.store.join("preferences.json"))
        {
            self.error = Some(error.to_string());
        }
    }
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let compact = ui.ctx().content_rect().height() < 650.0;
        if self.logo.is_none()
            && let Ok(icon) = image::load_from_memory(include_bytes!(
                "../../../assets/branding/yamantaka-logo-purple.png"
            ))
        {
            let icon = icon.to_rgba8();
            self.logo = Some(ui.ctx().load_texture(
                "Yamantaka brand",
                egui::ColorImage::from_rgba_unmultiplied(
                    [icon.width() as usize, icon.height() as usize],
                    icon.as_raw(),
                ),
                TextureOptions::LINEAR,
            ));
        }
        egui::Frame::new()
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(
                if compact { 18 } else { 24 },
                if compact { 4 } else { 8 },
            ))
            .show(ui, |ui| {
                let narrow = ui.available_width() < 540.0;
                ui.horizontal_wrapped(|ui| {
                    if let Some(logo) = &self.logo {
                        ui.add(
                            egui::Image::new(logo)
                                .fit_to_exact_size(egui::vec2(24.0, 24.0))
                                .corner_radius(12),
                        );
                    }
                    if !compact && !narrow {
                        ui.label(RichText::new("Yamantaka").size(16.0).strong());
                        ui.add_space(12.0);
                    }
                    let response = if narrow {
                        theme::icon_toggle(
                            ui,
                            self.tr("Biblioteca", "Library"),
                            theme::Icon::Book,
                            matches!(self.surface, Surface::Library),
                            p,
                        )
                    } else {
                        theme::control(
                            ui,
                            self.tr("Biblioteca", "Library"),
                            Some(theme::Icon::Book),
                            matches!(self.surface, Surface::Library),
                            false,
                            p,
                        )
                    };
                    if response.clicked() {
                        self.show_surface(Surface::Library);
                        self.refresh();
                    }
                    if self.active.is_some() {
                        let response = if narrow {
                            theme::icon_button(
                                ui,
                                self.tr("Lector", "Reader"),
                                theme::Icon::Right,
                                p,
                            )
                        } else {
                            theme::navigation(
                                ui,
                                self.tr("Lector", "Reader"),
                                matches!(self.surface, Surface::Reader),
                                p,
                            )
                        };
                        if response.clicked() {
                            self.show_surface(Surface::Reader);
                        }
                    }
                    if narrow {
                        if theme::primary_icon(
                            ui,
                            self.tr("Añadir PDF", "Add PDFs"),
                            theme::Icon::Add,
                            p,
                        )
                        .clicked()
                            && let Some(paths) = yamantaka_windows::pick_pdfs()
                        {
                            self.import(paths);
                        }
                        self.tools_menu(ui, true, p);
                        if matches!(self.surface, Surface::Library)
                            && let Some(document) = self
                                .documents
                                .iter()
                                .find(|doc| doc.reading_progress > 0.0)
                                .cloned()
                            && theme::icon_button(
                                ui,
                                self.tr("Retomar lectura", "Resume reading"),
                                theme::Icon::Right,
                                p,
                            )
                            .on_hover_text(document.display_title())
                            .clicked()
                        {
                            self.open(document);
                        }
                    } else {
                        let action_width = if compact { 330.0 } else { 245.0 };
                        ui.add_space(
                            (ui.available_size_before_wrap().x - action_width - 10.0).max(0.0),
                        );
                        ui.allocate_ui_with_layout(
                            egui::vec2(action_width, 40.0),
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                self.tools_menu(ui, false, p);
                                if theme::control(
                                    ui,
                                    self.tr("Añadir PDF", "Add PDFs"),
                                    Some(theme::Icon::Add),
                                    false,
                                    true,
                                    p,
                                )
                                .clicked()
                                    && let Some(paths) = yamantaka_windows::pick_pdfs()
                                {
                                    self.import(paths);
                                }
                                if compact
                                    && matches!(self.surface, Surface::Library)
                                    && let Some(document) = self
                                        .documents
                                        .iter()
                                        .find(|doc| doc.reading_progress > 0.0)
                                        .cloned()
                                    && theme::navigation(ui, self.tr("Retomar", "Resume"), false, p)
                                        .on_hover_text(document.display_title())
                                        .clicked()
                                {
                                    self.open(document);
                                }
                            },
                        );
                    }
                });
            });
    }
    fn library(&mut self, ui: &mut egui::Ui) {
        if ui.ctx().content_rect().height() < 420.0 {
            egui::ScrollArea::vertical()
                .id_salt("library-small-page")
                .auto_shrink([false, false])
                .show(ui, |ui| self.library_body(ui));
        } else {
            self.library_body(ui);
        }
    }
    fn library_body(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let compact = ui.ctx().content_rect().height() < 650.0;
        let margin = if ui.available_width() < 850.0 { 22 } else { 42 };
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(
                margin,
                if compact { 10 } else { 18 },
            ))
            .show(ui, |ui| {
                if compact {
                    ui.spacing_mut().item_spacing.y = 4.0;
                }
                let available = ui.available_width();
                let search_width = if available > 700.0 {
                    360.0
                } else {
                    (available - 200.0).max(200.0)
                };
                let title_width = if available > 500.0 {
                    (available - search_width - 18.0).max(150.0)
                } else {
                    available
                };
                ui.horizontal_wrapped(|ui| {
                    if available>=420.0 { ui.allocate_ui_with_layout(
                        egui::vec2(title_width, if available < 420.0 { 26.0 } else if compact { 52.0 } else { 64.0 }),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(title_width);
                            ui.label(RichText::new(self.tr("Tu biblioteca", "Your library")).font(
                                egui::FontId::new(if compact { 26.0 } else { 38.0 },
                                    if compact { egui::FontFamily::Proportional } else { egui::FontFamily::Name("Editorial".into()) }),
                            ));
                            if available>=420.0 {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(format!(
                                        "{} {}",
                                        self.count,
                                        if self.count==1 {self.tr("documento en tu colección","document in your collection")} else {self.tr(
                                            "documentos en tu colección",
                                            "documents in your collection"
                                        )}
                                    ))
                                    .color(p.secondary)
                                    .size(13.0),
                                )
                                .wrap(),
                            );
                            }
                        },
                    );
                    }
                    let search_width = if available <= 500.0 {
                        available
                    } else {
                        search_width
                    };
                    ui.allocate_ui_with_layout(
                        egui::vec2(search_width, 40.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(search_width);
                            let hint = self
                                .tr("Buscar por título o autor", "Search title or author")
                                .to_owned();
                            let response = ui
                                .scope(|ui| {
                            ui.visuals_mut().widgets.inactive.bg_stroke =
                                egui::Stroke::new(1.0, p.line.lerp_to_gamma(p.secondary, if self.dark {0.5}else{0.65}));
                                    ui.add_sized(
                                        [search_width, 40.0],
                                        egui::TextEdit::singleline(&mut self.query)
                                            .hint_text(hint)
                                            .background_color(p.surface)
                                            .margin(egui::vec2(36.0, 10.0)),
                                    )
                                })
                                .inner;
                            theme::icon(
                                ui.painter(),
                                egui::pos2(response.rect.left() + 18.0, response.rect.center().y),
                                theme::Icon::Search,
                                p.secondary,
                            );
                            if response.changed() {
                                self.offset = 0;
                                self.refresh();
                            }
                        },
                    );
                });
                ui.add_space(if compact { 4.0 } else { 12.0 });
                ui.horizontal_wrapped(|ui| {
                    if ui.ctx().content_rect().width()<420.0 {
                        let response=theme::icon_toggle(ui,self.tr("Filtros y orden","Filters and sorting"),theme::Icon::Sort,self.library_has_filters(),p);
                        let mut popup=egui::Popup::menu(&response);
                        if self.smart_menu_preview {popup=popup.open(true);}
                        popup.show(|ui| {
                            let height=(ui.ctx().content_rect().bottom()-ui.cursor().top()-12.0).max(40.0);
                            ui.set_max_height(height);
                            egui::ScrollArea::vertical().id_salt("compact-library-filters").max_height(height).show(ui,|ui|self.library_filters(ui,p));
                        });
                    }else{self.library_filters(ui,p);}
                    if !compact {
                        self.reading_shortcut(ui, p);
                    }
                    self.library_selection_control(ui, p);
                    let labelled = ui.available_width() > 900.0;
                    let view_width = if labelled { 300.0 } else { 140.0 };
                    ui.add_space((ui.available_size_before_wrap().x - view_width - 10.0).max(0.0));
                    ui.allocate_ui_with_layout(
                        egui::vec2(view_width, 40.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let response = if labelled {
                                theme::control(
                                    ui,
                                    self.tr("Lista", "List"),
                                    Some(theme::Icon::List),
                                    self.list,
                                    false,
                                    p,
                                )
                            } else {
                                theme::icon_toggle(
                                    ui,
                                    self.tr("Lista", "List"),
                                    theme::Icon::List,
                                    self.list,
                                    p,
                                )
                            };
                            if response.clicked() {
                                self.list = true;
                            }
                            let response = if labelled {
                                theme::control(
                                    ui,
                                    self.tr("Portadas", "Covers"),
                                    Some(theme::Icon::Grid),
                                    !self.list,
                                    false,
                                    p,
                                )
                            } else {
                                theme::icon_toggle(
                                    ui,
                                    self.tr("Portadas", "Covers"),
                                    theme::Icon::Grid,
                                    !self.list,
                                    p,
                                )
                            };
                            if response.clicked() {
                                self.list = false;
                            }
                            let response = if labelled {
                                theme::control(
                                    ui,
                                    self.tr("Detalles", "Details"),
                                    Some(theme::Icon::Panel),
                                    self.inspector,
                                    false,
                                    p,
                                )
                            } else {
                                theme::icon_toggle(
                                    ui,
                                    self.tr("Detalles", "Details"),
                                    theme::Icon::Panel,
                                    self.inspector,
                                    p,
                                )
                            };
                            if response.clicked() {
                                self.inspector = !self.inspector;
                            }
                        },
                    );
                });
                ui.add_space(if compact { 4.0 } else { 12.0 });
                if self.documents.is_empty() {
                    let filtered=self.library_has_filters();
                    ui.add_space(if compact{18.0}else{36.0});
                    theme::display(ui,if filtered{self.tr("No hay libros que coincidan","No matching books")}else{self.tr("Un lugar para tus próximas lecturas","A place for your next reads")},28.0);
                    ui.label(RichText::new(if filtered{self.tr("Prueba otro título o restablece los filtros para ver tu colección.","Try another title or reset the filters to see your collection.")}else{self.tr("Añade tus PDF o arrástralos a esta ventana para empezar.","Add your PDFs or drag them into this window to begin.")}).color(p.secondary));
                    ui.add_space(12.0);
                    if filtered {
                        if theme::control(ui,self.tr("Restablecer filtros","Reset filters"),None,false,true,p).clicked(){self.reset_library_filters();}
                    } else if theme::control(ui,self.tr("Elegir documentos","Choose documents"),Some(theme::Icon::Add),false,true,p).clicked() && let Some(paths)=yamantaka_windows::pick_pdfs(){self.import(paths);}
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("catalogue")
                        .show(ui, |ui| {
                            self.grouped_library(ui,compact,p);
                            ui.horizontal(|ui| {
                                if ui
                                    .add_enabled(
                                        self.offset > 0,
                                        egui::Button::new(self.tr("Anterior", "Previous")),
                                    )
                                    .clicked()
                                {
                                    self.offset = self.offset.saturating_sub(80);
                                    self.refresh();
                                }
                                ui.label(format!(
                                    "{}–{} / {}",
                                    self.offset + 1,
                                    (self.offset + self.documents.len()).min(self.count as usize),
                                    self.count
                                ));
                                if ui
                                    .add_enabled(
                                        self.offset + self.documents.len() < self.count as usize,
                                        egui::Button::new(self.tr("Siguiente", "Next")),
                                    )
                                    .clicked()
                                {
                                    self.offset += 80;
                                    self.refresh();
                                }
                            });
                        });
                }
            });
        self.persist_library_state();
    }
    fn library_filters(&mut self, ui: &mut egui::Ui, p: Palette) {
        self.tag_filter_control(ui, p);
        self.collection_filter(ui, p);
        if theme::navigation(
            ui,
            self.tr("Todos los libros", "All books"),
            !self.favorites
                && self.collections.filter.is_none()
                && self.smart_view.is_none()
                && self.library_tags.filters.is_empty(),
            p,
        )
        .clicked()
        {
            self.favorites = false;
            self.smart_view = None;
            self.clear_tag_filters();
            self.clear_collection_filter();
            self.offset = 0;
            self.refresh();
        }
        if theme::navigation(ui, self.tr("Favoritos", "Favorites"), self.favorites, p).clicked() {
            self.favorites = true;
            self.offset = 0;
            self.refresh();
        }
        self.library_state_filter(ui, p);
        let sort_name = self.tr(
            match self.sort {
                LibrarySort::LastOpened => "Recientes",
                LibrarySort::Title => "Título",
                LibrarySort::Author => "Autor",
                LibrarySort::Added => "Añadidos",
                LibrarySort::Progress => "Progreso",
                LibrarySort::PageCount => "Páginas",
            },
            match self.sort {
                LibrarySort::LastOpened => "Recent",
                LibrarySort::Title => "Title",
                LibrarySort::Author => "Author",
                LibrarySort::Added => "Added",
                LibrarySort::Progress => "Progress",
                LibrarySort::PageCount => "Pages",
            },
        );
        let sort_response = theme::control(ui, sort_name, Some(theme::Icon::Sort), false, false, p)
            .on_hover_text(self.tr("Ordenar biblioteca", "Sort library"));
        egui::Popup::menu(&sort_response).show(|ui| {
            let height = (ui.ctx().content_rect().height() - 40.0).max(40.0);
            egui::ScrollArea::vertical()
                .id_salt("library-sort-group-options")
                .max_height(height)
                .show(ui, |ui| {
                    for (sort, es, en) in [
                        (
                            LibrarySort::LastOpened,
                            "Abiertos recientemente",
                            "Recently opened",
                        ),
                        (LibrarySort::Title, "Título", "Title"),
                        (LibrarySort::Author, "Autor", "Author"),
                        (LibrarySort::Added, "Fecha de incorporación", "Date added"),
                        (
                            LibrarySort::Progress,
                            "Progreso de lectura",
                            "Reading progress",
                        ),
                        (LibrarySort::PageCount, "Número de páginas", "Page count"),
                    ] {
                        if theme::navigation(ui, self.tr(es, en), self.sort == sort, p).clicked() {
                            self.sort = sort;
                            self.sort_direction = SortDirection::Default;
                            self.offset = 0;
                            self.refresh();
                            ui.close();
                        }
                    }
                    ui.separator();
                    for (direction, es, en) in [
                        (SortDirection::Ascending, "Ascendente", "Ascending"),
                        (SortDirection::Descending, "Descendente", "Descending"),
                    ] {
                        if theme::navigation(
                            ui,
                            self.tr(es, en),
                            self.sort_direction == direction,
                            p,
                        )
                        .clicked()
                        {
                            self.sort_direction = direction;
                            self.offset = 0;
                            self.refresh();
                            ui.close();
                        }
                    }
                    ui.separator();
                    self.library_grouping_options(ui, p);
                });
        });
        if self.library_has_filters()
            && theme::navigation(
                ui,
                self.tr("Restablecer filtros", "Reset filters"),
                false,
                p,
            )
            .clicked()
        {
            self.reset_library_filters();
        }
    }
    fn reader(&mut self, ui: &mut egui::Ui) {
        let p = Palette::new(self.dark);
        let Some((document, info)) = self.active.clone() else {
            ui.label(self.tr(
                "Elige un documento en la biblioteca.",
                "Choose a document from your library.",
            ));
            return;
        };
        self.sync_tab_context();
        self.compact_reader_toolbar(ui, info.pages.len().max(1) as u32, p);
        ui.add_space(4.0);
        self.selection_toolbar(ui, p);
        self.reader_panel(ui, &document, &info, p);
        if self.panel && ui.ctx().content_rect().width() < 620.0 {
            return;
        }
        self.reader_viewport(ui, &document, &info, p);
        self.cache_reading_context();
    }
}
impl App for YamantakaApp {
    fn raw_input_hook(&mut self, _: &egui::Context, input: &mut egui::RawInput) {
        if self.demo && self.capture.is_some() {
            self.selection_preview
                .input(input, self.selection_keyboard_preview);
        }
        if self.demo
            && self.capture.is_some()
            && self.link_preview == links::LinkPreview::KeyPending
        {
            self.link_preview = links::LinkPreview::Activate;
            input.events.push(egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
    }

    fn logic(&mut self, ctx: &egui::Context, _: &mut Frame) {
        self.update_restore(ctx);
        if self.backup_restore.busy() && ctx.input(|input| input.viewport().close_requested()) {
            self.backup_restore.close_after = true;
            self.backup_restore.cancel();
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        self.update_backup(ctx);
        if self.backups.busy() && ctx.input(|input| input.viewport().close_requested()) {
            self.backups.close_after = true;
            self.backups.cancel();
            self.backup_restore.cancel();
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        self.update_legacy_import(ctx);
        if self.legacy_import.busy() && ctx.input(|input| input.viewport().close_requested()) {
            self.legacy_import.close_after = true;
            self.legacy_import.cancel();
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.request_repaint_after(Duration::from_millis(50));
        }

        if self
            .cover_picker
            .as_ref()
            .is_some_and(|picker| picker.saving.is_some())
            && ctx.input(|input| input.viewport().close_requested())
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        if self.library_tags.writing() && ctx.input(|input| input.viewport().close_requested()) {
            self.library_tags.close_after_write = true;
            self.library_tags.cancel();
            self.runtime.wake();
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        if self.library_selection.removing()
            && ctx.input(|input| input.viewport().close_requested())
        {
            self.library_selection.close_after_removal = true;
            self.library_selection.cancel();
            self.runtime.wake();
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        self.observe_reading_session(ctx);
        if let Some(presenter) = &mut self.presenter
            && let Err(error) = presenter.begin_frame()
        {
            self.error = Some(error.to_string());
        }
        if let Some((document, _)) = &self.active {
            self.annotations.bind(&document.id);
        }
        while !self.legacy_import.open
            && !self.backup_restore.open
            && !self.reading_queue.open
            && !self.workspaces.open
            && let Ok(paths) = self.activation.try_recv()
        {
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            if !paths.is_empty() {
                self.import(paths);
            }
        }
        while let Ok(update) = self.runtime.updates.try_recv() {
            match update {
                Update::PendingImports(result) => self.importing.accept_recovery(result),
                Update::SelectionOrganized(reply) => self.accept_library_bulk(ctx, *reply),
                Update::LibraryChecked(reply) => {
                    self.library_check.accept(*reply);
                    self.refresh();
                }
                Update::FileRelocated(reply) => {
                    let applied = matches!(
                        reply.result,
                        Ok(yamantaka_application::relocation::RelocationOutcome::Applied)
                    );
                    self.relocation.accept(*reply);
                    if applied {
                        self.refresh();
                    }
                }
                Update::Tags(result, cancel) => {
                    if self.bulk_target_matches(&cancel) {
                        self.accept_bulk_tags(result);
                    } else {
                        self.accept_tags(result, cancel);
                    }
                }
                Update::TagChanged(result, cancel) => self.accept_tag_change(result, cancel),
                Update::Collections(result, cancel) => {
                    if self.bulk_target_matches(&cancel) {
                        self.accept_bulk_collections(result);
                    } else {
                        self.accept_collections(result, cancel);
                    }
                }
                Update::CollectionChanged(result, cancel) => {
                    self.accept_collection_change(result, cancel)
                }
                Update::CoverCacheMaintained(reply) => self.cache_maintenance.accept(*reply),
                Update::FileChecked(reply) => {
                    self.file_check.accept(*reply);
                    self.refresh();
                }
                Update::FileComparison(reply) => self.file_comparison.accept(*reply),
                Update::AnnotationExport(reply) => self.annotation_export.accept(reply),
                Update::AnnotationMarks(reply) => self.annotation_paint.accept(reply),
                Update::Annotations(reply) => {
                    if self.global_notes.matches(&reply) {
                        self.accept_global_note_reply(reply);
                        continue;
                    }
                    let revision = self.annotations.revision;
                    if let Some(error) = self.annotations.accept(reply) {
                        self.error = Some(format!(
                            "{}: {error}",
                            self.tr(
                                "No se pudo guardar la nota del documento anterior",
                                "Could not save the note in the previous document"
                            )
                        ));
                    }
                    if self.annotations.revision != revision {
                        self.annotation_paint.invalidate();
                    }
                }
                Update::Library(documents, count, memberships, cancel) => {
                    if cancel.is_cancelled() || !self.query_cancel.same_request(&cancel) {
                        continue;
                    }
                    if documents.is_empty()
                        && self.offset > 0
                        && (count == 0 || self.offset as u64 >= count)
                    {
                        self.offset =
                            usize::try_from(count.saturating_sub(1) / 80 * 80).unwrap_or(0);
                        self.refresh();
                        continue;
                    }
                    self.library_loaded = true;
                    self.documents = documents;
                    self.grouping.memberships = memberships;
                    self.verify_grouping_preview();
                    self.library_selection
                        .model
                        .retain(&self.library_page_ids());
                    self.count = count;
                    if self.inspector && self.selected.is_none() {
                        self.selected = self.documents.first().map(|doc| doc.id.clone());
                    }
                    if self.preview_reader {
                        self.preview_reader = false;
                        if let Some(document) = self.documents.first().cloned() {
                            self.open(document);
                        }
                    }
                }
                Update::Outline(id, outline, cancel) => {
                    self.outline.accept(&id, &cancel, Ok(outline));
                    self.apply_outline_preview();
                }
                Update::OutlineFailed(id, error, cancel) => {
                    self.outline.accept(&id, &cancel, Err(error.to_string()))
                }
                Update::Bookmarks(id, _, page, cancel) => {
                    self.bookmarks.accept(&id, &cancel, Ok(page));
                }
                Update::BookmarkFailed(id, error, cancel) => {
                    if !cancel.is_cancelled()
                        && self
                            .active
                            .as_ref()
                            .is_some_and(|(document, _)| document.id != id)
                    {
                        // Leaving a document must not hide an accepted write failure.
                        self.error = Some(format!(
                            "{}: {error}",
                            self.tr(
                                "No se pudieron guardar los marcadores del documento anterior",
                                "Could not save bookmarks in the previous document",
                            )
                        ));
                    }
                    self.bookmarks.accept(&id, &cancel, Err(error.to_string()));
                }
                Update::Opened(document, info, cancel) => {
                    let editing =
                        self.annotations.editing() && self.opening.matches(&document.id, &cancel);
                    if editing {
                        self.error = Some(
                            self.tr(
                                "Termina o cancela la nota antes de cambiar de lectura.",
                                "Finish or cancel the note before switching readings.",
                            )
                            .into(),
                        );
                    }
                    if cancel.is_cancelled()
                        || !self.opening.matches(&document.id, &cancel)
                        || editing
                    {
                        if self.opening.fail(&document.id, &cancel, false) {
                            self.loading = false;
                            self.tabs.fail(&document.id, &cancel);
                            self.restoration.clear();
                        }
                        cancel.cancel();
                        self.open_confirmation = Some((document.id.clone(), cancel));
                        continue;
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
                            self.restoration.clear();
                            cancel.cancel();
                            self.open_confirmation = Some((document.id.clone(), cancel));
                            self.loading = false;
                            continue;
                        }
                    };
                    if let Err(error) = self.finish_reading_session() {
                        self.error = Some(error.to_string());
                        self.opening.fail(&document.id, &cancel, false);
                        self.tabs.fail(&document.id, &cancel);
                        self.restoration.clear();
                        cancel.cancel();
                        self.open_confirmation = Some((document.id.clone(), cancel));
                        self.loading = false;
                        continue;
                    }
                    let Some(preferences_cancel) = self.opening.accept(&document.id, &cancel)
                    else {
                        cancel.cancel();
                        self.open_confirmation = Some((document.id.clone(), cancel));
                        continue;
                    };
                    self.sync_tab_context();
                    self.tabs
                        .accept_open(&document.id, &cancel, preferences_cancel.clone());
                    self.forget_failed_retirement(&document.id);
                    self.replace_reader_state(preferences_cancel);
                    self.page = page;
                    self.navigation = Some(navigation);
                    self.open_confirmation = Some((document.id.clone(), cancel));
                    if target_page.is_some() {
                        self.send(Action::Navigate(document.id.clone(), self.page));
                    }
                    self.active = Some((*document, Arc::new(info)));
                    let id = self.active.as_ref().unwrap().0.id.clone();
                    self.send(Action::LoadReaderPreferences(
                        id,
                        self.reader_preferences_cancel.clone(),
                    ));
                    if reveal_reader {
                        self.show_surface(Surface::Reader);
                    }
                    self.loading = false;
                }
                Update::DocumentMatches(id, batch, cancel) => {
                    self.accept_find(&id, &cancel, Ok(batch))
                }
                Update::DocumentFindFailed(id, error, cancel) => {
                    self.accept_find(&id, &cancel, Err(error.to_string()))
                }
                Update::PageMatches(..) | Update::PageFindFailed(..) => {}
                Update::PageLinks(id, payload, cancel) => {
                    self.links.accept(&id, payload.page(), &cancel, Ok(payload));
                }
                Update::PageLinksFailed(id, page, error, cancel) => {
                    self.links
                        .accept(&id, page, &cancel, Err(error.to_string()));
                }
                Update::Cover(id, result, cancel) => {
                    if self.accept_metadata_reference(ctx, &id, &result, &cancel) {
                        continue;
                    }
                    if !self.accept_cover_picker_image(ctx, &id, &result, &cancel) {
                        self.accept_library_cover(ctx, &id, result, &cancel);
                    }
                }
                Update::CoverSourcePage(id, result, cancel) => {
                    self.accept_cover_source_page(&id, result, &cancel)
                }
                Update::MetadataSource(reply) => self.accept_metadata_source(reply),
                Update::ManualCover(reply) => self.accept_manual_cover(ctx, reply),
                Update::Page(page, cancel, _, _) => self.accept_rendered_page(ctx, &page, &cancel),
                Update::TextGeometry(id, payload, cancel) => {
                    self.selection
                        .accept(&id, payload.page(), &cancel, Ok(payload.clone()));
                    self.text.accept(&id, payload.page(), &cancel, Ok(payload));
                }
                Update::TextGeometryFailed(id, page, error, cancel) => {
                    self.selection
                        .accept(&id, page, &cancel, Err(error.to_string()));
                    self.text.accept(&id, page, &cancel, Err(error.to_string()));
                }
                Update::Text(..) => {}
                Update::Imported(_, cancel) => {
                    if !self.importing.finish(&cancel) {
                        continue;
                    }
                    self.send(Action::PendingImports);
                    self.refresh();
                }
                Update::OpenFailed(document, error, cancel) => {
                    let cancelled = cancel.is_cancelled();
                    let password = matches!(error, yamantaka_domain::Failure::PasswordRequired)
                        && self.open_intent.wants_reader();
                    if !self.opening.fail(&document.id, &cancel, password) {
                        continue;
                    }
                    self.loading = false;
                    if !password || cancelled {
                        self.tabs.fail(&document.id, &cancel);
                        self.restoration.clear();
                    }
                    if cancelled {
                        continue;
                    }
                    if matches!(error, yamantaka_domain::Failure::PasswordRequired)
                        && self.open_intent.wants_reader()
                    {
                        if let Some(action @ ("wrong" | "unlock")) =
                            self.preview_password_action.take()
                        {
                            let value = if action == "unlock" {
                                "fixture-password"
                            } else {
                                "synthetic incorrect credential"
                            };
                            self.retry_password(*document, value.into());
                            continue;
                        }
                        self.password = Some(password::PasswordChallenge::new(
                            *document,
                            self.password_attempted,
                        ));
                    } else if !matches!(error, yamantaka_domain::Failure::Cancelled) {
                        self.error = Some(error.to_string());
                    }
                }
                Update::Failed(error, cancel) => {
                    if cancel
                        .as_ref()
                        .is_some_and(|token| self.importing.finish(token))
                    {
                        if !matches!(error, yamantaka_domain::Failure::Cancelled) {
                            self.error = Some(error.to_string());
                        }
                        self.send(Action::PendingImports);
                        self.refresh();
                        continue;
                    }
                    if let Some(token) = &cancel {
                        self.tabs.fail_preferences(token);
                        if self.tabs.fail_lookup(token) {
                            self.error = Some(error.to_string());
                            continue;
                        }
                    }
                    if cancel
                        .as_ref()
                        .is_some_and(|token| self.thumbnails.fail(token, error.to_string()))
                    {
                        continue;
                    }
                    if cancel.is_some_and(|token| token.is_cancelled()) {
                        continue;
                    }
                    self.error = Some(error.to_string());
                    if self.reader_preferences.is_none() {
                        self.reader_preferences_loading = false;
                    }
                    self.render_retry = self.requested.is_some();
                    self.loading = false;
                }
                Update::ReaderClosed(reply) => {
                    if self.accept_tab_close(&reply) {
                        continue;
                    }
                    if let Err(error) = reply.result
                        && !matches!(error, yamantaka_domain::Failure::Cancelled)
                    {
                        self.error = Some(error.to_string());
                    }
                }
                Update::Committed(_, yamantaka_application::runtime::WriteKind::Index) => {
                    self.send(Action::IndexStatistics);
                    self.refresh_selected_index_state();
                }
                Update::Committed(_, _) => {}
                Update::Search(results, cancel) => {
                    if !cancel.is_cancelled() {
                        self.search.has_more = results.len() == 80;
                        self.search.results = results;
                        self.search.searching = false;
                        self.search.completed = true;
                        if self.search.preview_results
                            && self.search.selected.is_none()
                            && let Some(result) = if self.search.preview_action == Some("open") {
                                self.search
                                    .results
                                    .iter()
                                    .find(|result| result.page == Some(1))
                                    .cloned()
                            } else {
                                self.search.results.first().cloned()
                            }
                        {
                            self.select_search_result(result.clone());
                            match self.search.preview_action.take() {
                                Some("open") => self.activate_search_result(&result),
                                Some("exclude") => self.exclude_search_result(&result, true),
                                _ => {}
                            }
                        }
                    }
                }
                Update::SearchFailed(error, cancel) => {
                    if !cancel.is_cancelled() {
                        self.search.searching = false;
                        self.search.error = Some(error.to_string());
                    }
                }
                Update::IndexStatistics(stats) => {
                    self.search.stats = Some(stats);
                }
                Update::IndexState(id, state, cancel) => {
                    if !cancel.is_cancelled()
                        && self
                            .search
                            .selected
                            .as_ref()
                            .is_some_and(|result| result.document_id == id)
                    {
                        self.search.excluded = Some(state.is_some_and(|state| {
                            state.status == yamantaka_domain::search::IndexStatus::Excluded
                        }));
                    }
                }
                Update::Document(id, document, cancel) => {
                    if self.accept_tab_lookup(&id, document.clone(), &cancel) {
                        continue;
                    }
                    if let Some(target) = self.open_intent.accept_lookup(&id, &cancel) {
                        let page = target.page;
                        if let Some(document) =
                            document.filter(|document| document.id == id && !document.missing)
                        {
                            if self
                                .active
                                .as_ref()
                                .is_some_and(|(active, _)| active.id == id)
                            {
                                if let Some(page) = page {
                                    self.navigate(page);
                                }
                                self.show_surface(Surface::Reader);
                                if let Some(context) = target.context {
                                    self.restoration.arm(
                                        id,
                                        self.reader_preferences_cancel.clone(),
                                        context,
                                    );
                                    if let Some(preferences) = self.reader_preferences.clone() {
                                        self.load_reader_preferences(preferences);
                                    }
                                }
                            } else {
                                self.tabs.cancel();
                                if self.open_candidate(document) {
                                    if let Some(page) = page {
                                        self.open_intent.set_page(id.clone(), page);
                                    }
                                    if let Some(context) = target.context {
                                        self.restoration.arm(
                                            id,
                                            self.opening.preferences().unwrap(),
                                            context,
                                        );
                                    }
                                }
                            }
                        } else {
                            self.error = Some(self.tr("El documento ya no está disponible. Localízalo desde la biblioteca.", "The document is unavailable. Locate it from the library.").into());
                        }
                    }
                }
                Update::ReaderPreferences(id, preferences, cancel) => {
                    if cancel.is_cancelled()
                        || !cancel.same_request(&self.reader_preferences_cancel)
                        || !self.active.as_ref().is_some_and(|(doc, _)| doc.id == id)
                    {
                        continue;
                    }
                    self.load_reader_preferences(
                        preferences.unwrap_or_else(|| ReaderPreferences::new(id)),
                    );
                }
                Update::ReadingSession(reply) => {
                    let id = reply.id.clone();
                    match self.sessions.lifecycle.accept(reply) {
                        Some(Err(error)) => {
                            self.sessions.remember_failure(id);
                            self.error = Some(error.to_string());
                        }
                        Some(Ok(())) => self.send(Action::TrimReadingHistory(
                            self.preferences.reading_history_limit(),
                            Cancellation::default(),
                        )),
                        None => {}
                    }
                }
                Update::ReadingHistoryTrimmed(result, cancel) => {
                    if !cancel.is_cancelled()
                        && let Err(error) = result
                    {
                        self.error = Some(error.to_string());
                    }
                }
                Update::Metadata(reply) => self.accept_metadata(*reply),
                Update::ReadingGoal(reply) => self.accept_goal(reply),
                Update::ReadingSessionDeleted(reply) => self.accept_history_delete(reply),
                Update::ReadingHistory(reply) => {
                    self.history.accept(reply);
                }
                Update::ReaderContinuity(reply) => self.accept_reader_continuity(reply),
                Update::Workspace(reply) => self.accept_workspace(reply),
            }
        }
        self.advance_cover_picker_preview();
        self.advance_metadata_preview();
        self.advance_replacement_preview();
        self.advance_tab_preview();
        self.advance_queue_preview();
        self.advance_workspace_preview();
        self.advance_bulk_tab_preview();
        self.retire_closed_reader();
        self.poll_import_progress(ctx);
        self.sync_reading_session(ctx);
        self.pump_reader_continuity(ctx);
        if self.demo
            && self.capture.is_some()
            && self.selection_preview == selection::SelectionPreview::Waiting
            && matches!(self.surface, Surface::Reader)
            && self.active.is_some()
            && !self.selection.enabled
        {
            self.toggle_text_selection();
        }
        // Coalesce acknowledgement to the latest delivered open, then admit it
        // after draining results. Retry bounded control backpressure without
        // dropping the ownership handoff or continuously repainting while idle.
        if let Some((id, token)) = self.open_confirmation.take() {
            match self
                .runtime
                .submit(Action::ConfirmOpen(id.clone(), token.clone()))
            {
                Ok(()) => {}
                Err(yamantaka_domain::Failure::QueueFull) => {
                    self.open_confirmation = Some((id, token));
                    ctx.request_repaint_after(Duration::from_millis(100));
                }
                Err(error) => self.error = Some(error.to_string()),
            }
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Frame) {
        let ctx = ui.ctx().clone();
        ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::new("capture-control-bounds"),
                self.demo && self.capture.is_some(),
            );
            data.insert_temp(
                egui::Id::new("capture-controls"),
                Vec::<theme::ControlBounds>::new(),
            );
            data.insert_temp(
                egui::Id::new("yamantaka-reduced-motion"),
                self.reduced_motion,
            );
            data.insert_temp(egui::Id::new("yamantaka-english"), self.english);
        });
        let p = Palette::new(self.dark);
        self.poll_search(&ctx);
        self.selection.keyboard_input(
            &ctx,
            self.password.is_some()
                || self.links.confirmation_open()
                || self.bulk_picker.is_some()
                || self.selection_review_open()
                || self.tag_dialog_open()
                || self.legacy_import.open
                || self.backup_restore.open
                || self.reading_queue.open
                || self.workspaces.open,
        );
        self.library_selection_shortcuts(&ctx);
        for shortcut in shortcuts::read(
            &ctx,
            self.password.is_some()
                || self.links.confirmation_open()
                || self.bulk_picker.is_some()
                || self.selection_review_open()
                || self.tag_dialog_open()
                || self.legacy_import.open
                || self.backup_restore.open
                || self.reading_queue.open
                || self.workspaces.open,
            matches!(self.surface, Surface::Reader),
        ) {
            match shortcut {
                shortcuts::Shortcut::GlobalSearch => self.show_search(),
                shortcuts::Shortcut::DocumentFind => self.show_find(),
                shortcuts::Shortcut::PreviousMatch => self.step_find(false),
                shortcuts::Shortcut::NextMatch => self.step_find(true),
                shortcuts::Shortcut::PreviousPage => self.step_reader(false),
                shortcuts::Shortcut::NextPage => self.step_reader(true),
                shortcuts::Shortcut::NextTab => self.relative_reader_tab(false),
                shortcuts::Shortcut::PreviousTab => self.relative_reader_tab(true),
                shortcuts::Shortcut::CloseTab => {
                    if let Some(id) = self.tabs.model.active().map(|t| t.id().to_owned()) {
                        self.close_reader_tab(&id);
                    }
                }
                shortcuts::Shortcut::ReopenTab => self.reopen_reader_tab(),
                shortcuts::Shortcut::Fullscreen => {
                    let full = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!full));
                }
            }
        }
        self.apply_annotation_preview();
        self.apply_annotation_filter_preview();
        self.apply_annotation_bulk_preview();
        self.apply_annotation_tag_preview();
        self.apply_global_note_preview();
        self.apply_annotation_export_preview();
        self.update_file_comparison();
        self.update_library_selection_preview();
        self.update_removal_preview();
        self.update_cover_batch_preview(&ctx);
        self.update_bulk_organization_preview();
        self.update_tag_preview();
        self.update_collection_preview();
        self.update_cache_maintenance();
        self.update_file_check();
        self.update_relocation();
        self.update_library_check();
        self.apply_import_recovery_preview();
        let dropped = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_owned())
                .collect::<Vec<_>>()
        });
        if !dropped.is_empty() {
            self.import(dropped);
        }
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            egui::Frame::new().fill(p.canvas).show(ui, |ui| {
                if !matches!(self.surface, Surface::Reader) {
                    self.toolbar(ui);
                }
                if let Some(error) = self.error.clone() {
                    egui::Frame::new()
                        .fill(p.raised)
                        .inner_margin(egui::Margin::symmetric(24, 12))
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(error).color(p.ink));
                                if !self.sessions.failed.is_empty()
                                    && theme::primary_icon(
                                        ui,
                                        self.tr("Reintentar sesiones", "Retry saving sessions"),
                                        theme::Icon::Right,
                                        p,
                                    )
                                    .clicked()
                                {
                                    self.sessions.retry_failed();
                                    self.error = None;
                                }
                                if self.can_retry_reader_close()
                                    && theme::primary_icon(
                                        ui,
                                        self.tr("Reintentar cierre", "Retry close"),
                                        theme::Icon::Right,
                                        p,
                                    )
                                    .clicked()
                                {
                                    self.error = None;
                                    self.retry_reader_close();
                                }
                                if self.render_retry
                                    && theme::primary_icon(
                                        ui,
                                        self.tr("Reintentar página", "Retry page"),
                                        theme::Icon::Right,
                                        p,
                                    )
                                    .clicked()
                                {
                                    self.render_retry = false;
                                    self.reset_render_request();
                                    self.error = None;
                                }
                                if theme::navigation(
                                    ui,
                                    self.tr("Cerrar aviso", "Dismiss"),
                                    false,
                                    p,
                                )
                                .clicked()
                                {
                                    self.error = None;
                                }
                            });
                        });
                }
                if self.demo {
                    egui::Panel::bottom("synthetic").show(ui, |ui| {
                        ui.label(
                            RichText::new("Synthetic study collection · visual verification")
                                .size(11.0)
                                .color(p.secondary),
                        );
                    });
                }
                if matches!(self.surface, Surface::Library)
                    && self.inspector
                    && self.cover_picker.is_none()
                    && self.metadata.is_none()
                {
                    egui::Panel::bottom("document-details")
                        .default_size(148.0)
                        .size_range(80.0..=ui.ctx().content_rect().height().max(160.0) * 0.5)
                        .show(ui, |ui| self.details(ui));
                }
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(p.canvas))
                    .show(ui, |ui| match self.surface {
                        Surface::Library if self.metadata.is_some() => self.metadata_view(ui),
                        Surface::Library if self.cover_picker.is_some() => {
                            self.cover_picker_view(ui)
                        }
                        Surface::Library => self.library(ui),
                        Surface::Reader => self.reader(ui),
                        Surface::Settings => self.settings(ui),
                        Surface::Collections => self.collections_view(ui),
                        Surface::Tags => self.tags_view(ui),
                        Surface::Search => self.global_search(ui),
                        Surface::Notes => self.global_notes_panel(ui),
                        Surface::History => self.history_panel(ui),
                    });
            });
        });
        self.pump_library_removal(&ctx);
        self.update_cover_batch_progress();
        self.selection_review_dialog(&ctx);
        self.tag_management_dialogs(&ctx);
        self.legacy_import_dialog(&ctx);
        self.restore_dialog(&ctx);
        self.reading_queue_dialog(&ctx);
        self.workspace_dialog(&ctx);
        self.bulk_target_dialog(&ctx);
        self.password_dialog(&ctx);
        self.external_link_dialog(&ctx);
        self.pump_library_covers(&ctx);
        if self.library_selection.close_after_removal && !self.library_selection.removing() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if self.library_tags.close_after_write && !self.library_tags.writing() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if self.legacy_import.close_after && self.legacy_import.busy() {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        if self.backup_restore.close_after && self.backup_restore.busy() {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        if self.backups.close_after && self.backups.busy() {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        if let Some(path) = &self.capture {
            if !self.capture_requested {
                let ready = self.library_loaded
                    && (!matches!(self.surface, Surface::Reader)
                        || (!self.panel && self.reader_panel_tab != outline::ReaderPanelTab::Find)
                        || match self.reader_panel_tab {
                            outline::ReaderPanelTab::Bookmarks => self.bookmarks.ready(),
                            outline::ReaderPanelTab::Annotations => self.annotations.ready(),
                            outline::ReaderPanelTab::Outline => self.outline.ready(),
                            outline::ReaderPanelTab::Thumbnails => self.thumbnails.ready(),
                            outline::ReaderPanelTab::Find => self.find.ready(),
                            outline::ReaderPanelTab::Text => {
                                if self.selection.enabled {
                                    self.selection
                                        .text_state(self.page)
                                        .is_some_and(text::TextState::ready)
                                } else {
                                    self.text.ready()
                                }
                            }
                        })
                    && (self.link_preview == links::LinkPreview::Off
                        || self.links.ready()
                            && matches!(
                                self.link_preview,
                                links::LinkPreview::Focus | links::LinkPreview::Complete
                            ))
                    && (self.selection_preview == selection::SelectionPreview::Off
                        || self.selection_preview == selection::SelectionPreview::Complete
                            && self.selection.preview_ready())
                    && self.reader_destination.is_none()
                    && self.annotation_preview == 0
                    && self.annotation_export.ready()
                    && self.file_comparison.ready()
                    && self.bulk_organization_preview.is_none()
                    && self.bulk_picker.as_ref().is_none_or(|picker|picker.ready())
                    && self.library_selection.ready()
                    && self.library_tags.ready()
                    && self.collections.ready()
                    && self.cache_maintenance.ready()
                    && self.legacy_import.ready()
                    && self.backups.ready()
                    && self.backup_restore.ready()
                    && self.file_check.ready()
                    && self.library_check.ready()
                    && self.relocation.ready()
                    && (!matches!(self.surface, Surface::Notes) || self.global_notes.ready())
                    && (!matches!(self.surface, Surface::History) || self.history.ready())
                    && self.history.source_ready(if matches!(self.surface,Surface::Reader) {self.active.as_ref().map(|(doc,_)|(doc.id.as_str(),self.page))} else {None})
                    && self.restoration.ready()
                    && (!matches!(self.surface, Surface::Library) || !self.inspector || self.metadata.is_some() || self.goals.ready())
                    && self.metadata_preview.is_none_or(|(_,stage)|stage==3)
                    && self.metadata.as_ref().is_none_or(|editor|editor.capture_ready())
                    && self.library_covers.slots_ready()
                    && self.cover_picker_capture_ready()
                    && self.cover_picker.as_ref().is_none_or(|picker|picker.capture_ready())
                    && self.global_notes.source_ready(if matches!(self.surface,Surface::Reader) {self.active.as_ref().map(|(doc,_)|(doc.id.as_str(),self.page))} else {None})
                    // The compact drawer hides the PDF, so it has no visible marks to request.
                    && (!matches!(self.surface, Surface::Reader)
                        || self.panel && ctx.content_rect().width() < 620.0
                        || self.annotation_paint.ready())
                    && self.outline_preview_target.is_none()
                    && self.thumbnail_preview_page.is_none()
                    && !self.loading
                    && !self.importing.busy()
                    && self.importing.recovery_ready()
                    && self.opening.ready()
                    && self.tabs.ready()
                    && self.reading_queue.ready()
                    && self.workspaces.ready()
                    && match self.preview_password_target {
                        Some("unlock") => matches!(self.surface, Surface::Reader),
                        Some(_) => self.password.is_some(),
                        None => true,
                    }
                    && (!self.search.preview_open_page
                        || matches!(self.surface, Surface::Reader) && self.page == 1)
                    && (!self.search.preview_excluded
                        || self
                            .search
                            .stats
                            .as_ref()
                            .is_some_and(|stats| stats.excluded > 0))
                    && (!self.search.preview_results
                        || self.search.completed && !self.search.searching)
                    && (!matches!(self.surface, Surface::Reader)
                        || self.reader_preferences.is_some()
                            && (self.panel && ctx.content_rect().width() < 620.0
                                || !self.reader_pages.is_empty()
                                    && self
                                        .reader_pages
                                        .values()
                                        .all(|page| page.image.is_some())));
                if ready {
                    self.capture_ready_since.get_or_insert_with(Instant::now);
                } else {
                    self.capture_ready_since = None;
                }
                ctx.request_repaint_after(Duration::from_millis(100));
                if self
                    .capture_ready_since
                    .is_some_and(|since| since.elapsed() >= Duration::from_millis(350))
                {
                    self.capture_requested = true;
                    ctx.send_viewport_cmd(ViewportCommand::Screenshot(Default::default()));
                }
            }
            let screenshot = ctx.input(|i| {
                i.events.iter().find_map(|event| {
                    if let egui::Event::Screenshot { image, .. } = event {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
            });
            if let Some(screenshot) = screenshot {
                let bytes: Vec<u8> = screenshot
                    .pixels
                    .iter()
                    .flat_map(Color32::to_array)
                    .collect();
                let saved = image::save_buffer(
                    path,
                    &bytes,
                    screenshot.width() as u32,
                    screenshot.height() as u32,
                    image::ColorType::Rgba8,
                ).and_then(|_|{
                    let rect=|rect:egui::Rect|[rect.left(),rect.top(),rect.right(),rect.bottom()];
                    let controls=ctx.data(|data|data.get_temp::<Vec<theme::ControlBounds>>(egui::Id::new("capture-controls")).unwrap_or_default());
                    let controls:Vec<_>=controls.into_iter().map(|item|serde_json::json!({"label":item.label,"rect":rect(item.rect),"clip":rect(item.clip),"enabled":item.enabled})).collect();
                    let usage = self.governor.usage();
                    let mut record=serde_json::json!({"synthetic":self.demo,"libraryDocumentCount":self.count,"captureProcessId":std::process::id(),"viewport":rect(ctx.content_rect()),"pixelsPerPoint":ctx.pixels_per_point(),"controls":controls,
                        "pagePresenter":if self.presenter.is_some(){"native-wgpu"}else{"egui-glow"},
                        "readingSession":self.session_capture_summary(),
                        "visibleThumbnailSlots":self.thumbnails.resident_count(),
                        "libraryCovers":self.cover_summary(),
                        "coverPicker":self.cover_picker_summary(),
                        "coverPickerDemo":self.cover_picker_demo_summary(),
                        "metadataReview":self.metadata.as_ref().map(|editor|editor.summary()),
                        "documentFind":self.find.capture_summary(),
                        "documentLinks":self.links.capture_summary(),
                        "pageText": if self.selection.enabled { self.selection.text_state(self.page).map(text::TextState::capture_summary) } else { Some(self.text.capture_summary()) },
                        "textSelection":self.selection.capture_summary(),
                        "annotations":self.annotation_capture_summary(),
                        "annotationPaint":self.annotation_paint.capture_summary(),
                        "annotationExport":self.annotation_export.capture_summary(),
                        "fileComparison":self.file_comparison.summary(),
                        "importRecovery":self.importing.recovery_summary(),
                        "globalNotes":self.global_notes.summary(),
                        "readingHistory": self.history.summary(),
                        "readingRestoration": self.restoration_summary(),
                        "readerOpening":self.opening.summary(),
                        "readerTabs":self.tabs.summary(),
                        "readingGoal": self.goals.summary(),
                        "currentReaderPage":self.page,
                        "readerZoom":self.zoom,
                        "readerRenderEvents":self.capture_render_events,
                        "linkKeyboardActivated":self.link_preview == links::LinkPreview::Complete,
                        "trackedMemory":{"total":usage.total,"pages":usage.pages,"tiles":usage.tiles,"text":usage.text,"other":usage.other,"cpu":usage.cpu,"gpu":usage.gpu,"transient":usage.transient},
                        "memoryScope":"Tracked PDF buffers and native wgpu page payload/staging only; live text, Glow uploads, UI atlas and opaque driver allocations are not covered."});
                    record["bulkOrganization"]=self.bulk_organization_verified.clone().unwrap_or(serde_json::Value::Null);
                    record["bulkTargetPicker"]=self.bulk_picker_summary();
                    record["libraryVisibleFavorites"]=self.documents.iter().filter(|doc|doc.favorite).count().into();
                    record["librarySelection"]=self.library_selection.summary();
                    record["libraryGrouping"]=self.grouping_summary();
                    record["libraryFiltersActive"]=self.library_has_filters().into();
                    record["libraryQueryLoaded"]=self.library_loaded.into();
                    record["smartView"]=self.smart_view.map(|view|view.key()).into();
                    record["libraryTags"]=self.library_tags.summary();
                    record["legacyImport"]=self.legacy_import.summary();
                    record["backup"]=self.backups.summary();
                    record["restore"]=self.backup_restore.summary();
                    record["collections"]=self.collections.summary();
                    record["coverCache"]=self.cache_maintenance.summary();
                    record["fileCheck"]=self.file_check.summary();
                    record["relocation"]=self.relocation.summary();
                    record["libraryCheck"]=self.library_check.summary();
                    let bytes=serde_json::to_vec_pretty(&record).map_err(|error|image::ImageError::IoError(std::io::Error::other(error)))?;
                    std::fs::write(path.with_extension("controls.json"),bytes).map_err(image::ImageError::IoError)
                });
                match saved {
                    Ok(()) => ctx.send_viewport_cmd(ViewportCommand::Close),
                    Err(error) => {
                        self.error = Some(format!("Capture could not be saved: {error}"));
                        self.capture = None;
                        ctx.request_repaint();
                    }
                }
            }
        }
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        if let Err(error) = self.runtime.flush_import_recovery() {
            eprintln!("{error}");
            yamantaka_windows::show_startup_error(self.tr(
                "No se han podido guardar todas las selecciones de importación pendientes. Abre Yamantaka y vuelve a seleccionar esos archivos o carpetas.",
                "Some pending import choices could not be saved. Reopen Yamantaka and select those files or folders again.",
            ));
        }
        if let Err(error) = self.persist_reader_continuity_on_close() {
            eprintln!("{error}");
        }
        if let Err(error) = self.persist_reading_sessions_on_close() {
            eprintln!("{error}");
        }
        self.library_check.cancel();
        self.relocation.cancel();
        self.library_selection.cancel();
        self.library_tags.cancel();
        self.collections.cancel();
        self.workspaces.cancel();
        self.cache_maintenance.cancel();
        self.file_check.cancel();
        self.file_comparison.cancel();
        self.annotation_export.cancel();
        self.backups.cancel();
        self.backup_restore.cancel();
        self.global_notes.cancel();
        self.links.clear();
        self.annotation_paint.invalidate();
        self.text.clear();
        self.selection.clear();
        self.find = Default::default();
        self.cancel.cancel();
        self.query_cancel.cancel();
        self.reader_preferences_cancel.cancel();
        self.reader_pages.clear();
        self.thumbnails.clear();
        if let Some(presenter) = &mut self.presenter
            && let Err(error) = presenter.shutdown()
        {
            eprintln!("{error}");
        }
    }
}

/// Whole-page preview resolution remains valid at large zoom and small GPU caps.
/// Tiles will provide additional detail; this preview never silently stalls.
fn preview_dimensions(
    displayed: egui::Vec2,
    dpr: f32,
    device_limit: u32,
    bytes: usize,
) -> [u32; 2] {
    let width = (displayed.x * dpr).max(1.0) as f64;
    let height = (displayed.y * dpr).max(1.0) as f64;
    let dimension = device_limit.max(1) as f64;
    let scale = (dimension / width)
        .min(dimension / height)
        .min(((bytes / 4) as f64 / (width * height)).sqrt())
        .min(1.0);
    [
        (width * scale).floor().max(1.0) as u32,
        (height * scale).floor().max(1.0) as u32,
    ]
}
