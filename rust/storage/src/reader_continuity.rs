//! Independent Rust continuity; never rewrites Flutter's document-level history.
use crate::{database::storage, preferences::atomic_json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};
use yamantaka_domain::{
    Failure, Result,
    reader_continuity::ReaderContinuity,
    reader_preferences::{
        CoverPolicy, InitialSide, PageRotation, ReaderMode, ReaderPreferences, ReaderTheme,
        ZoomMode,
    },
    reader_tabs::{DocumentTabs, QueueEntry, ReaderTab},
    reading_sessions::restoration::SessionPanel,
};

/// Oversized input is preserved and reported, never partially restored.
pub const MAX_CONTINUITY_BYTES: usize = 4 * 1024 * 1024;

struct OutputBudget(usize);
impl Write for OutputBudget {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.0 {
            return Err(std::io::Error::other(
                "Reader continuity exceeds the four MiB output limit; original file is preserved",
            ));
        }
        self.0 -= bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    version: u32,
    #[serde(default)]
    workspace: Option<yamantaka_domain::workspaces::WorkspaceRevision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    comparison: Option<yamantaka_domain::workspaces::WorkspaceComparison>,
    active: Option<String>,
    tabs: Vec<StoredTab>,
    closed: Vec<StoredTab>,
    #[serde(flatten)]
    unknown: BTreeMap<String, Value>,
}
#[derive(Serialize, Deserialize)]
struct StoredTab {
    id: String,
    document: String,
    page: u32,
    pinned: bool,
    panel: Option<String>,
    preferences: StoredPreferences,
    #[serde(default)]
    queue: QueueEntry,
    #[serde(flatten)]
    unknown: BTreeMap<String, Value>,
}
#[derive(Serialize, Deserialize)]
struct StoredPreferences {
    mode: String,
    theme: String,
    zoom_mode: String,
    zoom: f64,
    layout: String,
    cover: String,
    side: String,
    rotation: i64,
    horizontal_margin: f64,
    vertical_margin: f64,
    auto_hide: bool,
    scroll: f64,
    updated_at: Option<String>,
    #[serde(flatten)]
    unknown: BTreeMap<String, Value>,
}
impl Snapshot {
    fn continuity(&self) -> Result<ReaderContinuity> {
        ReaderContinuity::new(self.model()?, self.comparison.clone())
    }
    fn model(&self) -> Result<DocumentTabs> {
        if self.version != 1 && self.version != 2 {
            return Err(Failure::Invalid(
                "Unsupported reader continuity version; original file is preserved".into(),
            ));
        }
        let mut model = DocumentTabs::default();
        model.restore_continuity(
            self.tabs
                .iter()
                .map(StoredTab::tab)
                .collect::<Result<_>>()?,
            self.active.as_deref(),
            self.closed
                .iter()
                .map(StoredTab::tab)
                .collect::<Result<_>>()?,
        )?;
        model.set_workspace_reference(self.workspace.clone())?;
        Ok(model)
    }
}
impl StoredTab {
    fn tab(&self) -> Result<ReaderTab> {
        let stored = &self.preferences;
        let mut p = ReaderPreferences::new(&self.document);
        p.view_mode = ReaderMode::from_persisted(&stored.mode)?;
        p.reading_theme = ReaderTheme::from_persisted(&stored.theme)?;
        p.zoom_mode = ZoomMode::from_persisted(&stored.zoom_mode)?;
        p.zoom_level = stored.zoom;
        p.load_layout_name(&stored.layout)?;
        p.cover_policy = CoverPolicy::from_persisted(&stored.cover)?;
        p.initial_side = InitialSide::from_persisted(&stored.side)?;
        p.page_rotation = PageRotation::from_degrees(stored.rotation)?;
        p.horizontal_margin = stored.horizontal_margin;
        p.vertical_margin = stored.vertical_margin;
        p.controls_auto_hide = stored.auto_hide;
        p.scroll_offset = stored.scroll;
        p.updated_at = stored.updated_at.clone();
        let panel = match self.panel.as_deref() {
            None => None,
            Some("outline") => Some(SessionPanel::Outline),
            Some("thumbnails") => Some(SessionPanel::Thumbnails),
            Some("search") => Some(SessionPanel::Search),
            Some("bookmarks") => Some(SessionPanel::Bookmarks),
            Some("annotations") => Some(SessionPanel::Annotations),
            Some("text") => Some(SessionPanel::Text),
            Some("positions") => Some(SessionPanel::Positions),
            Some(_) => {
                return Err(Failure::Invalid(
                    "Unknown continuity panel; original file is preserved".into(),
                ));
            }
        };
        ReaderTab::new(self.id.clone(), self.document.clone(), self.page, p, panel)?
            .with_pinned(self.pinned)
            .with_queue(self.queue.clone())
    }
    fn capture(tab: &ReaderTab, previous: Option<&Self>) -> Self {
        let p = tab.preferences();
        Self {
            id: tab.id().into(),
            queue: tab.queue().clone(),
            document: tab.document().into(),
            page: tab.page(),
            pinned: tab.pinned(),
            panel: tab.panel().map(|panel| {
                match panel {
                    SessionPanel::Outline => "outline",
                    SessionPanel::Thumbnails => "thumbnails",
                    SessionPanel::Search => "search",
                    SessionPanel::Bookmarks => "bookmarks",
                    SessionPanel::Annotations => "annotations",
                    SessionPanel::Text => "text",
                    SessionPanel::Positions => "positions",
                }
                .into()
            }),
            preferences: StoredPreferences {
                mode: p.view_mode.persisted_name().into(),
                theme: p.reading_theme.persisted_name().into(),
                zoom_mode: p.zoom_mode.persisted_name().into(),
                zoom: p.zoom_level,
                layout: p.persisted_layout_name().into(),
                cover: p.cover_policy.persisted_name().into(),
                side: p.initial_side.persisted_name().into(),
                rotation: p.page_rotation.degrees().into(),
                horizontal_margin: p.horizontal_margin,
                vertical_margin: p.vertical_margin,
                auto_hide: p.controls_auto_hide,
                scroll: p.scroll_offset,
                updated_at: p.updated_at.clone(),
                unknown: previous
                    .map(|old| old.preferences.unknown.clone())
                    .unwrap_or_default(),
            },
            unknown: previous.map(|old| old.unknown.clone()).unwrap_or_default(),
        }
    }
}
fn read(path: &Path) -> Result<Option<Snapshot>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(storage(error)),
    };
    let mut bytes = Vec::new();
    file.take((MAX_CONTINUITY_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(storage)?;
    if bytes.len() > MAX_CONTINUITY_BYTES {
        return Err(Failure::Invalid(
            "Reader continuity exceeds the four MiB input limit; original file is preserved".into(),
        ));
    }
    let snapshot: Snapshot = serde_json::from_slice(&bytes).map_err(storage)?;
    snapshot.continuity()?;
    Ok(Some(snapshot))
}
pub fn load(path: &Path) -> Result<Option<ReaderContinuity>> {
    read(path)?
        .map(|snapshot| snapshot.continuity())
        .transpose()
}
pub(crate) fn encode_workspace(model: &DocumentTabs) -> Result<Value> {
    let snapshot = Snapshot {
        version: 1,
        workspace: None,
        comparison: None,
        active: model.active().map(|tab| tab.id().into()),
        tabs: model
            .tabs()
            .iter()
            .map(|tab| StoredTab::capture(tab, None))
            .collect(),
        closed: Vec::new(),
        unknown: BTreeMap::new(),
    };
    snapshot.model()?;
    serde_json::to_writer(OutputBudget(MAX_CONTINUITY_BYTES), &snapshot).map_err(storage)?;
    serde_json::to_value(snapshot).map_err(storage)
}
pub(crate) fn decode_workspace(value: Value) -> Result<DocumentTabs> {
    let snapshot: Snapshot = serde_json::from_value(value).map_err(storage)?;
    if snapshot.version != 1 || snapshot.comparison.is_some() {
        return Err(Failure::Invalid(
            "Unsupported workspace reading context; stored data is preserved".into(),
        ));
    }
    snapshot.model()
}
pub fn save(path: &Path, continuity: &ReaderContinuity) -> Result<()> {
    let model = continuity.tabs();
    let previous = read(path)?;
    let capture = |tab: &ReaderTab| {
        let old = previous.as_ref().and_then(|s| {
            s.tabs
                .iter()
                .chain(&s.closed)
                .find(|old| old.id == tab.id() && old.document == tab.document())
        });
        StoredTab::capture(tab, old)
    };
    let snapshot = Snapshot {
        version: if continuity.comparison().is_some()
            || previous.as_ref().is_some_and(|old| old.version == 2)
        {
            2
        } else {
            1
        },
        workspace: model.workspace_reference().cloned(),
        comparison: continuity.comparison().cloned(),
        active: model.active().map(|tab| tab.id().into()),
        tabs: model.tabs().iter().map(&capture).collect(),
        closed: model.closed().iter().map(capture).collect(),
        unknown: previous.map(|old| old.unknown).unwrap_or_default(),
    };
    snapshot.continuity()?;
    serde_json::to_writer_pretty(OutputBudget(MAX_CONTINUITY_BYTES - 1), &snapshot)
        .map_err(storage)?;
    atomic_json(path, &snapshot)
}
