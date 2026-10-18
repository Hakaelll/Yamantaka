//! Canonical PDF navigation independent of viewport pixels, UI and PDFium.
use crate::{Failure, PageLayout, ReadingDirection, Result};
use std::{collections::VecDeque, num::NonZeroU32, ops::Range};

/// A point on the page before user rotation, independent of zoom and layout.
/// Its displayed x aligns with the viewport center; y aligns with the top edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReaderPosition {
    page: u32,
    normalized_offset: f64,
    normalized_horizontal: f64,
}

impl ReaderPosition {
    pub fn new(page: u32, normalized_offset: f64) -> Result<Self> {
        Self::at(page, [0.5, normalized_offset])
    }
    pub fn at(page: u32, point: [f64; 2]) -> Result<Self> {
        if point
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(Failure::Invalid(
                "Page coordinates must be finite and within 0..=1".into(),
            ));
        }
        Ok(Self {
            page,
            normalized_horizontal: point[0],
            normalized_offset: point[1],
        })
    }
    pub fn point(self) -> [f64; 2] {
        [self.normalized_horizontal, self.normalized_offset]
    }

    pub fn page(self) -> u32 {
        self.page
    }
    pub fn normalized_offset(self) -> f64 {
        self.normalized_offset
    }

    fn page_start(page: u32) -> Self {
        Self {
            page,
            normalized_offset: 0.0,
            normalized_horizontal: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderMode {
    Document,
    Book,
}

/// Domain vocabulary only; applying these environments belongs to presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingEnvironment {
    Light,
    Ivory,
    Sepia,
    Gray,
    Dark,
    Black,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PairingOffset {
    /// Spreads contain [0, 1], [2, 3], ... .
    #[default]
    PagesZeroOne,
    /// Page 0 stands alone, followed by [1, 2], [3, 4], ... .
    FirstPageAlone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpreadConfig {
    pub cover_alone: bool,
    pub pairing_offset: PairingOffset,
}

impl Default for SpreadConfig {
    fn default() -> Self {
        Self {
            cover_alone: true,
            pairing_offset: PairingOffset::PagesZeroOne,
        }
    }
}

impl SpreadConfig {
    pub fn from_preferences(preferences: &crate::reader_preferences::ReaderPreferences) -> Self {
        use crate::reader_preferences::{CoverPolicy, InitialSide};
        Self {
            cover_alone: preferences.cover_policy == CoverPolicy::FirstPageIsCover,
            pairing_offset: if preferences.initial_side == InitialSide::Right {
                PairingOffset::FirstPageAlone
            } else {
                PairingOffset::PagesZeroOne
            },
        }
    }
    fn first_alone(self) -> bool {
        self.cover_alone || self.pairing_offset == PairingOffset::FirstPageAlone
    }
}

/// Physical slots may be reversed, while logical bounds always increase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spread {
    pub left: Option<u32>,
    pub right: Option<u32>,
    first: u32,
    last: u32,
}

impl Spread {
    pub fn for_preferences(
        page_count: u32,
        page: u32,
        preferences: &crate::reader_preferences::ReaderPreferences,
        direction: ReadingDirection,
    ) -> Option<Self> {
        use crate::reader_preferences::{CoverPolicy, InitialSide};
        let mut spread = Self::for_page(
            page_count,
            page,
            SpreadConfig::from_preferences(preferences),
            direction,
        )?;
        // Flutter treats a non-cover initial right page as physically right in
        // both directions. A real cover follows the reading direction instead.
        if spread.first == 0
            && spread.last == 0
            && preferences.cover_policy == CoverPolicy::NoCover
            && preferences.initial_side == InitialSide::Right
        {
            spread.left = None;
            spread.right = Some(0);
        }
        Some(spread)
    }
    pub fn for_page(
        page_count: u32,
        page: u32,
        config: SpreadConfig,
        direction: ReadingDirection,
    ) -> Option<Self> {
        let last_page = page_count.checked_sub(1)?;
        let page = page.min(last_page);
        let standalone_first = config.first_alone();
        let first = if standalone_first && page == 0 {
            0
        } else if standalone_first {
            1 + (page - 1) / 2 * 2
        } else {
            page / 2 * 2
        };
        let last = if standalone_first && page == 0 {
            0
        } else {
            first.saturating_add(1).min(last_page)
        };
        let (left, right) = if standalone_first && page == 0 {
            (None, Some(0))
        } else {
            (Some(first), (last != first).then_some(last))
        };
        let (left, right) = match direction {
            ReadingDirection::Ltr => (left, right),
            ReadingDirection::Rtl => (right, left),
        };
        Some(Self {
            left,
            right,
            first,
            last,
        })
    }

    pub fn first(self) -> u32 {
        self.first
    }
    pub fn last(self) -> u32 {
        self.last
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkSlot {
    A,
    B,
}

/// Every destination (outline, search, links, marks and page controls) goes here.
/// An instance always represents a nonempty document; empty PDFs cannot open it.
#[derive(Debug, Clone)]
pub struct ReaderNavigation {
    page_count: NonZeroU32,
    position: ReaderPosition,
    layout: PageLayout,
    direction: ReadingDirection,
    spread_config: SpreadConfig,
    first_page_physical_right: bool,
    rotation: crate::reader_preferences::PageRotation,
    history: VecDeque<ReaderPosition>,
    history_cursor: usize,
    history_capacity: usize,
    marks: [Option<ReaderPosition>; 2],
}

impl ReaderNavigation {
    pub const DEFAULT_HISTORY_CAPACITY: usize = 128;

    pub fn new(page_count: u32, initial_page: u32) -> Result<Self> {
        Self::with_history_capacity(page_count, initial_page, Self::DEFAULT_HISTORY_CAPACITY)
    }

    pub fn with_history_capacity(
        page_count: u32,
        initial_page: u32,
        capacity: usize,
    ) -> Result<Self> {
        let page_count = NonZeroU32::new(page_count)
            .ok_or_else(|| Failure::Invalid("Cannot navigate an empty document".into()))?;
        if capacity == 0 {
            return Err(Failure::Invalid(
                "Navigation history capacity must be positive".into(),
            ));
        }
        let position = ReaderPosition::page_start(initial_page.min(page_count.get() - 1));
        Ok(Self {
            page_count,
            position,
            layout: PageLayout::Single,
            direction: ReadingDirection::Ltr,
            spread_config: SpreadConfig::default(),
            first_page_physical_right: false,
            rotation: crate::reader_preferences::PageRotation::Deg0,
            history: VecDeque::from([position]),
            history_cursor: 0,
            history_capacity: capacity,
            marks: [None, None],
        })
    }

    pub fn page_count(&self) -> u32 {
        self.page_count.get()
    }
    pub fn position(&self) -> ReaderPosition {
        self.position
    }
    pub fn layout(&self) -> PageLayout {
        self.layout
    }
    pub fn direction(&self) -> ReadingDirection {
        self.direction
    }
    pub fn spread_config(&self) -> SpreadConfig {
        self.spread_config
    }

    /// Presentation recalculates its viewport from the unchanged canonical anchor.
    pub fn set_layout(&mut self, layout: PageLayout) {
        self.layout = layout;
    }
    pub fn set_direction(&mut self, direction: ReadingDirection) {
        self.direction = direction;
    }
    pub fn set_spread_config(&mut self, config: SpreadConfig) {
        self.spread_config = config;
        self.first_page_physical_right = false;
    }
    /// Apply presentation preferences without changing the canonical anchor,
    /// history or marks. No persistent preference is silently normalized.
    pub fn apply_preferences(
        &mut self,
        preferences: &crate::reader_preferences::ReaderPreferences,
        direction: ReadingDirection,
    ) {
        use crate::reader_preferences::{CoverPolicy, InitialSide};
        self.layout = preferences.effective_layout();
        self.rotation = preferences.page_rotation;
        self.direction = direction;
        self.spread_config = SpreadConfig::from_preferences(preferences);
        self.first_page_physical_right = preferences.cover_policy == CoverPolicy::NoCover
            && preferences.initial_side == InitialSide::Right;
    }

    pub fn spread(&self) -> Spread {
        // The type guarantees a positive count, so this cannot be None.
        let mut spread = Spread::for_page(
            self.page_count(),
            self.position.page,
            self.spread_config,
            self.direction,
        )
        .expect("ReaderNavigation always has pages");
        if self.first_page_physical_right && spread.first == 0 && spread.last == 0 {
            spread.left = None;
            spread.right = Some(0);
        }
        spread
    }

    pub fn reading_unit(&self) -> Range<u32> {
        if self.layout == PageLayout::Spread {
            let spread = self.spread();
            spread.first..spread.last + 1
        } else {
            self.position.page..self.position.page + 1
        }
    }

    /// Clamp page boundaries while retaining the validated normalized offset.
    pub fn navigate_to(&mut self, position: ReaderPosition) -> bool {
        let position = ReaderPosition {
            page: position.page.min(self.page_count() - 1),
            ..position
        };
        if position == self.position {
            return false;
        }
        self.history.truncate(self.history_cursor + 1);
        self.history.push_back(position);
        if self.history.len() > self.history_capacity {
            self.history.pop_front();
        }
        self.history_cursor = self.history.len() - 1;
        self.position = position;
        true
    }

    pub fn go_to_page(&mut self, page: u32) -> bool {
        self.navigate_to(
            ReaderPosition::at(page, self.rotation.unrotate_point([0.5, 0.0]))
                .expect("Quarter-turn page starts are finite normalized points"),
        )
    }

    /// Scroll/pan feedback replaces the current anchor without adding history.
    /// Call navigate_to for deliberate jumps, and this method for viewport motion.
    pub fn update_anchor(&mut self, position: ReaderPosition) -> bool {
        let position = ReaderPosition {
            page: position.page.min(self.page_count() - 1),
            ..position
        };
        if position == self.position {
            return false;
        }
        self.position = position;
        self.history[self.history_cursor] = position;
        if self.history_cursor > 0 && self.history[self.history_cursor - 1] == position {
            self.history.remove(self.history_cursor);
            self.history_cursor -= 1;
        }
        if self.history.get(self.history_cursor + 1) == Some(&position) {
            self.history.remove(self.history_cursor + 1);
        }
        true
    }

    pub fn next_page(&mut self) -> bool {
        let next = self.reading_unit().end;
        if next >= self.page_count() {
            false
        } else {
            self.go_to_page(next)
        }
    }

    pub fn previous(&mut self) -> bool {
        let first = self.reading_unit().start;
        let Some(previous) = first.checked_sub(1) else {
            return false;
        };
        let target = if self.layout == PageLayout::Spread {
            Spread::for_page(
                self.page_count(),
                previous,
                self.spread_config,
                self.direction,
            )
            .expect("Previous page belongs to this document")
            .first
        } else {
            previous
        };
        self.go_to_page(target)
    }

    pub fn first(&mut self) -> bool {
        self.go_to_page(0)
    }
    pub fn last(&mut self) -> bool {
        let last = self.page_count() - 1;
        let target = if self.layout == PageLayout::Spread {
            Spread::for_page(self.page_count(), last, self.spread_config, self.direction)
                .expect("Last page belongs to this document")
                .first
        } else {
            last
        };
        self.go_to_page(target)
    }

    pub fn can_back(&self) -> bool {
        self.history_cursor > 0
    }
    pub fn can_forward(&self) -> bool {
        self.history_cursor + 1 < self.history.len()
    }
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    pub fn back(&mut self) -> bool {
        if !self.can_back() {
            return false;
        }
        self.history_cursor -= 1;
        self.position = self.history[self.history_cursor];
        true
    }

    pub fn forward(&mut self) -> bool {
        if !self.can_forward() {
            return false;
        }
        self.history_cursor += 1;
        self.position = self.history[self.history_cursor];
        true
    }

    pub fn set_mark(&mut self, slot: MarkSlot, position: ReaderPosition) -> Result<()> {
        if position.page >= self.page_count() {
            return Err(Failure::Invalid("Mark page is outside the document".into()));
        }
        self.marks[slot.index()] = Some(position);
        Ok(())
    }
    pub fn capture_mark(&mut self, slot: MarkSlot) {
        self.marks[slot.index()] = Some(self.position);
    }
    pub fn mark(&self, slot: MarkSlot) -> Option<ReaderPosition> {
        self.marks[slot.index()]
    }
    pub fn clear_mark(&mut self, slot: MarkSlot) {
        self.marks[slot.index()] = None;
    }
    pub fn jump_to_mark(&mut self, slot: MarkSlot) -> bool {
        self.mark(slot)
            .is_some_and(|position| self.navigate_to(position))
    }
}

impl MarkSlot {
    fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }
}

/// Validated page sizes and strictly increasing f64 offsets for virtual scrolling.
/// The gap is excluded from page height and normalized anchor calculations.
#[derive(Debug, Clone)]
pub struct PageGeometry {
    sizes: Vec<[f64; 2]>,
    starts: Vec<f64>,
    ends: Vec<f64>,
    total_height: f64,
}

impl PageGeometry {
    pub fn new(sizes: impl IntoIterator<Item = [f32; 2]>, gap: f64) -> Result<Self> {
        if !gap.is_finite() || gap < 0.0 {
            return Err(Failure::Invalid(
                "Page gap must be finite and nonnegative".into(),
            ));
        }
        let mut geometry = Self {
            sizes: vec![],
            starts: vec![],
            ends: vec![],
            total_height: 0.0,
        };
        for [width, height] in sizes {
            if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
                return Err(Failure::Invalid(
                    "Page dimensions must be finite and positive".into(),
                ));
            }
            let start = if geometry.sizes.is_empty() {
                0.0
            } else {
                geometry.total_height + gap
            };
            let end = start + f64::from(height);
            if !start.is_finite()
                || !end.is_finite()
                || end <= start
                || (!geometry.sizes.is_empty() && start < geometry.total_height)
            {
                return Err(Failure::Invalid(
                    "Page geometry exceeds finite coordinate precision".into(),
                ));
            }
            if geometry.sizes.len() >= u32::MAX as usize {
                return Err(Failure::Invalid(
                    "Page count exceeds navigation capacity".into(),
                ));
            }
            geometry.sizes.push([f64::from(width), f64::from(height)]);
            geometry.starts.push(start);
            geometry.ends.push(end);
            geometry.total_height = end;
        }
        Ok(geometry)
    }

    pub fn page_count(&self) -> usize {
        self.sizes.len()
    }
    pub fn size(&self, page: usize) -> Option<[f64; 2]> {
        self.sizes.get(page).copied()
    }
    pub fn offset(&self, page: usize) -> Option<f64> {
        self.starts.get(page).copied()
    }
    pub fn total_height(&self) -> f64 {
        self.total_height
    }

    /// Half-open viewport/page intersection, excluding blank gaps, in O(log n).
    pub fn visible(&self, top: f64, bottom: f64) -> Result<Range<usize>> {
        if !top.is_finite() || !bottom.is_finite() {
            return Err(Failure::Invalid(
                "Viewport coordinates must be finite".into(),
            ));
        }
        let start = self.ends.partition_point(|end| *end <= top);
        if bottom <= top {
            return Ok(start..start);
        }
        let end = self.starts.partition_point(|start| *start < bottom);
        Ok(start..end.max(start))
    }

    /// Gaps map to the start of the following page. Out-of-document offsets clamp.
    pub fn position_at(&self, offset: f64) -> Result<Option<ReaderPosition>> {
        if !offset.is_finite() {
            return Err(Failure::Invalid("Scroll coordinate must be finite".into()));
        }
        if self.sizes.is_empty() {
            return Ok(None);
        }
        let page = self
            .ends
            .partition_point(|end| *end <= offset)
            .min(self.sizes.len() - 1);
        let normalized_offset =
            ((offset - self.starts[page]) / self.sizes[page][1]).clamp(0.0, 1.0);
        Ok(Some(ReaderPosition::new(page as u32, normalized_offset)?))
    }

    /// Recreate pixels after zoom, resize, margins or layout geometry changes.
    pub fn offset_for(&self, position: ReaderPosition) -> Result<f64> {
        let page = position.page as usize;
        let size = self
            .sizes
            .get(page)
            .ok_or_else(|| Failure::Invalid("Anchor page is outside geometry".into()))?;
        Ok(self.starts[page] + size[1] * position.normalized_offset)
    }
}
