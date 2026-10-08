//! Pure reader viewport geometry. Build once per geometry/preference change;
//! continuous frames locate visible pages in O(log n + visible pages).
use crate::reader::{PageGeometry, ReaderPosition, Spread};
use crate::reader_preferences::{PageRotation, ReaderPreferences, ZoomMode};
use crate::{Failure, PageLayout, ReadingDirection, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalRect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}
impl LogicalRect {
    pub fn right(self) -> f64 {
        self.left + self.width
    }
    pub fn bottom(self) -> f64 {
        self.top + self.height
    }
    pub fn intersects(self, other: Self) -> bool {
        self.left < other.right()
            && self.right() > other.left
            && self.top < other.bottom()
            && self.bottom() > other.top
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisiblePage {
    pub page: u32,
    /// Content coordinates in logical points. Subtract ReaderFrame::scroll
    /// before painting in the viewport; raster size adds physical DPI separately.
    pub rect: LogicalRect,
}

#[derive(Debug, Clone)]
pub struct ReaderFrame {
    pub zoom: f64,
    pub viewport: [f64; 2],
    pub content_size: [f64; 2],
    pub scroll: [f64; 2],
    pub pages: Vec<VisiblePage>,
    /// Geometry content origin before page-zero's first pixel.
    origin_y: f64,
}

#[derive(Debug, Clone)]
pub struct ReaderLayout {
    geometry: PageGeometry,
    preferences: ReaderPreferences,
    direction: ReadingDirection,
    widest_page: f64,
    tallest_page: f64,
    pair_gap: f64,
}

impl ReaderLayout {
    pub fn new(
        sizes: impl IntoIterator<Item = [f32; 2]>,
        preferences: &ReaderPreferences,
        direction: ReadingDirection,
        gap_points: f64,
    ) -> Result<Self> {
        preferences.validate()?;
        if !gap_points.is_finite() || gap_points < 0.0 {
            return Err(Failure::Invalid(
                "Reader page gap must be finite and nonnegative".into(),
            ));
        }
        let rotated = matches!(
            preferences.page_rotation,
            PageRotation::Deg90 | PageRotation::Deg270
        );
        let geometry = PageGeometry::new(
            sizes
                .into_iter()
                .map(|[w, h]| if rotated { [h, w] } else { [w, h] }),
            gap_points,
        )?;
        if geometry.page_count() == 0 {
            return Err(Failure::Invalid("Cannot lay out an empty PDF".into()));
        }
        // Precomputed once. Frame fit and page placement never fold all pages.
        let mut widest_page: f64 = 0.0;
        let mut tallest_page: f64 = 0.0;
        for page in 0..geometry.page_count() {
            let [width, height] = geometry
                .size(page)
                .expect("PageGeometry has validated sizes");
            widest_page = widest_page.max(width);
            tallest_page = tallest_page.max(height);
        }
        Ok(Self {
            geometry,
            preferences: preferences.clone(),
            direction,
            widest_page,
            tallest_page,
            pair_gap: gap_points.min(2.0),
        })
    }
    pub fn layout(&self) -> PageLayout {
        self.preferences.effective_layout()
    }
    pub fn page_count(&self) -> u32 {
        self.geometry.page_count() as u32
    }
    pub fn rotation(&self) -> PageRotation {
        self.preferences.page_rotation
    }
    pub fn page_size(&self, page: u32) -> Option<[f64; 2]> {
        self.geometry.size(page as usize)
    }
    pub fn spread(&self, page: u32) -> Spread {
        Spread::for_preferences(self.page_count(), page, &self.preferences, self.direction)
            .expect("ReaderLayout always has validated nonempty page geometry")
    }
    fn check_anchor(&self, anchor: ReaderPosition) -> Result<()> {
        if anchor.page() >= self.page_count() {
            return Err(Failure::Invalid(
                "Reader anchor is outside this document".into(),
            ));
        }
        Ok(())
    }
    fn unit_size(&self, page: u32) -> [f64; 2] {
        if self.layout() == PageLayout::Continuous {
            // A stable fit throughout scrolling avoids resize loops on mixed-size pages.
            return [self.widest_page, self.tallest_page];
        }
        if self.layout() == PageLayout::Single {
            return self
                .geometry
                .size(page as usize)
                .expect("Validated page anchor");
        }
        let spread = self.spread(page);
        let first = self
            .geometry
            .size(spread.first() as usize)
            .expect("Validated spread");
        if spread.last() == spread.first() {
            return first;
        }
        let second = self
            .geometry
            .size(spread.last() as usize)
            .expect("Validated spread");
        [
            first[0] + self.pair_gap + second[0],
            first[1].max(second[1]),
        ]
    }
    fn zoom_for(
        &self,
        viewport: [f64; 2],
        anchor: ReaderPosition,
        mode: ZoomMode,
        custom: f64,
    ) -> Result<f64> {
        self.check_anchor(anchor)?;
        validate_viewport(viewport)?;
        let [width, height] = self.unit_size(anchor.page());
        let horizontal = (viewport[0] - self.preferences.horizontal_margin * 2.0).max(1.0);
        let vertical = (viewport[1] - self.preferences.vertical_margin * 2.0).max(1.0);
        let zoom = match mode {
            ZoomMode::Custom => custom,
            ZoomMode::FitWidth => horizontal / width,
            ZoomMode::FitPage => (horizontal / width).min(vertical / height),
        };
        if !zoom.is_finite() || zoom <= 0.0 {
            return Err(Failure::Invalid(
                "Reader zoom must be finite and positive".into(),
            ));
        }
        Ok(zoom)
    }
    pub fn frame(
        &self,
        viewport: [f64; 2],
        anchor: ReaderPosition,
        zoom_mode: ZoomMode,
        custom_zoom: f64,
        scroll: [f64; 2],
        overscan: f64,
    ) -> Result<ReaderFrame> {
        if !scroll.iter().all(|value| value.is_finite()) || !overscan.is_finite() || overscan < 0.0
        {
            return Err(Failure::Invalid(
                "Reader scroll and overscan must be finite".into(),
            ));
        }
        let zoom = self.zoom_for(viewport, anchor, zoom_mode, custom_zoom)?;
        let unit = self.unit_size(anchor.page());
        let continuous = self.layout() == PageLayout::Continuous;
        let raw_height = if continuous {
            self.geometry.total_height()
        } else {
            unit[1]
        };
        let raw_width = if continuous {
            self.widest_page
        } else {
            unit[0]
        };
        let content_width =
            (raw_width * zoom + self.preferences.horizontal_margin * 2.0).max(viewport[0]);
        let content_height =
            (raw_height * zoom + self.preferences.vertical_margin * 2.0).max(viewport[1]);
        if !content_width.is_finite()
            || !content_height.is_finite()
            || content_width <= 0.0
            || content_height <= 0.0
        {
            return Err(Failure::Invalid(
                "Reader content exceeds finite coordinate precision".into(),
            ));
        }
        let scroll = [
            scroll[0].clamp(0.0, content_width - viewport[0]),
            scroll[1].clamp(0.0, content_height - viewport[1]),
        ];
        let origin_y = (content_height - raw_height * zoom) / 2.0;
        let visible = LogicalRect {
            left: scroll[0] - overscan,
            top: scroll[1] - overscan,
            width: viewport[0] + overscan * 2.0,
            height: viewport[1] + overscan * 2.0,
        };
        if !visible.right().is_finite() || !visible.bottom().is_finite() {
            return Err(Failure::Invalid(
                "Reader overscan exceeds finite coordinates".into(),
            ));
        }
        let mut pages = Vec::new();
        if continuous {
            let range = self.geometry.visible(
                (visible.top - origin_y) / zoom,
                (visible.bottom() - origin_y) / zoom,
            )?;
            for page in range {
                let [width, height] = self
                    .geometry
                    .size(page)
                    .expect("Visible PageGeometry range is bounded");
                let rect = LogicalRect {
                    left: (content_width - width * zoom) / 2.0,
                    top: origin_y
                        + self
                            .geometry
                            .offset(page)
                            .expect("Visible page offset exists")
                            * zoom,
                    width: width * zoom,
                    height: height * zoom,
                };
                if rect.intersects(visible) {
                    pages.push(VisiblePage {
                        page: page as u32,
                        rect,
                    });
                }
            }
        } else if self.layout() == PageLayout::Single {
            let rect = LogicalRect {
                left: (content_width - unit[0] * zoom) / 2.0,
                top: origin_y,
                width: unit[0] * zoom,
                height: unit[1] * zoom,
            };
            if rect.intersects(visible) {
                pages.push(VisiblePage {
                    page: anchor.page(),
                    rect,
                });
            }
        } else {
            let spread = self.spread(anchor.page());
            let mut x = (content_width - unit[0] * zoom) / 2.0;
            for page in [spread.left, spread.right].into_iter().flatten() {
                let [width, height] = self
                    .geometry
                    .size(page as usize)
                    .expect("Spread page exists");
                let rect = LogicalRect {
                    left: x,
                    top: origin_y,
                    width: width * zoom,
                    height: height * zoom,
                };
                if rect.intersects(visible) {
                    pages.push(VisiblePage { page, rect });
                }
                x += (width + self.pair_gap) * zoom;
            }
        }
        Ok(ReaderFrame {
            zoom,
            viewport,
            content_size: [content_width, content_height],
            scroll,
            pages,
            origin_y,
        })
    }
    /// Preserve a normalized page anchor when changing zoom, rotation, layout,
    /// viewport size or margins. Near document edges the finite canvas clamps.
    pub fn scroll_for_anchor(
        &self,
        viewport: [f64; 2],
        anchor: ReaderPosition,
        zoom_mode: ZoomMode,
        custom_zoom: f64,
    ) -> Result<[f64; 2]> {
        let frame = self.frame(viewport, anchor, zoom_mode, custom_zoom, [0.0, 0.0], 0.0)?;
        let size = self
            .geometry
            .size(anchor.page() as usize)
            .expect("Anchor validated by frame");
        let page_start = if self.layout() == PageLayout::Continuous {
            self.geometry
                .offset(anchor.page() as usize)
                .expect("Validated page offset")
        } else {
            0.0
        };
        let y = frame.origin_y + (page_start + size[1] * anchor.normalized_offset()) * frame.zoom;
        Ok([
            (frame.content_size[0] - viewport[0]) / 2.0,
            y.clamp(0.0, frame.content_size[1] - viewport[1]),
        ])
    }
    /// Scroll feedback uses the same canonical ReaderPosition and must update
    /// ReaderNavigation::update_anchor, without adding a history entry.
    pub fn anchor_at_scroll(
        &self,
        frame: &ReaderFrame,
        previous: ReaderPosition,
    ) -> Result<ReaderPosition> {
        self.check_anchor(previous)?;
        if self.layout() == PageLayout::Continuous {
            return self
                .geometry
                .position_at((frame.scroll[1] - frame.origin_y) / frame.zoom)?
                .ok_or_else(|| {
                    Failure::Invalid("Cannot recover an anchor from empty geometry".into())
                });
        }
        let height = self
            .geometry
            .size(previous.page() as usize)
            .expect("Validated page anchor")[1]
            * frame.zoom;
        ReaderPosition::new(
            previous.page(),
            ((frame.scroll[1] - frame.origin_y) / height).clamp(0.0, 1.0),
        )
    }
}
fn validate_viewport(viewport: [f64; 2]) -> Result<()> {
    if viewport
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err(Failure::Invalid(
            "Reader viewport must be finite and positive".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::{ReaderMode, ReaderNavigation};
    use crate::reader_preferences::{CoverPolicy, InitialSide};
    fn preferences(layout: PageLayout) -> ReaderPreferences {
        let mut p = ReaderPreferences::new("document");
        p.page_layout = layout;
        p.horizontal_margin = 20.0;
        p.vertical_margin = 10.0;
        p
    }
    #[test]
    fn single_and_rotated_mixed_spreads_fit_the_actual_reading_unit() {
        let mut p = preferences(PageLayout::Single);
        let anchor = ReaderPosition::new(1, 0.0).unwrap();
        let sizes = [[600.0, 800.0], [900.0, 500.0], [300.0, 1000.0]];
        let single = ReaderLayout::new(sizes, &p, ReadingDirection::Ltr, 24.0).unwrap();
        let f = single
            .frame(
                [1000.0, 800.0],
                anchor,
                ZoomMode::FitPage,
                1.0,
                [0.0, 0.0],
                0.0,
            )
            .unwrap();
        assert_eq!(f.pages.len(), 1);
        assert_eq!(f.pages[0].page, 1);
        assert!((f.zoom - 960.0 / 900.0).abs() < 1e-10);
        assert!(f.pages[0].rect.left >= 20.0 && f.pages[0].rect.bottom() <= 790.0);
        p.page_layout = PageLayout::Spread;
        p.page_rotation = PageRotation::Deg90;
        let spread = ReaderLayout::new(sizes, &p, ReadingDirection::Rtl, 24.0).unwrap();
        let f = spread
            .frame(
                [1000.0, 800.0],
                anchor,
                ZoomMode::FitPage,
                1.0,
                [0.0, 0.0],
                0.0,
            )
            .unwrap();
        assert_eq!(spread.page_size(0), Some([800.0, 600.0]));
        assert_eq!(f.pages.iter().map(|p| p.page).collect::<Vec<_>>(), [1, 0]);
        assert!((f.zoom - (960.0 / 1302.0)).abs() < 1e-10);
        assert!(f.pages[0].rect.right() <= f.pages[1].rect.left);
        assert_eq!(f.pages[0].rect.top, f.pages[1].rect.top);
    }
    #[test]
    fn book_pairing_and_navigation_use_the_same_physical_slots_and_anchor() {
        for cover in [CoverPolicy::NoCover, CoverPolicy::FirstPageIsCover] {
            for initial_side in [InitialSide::Left, InitialSide::Right] {
                for direction in [ReadingDirection::Ltr, ReadingDirection::Rtl] {
                    let mut p = preferences(PageLayout::Single);
                    p.view_mode = ReaderMode::Book;
                    p.cover_policy = cover;
                    p.initial_side = initial_side;
                    let layout =
                        ReaderLayout::new([[600.0, 800.0]; 7], &p, direction, 2.0).unwrap();
                    assert_eq!(layout.layout(), PageLayout::Spread);
                    let mut nav = ReaderNavigation::new(7, 0).unwrap();
                    nav.apply_preferences(&p, direction);
                    let first = nav.spread();
                    assert_eq!(first, layout.spread(0));
                    if cover == CoverPolicy::NoCover && initial_side == InitialSide::Right {
                        assert_eq!((first.left, first.right), (None, Some(0)));
                    }
                    for page in 0..7 {
                        nav.go_to_page(page);
                        let a = ReaderPosition::new(page, 0.375).unwrap();
                        nav.update_anchor(a);
                        nav.apply_preferences(&p, direction);
                        assert_eq!(nav.position(), a);
                        assert_eq!(nav.spread(), layout.spread(page));
                        let f = layout
                            .frame([1400.0, 900.0], a, ZoomMode::FitPage, 1.0, [0.0, 0.0], 0.0)
                            .unwrap();
                        let spread = nav.spread();
                        assert_eq!(
                            f.pages.iter().map(|p| p.page).collect::<Vec<_>>(),
                            [spread.left, spread.right]
                                .into_iter()
                                .flatten()
                                .collect::<Vec<_>>()
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn continuous_five_thousand_mixed_pages_match_linear_visible_oracle() {
        let p = preferences(PageLayout::Continuous);
        let sizes: Vec<_> = (0..5000)
            .map(|i| [400.0 + (i % 3) as f32 * 80.0, 600.0 + (i % 5) as f32 * 90.0])
            .collect();
        let layout = ReaderLayout::new(sizes.clone(), &p, ReadingDirection::Ltr, 24.0).unwrap();
        let anchor = ReaderPosition::new(0, 0.0).unwrap();
        for index in (0..5000).step_by(79) {
            let offset = layout.geometry.offset(index).unwrap() + 17.0;
            let f = layout
                .frame(
                    [900.0, 1000.0],
                    anchor,
                    ZoomMode::Custom,
                    1.25,
                    [0.0, offset * 1.25],
                    80.0,
                )
                .unwrap();
            let mut start = 10.0;
            let expected: Vec<_> = sizes
                .iter()
                .enumerate()
                .filter_map(|(i, size)| {
                    let rect = LogicalRect {
                        left: (f.content_size[0] - f64::from(size[0]) * 1.25) / 2.0,
                        top: start,
                        width: f64::from(size[0]) * 1.25,
                        height: f64::from(size[1]) * 1.25,
                    };
                    start += f64::from(size[1]) * 1.25 + 24.0 * 1.25;
                    rect.intersects(LogicalRect {
                        left: f.scroll[0] - 80.0,
                        top: f.scroll[1] - 80.0,
                        width: 1060.0,
                        height: 1160.0,
                    })
                    .then_some(i as u32)
                })
                .collect();
            assert_eq!(f.pages.iter().map(|p| p.page).collect::<Vec<_>>(), expected);
            assert!(f.pages.len() <= 4);
        }
    }
    #[test]
    fn fit_stays_stable_while_scrolling_mixed_continuous_pages() {
        let p = preferences(PageLayout::Continuous);
        let layout = ReaderLayout::new(
            [[600.0, 800.0], [1600.0, 300.0], [400.0, 1900.0]],
            &p,
            ReadingDirection::Ltr,
            24.0,
        )
        .unwrap();
        for mode in [ZoomMode::FitPage, ZoomMode::FitWidth] {
            let zooms: Vec<_> = (0..3)
                .map(|page| {
                    layout
                        .frame(
                            [900.0, 800.0],
                            ReaderPosition::new(page, 0.3).unwrap(),
                            mode,
                            1.0,
                            [0.0, 0.0],
                            0.0,
                        )
                        .unwrap()
                        .zoom
                })
                .collect();
            assert_eq!(zooms, [zooms[0]; 3]);
        }
    }
    #[test]
    fn canonical_anchor_survives_zoom_rotation_and_viewport_changes() {
        let mut p = preferences(PageLayout::Continuous);
        let sizes = [[600.0, 800.0]; 20];
        let anchor = ReaderPosition::new(9, 0.375).unwrap();
        for rotation in [
            PageRotation::Deg0,
            PageRotation::Deg90,
            PageRotation::Deg180,
            PageRotation::Deg270,
        ] {
            p.page_rotation = rotation;
            let layout = ReaderLayout::new(sizes, &p, ReadingDirection::Ltr, 24.0).unwrap();
            for zoom in [0.5, 1.0, 2.0, 4.0] {
                for viewport in [[640.0, 480.0], [1440.0, 960.0]] {
                    let scroll = layout
                        .scroll_for_anchor(viewport, anchor, ZoomMode::Custom, zoom)
                        .unwrap();
                    let f = layout
                        .frame(viewport, anchor, ZoomMode::Custom, zoom, scroll, 0.0)
                        .unwrap();
                    let actual = layout.anchor_at_scroll(&f, anchor).unwrap();
                    assert_eq!(actual.page(), anchor.page());
                    assert!(
                        (actual.normalized_offset() - anchor.normalized_offset()).abs() < 1e-10
                    );
                }
            }
        }
    }
    #[test]
    fn malformed_inputs_fail_and_scroll_clamps_to_finite_content_bounds() {
        let p = preferences(PageLayout::Single);
        assert!(ReaderLayout::new([], &p, ReadingDirection::Ltr, 24.0).is_err());
        assert!(ReaderLayout::new([[f32::NAN, 800.0]], &p, ReadingDirection::Ltr, 24.0).is_err());
        let layout = ReaderLayout::new([[600.0, 800.0]], &p, ReadingDirection::Ltr, 24.0).unwrap();
        let a = ReaderPosition::new(0, 0.0).unwrap();
        for invalid in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
            assert!(
                layout
                    .frame([invalid, 480.0], a, ZoomMode::Custom, 1.0, [0.0, 0.0], 0.0)
                    .is_err()
            );
            assert!(
                layout
                    .frame(
                        [640.0, 480.0],
                        a,
                        ZoomMode::Custom,
                        invalid,
                        [0.0, 0.0],
                        0.0
                    )
                    .is_err()
            );
        }
        assert!(
            layout
                .frame(
                    [640.0, 480.0],
                    a,
                    ZoomMode::Custom,
                    1.0,
                    [f64::NAN, 0.0],
                    0.0
                )
                .is_err()
        );
        assert!(
            layout
                .frame([640.0, 480.0], a, ZoomMode::Custom, 1.0, [0.0, 0.0], -1.0)
                .is_err()
        );
        assert!(
            layout
                .frame(
                    [640.0, 480.0],
                    ReaderPosition::new(1, 0.0).unwrap(),
                    ZoomMode::Custom,
                    1.0,
                    [0.0, 0.0],
                    0.0
                )
                .is_err()
        );
        let f = layout
            .frame(
                [640.0, 480.0],
                a,
                ZoomMode::Custom,
                2.0,
                [-1000.0, 1e300],
                0.0,
            )
            .unwrap();
        assert_eq!(f.scroll, [0.0, f.content_size[1] - 480.0]);
        assert!(f.content_size.into_iter().all(f64::is_finite));
    }
}
