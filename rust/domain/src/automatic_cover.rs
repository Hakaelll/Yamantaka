use crate::{
    Document, Failure, RenderedPage, Result,
    cover_policy::{CoverChoice, MAX_COVER_CANDIDATES},
};

pub struct AutomaticCover {
    document: Document,
    source_modified: String,
    page: u32,
    confidence: f64,
    aspect: f64,
}
impl AutomaticCover {
    pub fn new(
        document: Document,
        source_modified: String,
        choice: CoverChoice,
        image: &RenderedPage,
    ) -> Result<Self> {
        if choice.page >= document.page_count.min(MAX_COVER_CANDIDATES as u32)
            || !choice.score.is_finite()
            || !choice.confidence.is_finite()
            || choice.confidence != ((choice.score + 20.0) / 100.0).clamp(0.0, 1.0)
            || image.key.document_id != document.id
            || image.key.page != choice.page
            || image.key.region.is_some()
            || image.key.rotation != 0
            || image.width == 0
            || image.height == 0
            || image.width > 512
            || image.height > 512
            || image.key.output_dimensions()? != [image.width, image.height]
            || image.pixels().len() != image.width as usize * image.height as usize * 4
        {
            return Err(Failure::Invalid(
                "Automatic cover does not match its document or preview".into(),
            ));
        }
        Ok(Self {
            document,
            source_modified,
            page: choice.page,
            confidence: choice.confidence,
            aspect: (f64::from(image.width) / f64::from(image.height)).clamp(0.1, 10.0),
        })
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    pub fn source_modified(&self) -> &str {
        &self.source_modified
    }
    pub fn page(&self) -> u32 {
        self.page
    }
    pub fn confidence(&self) -> f64 {
        self.confidence
    }
    pub fn aspect(&self) -> f64 {
        self.aspect
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderKey,
        memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
    };
    #[test]
    fn automatic_values_require_matching_finite_choices_and_bounded_rasters() {
        let document = Document {
            id: "d".into(),
            path: "synthetic.pdf".into(),
            file_name: "synthetic.pdf".into(),
            title: None,
            author: None,
            page_count: 7,
            last_page_index: 0,
            reading_progress: 0.0,
            favorite: false,
            missing: false,
            fingerprint: "synthetic".into(),
            thumbnail_path: None,
        };
        let pixels =
            TrackedBytesBuilder::try_zeroed(&MemoryGovernor::default(), MemoryCategory::Pages, 48)
                .unwrap()
                .finish();
        let mut image = RenderedPage::from_pixels(
            RenderKey {
                document_id: "d".into(),
                page: 1,
                region: None,
                physical_width: 3,
                physical_height: 4,
                rotation: 0,
                generation: 9,
            },
            pixels,
        )
        .unwrap();
        let valid = CoverChoice {
            page: 1,
            score: 59.0,
            confidence: 0.79,
        };
        let selected =
            AutomaticCover::new(document.clone(), "source-date".into(), valid, &image).unwrap();
        assert_eq!(selected.source_modified(), "source-date");
        assert_eq!(
            (selected.page(), selected.confidence(), selected.aspect()),
            (1, 0.79, 0.75)
        );
        for invalid in [
            CoverChoice {
                confidence: f64::NAN,
                ..valid
            },
            CoverChoice {
                score: f64::INFINITY,
                ..valid
            },
            CoverChoice {
                confidence: 0.1,
                ..valid
            },
            CoverChoice { page: 6, ..valid },
        ] {
            assert!(
                AutomaticCover::new(document.clone(), "source-date".into(), invalid, &image)
                    .is_err()
            );
        }
        image.width = 2;
        assert!(
            AutomaticCover::new(document.clone(), "source-date".into(), valid, &image).is_err()
        );
        image.width = 3;
        image.key.document_id = "other".into();
        assert!(
            AutomaticCover::new(document.clone(), "source-date".into(), valid, &image).is_err()
        );
        image.key.document_id = "d".into();
        image.key.page = 0;
        assert!(AutomaticCover::new(document, "source-date".into(), valid, &image).is_err());
    }
}
