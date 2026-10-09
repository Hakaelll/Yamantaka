use crate::{Failure, Result};

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
    pub fn normalized_rectangle(
        &self,
        [left, bottom, right, top]: [f64; 4],
    ) -> Result<Option<NormalizedPageRect>> {
        if [left, bottom, right, top]
            .into_iter()
            .any(|value| !value.is_finite())
            || left > right
            || bottom > top
        {
            return Err(Failure::Pdf("PDF rectangle is invalid".into()));
        }
        let mut min = [1.0f64; 2];
        let mut max = [0.0f64; 2];
        for point in [[left, top], [right, top], [left, bottom], [right, bottom]] {
            let point = self.normalized_point(point);
            if point.into_iter().any(|value| !value.is_finite()) {
                return Err(Failure::Pdf("PDF rectangle coordinates are invalid".into()));
            }
            for axis in 0..2 {
                min[axis] = min[axis].min(point[axis].clamp(0.0, 1.0));
                max[axis] = max[axis].max(point[axis].clamp(0.0, 1.0));
            }
        }
        if (0..2).all(|axis| min[axis] < max[axis]) {
            Ok(Some(NormalizedPageRect::new(min, max)?))
        } else {
            Ok(None)
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedPageRect {
    min: [f64; 2],
    max: [f64; 2],
}

impl NormalizedPageRect {
    pub fn rotated(self, rotation: crate::reader_preferences::PageRotation) -> Self {
        let mut min = [1.0f64; 2];
        let mut max = [0.0f64; 2];
        for point in [
            [self.min[0], self.min[1]],
            [self.max[0], self.min[1]],
            [self.min[0], self.max[1]],
            [self.max[0], self.max[1]],
        ] {
            let point = rotation.rotate_point(point);
            for axis in 0..2 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
        Self { min, max }
    }

    pub fn new(min: [f64; 2], max: [f64; 2]) -> Result<Self> {
        if min
            .into_iter()
            .chain(max)
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
            || (0..2).any(|axis| min[axis] >= max[axis])
        {
            return Err(Failure::Invalid(
                "Search rectangles must be finite normalized areas".into(),
            ));
        }
        Ok(Self { min, max })
    }
    pub fn min(&self) -> [f64; 2] {
        self.min
    }
    pub fn max(&self) -> [f64; 2] {
        self.max
    }
}

#[cfg(test)]
mod tests {
    use super::PageCoordinateSpace;
    #[test]
    fn rectangles_preserve_crop_clip_outside_areas_and_rotate_all_corners() {
        use crate::reader_preferences::PageRotation;
        let space = PageCoordinateSpace::new([20.0, 180.0], [120.0, 180.0], [20.0, 30.0]).unwrap();
        let rectangle = space
            .normalized_rectangle([40.0, 120.0, 60.0, 150.0])
            .unwrap()
            .unwrap();
        assert_eq!(rectangle.min(), [0.2, 0.2]);
        assert_eq!(rectangle.max(), [0.4, 0.4]);
        let rotated = rectangle.rotated(PageRotation::Deg90);
        assert_eq!(rotated.min(), [0.6, 0.2]);
        assert_eq!(rotated.max(), [0.8, 0.4]);
        assert_eq!(
            space
                .normalized_rectangle([0.0, 120.0, 40.0, 210.0])
                .unwrap()
                .unwrap()
                .min(),
            [0.0, 0.0]
        );
        assert_eq!(
            space
                .normalized_rectangle([200.0, 220.0, 240.0, 250.0])
                .unwrap(),
            None
        );
        assert_eq!(
            space
                .normalized_rectangle([40.0, 120.0, 40.0, 150.0])
                .unwrap(),
            None
        );
        assert!(
            space
                .normalized_rectangle([60.0, 120.0, 40.0, 150.0])
                .is_err()
        );
        assert!(
            space
                .normalized_rectangle([f64::NAN, 120.0, 60.0, 150.0])
                .is_err()
        );
    }
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
