mod annotations;
mod backup_restore;
mod backups;
pub use backup_restore::RestoreHandover;
mod automation;
mod bookmarks;
mod bulk_picker;
mod cache_maintenance;
mod catalogue_chooser;
mod close_review;
mod collections;
mod command_palette;
mod compact;
mod continuity_preview;
mod cover_picker;
mod cover_picker_preview;
mod cover_picker_view;
mod cover_rebuild;
mod cover_restore;
mod details;
mod diagnostics;
mod duplicates;
mod file_check;
mod file_comparison;
mod find;
mod folder_watch;
mod global_notes;
mod goals;
mod history;
mod legacy_import;
mod library_check;
mod library_columns;
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
mod page_prefetch;
mod page_refinement;
mod password;
mod quarantine;
pub(crate) mod reader_chrome;
mod reader_controls;
mod reader_destination;
mod reader_options;
mod reader_pane;
mod reader_profile;
mod reader_surface;
mod reader_viewport;
mod reading_queue;
mod reference;
mod relations;
mod relocation;
mod render_plan;
mod render_updates;
mod restoration;
mod safety;
mod search;
mod selection;
mod sessions;
mod settings;
mod shortcuts;
mod smart_views;
mod startup_repair;
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
    Duplicates,
    Search,
    Notes,
    History,
    Library,
    Reader,
    Settings,
    Collections,
    Relations,
    Automation,
    Tags,
}
enum PageImage {
    Native(PageTexture),
    Managed(crate::managed_pages::Texture),
}
struct ReaderPage {
    refinement: page_refinement::State,
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
    window_state: Option<crate::window_state::State>,
    startup_mode: yamantaka_domain::startup::StartupMode,
    safe_notice: bool,
    startup_repair: startup_repair::State,
    folder_watches: folder_watch::State,
    quarantine: quarantine::QuarantineState,
    shortcut_bindings: shortcuts::bindings::Bindings,
    shortcut_error: Option<String>,
    reader: reader_pane::ReaderPane,
    reading_focus: shortcuts::ReadingFocus,
    reference_shortcuts_preview: shortcuts::ShortcutPreview,
    reference: Option<reference::ReferenceView>,
    reference_mount: Option<reference::Mount>,
    reference_mount_preview: reference::PreparationPreview,
    reference_return: Option<Surface>,
    reference_demo: u8,
    reference_prepare_preview: reference::PreparationPreview,
    reference_close_demo: bool,
    reference_swap_demo: bool,
    reference_find_demo: bool,
    reference_links_demo: bool,
    reference_outline_demo: bool,
    reference_thumbnails_demo: bool,
    reference_bookmarks_demo: bool,
    reference_text_demo: bool,
    reference_native_closes: u64,
    reference_password_demo: u8,
    reference_password_attempts: u8,
    reference_chooser_demo: u8,
    reference_chooser_paged: usize,
    reference_sync_demo: u8,
    reference_region_preview: bool,
    reference_sync_linked: bool,
    reference_sync_position: bool,
    reference_sync_reverse: bool,
    reference_sync_differences: bool,
    preview_reference: bool,
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
    open_confirmation: Option<(String, Cancellation)>,
    password: Option<password::PasswordChallenge>,
    password_attempted: bool,
    preview_password_action: Option<&'static str>,
    import_password_preview: Option<password::ImportPreview>,
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
    library_density: yamantaka_domain::workspaces::WorkspaceDensity,
    library_columns: Vec<String>,
    reader_panel_width: f32,
    reader_panel_restore: bool,
    reader_chrome_hidden: bool,
    reader_chrome_blocked: bool,
    chrome_capture: Option<reader_chrome::Capture>,
    command_palette_preview: command_palette::PalettePreview,
    command_palette: command_palette::PaletteState,
    page_jump: Option<shortcuts::reader_commands::PageJump>,
    reader_focus: bool,
    offset: usize,
    selected: Option<String>,
    inspector: bool,
    find_preview_step: Option<bool>,
    pub find_preview_closed: bool,
    library_covers: library_covers::LibraryCovers,
    cover_picker: Option<cover_picker::CoverPicker>,
    cover_picker_demo: cover_picker_preview::CoverPickerDemo,
    thumbnail_preview_page: Option<u32>,
    outline_preview_target: Option<usize>,
    annotation_export: annotations::export::ExportState,
    file_comparison: file_comparison::ComparisonState,
    duplicates: duplicates::State,
    file_check: file_check::CheckState,
    cache_maintenance: cache_maintenance::CacheState,
    legacy_import: legacy_import::ImportState,
    backups: backups::BackupState,
    diagnostics: diagnostics::DiagnosticsState,
    backup_restore: backup_restore::RestoreState,
    collections: collections::CollectionState,
    relations: relations::RelationState,
    automation: automation::AutomationState,
    library_tags: tags::TagState,
    smart_view: Option<yamantaka_domain::LibrarySmartView>,
    smart_menu_preview: bool,
    library_check: library_check::CheckState,
    relocation: relocation::RelocationState,
    annotation_preview: u8,
    retired_annotation_writes: Vec<(String, Cancellation)>,
    reference_notes_preview: annotations::NotesPreview,
    reference_marks_preview: annotations::MarksPreview,
    annotation_edit_conflict: annotations::ConflictPreview,
    annotation_multi_preview: bool,
    selection_preview: selection::SelectionPreview,
    selection_keyboard_preview: bool,
    selection_copy_preview: Option<(String, String)>,
    comparison_selection_preview: selection::ComparisonSelectionPreview,
    presenter: Option<PagePresenter>,
    managed_pages: crate::managed_pages::Presenter,
    pub(crate) graphics_recovery: crate::graphics_recovery::State,
    preview_device_loss: Option<eframe::wgpu::Device>,
    device_loss_configured: bool,
    governor: MemoryGovernor,
    bulk_organization_preview: Option<bulk_picker::OrganizationPreview>,
    bulk_organization_verified: Option<serde_json::Value>,
    selection_admission_verified: Option<serde_json::Value>,
    bulk_picker: Option<bulk_picker::BulkPicker>,
    library_selection: library_selection::SelectionState,
    query_cancel: Cancellation,
    library_loaded: bool,
    error: Option<String>,
    close_review: close_review::CloseReview,
    importing: operations::BackgroundImport,
    cover_rebuild: cover_rebuild::State,
    cover_rebuild_preview: bool,
    protected_index_preview: bool,
    inspector_actions_preview: bool,
    preview_tools_menu: bool,
    activation: Receiver<Vec<PathBuf>>,
    _instance: std::rc::Rc<Instance>,
    _watch: ActivationWatch,
    capture: Option<PathBuf>,
    capture_requested: bool,
    capture_ready_since: Option<Instant>,
    pub(crate) idle_probe: Option<crate::idle_probe::Probe>,
    capture_render_events: Vec<serde_json::Value>,
    nearby_turn_preview: u8,
    nearby_rotation_preview: bool,
    nearby_expect_miss: bool,
    nearby_rotation_pause: bool,
    nearby_blank_before: [u64; 2],
    preview_reader: bool,
    logo: Option<TextureHandle>,
    preview_tilt: Option<egui::Vec2>,
    preview_list_focus: bool,
    preview_reader_options: bool,
    reader_options_section: usize,
    reader_profile_preview: u8,
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
        startup_mode: yamantaka_domain::startup::StartupMode,
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
        let library_density = if preferences
            .get("libraryDensity")
            .and_then(serde_json::Value::as_str)
            == Some("compact")
        {
            yamantaka_domain::workspaces::WorkspaceDensity::Compact
        } else {
            Default::default()
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
        let shortcut_bindings =
            shortcuts::bindings::Bindings::load(preferences.get("shortcutOverrides"));
        let mut reader = reader_pane::ReaderPane::default();
        reader.visual_profile = reader_profile::VisualProfile::load(&preferences);
        let mut app = Self {
            startup_mode,
            safe_notice: startup_mode.is_safe(),
            startup_repair: Default::default(),
            folder_watches: Default::default(),
            window_state: None,
            quarantine: Default::default(),
            reader,
            reading_focus: Default::default(),
            reference_shortcuts_preview: Default::default(),
            reference: None,
            reference_mount: None,
            reference_mount_preview: Default::default(),
            reference_return: None,
            reference_demo: 0,
            reference_prepare_preview: Default::default(),
            reference_close_demo: false,
            reference_swap_demo: false,
            reference_find_demo: false,
            reference_links_demo: false,
            reference_outline_demo: false,
            reference_thumbnails_demo: false,
            reference_bookmarks_demo: false,
            reference_text_demo: false,
            reference_native_closes: 0,
            reference_password_demo: 0,
            reference_password_attempts: 0,
            reference_chooser_demo: 0,
            reference_chooser_paged: 0,
            reference_sync_demo: 0,
            reference_region_preview: false,
            reference_sync_linked: false,
            reference_sync_position: false,
            reference_sync_reverse: false,
            reference_sync_differences: false,
            preview_reference: false,
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
            import_password_preview: None,
            preview_password_target: None,
            smart_view: preferences
                .get("librarySmartView")
                .and_then(serde_json::Value::as_str)
                .and_then(yamantaka_domain::LibrarySmartView::from_key),
            smart_menu_preview: false,
            library_tags: tags::TagState::new(&preferences),
            collections: collections::CollectionState::new(&preferences),
            relations: Default::default(),
            automation: Default::default(),
            runtime,
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
            library_density,
            library_columns: preferences
                .get("libraryColumns")
                .cloned()
                .and_then(|value| serde_json::from_value::<Vec<String>>(value).ok())
                .filter(|columns| yamantaka_domain::workspaces::validate_columns(columns).is_ok())
                .unwrap_or_else(yamantaka_domain::workspaces::default_columns),
            reader_panel_width: preferences
                .get("readerPanelWidth")
                .and_then(serde_json::Value::as_f64)
                .filter(|width| width.is_finite())
                .unwrap_or(300.0)
                .clamp(220.0, 500.0) as f32,
            reader_panel_restore: true,
            preferences,
            shortcut_bindings,
            shortcut_error: None,
            reader_chrome_hidden: false,
            reader_chrome_blocked: false,
            chrome_capture: None,
            command_palette_preview: Default::default(),
            command_palette: Default::default(),
            page_jump: None,
            reader_focus: false,
            offset: 0,
            selected: None,
            inspector,
            find_preview_step: None,
            find_preview_closed: false,
            library_covers: Default::default(),
            cover_picker: None,
            cover_picker_demo: Default::default(),
            thumbnail_preview_page: None,
            outline_preview_target: None,
            global_notes: Default::default(),
            history: Default::default(),
            goals: Default::default(),
            metadata: None,
            metadata_preview: None,
            annotation_export: Default::default(),
            file_comparison: Default::default(),
            duplicates: Default::default(),
            file_check: Default::default(),
            cache_maintenance: Default::default(),
            legacy_import: Default::default(),
            backups: Default::default(),
            diagnostics: Default::default(),
            backup_restore: Default::default(),
            library_check: Default::default(),
            relocation: Default::default(),
            annotation_preview: 0,
            retired_annotation_writes: Vec::new(),
            reference_notes_preview: Default::default(),
            reference_marks_preview: Default::default(),
            annotation_edit_conflict: Default::default(),
            annotation_multi_preview: false,
            selection_preview: Default::default(),
            selection_keyboard_preview: false,
            selection_copy_preview: None,
            comparison_selection_preview: Default::default(),
            presenter,
            managed_pages: crate::managed_pages::Presenter::new(governor.clone()),
            governor,
            bulk_organization_preview: None,
            bulk_organization_verified: None,
            selection_admission_verified: None,
            bulk_picker: None,
            library_selection: Default::default(),
            grouping,
            query_cancel: Cancellation::default(),
            library_loaded: false,
            error: None,
            close_review: Default::default(),
            importing: Default::default(),
            cover_rebuild: Default::default(),
            cover_rebuild_preview: false,
            protected_index_preview: false,
            inspector_actions_preview: false,
            preview_tools_menu: false,
            activation,
            _instance: instance,
            _watch: watch,
            capture,
            capture_requested: false,
            capture_ready_since: None,
            idle_probe: None,
            capture_render_events: Vec::new(),
            graphics_recovery: Default::default(),
            preview_device_loss: None,
            device_loss_configured: false,
            nearby_turn_preview: 0,
            nearby_rotation_preview: false,
            nearby_expect_miss: false,
            nearby_rotation_pause: false,
            nearby_blank_before: [0; 2],
            preview_reader: false,
            logo: None,
            preview_tilt: None,
            preview_list_focus: false,
            preview_reader_options: false,
            reader_options_section: 0,
            reader_profile_preview: 0,
        };
        app.check_tag_filters();
        app.check_collection_filter();
        app.refresh();
        app.request_import_rule_context(false);
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
        self.reader.viewport.reset_requests();
    }
    fn persist_library_state(&mut self) {
        if self.demo {
            return;
        }
        let state = [
            ("libraryColumns", serde_json::json!(self.library_columns)),
            (
                "readerPanelWidth",
                serde_json::json!(self.reader_panel_width),
            ),
            (
                "libraryDensity",
                serde_json::Value::from(
                    if self.library_density
                        == yamantaka_domain::workspaces::WorkspaceDensity::Compact
                    {
                        "compact"
                    } else {
                        "comfortable"
                    },
                ),
            ),
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
        self.dismiss_password_challenge();
        let cancel = self.importing.begin();
        if let Err(error) = self.runtime.submit(Action::Import(paths, cancel.clone())) {
            self.importing.submission_failed(&cancel);
            self.error = Some(error.to_string());
        }
    }
    pub fn import_folder(&mut self, root: PathBuf) {
        self.tabs.continuity.overridden = true;
        self.open_intent.leave_destination();
        self.dismiss_password_challenge();
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
    pub fn preview_graphics_loss(&mut self, device: eframe::wgpu::Device) {
        if self.demo && self.capture.is_some() {
            self.enable_continuity_capture();
            self.preview_device_loss = Some(device);
        }
    }
    pub fn text_scale(&self) -> f32 {
        self.text_scale
    }
    pub fn configure_window_state(&mut self, state: Option<crate::window_state::State>) {
        self.window_state = state;
    }
    /// Capture-only synthetic pointer, using the same spring and rigid transform.
    pub fn preview_tilt(&mut self, negative: bool) {
        self.preview_tilt = (self.demo && self.capture.is_some()).then(|| {
            if negative {
                egui::vec2(-1.0, -1.0)
            } else {
                egui::vec2(0.95, 0.75)
            }
        });
    }
    pub fn preview_layout(&mut self, list: bool, details: bool, panel: bool) {
        if self.demo && self.capture.is_some() {
            self.list = list;
            self.inspector = details;
            self.reader.panel = panel;
            self.reader.reader_panel_tab = outline::ReaderPanelTab::Text;
        }
    }
    pub fn preview_bookmarks(&mut self) {
        if self.demo && self.capture.is_some() {
            self.reader.panel = true;
            self.reader.reader_panel_tab = outline::ReaderPanelTab::Bookmarks;
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
        if matches!(surface, Surface::Reader) && !matches!(self.surface, Surface::Reader) {
            self.reader.chrome.reveal();
        }
        if !matches!(surface, Surface::Reader) && !self.request_reference_close() {
            self.reference_return = Some(surface);
            return;
        }
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
            self.page_jump = None;
            if self.reference.is_some() {
                self.reader.viewport.scroll_anchor = true;
            }
            self.reference = None;
            self.tabs.leave_destination();
            self.reader.restoration.clear();
        }
        if matches!(self.surface, Surface::Duplicates) {
            self.duplicates.cancel();
        }
        if matches!(self.surface, Surface::Automation) {
            self.automation.leave();
        }
        if matches!(self.surface, Surface::Tags) {
            self.library_tags.leave();
        }
        if matches!(self.surface, Surface::Collections) {
            self.collections.leave();
        }
        if matches!(self.surface, Surface::Relations) {
            self.relations.leave();
        }
        if matches!(self.surface, Surface::Notes) {
            self.global_notes.leave();
        }
        if matches!(self.surface, Surface::History) {
            self.history.leave();
        }
        self.reader.links.clear();
        self.reader.annotation_paint.invalidate();
        self.reader.text.clear();
        self.reader.selection.clear();
        self.reader.find.stop();
        self.reader.thumbnails.clear();
        self.open_intent.leave_destination();
        self.dismiss_password_challenge();
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
        if self.reference_blocks_switch() {
            return false;
        }
        if self.library_selection.removing() {
            return false;
        }
        if self.tabs.closing.busy() {
            return false;
        }
        if self.reader.annotations.editing() {
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
        self.reader.restoration.clear();
        self.dismiss_password_challenge();
        self.password_attempted = false;
        self.open_intent.begin_open();
        self.reader.loading = true;
        true
    }

    fn navigate(&mut self, page: u32) {
        if self
            .reader
            .navigation
            .as_mut()
            .is_some_and(|nav| nav.go_to_page(page))
        {
            self.apply_navigation();
        }
    }
    fn apply_navigation(&mut self) {
        if let Some((document, page)) = self.reader.navigation_changed() {
            self.send(Action::Navigate(document, page));
        }
    }
    pub(in crate::app) fn execute_shortcut(
        &mut self,
        ctx: &egui::Context,
        shortcut: shortcuts::Shortcut,
    ) {
        if self.dispatch_reading_shortcut(ctx, &shortcut) {
            return;
        }
        match shortcut {
            shortcuts::Shortcut::SwitchPane
            | shortcuts::Shortcut::ZoomIn
            | shortcuts::Shortcut::ZoomOut
            | shortcuts::Shortcut::FitPage
            | shortcuts::Shortcut::HistoryBack
            | shortcuts::Shortcut::HistoryForward => unreachable!(),
            shortcuts::Shortcut::GoToPage
            | shortcuts::Shortcut::Navigator
            | shortcuts::Shortcut::TogglePanel
            | shortcuts::Shortcut::ToggleBookmark
            | shortcuts::Shortcut::ToggleChrome
            | shortcuts::Shortcut::ToggleImmersive => unreachable!(),
            shortcuts::Shortcut::CommandPalette => self.open_command_palette(ctx),
            shortcuts::Shortcut::UndoAnnotation => self.annotation_history_command(false),
            shortcuts::Shortcut::RedoAnnotation => self.annotation_history_command(true),
            shortcuts::Shortcut::OpenDocument => {
                if let Some(paths) = yamantaka_windows::pick_pdfs() {
                    self.import(paths);
                }
            }
            shortcuts::Shortcut::ActualSize | shortcuts::Shortcut::FitWidth => unreachable!(),
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
            shortcuts::Shortcut::Focus => {
                self.toggle_immersive_reader(ctx);
            }
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
                                .corner_radius(5),
                        );
                    }
                    if !compact && !narrow {
                        if self.startup_mode.is_safe() {
                            if theme::navigation(ui, self.tr("Modo seguro", "Safe mode"), true, p)
                                .clicked()
                            {
                                self.safe_notice = true;
                            }
                        } else {
                            ui.label(RichText::new("Yamantaka").size(16.0).strong());
                        }
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
                    if self.reader.active.is_some() {
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
                self.library_header(ui, compact, p);
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
                    if ui.ctx().content_rect().height() < 420.0 {
                        self.library_catalogue(ui,compact,p);
                    } else {
                        egui::ScrollArea::vertical().id_salt("catalogue").show(ui,|ui|self.library_catalogue(ui,compact,p));
                    }
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
        let p = Palette::new(self.dark && !self.electronic_ink());
        if self.electronic_ink() {
            ui.ctx()
                .data_mut(|data| data.insert_temp(egui::Id::new("yamantaka-reduced-motion"), true));
            ui.painter()
                .rect_filled(ui.available_rect_before_wrap(), 0, Color32::WHITE);
            *ui.visuals_mut() = egui::Visuals::light();
            ui.visuals_mut().override_text_color = Some(p.ink);
            ui.visuals_mut().dark_mode = false;
            ui.style_mut().animation_time = 0.0;
        }
        let Some((document, info)) = self.reader.active.clone() else {
            ui.label(self.tr(
                "Elige un documento en la biblioteca.",
                "Choose a document from your library.",
            ));
            return;
        };
        self.sync_tab_context();
        if self.reader_focus {
            self.prepare_annotations(&document.id);
            self.reader.text.clear();
            self.reader.thumbnails.clear();
        }
        if self.reader.panel && !self.reader_focus && ui.ctx().content_rect().width() < 620.0 {
            if self.reader_chrome_hidden {
                self.reader_focus_exit(ui, p);
            } else {
                self.compact_reader_toolbar(ui, info.pages.len().max(1) as u32, p);
                ui.add_space(4.0);
            }
        }
        self.selection_toolbar(ui, p);
        if !self.reader_focus {
            self.reader_panel(ui, &document, &info, p);
        }
        if self.reader.panel && !self.reader_focus && ui.ctx().content_rect().width() < 620.0 {
            return;
        }
        if self.preview_reference
            && self.reader.reader_preferences.is_some()
            && self.open_confirmation.is_none()
            && !self.opening.busy()
        {
            self.preview_reference = false;
            self.toggle_reference(ui.ctx());
        }
        self.prepare_reference_mount(ui);
        self.prepare_saved_primary(ui);
        if self
            .reference
            .as_mut()
            .is_some_and(|reference| !reference.deferred)
        {
            self.comparison_viewport(ui, &document, p);
        } else {
            self.reader_viewport(ui, &document, p);
        }
        self.advance_profile_return();
        self.cache_reading_context();
    }
}
impl App for YamantakaApp {
    fn raw_input_hook(&mut self, _: &egui::Context, input: &mut egui::RawInput) {
        if self.demo && self.capture.is_some() {
            self.selection_preview
                .input(input, self.selection_keyboard_preview);
            self.reference_shortcuts_preview.input(input);
            self.command_palette_preview.input(input);
            self.comparison_selection_preview.input(input);
            if let Some(reference) = &mut self.reference {
                reference
                    .selection_preview
                    .input(input, reference.selection_keyboard_preview);
            }
        }
        if self.demo
            && self.capture.is_some()
            && (self.reader.links.preview == links::LinkPreview::KeyPending
                || self.reference.as_ref().is_some_and(|reference| {
                    reference.pane.links.preview == links::LinkPreview::KeyPending
                }))
        {
            if self.reader.links.preview == links::LinkPreview::KeyPending {
                self.reader.links.preview = links::LinkPreview::Activate;
            } else if let Some(reference) = &mut self.reference {
                reference.pane.links.preview = links::LinkPreview::Activate;
            }
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
        if self.graphics_recovery.faulted() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        self.guard_duplicate_merge_close(ctx);
        self.guard_workspace_close(ctx);
        self.guard_editor_close(ctx);
        self.guard_quarantine_close(ctx);
        self.update_restore(ctx);
        if self.backup_restore.busy() && ctx.input(|input| input.viewport().close_requested()) {
            self.backup_restore.close_after = true;
            self.backup_restore.cancel();
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        self.update_backup(ctx);
        self.update_diagnostics(ctx);
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
        if let Some((document, _)) = &self.reader.active {
            self.reader.annotations.bind(&document.id);
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
                Update::Quarantine(reply) => self.accept_quarantine(ctx, reply),
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
                Update::RuleSimulated(reply) => self.accept_rule_simulation(*reply),
                Update::RulePrepared(reply) => self.accept_rule_review(*reply),
                Update::InheritedAutomationRule(result, cancel) => {
                    self.accept_inherited_rule(result, cancel)
                }
                Update::RuleExecuted(reply) => self.accept_rule_execution(*reply),
                Update::AutomationJournals(result, cancel) => {
                    self.accept_rule_journals(result, cancel)
                }
                Update::AutomationRules(result, cancel) => {
                    self.accept_automation_rules(result, cancel)
                }
                Update::AutomationRuleChanged(result, cancel) => {
                    self.accept_automation_change(result, cancel)
                }
                Update::RevisionGroups(result, cancel) => {
                    self.accept_revision_groups(result, cancel)
                }
                Update::RevisionMembers(result, cancel) => {
                    self.accept_revision_members(result, cancel)
                }
                Update::RevisionChanged(result, cancel) => {
                    self.accept_revision_change(result, cancel)
                }
                Update::Relations(result, cancel) => self.accept_relations(result, cancel),
                Update::RelationChanged(result, cancel) => {
                    self.accept_relation_change(result, cancel)
                }
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
                Update::DuplicateMerge(reply) => self.accept_duplicate_merge(ctx, *reply),
                Update::Duplicates(reply) => self.duplicates.accept(*reply),
                Update::FileComparison(reply) => self.file_comparison.accept(*reply),
                Update::AnnotationExport(reply) => self.annotation_export.accept(reply),
                Update::AnnotationMarks(reply) => {
                    if let Some(reference) = &mut self.reference
                        && reference.pane.annotation_paint.owns(&reply)
                    {
                        reference.pane.annotation_paint.accept(reply);
                    } else {
                        self.reader.annotation_paint.accept(reply);
                    }
                }
                Update::Annotations(reply) => self.accept_annotation_reply(reply),
                Update::CoverRebuilt(reply) => self.accept_cover_rebuilt(*reply, ctx),
                Update::AnnotationRecovered(reply) => self.accept_stored_note_recovery(*reply),
                Update::SelectionRecovered(reply) => self.accept_selection_note_recovery(*reply),
                Update::Library(documents, count, memberships, cancel) => {
                    if self.command_palette.owns(&cancel) {
                        self.command_palette.accept(documents, cancel);
                        continue;
                    }

                    if self.relations.chooser.accept(&documents, count, &cancel) {
                        continue;
                    }
                    if self.reference.as_mut().is_some_and(|reference| {
                        reference.chooser.accept(&documents, count, &cancel)
                    }) {
                        continue;
                    }
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
                    if let Some(reference) = self
                        .reference
                        .as_mut()
                        .filter(|reference| reference.pane.outline.owns(&id, &cancel))
                    {
                        reference.pane.outline.accept(&id, &cancel, Ok(outline));
                    } else {
                        self.reader.outline.accept(&id, &cancel, Ok(outline));
                        self.apply_outline_preview();
                    }
                }
                Update::OutlineFailed(id, error, cancel) => {
                    if let Some(reference) = self
                        .reference
                        .as_mut()
                        .filter(|reference| reference.pane.outline.owns(&id, &cancel))
                    {
                        reference
                            .pane
                            .outline
                            .accept(&id, &cancel, Err(error.to_string()));
                    } else {
                        self.reader
                            .outline
                            .accept(&id, &cancel, Err(error.to_string()));
                    }
                }
                Update::Bookmarks(id, _, page, cancel) => {
                    self.accept_bookmarks(&id, &cancel, Ok(page), false);
                }
                Update::BookmarkChanged(id, page, cancel) => {
                    self.accept_bookmarks(&id, &cancel, Ok(page), true);
                }
                Update::BookmarkFailed(id, error, cancel) => {
                    self.accept_bookmarks(&id, &cancel, Err(error.to_string()), false);
                }
                Update::Opened(document, info, cancel) => {
                    if self.accept_reference_opened(ctx, &document, &info, &cancel) {
                        continue;
                    }
                    self.begin_open_preparation(document, info, cancel);
                }
                Update::DocumentMatches(id, batch, cancel) => {
                    if let Some(reference) = self
                        .reference
                        .as_mut()
                        .filter(|reference| reference.pane.find.owns(&id, &cancel))
                    {
                        reference.accept_reference_find(&self.runtime, &cancel, Ok(batch));
                    } else {
                        self.accept_find(&id, &cancel, Ok(batch));
                    }
                }
                Update::DocumentFindFailed(id, error, cancel) => {
                    if let Some(reference) = self
                        .reference
                        .as_mut()
                        .filter(|reference| reference.pane.find.owns(&id, &cancel))
                    {
                        reference.accept_reference_find(
                            &self.runtime,
                            &cancel,
                            Err(error.to_string()),
                        );
                    } else {
                        self.accept_find(&id, &cancel, Err(error.to_string()));
                    }
                }
                Update::PageMatches(id, payload, cancel) => {
                    self.reader
                        .find
                        .accept_highlights(&id, payload.clone(), &cancel);
                    if let Some(reference) = &mut self.reference {
                        reference.pane.find.accept_highlights(&id, payload, &cancel);
                    }
                }
                Update::PageFindFailed(id, error, cancel) => {
                    self.reader.find.fail_highlights(&id, &error, &cancel);
                    if let Some(reference) = &mut self.reference {
                        reference.pane.find.fail_highlights(&id, &error, &cancel);
                    }
                }
                Update::PageLinks(id, payload, cancel) => {
                    let page = payload.page();
                    if let Some(reference) = self
                        .reference
                        .as_mut()
                        .filter(|reference| reference.pane.links.owns(&id, page, &cancel))
                    {
                        reference.pane.links.accept(&id, page, &cancel, Ok(payload));
                    } else {
                        self.reader.links.accept(&id, page, &cancel, Ok(payload));
                    }
                }
                Update::PageLinksFailed(id, page, error, cancel) => {
                    if let Some(reference) = self
                        .reference
                        .as_mut()
                        .filter(|reference| reference.pane.links.owns(&id, page, &cancel))
                    {
                        reference
                            .pane
                            .links
                            .accept(&id, page, &cancel, Err(error.to_string()));
                    } else {
                        self.reader
                            .links
                            .accept(&id, page, &cancel, Err(error.to_string()));
                    }
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
                Update::Page(page, cancel, _, _) => {
                    if !self.accept_open_preparation_image(ctx, &page, &cancel)
                        && !self.accept_reference_mount_image(ctx, &page, &cancel)
                        && !self.accept_saved_primary_image(ctx, &page, &cancel)
                        && !self.accept_reference_preparation_image(ctx, &page, &cancel)
                    {
                        self.accept_rendered_page(ctx, &page, &cancel);
                    }
                }
                Update::Prefetched(key, result, cancel) => {
                    if !self.reader.viewport.prefetch.accept(
                        &key,
                        &cancel,
                        &result,
                        self.presenter.as_mut(),
                    ) && let Some(reference) = &mut self.reference
                    {
                        reference.pane.viewport.prefetch.accept(
                            &key,
                            &cancel,
                            &result,
                            self.presenter.as_mut(),
                        );
                    }
                }
                Update::TextGeometry(id, payload, cancel) => {
                    let page = payload.page();
                    if self.reference.as_mut().is_some_and(|reference| {
                        reference
                            .pane
                            .accept_text(&id, page, &cancel, Ok(payload.clone()))
                    }) {
                        continue;
                    }
                    self.reader.accept_text(&id, page, &cancel, Ok(payload));
                }
                Update::TextGeometryFailed(id, page, error, cancel) => {
                    let error = error.to_string();
                    if self.reference.as_mut().is_some_and(|reference| {
                        reference
                            .pane
                            .accept_text(&id, page, &cancel, Err(error.clone()))
                    }) {
                        continue;
                    }
                    self.reader.accept_text(&id, page, &cancel, Err(error));
                }
                Update::Text(..) => {}
                Update::ImportRuleContext(cancel, result) => {
                    if self
                        .importing
                        .context_pending
                        .as_ref()
                        .is_some_and(|pending| pending.same_request(&cancel))
                    {
                        self.importing.context_pending = None;
                        if !cancel.is_cancelled() {
                            match result {
                                Ok(batch) => self.importing.last_batch = batch,
                                Err(error) => self.error = Some(error.to_string()),
                            }
                        }
                    }
                }
                Update::Imported(_, cancel, batch) => {
                    if !self.importing.finish(&cancel) {
                        continue;
                    }
                    self.importing.last_batch = batch;
                    self.send(Action::PendingImports);
                    self.refresh();
                }
                Update::OpenFailed(document, error, cancel) => {
                    if self.accept_reference_failure(&document, &cancel, &error) {
                        if !cancel.is_cancelled() {
                            self.record_reader_failure(&document, &error);
                        }
                        continue;
                    }
                    let cancelled = cancel.is_cancelled();
                    let password = matches!(error, yamantaka_domain::Failure::PasswordRequired)
                        && self.open_intent.wants_reader();
                    if !self.opening.fail(&document.id, &cancel, password) {
                        continue;
                    }
                    if !cancelled {
                        self.record_reader_failure(&document, &error);
                    }
                    self.reader.loading = false;
                    if !password || cancelled {
                        self.tabs.fail(&document.id, &cancel);
                        self.reader.restoration.clear();
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
                    if self.fail_open_preparation(&error, cancel.as_ref()) {
                        continue;
                    }
                    if self.fail_reference_preparation(ctx, &error, cancel.as_ref()) {
                        continue;
                    }
                    if self.fail_reference_mount(ctx, &error, cancel.as_ref()) {
                        continue;
                    }
                    if self.fail_saved_primary(ctx, &error, cancel.as_ref()) {
                        continue;
                    }

                    if matches!(error, yamantaka_domain::Failure::PdfSourceChanged)
                        && let Some(token) = &cancel
                    {
                        let primary = self.reader.viewport.block_changed_source(token);
                        let reference = self.reference.as_mut().is_some_and(|reference| {
                            let changed = reference.pane.viewport.block_changed_source(token);
                            if changed {
                                reference.pane.loading = false;
                            }
                            changed
                        });
                        if primary {
                            self.reader.loading = false;
                            self.error =
                                Some(reader_surface::source_change_message(self.english).into());
                        }
                        if primary || reference {
                            continue;
                        }
                    }

                    if let Some(token) = &cancel
                        && (self.reader.viewport.fail_refinement(token)
                            || self.reference.as_mut().is_some_and(|reference| {
                                reference.pane.viewport.fail_refinement(token)
                            }))
                    {
                        continue;
                    }
                    if let Some(token) = &cancel
                        && self.command_palette.fail(token)
                    {
                        continue;
                    }
                    if let Some(token) = &cancel {
                        if self.relations.chooser.fail(token, &error) {
                            continue;
                        }
                        if self
                            .relations
                            .context_lookup
                            .as_ref()
                            .is_some_and(|(_, pending)| pending.same_request(token))
                        {
                            self.relations.context_lookup = None;
                            self.relations.error = Some(error.to_string());
                            continue;
                        }
                    }
                    if let Some(token) = &cancel
                        && self
                            .reference
                            .as_mut()
                            .is_some_and(|reference| reference.chooser.fail(token, &error))
                    {
                        continue;
                    }
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
                        .is_some_and(|token| self.reader.thumbnails.fail(token, error.to_string()))
                    {
                        continue;
                    }
                    if cancel.as_ref().is_some_and(|token| {
                        self.reference.as_mut().is_some_and(|reference| {
                            reference.pane.thumbnails.fail(token, error.to_string())
                        })
                    }) {
                        continue;
                    }
                    if cancel.as_ref().is_some_and(|token| token.is_cancelled()) {
                        continue;
                    }
                    if let Some(token) = &cancel
                        && self
                            .reference
                            .as_mut()
                            .is_some_and(|reference| reference.fail_render(token, &error))
                    {
                        continue;
                    }
                    self.error = Some(error.to_string());
                    if self.reader.reader_preferences.is_none() {
                        self.reader.reader_preferences_loading = false;
                    }
                    self.reader.viewport.retry = self.reader.viewport.requested.is_some();
                    self.reader.loading = false;
                }
                Update::ReaderClosed(reply) => {
                    if self.accept_reference_close(&reply) {
                        continue;
                    }
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
                    self.refresh_search_exclusions();
                }
                Update::OpenConfirmed {
                    document,
                    cancel,
                    result,
                } => {
                    if self.accept_reference_confirmation(&document, &cancel, &result) {
                        continue;
                    }
                    if let Err(error) = result {
                        if matches!(error, yamantaka_domain::Failure::QueueFull) {
                            if self.open_confirmation.is_none() {
                                self.open_confirmation = Some((document, cancel));
                                ctx.request_repaint_after(Duration::from_millis(100));
                            }
                        } else if !matches!(error, yamantaka_domain::Failure::Cancelled) {
                            self.error = Some(error.to_string());
                        }
                    }
                }
                Update::Committed(_, _) => {}
                Update::SearchExclusions(result, cancel) => {
                    self.accept_search_exclusions(result, &cancel)
                }
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
                    if self.accept_relation_context(&id, document.as_ref(), &cancel) {
                        continue;
                    }
                    if self.accept_tab_lookup(&id, document.clone(), &cancel) {
                        continue;
                    }
                    if let Some(target) = self.open_intent.accept_lookup(&id, &cancel) {
                        let page = target.page;
                        if let Some(document) =
                            document.filter(|document| document.id == id && !document.missing)
                        {
                            if self
                                .reader
                                .active
                                .as_ref()
                                .is_some_and(|(active, _)| active.id == id)
                            {
                                if let Some(page) = page {
                                    self.navigate(page);
                                }
                                self.show_surface(Surface::Reader);
                                if let Some(context) = target.context {
                                    self.reader.restoration.arm(
                                        id,
                                        self.reader.reader_preferences_cancel.clone(),
                                        context,
                                    );
                                    if let Some(preferences) =
                                        self.reader.reader_preferences.clone()
                                    {
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
                                        self.opening.arm_context(context);
                                    }
                                }
                            }
                        } else {
                            self.error = Some(self.tr("El documento ya no está disponible. Localízalo desde la biblioteca.", "The document is unavailable. Locate it from the library.").into());
                        }
                    }
                }
                Update::ReaderPreferences(id, preferences, cancel) => {
                    if self.accept_open_preparation_preferences(&id, preferences.clone(), &cancel) {
                        continue;
                    }

                    if cancel.is_cancelled()
                        || !cancel.same_request(&self.reader.reader_preferences_cancel)
                        || !self
                            .reader
                            .active
                            .as_ref()
                            .is_some_and(|(doc, _)| doc.id == id)
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
                Update::ReaderContinuity(reply) => self.accept_reader_continuity(*reply),
                Update::Workspace(reply) => self.accept_workspace(reply),
            }
        }
        self.advance_cover_picker_preview();
        self.advance_metadata_preview();
        self.advance_replacement_preview();
        self.advance_first_raster_probe();
        self.advance_tab_preview();
        self.advance_queue_preview();
        self.pump_workspace_startup(ctx);
        self.pump_workspace_context(ctx);
        self.advance_workspace_preview();
        self.advance_bulk_tab_preview();
        self.retire_closed_reader();
        self.pump_reference(ctx);
        self.advance_reference_demo();
        self.advance_reference_chooser();
        self.advance_reference_sync();
        self.poll_import_progress(ctx);
        self.sync_reading_session(ctx);
        if self.advance_graphics_loss_preview(ctx) {
            return;
        }
        self.pump_reader_continuity(ctx);
        if self.demo
            && self.capture.is_some()
            && self.selection_preview == selection::SelectionPreview::Waiting
            && matches!(self.surface, Surface::Reader)
            && self.reader.active.is_some()
            && !self.reader.selection.enabled
        {
            self.toggle_text_selection();
        }
        // Coalesce acknowledgement to the latest delivered open, then admit it
        // after draining results. Retry bounded control backpressure without
        // dropping the ownership handoff or continuously repainting while idle.
        if let Some((id, token)) = self.open_confirmation.take() {
            match self.runtime.submit(Action::ConfirmOpen {
                document: id.clone(),
                visible: vec![id.clone()],
                cancel: token.clone(),
            }) {
                Ok(()) => {}
                Err(yamantaka_domain::Failure::QueueFull) => {
                    self.open_confirmation = Some((id, token));
                    ctx.request_repaint_after(Duration::from_millis(100));
                }
                Err(error) => self.error = Some(error.to_string()),
            }
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut Frame) {
        if self.reduced_motion || (self.electronic_ink() && matches!(self.surface, Surface::Reader))
        {
            ui.ctx().input_mut(|input| input.finish_scroll_smoothing());
        }
        if self.reduced_motion {
            ui.style_mut().animation_time = 0.0;
        }
        if self.graphics_recovery.faulted() {
            return;
        }
        if let Some(gl) = frame.gl()
            && let Err(error) = self.managed_pages.begin_frame(ui.ctx(), gl)
        {
            self.error = Some(error.to_string());
        }
        let prefetch_allowed =
            !self.startup_mode.is_safe() && matches!(self.surface, Surface::Reader);
        self.reader.viewport.regions_enabled = prefetch_allowed && self.presenter.is_some();
        self.reader
            .viewport
            .prefetch
            .set_enabled(prefetch_allowed, self.presenter.is_some());
        if let Some(reference) = &mut self.reference {
            reference.pane.viewport.regions_enabled = prefetch_allowed && self.presenter.is_some();
            reference
                .pane
                .viewport
                .prefetch
                .set_enabled(prefetch_allowed, self.presenter.is_some());
        }

        if let Some(state) = &mut self.window_state
            && state.observe(ui.ctx(), frame).is_err()
        {
            self.error = Some(
                self.tr(
                    "No se pudo guardar la posición de la ventana.",
                    "Window placement could not be saved.",
                )
                .into(),
            );
        }
        if let Some(state) = &mut self.window_state {
            state.advance_verification(ui.ctx());
            if let Some(snapshot) = state.take_snapshot() {
                self.preferences
                    .rust
                    .insert("nativeWindowPlacement".into(), snapshot);
            }
        }
        if self.workspaces.close_after {
            ui.disable();
        }
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
        self.poll_folder_watches(&ctx);
        self.poll_startup_record_repair(&ctx);
        let shortcut_modal = self.duplicates.merge_open()
            || self.command_palette.open
            || self.close_review.open()
            || self.safe_notice
            || self.quarantine.open
            || self.quarantine.closing()
            || self.diagnostics.open
            || self.page_jump.is_some()
            || self.password.is_some()
            || self.link_confirmation_open()
            || self.bulk_picker.is_some()
            || self.selection_review_open()
            || self.tag_dialog_open()
            || self.legacy_import.open
            || self.backup_restore.open
            || self.reading_queue.open
            || self.workspaces.open
            || self.workspaces.close_after;
        self.reader_chrome_blocked = shortcut_modal;
        self.reading_focus.observe(
            &ctx,
            shortcut_modal,
            self.reference
                .as_ref()
                .filter(|_| {
                    matches!(self.surface, Surface::Reader)
                        && !(self.reader.panel
                            && !self.reader_focus
                            && ctx.content_rect().width() < 620.0)
                })
                .map(|reference| &reference.input_layers),
        );
        if let Some(reference) = &mut self.reference {
            reference.input_layers.clear();
        }
        self.reader.selection.keyboard_input(
            &ctx,
            shortcut_modal
                || self.reading_focus.reference()
                || !matches!(self.surface, Surface::Reader),
        );
        if let Some(reference) = &mut self.reference {
            let reference_busy = reference.busy();
            reference.pane.selection.keyboard_input(
                &ctx,
                shortcut_modal
                    || reference_busy
                    || !self.reading_focus.reference()
                    || !matches!(self.surface, Surface::Reader),
            );
        }
        let selection_blocked = shortcut_modal || !matches!(self.surface, Surface::Reader);
        let reference_selection = self.reading_focus.reference();
        let copy = if reference_selection {
            self.reference.as_mut().is_some_and(|reference| {
                let blocked = selection_blocked || reference.busy();
                reference.pane.selection.edit_commands(&ctx, blocked)
            })
        } else {
            self.reader.selection.edit_commands(&ctx, selection_blocked)
        };
        if copy {
            self.copy_selection(&ctx, reference_selection);
        }
        self.library_selection_shortcuts(&ctx);
        for shortcut in shortcuts::read(
            &ctx,
            shortcut_modal,
            matches!(self.surface, Surface::Reader),
            self.reading_focus.canvas_focused(&ctx)
                || self.reader.selection.focused_page(&ctx).is_some()
                || self
                    .reference
                    .as_ref()
                    .is_some_and(|reference| reference.pane.selection.focused_page(&ctx).is_some()),
            &self.shortcut_bindings,
        ) {
            if (self.command_palette.open || self.page_jump.is_some())
                && shortcut != shortcuts::Shortcut::Fullscreen
            {
                continue;
            }
            if !matches!(self.surface, Surface::Reader)
                && !shortcuts::available_outside_reader(shortcut)
            {
                continue;
            }
            self.execute_shortcut(&ctx, shortcut);
        }
        self.poll_rule_simulation(&ctx);
        self.poll_rule_review(&ctx);
        self.poll_rule_execution(&ctx);
        self.advance_rule_simulation_preview();
        self.advance_inherited_journal_preview();
        self.advance_automation_preview();
        self.advance_revision_preview();
        self.advance_relations_preview();
        self.advance_reference_shortcuts(&ctx);
        self.advance_palette_preview(&ctx);
        self.advance_quarantine_preview();
        self.advance_nearby_turn();
        self.advance_comparison_selection(&ctx);
        self.apply_annotation_preview();
        self.advance_annotation_conflict();
        self.advance_reference_marks();
        self.advance_reference_notes();
        self.advance_editor_close_preview(&ctx);
        self.apply_annotation_filter_preview();
        self.apply_annotation_bulk_preview();
        self.apply_annotation_tag_preview();
        self.apply_global_note_preview();
        self.apply_annotation_export_preview();
        self.update_file_comparison();
        self.update_library_selection_preview();
        self.update_removal_preview();
        self.update_cover_batch_preview(&ctx);
        self.update_cover_source_preview(&ctx);
        self.update_bulk_organization_preview();
        self.update_tag_preview();
        self.update_collection_preview();
        self.update_cache_maintenance();
        self.update_file_check();
        self.update_relocation();
        self.update_library_check();
        self.pump_duplicate_merge_preview(&ctx);
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
                let before_toolbar = ui.available_rect_before_wrap().top();
                if !matches!(self.surface, Surface::Reader) {
                    self.toolbar(ui);
                }
                let toolbar_extent = ui.available_rect_before_wrap().top() - before_toolbar;
                if let Some(error) = self.error.clone().or_else(|| {
                    self.reader
                        .viewport
                        .source_changed
                        .then(|| reader_surface::source_change_message(self.english).into())
                }) {
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
                                if self.reader.viewport.source_changed {
                                    let has_reference = self.reference.is_some();
                                    let label = if has_reference {
                                        self.tr("Cerrar comparación", "Close comparison")
                                    } else {
                                        self.tr("Cerrar lectura", "Close reading")
                                    };
                                    if theme::navigation(ui, label, false, p).clicked() {
                                        if has_reference {
                                            self.request_reference_close();
                                        } else if let Some(id) =
                                            self.tabs.model.active().map(|tab| tab.id().to_owned())
                                        {
                                            self.close_reader_tab(&id);
                                        }
                                    }
                                }
                                if self.reader.viewport.retry
                                    && !self.reader.viewport.source_changed
                                    && theme::primary_icon(
                                        ui,
                                        self.tr("Reintentar página", "Retry page"),
                                        theme::Icon::Right,
                                        p,
                                    )
                                    .clicked()
                                {
                                    self.reader.viewport.retry = false;
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
                let mut opening_rect = ui.available_rect_before_wrap();
                opening_rect.min.y -= toolbar_extent;
                self.measure_opening_view(ui, opening_rect);
                if self.opening.busy() && ui.ctx().current_pass_index() == 0 {
                    let previous = self.reader.reader_preferences_cancel.clone();
                    self.pump_open_preparation(ui.ctx());
                    if !previous.same_request(&self.reader.reader_preferences_cancel) {
                        ui.ctx()
                            .request_discard("Opening adoption updates the reader composition");
                        if ui.ctx().will_discard() {
                            return;
                        }
                    }
                } else if self.opening.busy() {
                    ui.ctx().request_repaint();
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
                        Surface::Relations => self.relations_view(ui),
                        Surface::Automation => self.automation_rules_view(ui),
                        Surface::Duplicates => self.duplicates_view(ui),
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
        self.duplicate_merge_dialog(&ctx);
        self.restore_dialog(&ctx);
        self.reading_queue_dialog(&ctx);
        self.workspace_dialog(&ctx);
        self.bulk_target_dialog(&ctx);
        self.command_palette_dialog(&ctx);
        self.diagnostics_dialog(&ctx);
        self.safe_mode_dialog(&ctx);
        self.quarantine_dialog(&ctx);
        self.page_jump_dialog(&ctx);
        self.advance_cover_rebuild_preview();
        self.password_dialog(&ctx);
        self.external_link_dialog(&ctx);
        self.editor_close_dialog(&ctx);
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
                    && self.stored_note_recovery_ready()
                    && self.selection_note_recovery_ready()
                    && matches!(self.annotation_edit_conflict.phase,0|5)
                    && self.editor_close_capture_ready()
                    && self.managed_pages.settled()
                    && self.preview_device_loss.is_none()
                    && matches!(self.nearby_turn_preview, 0 | 3)
                    && self.reader.viewport.refinements_settled()
                    && self.reader_profile_preview != 1
                    && (self.idle_probe.is_none() || !matches!(self.surface,Surface::Reader) || self.reader_chrome_hidden ||
                        (self.reader.panel || self.reader.chrome.settled()) && self.reference.as_ref().is_none_or(|reference|reference.pane.chrome.settled()))
                    && self.reference.as_ref().is_none_or(|reference|reference.pane.viewport.refinements_settled())
                    && self.reader.viewport.prefetch.settled()
                    && self.reference.as_ref().is_none_or(|reference|reference.pane.viewport.prefetch.settled())
                    && self.window_state.as_ref().is_none_or(crate::window_state::State::verification_ready)
                    && (!matches!(self.surface, Surface::Reader)
                        || (!self.reader.panel && self.reader.reader_panel_tab != outline::ReaderPanelTab::Find)
                        || match self.reader.reader_panel_tab {
                            outline::ReaderPanelTab::Positions => true,
                            outline::ReaderPanelTab::Bookmarks => self.reader.bookmarks.ready(),
                            outline::ReaderPanelTab::Annotations => self.reader.annotations.ready(),
                            outline::ReaderPanelTab::Outline => self.reader.outline.ready(),
                            outline::ReaderPanelTab::Thumbnails => self.reader.thumbnails.ready(),
                            outline::ReaderPanelTab::Find => self.reader.find.ready(),
                            outline::ReaderPanelTab::Text => {
                                if self.reader.selection.enabled {
                                    self.reader.selection
                                        .text_state(self.reader.page)
                                        .is_some_and(text::TextState::ready)
                                } else {
                                    self.reader.text.ready()
                                }
                            }
                        })
                    && (self.reader.links.preview == links::LinkPreview::Off
                        || self.reader.links.ready()
                            && matches!(
                                self.reader.links.preview,
                                links::LinkPreview::Focus | links::LinkPreview::Complete
                            ))
                    && (self.selection_preview == selection::SelectionPreview::Off
                        || self.selection_preview == selection::SelectionPreview::Complete
                            && self.reader.selection.preview_ready())
                    && self.reader.reader_destination.is_none()
                    && self.annotation_preview == 0
                    && self.reference_marks_preview.ready()
                    && self.reference_notes_preview.ready()
                    && self.annotation_export.ready()
                    && self.duplicates.ready()
                    && self.importing.context_pending.is_none()
                    && self.file_comparison.ready()
                    && self.bulk_organization_preview.is_none()
                    && self.bulk_picker.as_ref().is_none_or(|picker|picker.ready())
                    && self.library_selection.ready()
                    && self.library_tags.ready()
                    && self.collections.ready()
                    && self.relations.ready()
                    && self.automation.ready()
                    && self.cache_maintenance.ready()
                    && self.legacy_import.ready()
                    && self.backups.ready()
                    && self.diagnostics.ready()
                    && self.quarantine.ready()
                    && self.backup_restore.ready()
                    && self.file_check.ready()
                    && self.library_check.ready()
                    && self.relocation.ready()
                    && (!matches!(self.surface, Surface::Notes) || self.global_notes.ready())
                    && (!matches!(self.surface, Surface::History) || self.history.ready())
                    && self.history.source_ready(if matches!(self.surface,Surface::Reader) {self.reader.active.as_ref().map(|(doc,_)|(doc.id.as_str(),self.reader.page))} else {None})
                    && self.reader.restoration.ready()
                    && self.reference.as_ref().is_none_or(|reference| !reference.pane.loading && !reference.pane.viewport.pages.is_empty())
                    && self.reference_shortcuts_preview.ready()
                    && self.command_palette_preview.ready()
                    && self.comparison_selection_preview.ready()
                    && matches!(self.reference_demo, 0 | 3 | 5 | 7 | 9 | 11 | 13 | 16 | 19 | 28 | 35 | 39 | 40)
                    && matches!(self.reference_chooser_demo, 0 | 3)
                    && matches!(self.reference_sync_demo, 0 | 10)
                    && (self.reference_chooser_demo == 0 || self.reference.as_ref().is_some_and(|reference|reference.chooser.loaded && reference.chooser.total == 1))
                    && (!matches!(self.surface, Surface::Library) || !self.inspector || self.metadata.is_some() || self.goals.ready())
                    && self.metadata_preview.is_none_or(|(_,stage)|stage==3)
                    && self.metadata.as_ref().is_none_or(|editor|editor.capture_ready())
                    && self.library_covers.slots_ready()
                    && self.cover_picker_capture_ready()
                    && self.cover_picker.as_ref().is_none_or(|picker|picker.capture_ready())
                    && self.global_notes.source_ready(if matches!(self.surface,Surface::Reader) {self.reader.active.as_ref().map(|(doc,_)|(doc.id.as_str(),self.reader.page))} else {None})
                    // The compact drawer hides the PDF, so it has no visible marks to request.
                    && (!matches!(self.surface, Surface::Reader)
                        || self.reader.panel && ctx.content_rect().width() < 620.0
                        || self.reader.annotation_paint.ready())
                    && self.outline_preview_target.is_none()
                    && self.thumbnail_preview_page.is_none()
                    && !self.reader.loading
                    && (!self.importing.busy() || self.import_password_waiting_capture())
                    && self.import_password_capture_ready()
                    && self.cover_rebuild_capture_ready()
                    && self.protected_index_capture_ready()
                    && self.importing.recovery_ready()
                    && self.opening.ready()
                    && self.reference_mount.is_none()
                    && self.reference.as_ref().is_none_or(|reference|!reference.deferred)
                    && !self.preview_reference
                    && self.tabs.ready()
                    && self.reading_queue.ready()
                    && self.startup_record_repair_ready()
                    && self.search_exclusions_ready()
                    && self.workspaces.ready()
                    && match self.preview_password_target {
                        Some("unlock") => matches!(self.surface, Surface::Reader),
                        Some(_) => self.password.is_some(),
                        None => true,
                    }
                    && (!self.search.preview_open_page
                        || matches!(self.surface, Surface::Reader) && self.reader.page == 1)
                    && (!self.search.preview_excluded
                        || self
                            .search
                            .stats
                            .as_ref()
                            .is_some_and(|stats| stats.excluded > 0))
                    && (!self.search.preview_results
                        || self.search.completed && !self.search.searching)
                    && (!matches!(self.surface, Surface::Reader)
                        || self.reader.reader_preferences.is_some()
                            && (self.reader.panel && ctx.content_rect().width() < 620.0
                                || !self.reader.viewport.pages.is_empty()
                                    && self.reader.viewport.pages
                                        .values()
                                        .all(|page| page.image.is_some())));
                let ready = if let Some(capture) = &mut self.chrome_capture {
                    capture.advance(
                        &ctx,
                        ready,
                        &mut self.reader.chrome,
                        self.reference
                            .as_mut()
                            .map(|reference| &mut reference.pane.chrome),
                    )
                } else {
                    ready
                };
                if ready {
                    self.capture_ready_since.get_or_insert_with(Instant::now);
                } else {
                    self.capture_ready_since = None;
                }
                let capture_now = if let Some(probe) = &mut self.idle_probe {
                    probe.advance(&ctx, ready)
                } else {
                    self.capture_ready_since
                        .is_some_and(|since| since.elapsed() >= Duration::from_millis(350))
                };
                if !self
                    .idle_probe
                    .as_ref()
                    .is_some_and(|probe| probe.observing())
                {
                    ctx.request_repaint_after(Duration::from_millis(100));
                }
                if capture_now {
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
                        "visibleThumbnailSlots":self.reader.thumbnails.resident_count(),
                        "libraryCovers":self.cover_summary(),
                        "coverPicker":self.cover_picker_summary(),
                        "coverPickerDemo":self.cover_picker_demo_summary(),
                        "metadataReview":self.metadata.as_ref().map(|editor|editor.summary()),
                        "documentFind":self.reader.find.capture_summary(),
                        "documentLinks":self.reader.links.capture_summary(),
                        "pageText": if self.reader.selection.enabled { self.reader.selection.text_state(self.reader.page).map(text::TextState::capture_summary) } else { Some(self.reader.text.capture_summary()) },
                        "textSelection":self.reader.selection.capture_summary(),
                        "annotations":self.annotation_capture_summary(),
                        "annotationPaint":self.reader.annotation_paint.capture_summary(),
                        "annotationExport":self.annotation_export.capture_summary(),
                        "fileComparison":self.file_comparison.summary(),
                        "importRecovery":self.importing.recovery_summary(),
                        "globalNotes":self.global_notes.summary(),
                        "readingHistory": self.history.summary(),
                        "readingRestoration": self.restoration_summary(),
                        "readerOpening":self.opening.summary(),
                        "readerTabs":self.tabs.summary(),
                        "readingGoal": self.goals.summary(),
                        "currentReaderPage":self.reader.page,
                        "readerZoom":self.reader.zoom,
                        "readerRenderEvents":self.capture_render_events,
                        "linkKeyboardActivated":self.reader.links.preview == links::LinkPreview::Complete,
                        "trackedMemory":{"total":usage.total,"pages":usage.pages,"tiles":usage.tiles,"text":usage.text,"other":usage.other,"cpu":usage.cpu,"gpu":usage.gpu,"transient":usage.transient},
                        "memoryScope":"Tracked PDF buffers, extracted-text geometry and selected-text payloads, page/thumbnail/PDF-cover GPU payloads, wgpu staging and Glow image copies. egui text layout, UI/font textures and opaque driver allocations are not covered."});
                    record["selectionAdmission"]=self.selection_admission_verified.clone().unwrap_or(serde_json::Value::Null);
                    record["idleObservation"] = self.idle_probe.as_ref().map(|probe|probe.summary()).unwrap_or_default();
                    record["readerVisualProfile"] = self.reader.profile_summary();
                    record["readerVisualProfileReturn"] = self.reader_profile_preview.into();
                    record["readerAutoHide"] = self.reader.chrome.summary();
                    record["readerChromeTransition"] = self.chrome_capture.as_ref().map(|capture|capture.summary()).unwrap_or_default();
                    record["referenceAutoHide"] = self.reference.as_ref().map(|reference|reference.pane.chrome.summary()).unwrap_or_default();
                    record["referenceVisualProfile"] = self.reference.as_ref().map(|reference|reference.pane.profile_summary()).unwrap_or_default();
                    record["bulkOrganization"]=self.bulk_organization_verified.clone().unwrap_or(serde_json::Value::Null);
                    record["bulkTargetPicker"]=self.bulk_picker_summary();
                    record["libraryVisibleFavorites"]=self.documents.iter().filter(|doc|doc.favorite).count().into();
                    record["librarySelection"]=self.library_selection.summary();
                    record["libraryGrouping"]=self.grouping_summary();
                    record["workspaceLayout"]=serde_json::json!({"columns":self.library_columns,"panelWidth":self.reader_panel_width,"panelRestorePending":self.reader_panel_restore,"panelActualWidth":egui::containers::panel::PanelState::load(&ctx,egui::Id::new("reader-panel")).map(|state|state.size().x)});
                    record["libraryFiltersActive"]=self.library_has_filters().into();
                    record["libraryQueryLoaded"]=self.library_loaded.into();
                    record["smartView"]=self.smart_view.map(|view|view.key()).into();
                    record["libraryTags"]=self.library_tags.summary();
                    record["legacyImport"]=self.legacy_import.summary();
                    record["backup"]=self.backups.summary();
                    record["restore"]=self.backup_restore.summary();
                    record["collections"]=self.collections.summary();
                    record["searchExclusions"]=self.search_exclusions_summary();
                    record["workspaces"]=self.workspace_capture_summary();
                    record["coverCache"]=self.cache_maintenance.summary();
                    record["referenceShortcutPhase"] = serde_json::json!(self.reference_shortcuts_preview.phase());
                    record["ruleSimulation"] = self.automation.simulation_summary();
                    record["ruleReview"] = self.automation.review_summary();
                    record["ruleExecution"] = self.automation.execution_summary();
                    record["automationRules"] = self.automation.summary();
                    record["revisionGroups"] = self.revision_capture_summary();
                    record["commandPalette"]=self.command_palette.summary();
                    record["diagnostics"]=self.diagnostics.summary();
                    record["startupMode"] = serde_json::to_value(self.startup_mode).unwrap();
                    record["safeStartup"] = self.safe_summary();
                    record["startupRecordRepair"] = self.startup_record_repair_summary();
                    record["quarantine"] = self.quarantine.summary();
                    record["nearbyTurnPreview"] = self.nearby_turn_preview.into();
                    record["nearbyRotationTurn"]=serde_json::json!({"enabled":self.nearby_rotation_preview,"primaryEmptyPaints":self.reader.viewport.empty_paints.saturating_sub(self.nearby_blank_before[0]),"referenceEmptyPaints":self.reference.as_ref().map(|reference|reference.pane.viewport.empty_paints.saturating_sub(self.nearby_blank_before[1]))});
                    record["graphicsStartupFallback"] = crate::graphics_recovery::is_fallback().into();
                    record["readerContinuity"] = self.continuity_summary();
                    record["continuityStoreToken"] = if self.demo {self.store.file_name().and_then(|name|name.to_str()).into()} else {serde_json::Value::Null};
                    record["graphicsRuntimeFallback"] = crate::graphics_recovery::runtime_fallback().into();
                    record["managedPages"] = self.managed_pages.summary();
                    record["graphicsRecoveryFavorite"] = self.reader.active.as_ref().map(|(document,_)|document.favorite).into();
                    record["graphicsRecoveryRestorePreference"] = self.preferences.get("restoreLastSession").cloned().unwrap_or(serde_json::Value::Null);
                    record["pagePrefetch"] = self.reader.viewport.prefetch.summary();
                    record["pageRefinement"] = self.reader.viewport.refinement_summary();
                    record["referenceRefinement"] = self.reference.as_ref().map(|reference|reference.pane.viewport.refinement_summary()).unwrap_or(serde_json::Value::Null);
                    record["readerPresentationSlots"] = self.reader.viewport.pages.len().into();
                    record["referencePrefetch"] = self.reference.as_ref().map(|reference|reference.pane.viewport.prefetch.summary()).unwrap_or(serde_json::Value::Null);
                    record["windowPlacement"] = self.window_state.as_ref().map(crate::window_state::State::summary).unwrap_or(serde_json::Value::Null);
                    record["windowPlacementBackupSetting"] = self.preferences.get("nativeWindowPlacement").cloned().unwrap_or(serde_json::Value::Null);
                    record["annotationHistoryPhase"]=self.command_palette_preview.history_phase().into();
                    record["readerControls"]=serde_json::json!({"chromeHidden":self.reader_chrome_hidden,"immersive":self.reader_focus,"pageJump":self.page_jump.is_some(),"primaryPanel":self.reader.panel});
                    record["shortcutBindings"]=self.shortcut_bindings.summary();
                    record["shortcutConflict"]=self.shortcut_error.is_some().into();
                    record["relations"] = self.relations.summary();
                    record["referenceNotesPhase"] = serde_json::json!(self.reference_notes_preview.phase());
                    record["primaryAnnotations"] = self.reader.annotations.summary();
                    record["annotationEditConflict"] = self.annotation_conflict_summary();
                    record["referenceAnnotations"] = self.reference.as_ref().map(|view|view.pane.annotations.summary()).unwrap_or_default();
                    record["retiredAnnotationWrites"] = self.retired_annotation_writes.len().into();
                    record["referenceMarksPhase"] = serde_json::json!(self.reference_marks_preview.phase());
                    record["referenceMarks"] = self.reference.as_ref().map(|reference| reference.pane.annotation_paint.capture_summary()).unwrap_or_default();
                    record["referenceMarksRotation"] = self.reference.as_ref().and_then(|reference| reference.pane.viewport.layout.as_ref().map(|layout|layout.rotation().degrees())).into();
                    record["referenceSelectionPhase"] = serde_json::json!(self.comparison_selection_preview.phase());
                    record["referenceSelection"] = self.reference.as_ref().map(|reference| reference.pane.selection.capture_summary()).unwrap_or_default();
                    record["selectionClipboard"] = serde_json::json!(self.selection_copy_preview);
                    record["primaryNavigation"] = navigation::summary(self.reader.navigation.as_ref());
                    record["referenceNavigation"] = navigation::summary(self.reference.as_ref().and_then(|reference| reference.pane.navigation.as_ref()));
                    record["readingFocus"] = serde_json::json!(format!("{:?}", self.reading_focus.target));
                    record["documentOutline"] = self.reader.outline.capture_summary();
                    record["documentBookmarks"] = self.reader.bookmarks.capture_summary();
                    record["reference"]=self.reference.as_ref().map(|reference|serde_json::json!({"page":reference.pane.page,"zoom":reference.pane.zoom,"scale":reference.pane.viewport.scale,"images":reference.pane.viewport.pages.values().filter(|slot|slot.image.is_some()).count(),"followPage":reference.follow_page,"followZoom":reference.follow_zoom,"followPosition":reference.follow_position,"bidirectional":reference.bidirectional,"swapped":reference.swapped,"find":reference.pane.find.capture_summary(),"links":reference.pane.links.capture_summary(),"outline":reference.pane.outline.capture_summary(),"bookmarks":reference.pane.bookmarks.capture_summary(),"text":reference.pane.text.capture_summary(),"thumbnailSlots":reference.pane.thumbnails.resident_count(),"differences":reference.differences,"layout":reference.layout,"stacked":reference.stacked,"document":reference.pane.active.as_ref().map(|(doc,_)|&doc.id)})).unwrap_or_default();
                    record["referenceSync"]=serde_json::json!({"demoStage":self.reference_sync_demo,"linkedConfirmed":self.reference_sync_linked,"positionConfirmed":self.reference_sync_position,"reverseConfirmed":self.reference_sync_reverse,"differencesConfirmed":self.reference_sync_differences,"primaryScale":self.reader.viewport.scale});
                    record["closeReview"] = self.editor_close_summary();
                    record["pageNoteRecovery"] = self.page_note_recovery_summary();
                    record["metadataDraftRecovery"] = self.metadata_draft_summary();
                    record["coverRebuild"] = self.cover_rebuild_summary();
                    record["importPassword"] = self.import_password_summary();
                    record["storedNoteRecovery"] = self.stored_note_recovery_summary();
                    record["selectionNoteRecovery"] = self.selection_note_recovery_summary();
                    record["referenceAdmission"]=serde_json::json!({"demoStage":self.reference_demo,"nativeCloses":self.reference_native_closes,"passwordAttempts":self.reference_password_attempts});
                    record["referenceOpening"]=self.reference.as_ref().and_then(|reference|reference.opening_summary()).unwrap_or_default();
                    record["referenceMount"]=self.reference_mount_summary();
                    record["referencePreparationPreview"]=serde_json::json!({"cancelledBeforeAdoption":self.reference_prepare_preview==reference::PreparationPreview::Complete});
                    record["referenceChooser"]=self.reference.as_ref().map(|reference|serde_json::json!({"demoStage":self.reference_chooser_demo,"pagedRows":self.reference_chooser_paged,"offset":reference.chooser.offset,"count":reference.chooser.total,"rows":reference.chooser.entries.len(),"outsideVisible":reference.chooser.entries.first().is_some_and(|entry|!self.documents.iter().any(|doc|doc.id==entry.id))})).unwrap_or_default();
                    record["fileCheck"]=self.file_check.summary();
                    record["relocation"]=self.relocation.summary();
                    record["duplicates"]=self.duplicates.summary();
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
        if self.close_review.open() && !self.capture_requested && !self.graphics_recovery.faulted()
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
    }
    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.graphics_recovery.begin_close();
        if self.graphics_recovery.faulted()
            && let Err(error) = self.runtime.flush_changes()
        {
            self.graphics_recovery.checkpoint_failed();
            eprintln!("{error}");
        }
        if self.graphics_recovery.faulted() {
            while let Ok(update) = self.runtime.updates.try_recv() {
                match update {
                    Update::Annotations(reply) => self.accept_annotation_reply(reply),
                    Update::Metadata(reply) => {
                        if let Some(editor) = &mut self.metadata {
                            editor.accept_shutdown_reply(*reply);
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Some(state) = &mut self.window_state
            && let Err(error) = state.finish()
        {
            eprintln!("{error}");
        }
        if let Err(error) = self.runtime.flush_import_recovery() {
            self.graphics_recovery.checkpoint_failed();
            eprintln!("{error}");
            yamantaka_windows::show_startup_error(self.tr(
                "No se han podido guardar todas las selecciones de importación pendientes. Abre Yamantaka y vuelve a seleccionar esos archivos o carpetas.",
                "Some pending import choices could not be saved. Reopen Yamantaka and select those files or folders again.",
            ));
        }
        if let Err(error) = self.persist_reader_continuity_on_close() {
            self.graphics_recovery.checkpoint_failed();
            eprintln!("{error}");
        }
        if let Err(error) = self.persist_reading_sessions_on_close() {
            self.graphics_recovery.checkpoint_failed();
            eprintln!("{error}");
        }
        self.duplicates.cancel();
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
        self.reader.links.clear();
        self.reader.annotation_paint.invalidate();
        self.reader.text.clear();
        self.reader.selection.clear();
        self.reader.find = Default::default();
        self.reference = None;
        self.reference_mount = None;
        self.reader.viewport.clear();
        self.query_cancel.cancel();
        self.reader.reader_preferences_cancel.cancel();
        self.reader.thumbnails.clear();
        self.cover_picker = None;
        self.library_covers.clear();
        if let Some(gl) = gl {
            self.managed_pages.shutdown(gl);
        }
        if let Some(presenter) = &mut self.presenter
            && let Err(error) = presenter.shutdown()
        {
            eprintln!("{error}");
        }
        self.graphics_recovery.closed();
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
