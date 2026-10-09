mod bookmarks;
mod compact;
mod details;
mod find;
mod links;
mod list;
mod navigation;
mod open_intent;
mod outline;
mod password;
mod reader_controls;
mod reader_destination;
mod reader_options;
mod reader_viewport;
mod render_updates;
mod search;
mod settings;
mod shortcuts;
mod thumbnails;

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
    Library,
    Reader,
    Settings,
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
    open_intent: open_intent::OpenIntent,
    open_confirmation: Option<(String, Cancellation)>,
    password: Option<password::PasswordChallenge>,
    password_attempted: bool,
    preview_password_action: Option<&'static str>,
    preview_password_target: Option<&'static str>,
    runtime: Runtime,
    preferences: Preferences,
    store: PathBuf,
    dark: bool,
    english: bool,
    demo: bool,
    reduced_motion: bool,
    text_scale: f32,
    surface: Surface,
    documents: Vec<Document>,
    count: u64,
    query: String,
    favorites: bool,
    reading_state: LibraryReadingState,
    sort: LibrarySort,
    sort_direction: SortDirection,
    missing_only: bool,
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
    thumbnail_preview_page: Option<u32>,
    outline_preview_target: Option<usize>,
    reader_destination: Option<(
        String,
        yamantaka_domain::pdf_destination::NavigationDestination,
    )>,
    bookmarks: bookmarks::BookmarkState,
    panel_text: String,
    panel_requested: Option<(String, u32, u64)>,
    reader_pages: HashMap<u32, ReaderPage>,
    reader_layout: Option<Arc<ReaderLayout>>,
    reader_scroll_anchor: bool,
    presenter: Option<PagePresenter>,
    governor: MemoryGovernor,
    generation: u64,
    requested: Option<RenderKey>,
    render_retry: bool,
    cancel: Cancellation,
    query_cancel: Cancellation,
    error: Option<String>,
    loading: bool,
    activation: Receiver<Vec<PathBuf>>,
    _instance: Instance,
    _watch: ActivationWatch,
    capture: Option<PathBuf>,
    capture_requested: bool,
    capture_ready_since: Option<Instant>,
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
        instance: Instance,
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
        let mut app = Self {
            search: search::SearchState::new(
                preferences
                    .get("globalIndexEnabled")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
            ),
            open_intent: open_intent::OpenIntent::default(),
            open_confirmation: None,
            password: None,
            password_attempted: false,
            preview_password_action: None,
            preview_password_target: None,
            runtime,
            preferences,
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
            thumbnail_preview_page: None,
            outline_preview_target: None,
            reader_destination: None,
            bookmarks: bookmarks::BookmarkState::default(),
            panel_text: String::new(),
            panel_requested: None,
            reader_pages: HashMap::new(),
            reader_layout: None,
            reader_scroll_anchor: true,
            presenter,
            governor,
            generation: 0,
            requested: None,
            render_retry: false,
            cancel: Cancellation::default(),
            query_cancel: Cancellation::default(),
            error: None,
            loading: false,
            activation,
            _instance: instance,
            _watch: watch,
            capture,
            capture_requested: false,
            capture_ready_since: None,
            preview_reader: false,
            logo: None,
            preview_tilt: false,
            preview_list_focus: false,
            preview_reader_options: false,
            reader_options_section: 0,
        };
        app.refresh();
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
        self.query_cancel.cancel();
        self.query_cancel = Cancellation::default();
        self.send(Action::Query(
            LibraryQuery {
                text: self.query.clone(),
                favorites_only: self.favorites,
                reading_state: self.reading_state,
                sort: self.sort,
                sort_direction: self.sort_direction,
                missing_only: self.missing_only,
                offset: self.offset,
                ..Default::default()
            },
            self.query_cancel.clone(),
        ));
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
        self.open_intent.leave_destination();
        self.password = None;
        self.cancel.cancel();
        self.cancel = Cancellation::default();
        self.loading = true;
        self.send(Action::Import(paths, self.cancel.clone()));
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
            self.preview_list_focus = true;
        }
    }
    fn show_surface(&mut self, surface: Surface) {
        self.links.clear();
        self.find.stop();
        self.thumbnails.clear();
        self.open_intent.leave_destination();
        self.password = None;
        self.surface = surface;
    }
    fn open(&mut self, document: Document) {
        self.links.clear();
        self.thumbnails.clear();
        let preview = self.find.take_preview();
        self.find = Default::default();
        self.find.restore_preview(preview);
        self.reader_destination = None;
        self.outline = Default::default();
        self.password = None;
        self.password_attempted = false;
        self.open_intent.begin_open();
        self.reader_preferences_cancel.cancel();
        self.reader_preferences_cancel = Cancellation::default();
        self.reader_preferences = None;
        self.reader_preferences_loading = true;
        self.cancel.cancel();
        self.cancel = Cancellation::default();
        self.loading = true;
        self.reader_pages.clear();
        self.reader_layout = None;
        self.reset_render_request();
        self.generation += 1;
        self.send(Action::Open(document, None, self.cancel.clone()));
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
                            theme::display(
                                ui,
                                self.tr("Tu biblioteca", "Your library"),
                                if available <420.0 {18.0} else if compact { 26.0 } else { 34.0 },
                            );
                            if available>=420.0 {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(format!(
                                        "{} {}",
                                        self.count,
                                        self.tr(
                                            "documentos en tu colección",
                                            "documents in your collection"
                                        )
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
                        let response=theme::icon_toggle(ui,self.tr("Filtros y orden","Filters and sorting"),theme::Icon::Sort,self.favorites||self.reading_state!=LibraryReadingState::All||self.missing_only,p);
                        egui::Popup::menu(&response).show(|ui|self.library_filters(ui,p));
                    }else{self.library_filters(ui,p);}
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
                if let Some(document) = self
                    .documents
                    .iter()
                    .find(|doc| doc.reading_progress > 0.0)
                    .cloned()
                    && !compact
                {
                    egui::Frame::new()
                        .fill(p.surface)
                        .inner_margin(egui::Margin::symmetric(14, 8))
                        .corner_radius(12.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let response = ui
                                    .push_id("continuity-book", |ui| {
                                        book::miniature(ui, &document, 42.0, self.reduced_motion, p)
                                    })
                                    .inner;
                                if response.clicked() {
                                    self.open(document.clone());
                                }
                                ui.add_space(10.0);
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y=2.0;
                                    ui.spacing_mut().interact_size.y=0.0;
                                    ui.add_space(2.0);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(document.display_title()).font(
                                                egui::FontId::new(
                                                    20.0,
                                                    egui::FontFamily::Name("Editorial".into()),
                                                ),
                                            ),
                                        )
                                        .truncate(),
                                    );
                                    ui.label(
                                        RichText::new(document.author.as_deref().unwrap_or(""))
                                            .size(13.0)
                                            .color(p.secondary),
                                    );
                                    ui.add_space(2.0);
                                    ui.horizontal(|ui| {
                                        let (track, _) = ui.allocate_exact_size(
                                            egui::vec2(100.0, 3.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().rect_filled(track, 2.0, p.line);
                                        let mut progress = track;
                                        progress.max.x = progress.min.x
                                            + track.width()
                                                * document.reading_progress.clamp(0.0, 1.0) as f32;
                                        ui.painter().rect_filled(progress, 2.0, p.accent);
                                        ui.label(
                                            RichText::new(format!(
                                                "{}%  ·  {} {} {} {}",
                                                (document.reading_progress * 100.0).round(),
                                                self.tr("página", "page"),
                                                document.last_page_index + 1,
                                                self.tr("de", "of"),
                                                document.page_count
                                            ))
                                            .size(12.0)
                                            .color(p.secondary),
                                        );
                                    });
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if theme::control(
                                            ui,
                                            self.tr("Continuar leyendo", "Continue reading"),
                                            Some(theme::Icon::Right),
                                            false,
                                            true,
                                            p,
                                        )
                                        .clicked()
                                        {
                                            self.open(document.clone());
                                        }
                                    },
                                );
                            });
                        });
                    ui.add_space(10.0);
                }
                if self.documents.is_empty() {
                    let filtered=!self.query.is_empty()||self.favorites||self.reading_state!=LibraryReadingState::All||self.missing_only;
                    ui.add_space(if compact{18.0}else{36.0});
                    theme::display(ui,if filtered{self.tr("No hay libros que coincidan","No matching books")}else{self.tr("Un lugar para tus próximas lecturas","A place for your next reads")},28.0);
                    ui.label(RichText::new(if filtered{self.tr("Prueba otro título o restablece los filtros para ver tu colección.","Try another title or reset the filters to see your collection.")}else{self.tr("Añade tus PDF o arrástralos a esta ventana para empezar.","Add your PDFs or drag them into this window to begin.")}).color(p.secondary));
                    ui.add_space(12.0);
                    if filtered {
                        if theme::control(ui,self.tr("Restablecer filtros","Reset filters"),None,false,true,p).clicked(){self.query.clear();self.favorites=false;self.reading_state=LibraryReadingState::All;self.missing_only=false;self.offset=0;self.refresh();}
                    } else if theme::control(ui,self.tr("Elegir documentos","Choose documents"),Some(theme::Icon::Add),false,true,p).clicked() && let Some(paths)=yamantaka_windows::pick_pdfs(){self.import(paths);}
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("catalogue")
                        .show(ui, |ui| {
                            let documents = self.documents.clone();
                            if self.list {
                                self.list_documents(ui,documents,p);
                            } else if ui.ctx().content_rect().width()<420.0 {self.compact_covers(ui,documents,p);} else {
                                let columns = (ui.available_width()
                                    / if compact { 260.0 } else { 240.0 })
                                .floor()
                                .max(1.0) as usize;
                                let gap = 36.0;
                                let slot = ((ui.available_width() - gap * (columns - 1) as f32)
                                    / columns as f32)
                                    .max(100.0);
                                let width = if compact {
                                    slot.min(
                                        ((ui.available_height() - 83.0) / 1.34).clamp(110.0, 180.0),
                                    )
                                } else {
                                    slot.min(228.0)
                                };
                                for row in documents.chunks(columns) {
                                    ui.horizontal_top(|ui| {
                                        ui.spacing_mut().item_spacing.x = gap;
                                        for document in row {
                                            let response = ui
                                                .allocate_ui_with_layout(
                                                    egui::vec2(slot, width * 1.34 + 78.0),
                                                    egui::Layout::top_down(egui::Align::Center),
                                                    |ui| {
                                                        ui.push_id(&document.id, |ui| {
                                                            book::cover(
                                                                ui,
                                                                document,
                                                                width,
                                                                self.selected.as_deref()
                                                                    == Some(&document.id),
                                                                self.reduced_motion,
                                                                p,
                                                                if self.preview_tilt
                                                                    && document.id == "demo-0"
                                                                {
                                                                    Some(egui::vec2(0.95, 0.75))
                                                                } else {
                                                                    None
                                                                },
                                                            )
                                                        })
                                                        .inner
                                                    },
                                                )
                                                .inner;
                                            if response.clicked() {
                                                self.selected = Some(document.id.clone());
                                            }
                                            if response.double_clicked()
                                                || response.has_focus()
                                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                            {
                                                self.open(document.clone());
                                            }
                                            response.context_menu(|ui| {
                                                if ui
                                                    .button(
                                                        self.tr("Abrir documento", "Open document"),
                                                    )
                                                    .clicked()
                                                {
                                                    self.open(document.clone());
                                                    ui.close();
                                                }
                                                if ui
                                                    .button(self.tr(
                                                        "Mostrar en Explorer",
                                                        "Show in Explorer",
                                                    ))
                                                    .clicked()
                                                {
                                                    if let Err(error) =
                                                        yamantaka_windows::reveal(&document.path)
                                                    {
                                                        self.error = Some(error.to_string());
                                                    }
                                                    ui.close();
                                                }
                                                if ui
                                                    .button(
                                                        self.tr(
                                                            "Cambiar favorito",
                                                            "Toggle favorite",
                                                        ),
                                                    )
                                                    .clicked()
                                                {
                                                    self.send(Action::Favorite(
                                                        document.id.clone(),
                                                        !document.favorite,
                                                    ));
                                                    self.refresh();
                                                    ui.close();
                                                }
                                            });
                                        }
                                    });
                                    ui.add_space(14.0);
                                }
                            }
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
        if theme::navigation(
            ui,
            self.tr("Todos los libros", "All books"),
            !self.favorites,
            p,
        )
        .clicked()
        {
            self.favorites = false;
            self.offset = 0;
            self.refresh();
        }
        if theme::navigation(ui, self.tr("Favoritos", "Favorites"), self.favorites, p).clicked() {
            self.favorites = true;
            self.offset = 0;
            self.refresh();
        }
        let status = self.tr(
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
        );
        let response = theme::navigation(
            ui,
            status,
            self.reading_state != LibraryReadingState::All || self.missing_only,
            p,
        );
        egui::Popup::menu(&response).show(|ui| {
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
                if theme::navigation(ui, self.tr(es, en), self.reading_state == state, p).clicked()
                {
                    self.reading_state = state;
                    self.offset = 0;
                    self.refresh();
                    ui.close();
                }
            }
            ui.separator();
            let label = self
                .tr("Solo archivos ausentes", "Only missing files")
                .to_owned();
            if ui.checkbox(&mut self.missing_only, label).changed() {
                self.offset = 0;
                self.refresh();
            }
        });
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
                if theme::navigation(ui, self.tr(es, en), self.sort_direction == direction, p)
                    .clicked()
                {
                    self.sort_direction = direction;
                    self.offset = 0;
                    self.refresh();
                    ui.close();
                }
            }
        });
        if (!self.query.is_empty()
            || self.favorites
            || self.reading_state != LibraryReadingState::All
            || self.missing_only)
            && theme::navigation(
                ui,
                self.tr("Restablecer filtros", "Reset filters"),
                false,
                p,
            )
            .clicked()
        {
            self.query.clear();
            self.reading_state = LibraryReadingState::All;
            self.missing_only = false;
            self.favorites = false;
            self.offset = 0;
            self.refresh();
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
        if ui.ctx().content_rect().width() < 420.0 {
            self.compact_reader_toolbar(ui, info.pages.len().max(1) as u32, p);
        } else {
            egui::Frame::new()
                .fill(p.surface)
                .inner_margin(egui::Margin::symmetric(18, 8))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if theme::control(
                            ui,
                            self.tr("Biblioteca", "Library"),
                            Some(theme::Icon::Left),
                            false,
                            false,
                            p,
                        )
                        .clicked()
                        {
                            self.show_surface(Surface::Library);
                            self.refresh();
                        }
                        if ui.available_width() > 880.0 {
                            ui.add_space(12.0);
                            ui.add_sized(
                                [ui.available_width().min(320.0), 40.0],
                                egui::Label::new(
                                    RichText::new(document.display_title()).size(18.0),
                                )
                                .truncate(),
                            )
                            .on_hover_text(document.display_title());
                        }
                        ui.add_space(8.0);
                        if ui
                            .add_enabled_ui(self.page > 0, |ui| {
                                theme::icon_button(
                                    ui,
                                    self.tr("Página anterior", "Previous page"),
                                    theme::Icon::Left,
                                    p,
                                )
                            })
                            .inner
                            .clicked()
                        {
                            self.step_reader(false);
                        }
                        let mut shown = self.page + 1;
                        let page_label = ui.label(
                            RichText::new(self.tr("Página", "Page"))
                                .size(13.0)
                                .color(p.secondary),
                        );
                        if ui
                            .add(
                                egui::DragValue::new(&mut shown)
                                    .range(1..=info.pages.len().max(1) as u32),
                            )
                            .labelled_by(page_label.id)
                            .changed()
                        {
                            self.navigate(shown.saturating_sub(1));
                        }
                        ui.label(
                            RichText::new(format!("/ {}", info.pages.len()))
                                .size(13.0)
                                .color(p.secondary),
                        );
                        if ui
                            .add_enabled_ui((self.page as usize) + 1 < info.pages.len(), |ui| {
                                theme::icon_button(
                                    ui,
                                    self.tr("Página siguiente", "Next page"),
                                    theme::Icon::Right,
                                    p,
                                )
                            })
                            .inner
                            .clicked()
                        {
                            self.step_reader(true);
                        }
                        ui.add_space(8.0);
                        if theme::icon_button(
                            ui,
                            self.tr("Alejar", "Zoom out"),
                            theme::Icon::Minus,
                            p,
                        )
                        .clicked()
                        {
                            self.set_reader_zoom(self.reader_scale / 1.2);
                        }
                        ui.label(
                            RichText::new(self.reader_zoom_label())
                                .size(13.0)
                                .color(p.secondary),
                        );
                        if theme::icon_button(
                            ui,
                            self.tr("Acercar", "Zoom in"),
                            theme::Icon::Plus,
                            p,
                        )
                        .clicked()
                        {
                            self.set_reader_zoom(self.reader_scale * 1.2);
                        }
                        if theme::icon_button(
                            ui,
                            self.tr("Ajustar página", "Fit page"),
                            theme::Icon::Fit,
                            p,
                        )
                        .clicked()
                        {
                            self.set_reader_fit(ZoomMode::FitPage);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if theme::control(
                                ui,
                                self.tr("Navegador", "Navigator"),
                                Some(theme::Icon::Panel),
                                self.panel,
                                false,
                                p,
                            )
                            .clicked()
                            {
                                self.toggle_reader_panel();
                            }
                            let response = theme::icon_button(
                                ui,
                                self.tr("Opciones de lectura", "Reading options"),
                                theme::Icon::Settings,
                                p,
                            );
                            self.reader_options_popup(&response)
                                .show(|ui| self.reader_options_menu(ui, p));
                        });
                    });
                });
        }
        ui.add_space(16.0);
        self.reader_panel(ui, &document, &info, p);
        if self.panel && ui.ctx().content_rect().width() < 620.0 {
            return;
        }
        self.reader_viewport(ui, &document, &info, p);
    }
}
impl App for YamantakaApp {
    fn raw_input_hook(&mut self, _: &egui::Context, input: &mut egui::RawInput) {
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
        if let Some(presenter) = &mut self.presenter
            && let Err(error) = presenter.begin_frame()
        {
            self.error = Some(error.to_string());
        }
        while let Ok(paths) = self.activation.try_recv() {
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            if !paths.is_empty() {
                self.import(paths);
            }
        }
        while let Ok(update) = self.runtime.updates.try_recv() {
            match update {
                Update::Library(documents, count, cancel) => {
                    if cancel.is_cancelled() {
                        continue;
                    }
                    self.documents = documents;
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
                    if cancel.is_cancelled() {
                        self.open_confirmation = Some((document.id.clone(), cancel));
                        continue;
                    }
                    self.page = document
                        .last_page_index
                        .min(info.pages.len().saturating_sub(1) as u32);
                    let (target_page, reveal_reader) = self.open_intent.finish_open(&document.id);
                    if let Some(page) = target_page {
                        self.page = page.min(info.pages.len().saturating_sub(1) as u32);
                    }
                    let count = u32::try_from(info.pages.len()).unwrap_or(0);
                    match ReaderNavigation::new(count, self.page) {
                        Ok(navigation) => self.navigation = Some(navigation),
                        Err(error) => {
                            self.error = Some(error.to_string());
                            self.loading = false;
                            cancel.cancel();
                            self.open_confirmation = Some((document.id.clone(), cancel));
                            continue;
                        }
                    }
                    self.open_confirmation = Some((document.id.clone(), cancel));
                    // Opening acknowledgement and subsequent rendering have
                    // independent cancellation lifetimes. Preference application
                    // must not cancel an already accepted ownership handoff.
                    self.cancel = Cancellation::default();
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
                Update::Page(page, cancel, _, _) => self.accept_rendered_page(ctx, &page, &cancel),
                Update::Text(id, page, text, cancel) => {
                    if !cancel.is_cancelled()
                        && page == self.page
                        && self.active.as_ref().is_some_and(|(doc, _)| doc.id == id)
                    {
                        self.panel_text = text;
                    }
                }
                Update::Imported(_, cancel) => {
                    if cancel.is_cancelled() {
                        continue;
                    }
                    self.loading = false;
                    self.refresh();
                }
                Update::OpenFailed(document, error, cancel) => {
                    if cancel.is_cancelled() {
                        continue;
                    }
                    self.loading = false;
                    self.reader_preferences_loading = false;
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
                    if let Some(page) = self.open_intent.accept_lookup(&id, &cancel) {
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
                            } else {
                                self.open(document);
                                if let Some(page) = page {
                                    self.open_intent.set_page(id, page);
                                }
                            }
                        } else {
                            self.error = Some(self.tr("El documento ya no está disponible. Localízalo desde la biblioteca.", "The document is unavailable. Locate it from the library.").into());
                        }
                    }
                }
                Update::ReaderPreferences(id, preferences, cancel) => {
                    if cancel.is_cancelled()
                        || !self.active.as_ref().is_some_and(|(doc, _)| doc.id == id)
                    {
                        continue;
                    }
                    self.load_reader_preferences(
                        preferences.unwrap_or_else(|| ReaderPreferences::new(id)),
                    );
                }
            }
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
        for shortcut in shortcuts::read(
            &ctx,
            self.password.is_some() || self.links.confirmation_open(),
            matches!(self.surface, Surface::Reader),
        ) {
            match shortcut {
                shortcuts::Shortcut::GlobalSearch => self.show_search(),
                shortcuts::Shortcut::DocumentFind => self.show_find(),
                shortcuts::Shortcut::PreviousMatch => self.step_find(false),
                shortcuts::Shortcut::NextMatch => self.step_find(true),
                shortcuts::Shortcut::PreviousPage => self.step_reader(false),
                shortcuts::Shortcut::NextPage => self.step_reader(true),
                shortcuts::Shortcut::Fullscreen => {
                    let full = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!full));
                }
            }
        }
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
                if self.loading {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(self.tr("Procesando documentos…", "Working on your documents…"));
                        if ui.button(self.tr("Cancelar", "Cancel")).clicked() {
                            self.cancel.cancel();
                            self.loading = false;
                            self.refresh();
                        }
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
                if matches!(self.surface, Surface::Library) && self.inspector {
                    egui::Panel::bottom("document-details")
                        .default_size(148.0)
                        .size_range(80.0..=ui.ctx().content_rect().height().max(160.0) * 0.5)
                        .show(ui, |ui| self.details(ui));
                }
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(p.canvas))
                    .show(ui, |ui| match self.surface {
                        Surface::Library => self.library(ui),
                        Surface::Reader => self.reader(ui),
                        Surface::Settings => self.settings(ui),
                        Surface::Search => self.global_search(ui),
                    });
            });
        });
        self.password_dialog(&ctx);
        self.external_link_dialog(&ctx);
        if let Some(path) = &self.capture {
            if !self.capture_requested {
                let ready = !self.documents.is_empty()
                    && ((!self.panel && self.reader_panel_tab != outline::ReaderPanelTab::Find)
                        || match self.reader_panel_tab {
                            outline::ReaderPanelTab::Bookmarks => self.bookmarks.ready(),
                            outline::ReaderPanelTab::Outline => self.outline.ready(),
                            outline::ReaderPanelTab::Thumbnails => self.thumbnails.ready(),
                            outline::ReaderPanelTab::Find => self.find.ready(),
                            outline::ReaderPanelTab::Text => true,
                        })
                    && (self.link_preview == links::LinkPreview::Off
                        || self.links.ready()
                            && matches!(
                                self.link_preview,
                                links::LinkPreview::Focus | links::LinkPreview::Complete
                            ))
                    && self.reader_destination.is_none()
                    && self.outline_preview_target.is_none()
                    && self.thumbnail_preview_page.is_none()
                    && !self.loading
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
                    let record=serde_json::json!({"synthetic":self.demo,"viewport":rect(ctx.content_rect()),"pixelsPerPoint":ctx.pixels_per_point(),"controls":controls,
                        "pagePresenter":if self.presenter.is_some(){"native-wgpu"}else{"egui-glow"},
                        "visibleThumbnailSlots":self.thumbnails.resident_count(),
                        "documentFind":self.find.capture_summary(),
                        "documentLinks":self.links.capture_summary(),
                        "currentReaderPage":self.page,
                        "readerZoom":self.zoom,
                        "linkKeyboardActivated":self.link_preview == links::LinkPreview::Complete,
                        "trackedMemory":{"total":usage.total,"pages":usage.pages,"tiles":usage.tiles,"text":usage.text,"other":usage.other,"cpu":usage.cpu,"gpu":usage.gpu,"transient":usage.transient},
                        "memoryScope":"Tracked PDF buffers and native wgpu page payload/staging only; live text, Glow uploads, UI atlas and opaque driver allocations are not covered."});
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
        self.links.clear();
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn large_zoom_preview_respects_both_gpu_and_byte_limits() {
        for (size, dpr, limit) in [
            (egui::vec2(48000.0, 72000.0), 2.0, 16384),
            (egui::vec2(600.0, 800.0), 1.25, 1024),
            (egui::vec2(100000.0, 1.0), 2.0, 4096),
        ] {
            let dimensions = preview_dimensions(size, dpr, limit, 72 * 1024 * 1024);
            assert!(dimensions.iter().all(|value| *value > 0 && *value <= limit));
            assert!(dimensions[0] as u64 * dimensions[1] as u64 * 4 <= 72 * 1024 * 1024);
        }
    }
}
