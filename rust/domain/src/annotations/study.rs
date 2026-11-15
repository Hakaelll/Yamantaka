use super::AnnotationKind;
use crate::{Failure, Result};
use serde::{Deserialize, Serialize};

pub const MAX_STUDY_POINTS: usize = 2048;
const MAX_ENCODED_BYTES: usize = 128 * 1024;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StrokeWeight {
    Fine,
    #[default]
    Medium,
    Thick,
}

impl StrokeWeight {
    pub fn factor(self) -> f32 {
        match self {
            Self::Fine => 0.65,
            Self::Medium => 1.0,
            Self::Thick => 2.0,
        }
    }

    fn is_medium(&self) -> bool {
        *self == Self::Medium
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StudyGeometry {
    version: u8,
    kind: AnnotationKind,
    points: Vec<[f64; 2]>,
    #[serde(skip_serializing_if = "StrokeWeight::is_medium")]
    weight: StrokeWeight,
}

impl StudyGeometry {
    pub fn new(kind: AnnotationKind, points: Vec<[f64; 2]>) -> Result<Self> {
        let valid_shape = match kind {
            AnnotationKind::Ink => {
                (2..=MAX_STUDY_POINTS).contains(&points.len())
                    && points.windows(2).any(|pair| pair[0] != pair[1])
            }
            AnnotationKind::Arrow | AnnotationKind::Line => {
                points.len() == 2 && points[0] != points[1]
            }
            AnnotationKind::Rectangle | AnnotationKind::Ellipse | AnnotationKind::Crop => {
                points.len() == 2 && points[0][0] != points[1][0] && points[0][1] != points[1][1]
            }
            _ => false,
        };
        if !valid_shape
            || !points
                .iter()
                .flatten()
                .all(|value| (0.0..=1.0).contains(value))
        {
            return Err(Failure::Invalid("Invalid drawing geometry".into()));
        }
        Ok(Self {
            version: 1,
            kind,
            points,
            weight: StrokeWeight::Medium,
        })
    }

    pub fn with_weight(mut self, weight: StrokeWeight) -> Self {
        self.version = if weight == StrokeWeight::Medium { 1 } else { 2 };
        self.weight = weight;
        self
    }

    pub fn bounds(&self) -> [[f64; 2]; 2] {
        let mut bounds = [self.points[0]; 2];
        for point in &self.points[1..] {
            for (axis, coordinate) in point.iter().copied().enumerate() {
                bounds[0][axis] = bounds[0][axis].min(coordinate);
                bounds[1][axis] = bounds[1][axis].max(coordinate);
            }
        }
        bounds
    }

    pub fn with_bounds(&self, bounds: [[f64; 2]; 2]) -> Result<Self> {
        let original = self.bounds();
        let mut points = self.points.clone();
        for point in &mut points {
            for (axis, coordinate) in point.iter_mut().enumerate() {
                let span = original[1][axis] - original[0][axis];
                let fraction = if span == 0.0 {
                    0.0
                } else {
                    (*coordinate - original[0][axis]) / span
                };
                *coordinate = (bounds[0][axis] + fraction * (bounds[1][axis] - bounds[0][axis]))
                    .clamp(0.0, 1.0);
            }
        }
        Ok(Self::new(self.kind, points)?.with_weight(self.weight))
    }

    pub fn weight(&self) -> StrokeWeight {
        self.weight
    }

    pub fn kind(&self) -> AnnotationKind {
        self.kind
    }

    pub fn points(&self) -> &[[f64; 2]] {
        &self.points
    }

    pub(super) fn allocated_bytes(&self) -> usize {
        self.points.capacity() * std::mem::size_of::<[f64; 2]>()
    }

    pub fn decode(kind: AnnotationKind, json: &str) -> Result<Self> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields {
            version: u8,
            kind: AnnotationKind,
            points: Vec<[f64; 2]>,
            weight: Option<StrokeWeight>,
        }
        if json.len() > MAX_ENCODED_BYTES {
            return Err(Failure::Invalid(
                "Drawing geometry exceeds its size limit".into(),
            ));
        }
        let fields: Fields = serde_json::from_str(json)
            .map_err(|_| Failure::Invalid("Drawing geometry cannot be decoded safely".into()))?;
        if !matches!((fields.version, fields.weight), (1, None) | (2, Some(_)))
            || fields.kind != kind
        {
            return Err(Failure::Invalid(
                "Drawing kind or version does not match".into(),
            ));
        }
        Ok(Self::new(kind, fields.points)?.with_weight(fields.weight.unwrap_or_default()))
    }
}
