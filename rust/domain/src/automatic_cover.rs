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
