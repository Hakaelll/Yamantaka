//! Canonical PDF navigation independent of viewport pixels, UI and PDFium.
use crate::{Failure, PageLayout, ReadingDirection, Result};
use std::{collections::VecDeque, num::NonZeroU32, ops::Range};

/// A PDF page index and a fraction of its height, unaffected by zoom or layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReaderPosition {
    page: u32,
    normalized_offset: f64,
}

impl ReaderPosition {
    pub fn new(page: u32, normalized_offset: f64) -> Result<Self> {
        if !normalized_offset.is_finite() || !(0.0..=1.0).contains(&normalized_offset) {
            return Err(Failure::Invalid(
                "Page offset must be finite and within 0..=1".into(),
            ));
        }
        Ok(Self {
            page,
            normalized_offset,
        })
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
        self.navigate_to(ReaderPosition::page_start(page))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_small_spreads_partition_pages_and_reverse_only_physical_slots() {
        for count in 0..=7 {
            for cover_alone in [false, true] {
                for pairing_offset in [PairingOffset::PagesZeroOne, PairingOffset::FirstPageAlone] {
                    let config = SpreadConfig {
                        cover_alone,
                        pairing_offset,
                    };
                    if count == 0 {
                        for direction in [ReadingDirection::Ltr, ReadingDirection::Rtl] {
                            assert!(Spread::for_page(count, 0, config, direction).is_none());
                        }
                        assert!(ReaderNavigation::new(count, 0).is_err());
                        continue;
                    }
                    let mut seen = vec![];
                    for page in 0..count {
                        let ltr =
                            Spread::for_page(count, page, config, ReadingDirection::Ltr).unwrap();
                        let rtl =
                            Spread::for_page(count, page, config, ReadingDirection::Rtl).unwrap();
                        assert_eq!((ltr.left, ltr.right), (rtl.right, rtl.left));
                        assert_eq!((ltr.first, ltr.last), (rtl.first, rtl.last));
                        assert!(ltr.first <= page && page <= ltr.last && ltr.last < count);
                        let expected_first = if config.first_alone() {
                            if page == 0 { 0 } else { 1 + (page - 1) / 2 * 2 }
                        } else {
                            page / 2 * 2
                        };
                        assert_eq!(ltr.first, expected_first);
                        if page == ltr.first {
                            seen.extend(ltr.first..=ltr.last);
                        }
                    }
                    assert_eq!(seen, (0..count).collect::<Vec<_>>());
                    for direction in [ReadingDirection::Ltr, ReadingDirection::Rtl] {
                        let mut reader = ReaderNavigation::new(count, 0).unwrap();
                        reader.set_layout(PageLayout::Spread);
                        reader.set_direction(direction);
                        reader.set_spread_config(config);
                        let mut starts = vec![0];
                        while reader.next_page() {
                            starts.push(reader.position.page());
                        }
                        assert_eq!(reader.reading_unit().end, count);
                        for expected in starts.iter().rev().skip(1) {
                            assert!(reader.previous());
                            assert_eq!(reader.position.page(), *expected);
                        }
                        assert!(!reader.previous());
                        assert_eq!(reader.position.page(), 0);
                        reader.last();
                        assert_eq!(reader.reading_unit().end, count);
                        reader.go_to_page(u32::MAX);
                        assert_eq!(reader.position.page(), count - 1);
                        assert!(!reader.next_page());
                    }
                }
            }
        }
    }

    #[test]
    fn layouts_and_geometry_changes_preserve_the_actual_page_anchor() {
        let mut reader = ReaderNavigation::new(7, 2).unwrap();
        let anchor = ReaderPosition::new(2, 0.625).unwrap();
        reader.update_anchor(anchor);
        for layout in [
            PageLayout::Single,
            PageLayout::Spread,
            PageLayout::Continuous,
        ] {
            reader.set_layout(layout);
            reader.set_direction(ReadingDirection::Rtl);
            reader.set_spread_config(SpreadConfig {
                cover_alone: false,
                pairing_offset: PairingOffset::FirstPageAlone,
            });
            assert_eq!(reader.position(), anchor);
        }
        let before = PageGeometry::new([[400.0, 600.0]; 7], 20.0).unwrap();
        let after = PageGeometry::new([[800.0, 1200.0]; 7], 40.0).unwrap();
        assert_eq!(
            after.offset_for(anchor).unwrap(),
            before.offset_for(anchor).unwrap() * 2.0
        );
        assert_eq!(
            after
                .position_at(after.offset_for(anchor).unwrap())
                .unwrap(),
            Some(anchor)
        );
        assert_eq!(reader.history_len(), 1);
    }

    #[test]
    fn single_and_continuous_step_every_page_and_share_canonical_history() {
        for count in 1..=7 {
            for layout in [PageLayout::Single, PageLayout::Continuous] {
                let mut reader = ReaderNavigation::new(count, 0).unwrap();
                reader.set_layout(layout);
                reader.set_direction(ReadingDirection::Rtl);
                for page in 0..count {
                    assert_eq!(reader.position.page(), page);
                    assert_eq!(reader.reading_unit(), page..page + 1);
                    assert_eq!(reader.next_page(), page + 1 < count);
                }
                for page in (0..count).rev() {
                    assert_eq!(reader.position.page(), page);
                    assert_eq!(reader.previous(), page > 0);
                }
            }
        }
    }

    #[test]
    fn history_is_bounded_coalesces_duplicates_and_discards_new_jump_branch() {
        assert!(ReaderNavigation::with_history_capacity(8, 0, 0).is_err());
        let mut reader = ReaderNavigation::with_history_capacity(8, 0, 3).unwrap();
        for page in 1..=4 {
            assert!(reader.go_to_page(page));
        }
        assert_eq!(reader.history_len(), 3);
        assert!(!reader.go_to_page(4));
        assert!(reader.back());
        assert!(reader.back());
        assert_eq!(reader.position.page(), 2);
        assert!(!reader.back());
        assert_eq!(reader.history_len(), 3);
        assert!(reader.forward());
        assert!(reader.go_to_page(7));
        assert!(!reader.can_forward());
        assert!(reader.back());
        assert_eq!(reader.position.page(), 3);
        assert!(reader.forward());
        assert_eq!(reader.position.page(), 7);
        let mut one = ReaderNavigation::with_history_capacity(3, 0, 1).unwrap();
        one.next_page();
        assert_eq!(one.history_len(), 1);
        assert!(!one.can_back());
        let mut scroll = ReaderNavigation::new(3, 0).unwrap();
        scroll.go_to_page(1);
        scroll.go_to_page(2);
        scroll.back();
        scroll.update_anchor(ReaderPosition::new(0, 0.0).unwrap());
        assert_eq!(scroll.history_len(), 2);
        assert!(!scroll.can_back());
        assert!(scroll.forward());
        assert_eq!(scroll.position.page(), 2);
    }

    #[test]
    fn marks_keep_fractional_canonical_positions_and_reject_invalid_pages() {
        let mut reader = ReaderNavigation::new(7, 0).unwrap();
        let a = ReaderPosition::new(2, 0.3).unwrap();
        reader.set_mark(MarkSlot::A, a).unwrap();
        reader.navigate_to(ReaderPosition::new(5, 0.7).unwrap());
        reader.capture_mark(MarkSlot::B);
        reader.set_layout(PageLayout::Spread);
        assert!(reader.jump_to_mark(MarkSlot::A));
        assert_eq!(reader.position(), a);
        assert!(reader.jump_to_mark(MarkSlot::B));
        assert_eq!(reader.position(), ReaderPosition::new(5, 0.7).unwrap());
        assert!(
            reader
                .set_mark(MarkSlot::A, ReaderPosition::new(7, 0.0).unwrap())
                .is_err()
        );
        assert_eq!(reader.mark(MarkSlot::A), Some(a));
        reader.clear_mark(MarkSlot::A);
        assert!(!reader.jump_to_mark(MarkSlot::A));
    }

    #[test]
    fn geometry_rejects_nonfinite_sizes_gaps_positions_and_viewports() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -1.0] {
            assert!(PageGeometry::new([[invalid, 800.0]], 20.0).is_err());
            assert!(PageGeometry::new([[600.0, invalid]], 20.0).is_err());
        }
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
            assert!(PageGeometry::new([[600.0, 800.0]], invalid).is_err());
        }
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 1.1] {
            assert!(ReaderPosition::new(0, invalid).is_err());
        }
        assert!(PageGeometry::new([[1.0, 1.0]; 2], f64::MAX).is_err());
        let geometry = PageGeometry::new([[600.0, 800.0]], 0.0).unwrap();
        assert!(geometry.visible(f64::NAN, 100.0).is_err());
        assert!(geometry.position_at(f64::INFINITY).is_err());
        assert!(
            geometry
                .offset_for(ReaderPosition::new(1, 0.0).unwrap())
                .is_err()
        );
    }

    #[test]
    fn mixed_five_thousand_page_geometry_matches_linear_intersection_oracle() {
        let geometry = PageGeometry::new(
            (0..5000).map(|page| {
                [
                    500.0 + (page % 3) as f32 * 30.0,
                    700.0 + (page % 5) as f32 * 45.0,
                ]
            }),
            20.0,
        )
        .unwrap();
        assert_eq!(geometry.page_count(), 5000);
        for page in (0..5000).step_by(73) {
            let top = geometry.offset(page).unwrap() + 17.0;
            let bottom = top + 1600.0;
            let actual = geometry.visible(top, bottom).unwrap();
            let expected: Vec<_> = (0..5000)
                .filter(|&index| geometry.ends[index] > top && geometry.starts[index] < bottom)
                .collect();
            assert_eq!(actual.collect::<Vec<_>>(), expected);
            let position = ReaderPosition::new(page as u32, 0.375).unwrap();
            assert_eq!(
                geometry
                    .position_at(geometry.offset_for(position).unwrap())
                    .unwrap(),
                Some(position)
            );
        }
        let total = geometry.total_height();
        assert!(total.is_finite());
        assert_eq!(geometry.visible(total, total + 1000.0).unwrap(), 5000..5000);
        assert_eq!(geometry.visible(-100.0, -1.0).unwrap(), 0..0);
        assert_eq!(geometry.visible(800.0, 700.0).unwrap(), 1..1);
        let small = PageGeometry::new([[600.0, 700.0], [600.0, 900.0]], 20.0).unwrap();
        assert_eq!(small.visible(700.0, 720.0).unwrap(), 1..1);
        assert_eq!(
            small.position_at(710.0).unwrap(),
            Some(ReaderPosition::new(1, 0.0).unwrap())
        );
        let empty = PageGeometry::new([], 20.0).unwrap();
        assert_eq!(empty.visible(0.0, 100.0).unwrap(), 0..0);
        assert_eq!(empty.position_at(0.0).unwrap(), None);
    }
}
