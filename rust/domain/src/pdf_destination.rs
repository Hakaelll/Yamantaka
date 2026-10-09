use crate::pdf_geometry::PageCoordinateSpace;

/// PDF coordinates are preserved in points until canonical navigation applies
/// the destination using the actual page geometry and current reader settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DestinationView {
    Unknown,
    Coordinates {
        x: Option<f32>,
        y: Option<f32>,
        zoom: Option<f32>,
    },
    FitPage,
    FitWidth {
        top: Option<f32>,
    },
    FitHeight {
        left: Option<f32>,
    },
    FitRectangle {
        left: f32,
        bottom: f32,
        right: f32,
        top: f32,
    },
    FitBounds,
    FitBoundsWidth {
        top: Option<f32>,
    },
    FitBoundsHeight {
        left: Option<f32>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavigationDestination {
    pub page: u32,
    pub view: DestinationView,
    pub coordinates: PageCoordinateSpace,
    pub content_bounds: Option<DestinationRectangle>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DestinationRectangle([f64; 4]);
impl DestinationRectangle {
    pub fn new([left, bottom, right, top]: [f64; 4]) -> crate::Result<Self> {
        if [left, bottom, right, top]
            .iter()
            .any(|value| !value.is_finite())
            || left >= right
            || bottom >= top
        {
            return Err(crate::Failure::Pdf(
                "Invalid PDF destination rectangle".into(),
            ));
        }
        Ok(Self([left, bottom, right, top]))
    }
    pub fn bounds(self) -> [f64; 4] {
        self.0
    }
}
