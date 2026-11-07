use super::AnnotationKind;
use crate::{Failure, Result};
use serde::{Deserialize, Serialize};

pub const MAX_STUDY_POINTS: usize = 2048;
const MAX_ENCODED_BYTES: usize = 128 * 1024;

#[derive(Debug, PartialEq, Serialize)]
pub struct StudyGeometry {
    version: u8,
    kind: AnnotationKind,
    points: Vec<[f64; 2]>,
}

impl StudyGeometry {
    pub fn new(kind: AnnotationKind, points: Vec<[f64; 2]>) -> Result<Self> {
        let valid_shape = match kind {
            AnnotationKind::Ink => {
                (2..=MAX_STUDY_POINTS).contains(&points.len())
                    && points.windows(2).any(|pair| pair[0] != pair[1])
            }
            AnnotationKind::Arrow => points.len() == 2 && points[0] != points[1],
            AnnotationKind::Rectangle => {
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
        })
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
        }
        if json.len() > MAX_ENCODED_BYTES {
            return Err(Failure::Invalid(
                "Drawing geometry exceeds its size limit".into(),
            ));
        }
        let fields: Fields = serde_json::from_str(json)
            .map_err(|_| Failure::Invalid("Drawing geometry cannot be decoded safely".into()))?;
        if fields.version != 1 || fields.kind != kind {
            return Err(Failure::Invalid(
                "Drawing kind or version does not match".into(),
            ));
        }
        Self::new(kind, fields.points)
    }
}
