//! UI and FFI independent contracts and persisted domain semantics.
pub mod annotations;
pub mod automatic_cover;
pub mod bookmarks;
pub mod cover_cache;
pub mod cover_policy;
pub mod cover_revision;
pub mod cover_selection;
pub mod cover_snapshot;
pub mod external_uri;
pub mod front_matter;
pub mod import_metadata;
pub mod memory;
pub mod outline;

use serde::{Deserialize, Serialize};
pub mod password;
pub mod pdf_destination;
pub mod pdf_find;
pub mod pdf_geometry;
pub mod pdf_links;
pub mod pdf_pages;
pub mod pdf_text;
pub mod pdf_ticket;
pub use password::PdfPassword;
pub mod reader;
pub mod reader_destination;
pub mod reader_layout;
pub mod reader_preferences;
pub mod reader_tabs;
pub mod reading_activity;
pub mod reading_sessions;
pub mod search;
pub mod source_revision;
pub mod text_selection;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub const SCHEMA_VERSION: u32 = 11;
pub const MAX_TABS: usize = 12;
pub const MAX_HOT_DOCUMENTS: usize = 2;
pub const MAX_LIBRARY_PAGE: usize = 80;
pub const MAX_RENDER_DIMENSION: u32 = 16_384;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Failure {
    #[error("PDF text exceeds the requested work limit")]
    TextWorkLimit,
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    #[error("Storage operation failed: {0}")]
    Storage(String),
    #[error("Invalid data: {0}")]
    Invalid(String),
    #[error("The Rust library already contains data. Choose an empty destination.")]
    DestinationOccupied,
    #[error("Close Flutter before capturing its database and preferences.")]
    LegacyRunning,
    #[error("Operation cancelled")]
    Cancelled,
    #[error("Work queue is full; drain results or cancel pending work")]
    QueueFull,
    #[error("PDF operation failed: {0}")]
    Pdf(String),
    #[error("The PDF requires a password")]
    PasswordRequired,
    #[error(
        "The live render memory budget is full; release unused pages or retry at a smaller size"
    )]
    MemoryBudget,
}
pub type Result<T> = std::result::Result<T, Failure>;

#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    /// Identity of a request, including clones carried by its terminal event.
    pub fn same_request(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    pub fn check(&self) -> Result<()> {
        if self.is_cancelled() {
            Err(Failure::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub path: PathBuf,
    pub file_name: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: u32,
    pub last_page_index: u32,
    pub reading_progress: f64,
    pub favorite: bool,
    pub missing: bool,
    pub fingerprint: String,
    pub thumbnail_path: Option<PathBuf>,
}
impl Document {
    pub fn display_title(&self) -> &str {
        self.title
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&self.file_name)
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LibraryReadingState {
    #[default]
    All,
    InProgress,
    NotStarted,
    Completed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LibrarySort {
    #[default]
    LastOpened,
    Title,
    Author,
    Added,
    Progress,
    PageCount,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortDirection {
    /// Titles/authors ascend; dates, progress and page count descend.
    #[default]
    Default,
    Ascending,
    Descending,
}

#[derive(Debug, Clone)]
pub struct LibraryQuery {
    pub text: String,
    pub favorites_only: bool,
    pub missing_only: bool,
    pub include_missing: bool,
    pub reading_state: LibraryReadingState,
    pub sort: LibrarySort,
    pub sort_direction: SortDirection,
    /// Any selected effective author, normalized with trim/lowercase.
    pub authors: Vec<String>,
    /// Any selected persisted topic key (for example "religionMythology").
    pub topics: Vec<String>,
    /// Any selected organization tag ID; combined with other filters using AND.
    pub tag_ids: Vec<String>,
    pub collection_id: Option<String>,
    pub offset: usize,
    pub page_size: usize,
}
impl Default for LibraryQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            favorites_only: false,
            missing_only: false,
            include_missing: true,
            reading_state: LibraryReadingState::All,
            sort: LibrarySort::LastOpened,
            sort_direction: SortDirection::Default,
            authors: Vec::new(),
            topics: Vec::new(),
            tag_ids: Vec::new(),
            collection_id: None,
            offset: 0,
            page_size: 0,
        }
    }
}
pub trait LibraryRepository {
    fn cover_snapshot(&self, _id: &str) -> Result<cover_snapshot::CoverSnapshot> {
        Err(Failure::Unsupported("Cover source snapshot".into()))
    }
    fn cover_revision(&self, _id: &str) -> Result<Option<cover_revision::CoverRevision>> {
        Err(Failure::Unsupported("Cover decision revision".into()))
    }
    fn replace_automatic_cover(
        &mut self,
        _choice: &automatic_cover::AutomaticCover,
        _expected: Option<cover_revision::CoverRevision>,
    ) -> Result<bool> {
        Err(Failure::Unsupported("Restore automatic PDF cover".into()))
    }
    /// Returns false when a newer stored choice already owns the cover.
    fn save_automatic_cover(&mut self, _choice: &automatic_cover::AutomaticCover) -> Result<bool> {
        Err(Failure::Unsupported(
            "Automatic PDF cover persistence".into(),
        ))
    }
    fn save_manual_cover(&mut self, _choice: &cover_selection::ManualCover) -> Result<()> {
        Err(Failure::Unsupported("Manual PDF cover selection".into()))
    }
    fn cover_source_page(&self, _document_id: &str) -> Result<Option<u32>> {
        Ok(None)
    }
    fn reader_continuity(&self) -> Result<Option<reader_tabs::DocumentTabs>> {
        Ok(None)
    }
    fn save_reader_continuity(&mut self, _tabs: &reader_tabs::DocumentTabs) -> Result<()> {
        Err(Failure::Unsupported("Reader continuity persistence".into()))
    }
    fn query(&self, query: &LibraryQuery) -> Result<Vec<Document>>;
    fn count(&self, query: &LibraryQuery) -> Result<u64>;
    fn set_favorite(&mut self, id: &str, favorite: bool) -> Result<()>;
    fn save_position(&mut self, id: &str, page: u32) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<Document>>;
    fn get_reader_preferences(
        &self,
        _document_id: &str,
    ) -> Result<Option<reader_preferences::ReaderPreferences>> {
        Ok(None)
    }
    fn save_reader_preferences(
        &mut self,
        _document_id: &str,
        _preferences: &reader_preferences::ReaderPreferences,
    ) -> Result<()> {
        Err(Failure::Unsupported("Saving reader preferences".into()))
    }
    fn import(&mut self, document: &NewDocument) -> Result<String>;
}
pub struct NewDocument {
    pub path: PathBuf,
    pub info: PdfInfo,
    pub source: source_revision::SourceRevision,
    pub analysis: front_matter::ImportAnalysis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PageLayout {
    Single,
    Spread,
    Continuous,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReadingDirection {
    Ltr,
    Rtl,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Priority {
    Visible,
    Nearby,
    Prefetch,
    Thumbnail,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderKey {
    pub document_id: String,
    pub page: u32,
    /// Optional `[x, y, width, height]` tile in the final rotated page's
    /// top-left-origin physical pixel coordinates. The rectangle is half-open
    /// and must fit inside the full `physical_width` × `physical_height` canvas.
    pub region: Option<[u32; 4]>,
    /// Full page canvas dimensions, including the requested rotation. With a
    /// region, these control page scale rather than the allocated tile size.
    pub physical_width: u32,
    pub physical_height: u32,
    pub rotation: u16,
    pub generation: u64,
}
impl RenderKey {
    pub fn output_dimensions(&self) -> Result<[u32; 2]> {
        if let Some([x, y, width, height]) = self.region {
            if width == 0
                || height == 0
                || x.checked_add(width)
                    .is_none_or(|end| end > self.physical_width)
                || y.checked_add(height)
                    .is_none_or(|end| end > self.physical_height)
            {
                return Err(Failure::Invalid(
                    "Render region is outside the page canvas".into(),
                ));
            }
            Ok([width, height])
        } else {
            Ok([self.physical_width, self.physical_height])
        }
    }
    pub fn byte_size(&self, device_limit: u32, budget: usize) -> Result<usize> {
        if !self.rotation.is_multiple_of(90) {
            return Err(Failure::Invalid(
                "Page rotation must be a multiple of 90 degrees".into(),
            ));
        }
        let limit = device_limit.min(MAX_RENDER_DIMENSION);
        if self.physical_width == 0
            || self.physical_height == 0
            || self.physical_width > limit
            || self.physical_height > limit
        {
            return Err(Failure::Invalid(
                "Render dimensions exceed device limits".into(),
            ));
        }
        let [width, height] = self.output_dimensions()?;
        let bytes = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| Failure::Invalid("Render allocation overflow".into()))?;
        if bytes > budget {
            return Err(Failure::Invalid("Render exceeds memory budget".into()));
        }
        Ok(bytes)
    }
}
pub struct RenderedPage {
    pub key: RenderKey,
    pub width: u32,
    pub height: u32,
    pixels: memory::TrackedBytes,
}
impl RenderedPage {
    pub fn from_pixels(key: RenderKey, pixels: memory::TrackedBytes) -> Result<Self> {
        let bytes = key.byte_size(MAX_RENDER_DIMENSION, usize::MAX)?;
        if pixels.len() != bytes {
            return Err(Failure::Invalid(
                "Page buffer does not match its dimensions".into(),
            ));
        }
        let [width, height] = key.output_dimensions()?;
        Ok(Self {
            width,
            height,
            key,
            pixels,
        })
    }
    pub fn pixels(&self) -> &[u8] {
        self.pixels.as_slice()
    }
}
#[derive(Debug, Clone)]
pub struct PdfInfo {
    pub pages: Arc<pdf_pages::PdfPages>,
    pub title: Option<String>,
    pub author: Option<String>,
}
pub trait PdfEngine {
    fn open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<crate::PdfPassword>,
        cancel: Cancellation,
    ) -> Result<PdfInfo>;
    fn render(
        &self,
        key: RenderKey,
        priority: Priority,
        cancel: Cancellation,
    ) -> Result<Arc<RenderedPage>>;
    fn text(&self, id: String, page: u32, cancel: Cancellation) -> Result<String>;
    fn outline(&self, _id: String, _cancel: Cancellation) -> Result<Arc<outline::PdfOutline>> {
        Err(Failure::Unsupported("PDF outline".into()))
    }
    fn close(&self, id: String) -> Result<()>;
}

/// Prefix sums permit O(log n) continuous viewport lookup for mixed page sizes.
pub struct PageOffsets(Vec<f32>);
impl PageOffsets {
    pub fn new(heights: impl IntoIterator<Item = f32>, gap: f32) -> Self {
        let mut offsets = vec![0.0];
        for height in heights {
            offsets.push(offsets.last().copied().unwrap_or(0.0) + height + gap);
        }
        Self(offsets)
    }
    pub fn visible(&self, top: f32, bottom: f32) -> std::ops::Range<usize> {
        let count = self.0.len().saturating_sub(1);
        let start = self
            .0
            .partition_point(|y| *y <= top)
            .saturating_sub(1)
            .min(count);
        let end = self.0.partition_point(|y| *y < bottom).min(count);
        start..end.max(start)
    }
    pub fn offset(&self, page: usize) -> Option<f32> {
        self.0.get(page).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixed_page_lookup_is_bounded() {
        let offsets = PageOffsets::new(
            (0..5000).map(|n| if n % 2 == 0 { 700.0 } else { 900.0 }),
            20.0,
        );
        assert_eq!(offsets.visible(710.0, 1000.0), 0..2);
        assert_eq!(offsets.visible(4100000.0, 4200000.0), 5000..5000);
    }
    #[test]
    fn reject_oversize_before_allocating() {
        let mut key = RenderKey {
            document_id: "d".into(),
            page: 0,
            region: None,
            physical_width: 16384,
            physical_height: 16384,
            rotation: 0,
            generation: 1,
        };
        assert!(key.byte_size(16384, 72 * 1024 * 1024).is_err());
        key.physical_width = 512;
        key.physical_height = 512;
        assert_eq!(key.byte_size(4096, 72 * 1024 * 1024).unwrap(), 1048576);
        assert!(key.byte_size(256, 72 * 1024 * 1024).is_err());
    }
    #[test]
    fn tile_bounds_and_allocation_use_the_rotated_device_canvas() {
        let mut key = RenderKey {
            document_id: "d".into(),
            page: 0,
            region: Some([15984, 16084, 400, 300]),
            physical_width: 16384,
            physical_height: 16384,
            rotation: 270,
            generation: 1,
        };
        assert_eq!(key.output_dimensions().unwrap(), [400, 300]);
        assert_eq!(key.byte_size(16384, 480000).unwrap(), 480000);
        assert!(key.byte_size(16384, 479999).is_err());
        for region in [
            [0, 0, 0, 1],
            [0, 0, 1, 0],
            [16384, 0, 1, 1],
            [0, 16384, 1, 1],
            [u32::MAX, 0, 2, 1],
            [0, u32::MAX, 1, 2],
        ] {
            key.region = Some(region);
            assert!(key.byte_size(16384, usize::MAX).is_err(), "{region:?}");
        }
        key.region = Some([0, 0, 1, 1]);
        key.rotation = 45;
        assert!(key.byte_size(16384, usize::MAX).is_err());
    }
}
