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
    pub(crate) fn destination_viewport(&self, viewport: [f64; 2]) -> Result<[f64; 2]> {
        validate_viewport(viewport)?;
        Ok([
            (viewport[0] - self.preferences.horizontal_margin * 2.0).max(1.0),
            (viewport[1] - self.preferences.vertical_margin * 2.0).max(1.0),
        ])
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
        let [horizontal, vertical] = self.destination_viewport(viewport)?;
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
                let rect = self.page_rect(page as u32, content_width, origin_y, zoom);
                if rect.intersects(visible) {
                    pages.push(VisiblePage {
                        page: page as u32,
                        rect,
                    });
                }
            }
        } else if self.layout() == PageLayout::Single {
            let rect = self.page_rect(anchor.page(), content_width, origin_y, zoom);
            if rect.intersects(visible) {
                pages.push(VisiblePage {
                    page: anchor.page(),
                    rect,
                });
            }
        } else {
            let spread = self.spread(anchor.page());
            for page in [spread.left, spread.right].into_iter().flatten() {
                let rect = self.page_rect(page, content_width, origin_y, zoom);
                if rect.intersects(visible) {
                    pages.push(VisiblePage { page, rect });
                }
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
    fn page_rect(&self, page: u32, content_width: f64, origin_y: f64, zoom: f64) -> LogicalRect {
        let [width, height] = self
            .geometry
            .size(page as usize)
            .expect("Validated page anchor");
        let mut left = (content_width - width * zoom) / 2.0;
        let mut top = origin_y;
        if self.layout() == PageLayout::Continuous {
            top += self
                .geometry
                .offset(page as usize)
                .expect("Validated page offset")
                * zoom;
        } else if self.layout() == PageLayout::Spread {
            left = (content_width - self.unit_size(page)[0] * zoom) / 2.0;
            let spread = self.spread(page);
            if spread.right == Some(page)
                && let Some(first) = spread.left
            {
                left += (self
                    .geometry
                    .size(first as usize)
                    .expect("Validated spread")[0]
                    + self.pair_gap)
                    * zoom;
            }
        }
        LogicalRect {
            left,
            top,
            width: width * zoom,
            height: height * zoom,
        }
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
        let rect = self.page_rect(
            anchor.page(),
            frame.content_size[0],
            frame.origin_y,
            frame.zoom,
        );
        let point = self.rotation().rotate_point(anchor.point());
        Ok([
            (rect.left + rect.width * point[0] - viewport[0] / 2.0)
                .clamp(0.0, frame.content_size[0] - viewport[0]),
            (rect.top + rect.height * point[1]).clamp(0.0, frame.content_size[1] - viewport[1]),
        ])
    }
    /// Recover the viewport's top-left PDF point, including blank page margins.
    /// The frame and coordinate space must belong to this layout and page.
    pub fn pdf_point_at_viewport(
        &self,
        frame: &ReaderFrame,
        page: u32,
        coordinates: crate::pdf_geometry::PageCoordinateSpace,
    ) -> Result<[f64; 2]> {
        self.check_anchor(ReaderPosition::new(page, 0.0)?)?;
        let rect = self.page_rect(page, frame.content_size[0], frame.origin_y, frame.zoom);
        let displayed = [
            (frame.scroll[0] - rect.left) / rect.width,
            (frame.scroll[1] - rect.top) / rect.height,
        ];
        let point = coordinates.pdf_point(self.rotation().unrotate_point(displayed));
        if point.iter().any(|value| !value.is_finite()) {
            return Err(Failure::Invalid(
                "Viewport PDF point exceeds finite coordinates".into(),
            ));
        }
        Ok(point)
    }
    /// Scroll feedback uses the same canonical ReaderPosition and must update
    /// ReaderNavigation::update_anchor, without adding a history entry.
    pub fn anchor_at_scroll(
        &self,
        frame: &ReaderFrame,
        previous: ReaderPosition,
    ) -> Result<ReaderPosition> {
        self.check_anchor(previous)?;
        let page = if self.layout() == PageLayout::Continuous {
            self.geometry
                .position_at((frame.scroll[1] - frame.origin_y) / frame.zoom)?
                .ok_or_else(|| {
                    Failure::Invalid("Cannot recover an anchor from empty geometry".into())
                })?
                .page()
        } else {
            previous.page()
        };
        let rect = self.page_rect(page, frame.content_size[0], frame.origin_y, frame.zoom);
        let displayed = [
            ((frame.scroll[0] + frame.viewport[0] / 2.0 - rect.left) / rect.width).clamp(0.0, 1.0),
            ((frame.scroll[1] - rect.top) / rect.height).clamp(0.0, 1.0),
        ];
        ReaderPosition::at(page, self.rotation().unrotate_point(displayed))
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
