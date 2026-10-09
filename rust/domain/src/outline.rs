use crate::memory::ByteLease;

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
pub struct OutlineDestination {
    pub page: u32,
    pub view: DestinationView,
    pub coordinates: PageCoordinateSpace,
}

/// PDF points at the displayed page's top-left, top-right and bottom-left.
/// PDFium supplies these corners, including crop offsets and intrinsic rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageCoordinateSpace {
    origin: [f64; 2],
    horizontal: [f64; 2],
    vertical: [f64; 2],
}
impl PageCoordinateSpace {
    pub fn new(origin: [f64; 2], right: [f64; 2], bottom: [f64; 2]) -> crate::Result<Self> {
        let horizontal = [right[0] - origin[0], right[1] - origin[1]];
        let vertical = [bottom[0] - origin[0], bottom[1] - origin[1]];
        let determinant = horizontal[0] * vertical[1] - horizontal[1] * vertical[0];
        if origin
            .into_iter()
            .chain(right)
            .chain(bottom)
            .any(|value| !value.is_finite())
            || !determinant.is_finite()
            || determinant == 0.0
        {
            return Err(crate::Failure::Pdf(
                "Invalid PDF page coordinate space".into(),
            ));
        }
        Ok(Self {
            origin,
            horizontal,
            vertical,
        })
    }
    /// Unclamped coordinates retain destinations outside the visible crop box.
    pub fn normalized_point(self, point: [f64; 2]) -> [f64; 2] {
        let delta = [point[0] - self.origin[0], point[1] - self.origin[1]];
        let determinant =
            self.horizontal[0] * self.vertical[1] - self.horizontal[1] * self.vertical[0];
        [
            (delta[0] * self.vertical[1] - delta[1] * self.vertical[0]) / determinant,
            (self.horizontal[0] * delta[1] - self.horizontal[1] * delta[0]) / determinant,
        ]
    }
    pub fn pdf_point(self, normalized: [f64; 2]) -> [f64; 2] {
        [
            self.origin[0] + self.horizontal[0] * normalized[0] + self.vertical[0] * normalized[1],
            self.origin[1] + self.horizontal[1] * normalized[0] + self.vertical[1] * normalized[1],
        ]
    }
}

#[derive(Debug, PartialEq)]
pub struct OutlineEntry {
    pub title: String,
    /// Parent is an earlier entry in this depth-first list.
    pub parent: Option<usize>,
    /// External actions and headings without a local destination have no target.
    pub destination: Option<OutlineDestination>,
}

/// Shared replies retain their memory reservation until the final owner drops.
#[derive(Debug)]
pub struct PdfOutline {
    entries: Vec<OutlineEntry>,
    _lease: ByteLease,
}
impl PdfOutline {
    pub fn entries(&self) -> &[OutlineEntry] {
        &self.entries
    }
    pub fn from_reserved(entries: Vec<OutlineEntry>, mut lease: ByteLease) -> crate::Result<Self> {
        let bytes = entries
            .capacity()
            .checked_mul(std::mem::size_of::<OutlineEntry>())
            .and_then(|bytes| {
                entries.iter().try_fold(bytes, |bytes, entry| {
                    bytes.checked_add(entry.title.capacity())
                })
            })
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())
            })
            .ok_or(crate::Failure::MemoryBudget)?;
        lease
            .try_resize(bytes)
            .map_err(|_| crate::Failure::MemoryBudget)?;
        Ok(Self {
            entries,
            _lease: lease,
        })
    }
}

#[cfg(test)]
mod coordinate_tests {
    use super::PageCoordinateSpace;
    #[test]
    fn coordinates_preserve_crop_offsets_rotations_and_outside_points() {
        let corners = [
            ([20.0, 180.0], [120.0, 180.0], [20.0, 30.0]),
            ([20.0, 30.0], [20.0, 180.0], [120.0, 30.0]),
            ([120.0, 30.0], [20.0, 30.0], [120.0, 180.0]),
            ([120.0, 180.0], [120.0, 30.0], [20.0, 180.0]),
        ];
        let expected = [[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]];
        for ((origin, right, bottom), expected) in corners.into_iter().zip(expected) {
            let space = PageCoordinateSpace::new(origin, right, bottom).unwrap();
            let actual = space.normalized_point([40.0, 150.0]);
            for axis in 0..2 {
                assert!((actual[axis] - expected[axis]).abs() < 1e-12);
            }
            assert_eq!(space.pdf_point(actual), [40.0, 150.0]);
        }
        let space = PageCoordinateSpace::new([20.0, 180.0], [120.0, 180.0], [20.0, 30.0]).unwrap();
        assert_eq!(space.normalized_point([0.0, 210.0]), [-0.2, -0.2]);
        assert!(PageCoordinateSpace::new([0.0, 0.0], [0.0, 0.0], [1.0, 1.0]).is_err());
        assert!(PageCoordinateSpace::new([f64::NAN, 0.0], [1.0, 0.0], [0.0, 1.0]).is_err());
    }
}
