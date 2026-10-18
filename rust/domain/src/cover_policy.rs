use crate::{Cancellation, Failure, Result};

pub const COVER_ALGORITHM_VERSION: u32 = 3;
pub const MAX_COVER_CANDIDATES: usize = 6;

pub struct DetectedCover {
    pub image: std::sync::Arc<crate::RenderedPage>,
    pub choice: CoverChoice,
}

#[derive(Clone, Copy, Debug)]
pub struct CoverMetrics {
    page: u32,
    uniform: f64,
    edges: f64,
    contrast: f64,
    text_units: usize,
    lines: usize,
    contents: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoverChoice {
    pub page: u32,
    pub score: f64,
    pub confidence: f64,
}

impl CoverMetrics {
    pub fn new(
        page: u32,
        uniform: f64,
        edges: f64,
        contrast: f64,
        text_units: usize,
        lines: usize,
        contents: bool,
    ) -> Result<Self> {
        if page >= MAX_COVER_CANDIDATES as u32
            || [uniform, edges, contrast]
                .iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(Failure::Invalid("Invalid cover candidate metrics".into()));
        }
        Ok(Self {
            page,
            uniform,
            edges,
            contrast,
            text_units,
            lines,
            contents,
        })
    }
    pub fn score(&self) -> CoverChoice {
        let mut score = [40.0, 22.0, 12.0, 8.0, 5.0, 3.0][self.page as usize];
        if (0.28..=0.82).contains(&self.uniform) {
            score += 10.0;
        } else if self.uniform > 0.94 {
            score -= 12.0;
        }
        if (0.025..=0.34).contains(&self.edges) {
            score += 12.0;
        } else if self.edges > 0.48 {
            score -= 15.0;
        }
        if self.contrast >= 0.12 {
            score += 5.0;
        }
        if self.text_units <= 220 && self.lines <= 12 {
            score += 10.0;
        }
        if self.text_units > 900 || self.lines > 34 {
            score -= 20.0;
        }
        if self.contents {
            score -= 18.0;
        }
        CoverChoice {
            page: self.page,
            score,
            confidence: ((score + 20.0) / 100.0).clamp(0.0, 1.0),
        }
    }
    pub fn measure(
        page: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
        text: &str,
        cancel: &Cancellation,
    ) -> Result<Self> {
        cancel.check()?;
        if width == 0
            || height == 0
            || width > 512
            || height > 512
            || rgba.len() != width as usize * height as usize * 4
        {
            return Err(Failure::Invalid("Invalid cover candidate raster".into()));
        }
        let (mut samples, mut uniform, mut edges) = (0usize, 0usize, 0usize);
        let (mut sum, mut squares) = (0.0, 0.0);
        for y in (0..height).step_by((height / 48).max(1) as usize) {
            cancel.check()?;
            let mut previous: Option<f64> = None;
            for x in (0..width).step_by((width / 48).max(1) as usize) {
                let offset = ((y * width + x) * 4) as usize;
                let luminance = (f64::from(rgba[offset]) * 0.2126
                    + f64::from(rgba[offset + 1]) * 0.7152
                    + f64::from(rgba[offset + 2]) * 0.0722)
                    / 255.0;
                samples += 1;
                sum += luminance;
                squares += luminance * luminance;
                if luminance >= 0.94 || luminance <= 0.06 {
                    uniform += 1;
                }
                if previous.is_some_and(|before| (luminance - before).abs() > 0.16) {
                    edges += 1;
                }
                previous = Some(luminance);
            }
        }
        let mean = sum / samples as f64;
        let text = text.trim();
        let contents = text
            .split(|character: char| !character.is_alphanumeric() && character != '_')
            .any(|word| {
                [
                    "índice",
                    "indice",
                    "contenido",
                    "contents",
                    "chapter",
                    "capítulo",
                ]
                .iter()
                .any(|marker| word.chars().flat_map(char::to_lowercase).eq(marker.chars()))
            });
        cancel.check()?;
        Self::new(
            page,
            uniform as f64 / samples as f64,
            edges as f64 / samples as f64,
            (squares / samples as f64 - mean * mean).max(0.0).sqrt(),
            text.encode_utf16().count(),
            if text.is_empty() {
                0
            } else {
                text.split('\n').count()
            },
            contents,
        )
    }
}

pub fn choose(candidates: &[CoverMetrics]) -> Result<CoverChoice> {
    if candidates.is_empty()
        || candidates.len() > MAX_COVER_CANDIDATES
        || candidates
            .iter()
            .enumerate()
            .any(|(index, metrics)| metrics.page as usize != index)
    {
        return Err(Failure::Invalid(
            "Cover candidates must be the ordered first pages".into(),
        ));
    }
    let first = candidates[0].score();
    let best = candidates.iter().skip(1).fold(first, |best, metrics| {
        let next = metrics.score();
        if next.score > best.score { next } else { best }
    });
    if best.page == first.page
        || best.confidence >= 0.70
        || (best.confidence >= 0.45 && best.score >= first.score + 12.0)
    {
        Ok(best)
    } else {
        Ok(first)
    }
}
