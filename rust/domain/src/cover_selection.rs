use crate::{Document, Failure, Result};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct ManualCover {
    document: String,
    path: PathBuf,
    fingerprint: String,
    page: u32,
    aspect: f64,
}
impl ManualCover {
    pub fn new(document: &Document, page: u32, aspect: f64) -> Result<Self> {
        if page >= document.page_count || !aspect.is_finite() || !(0.1..=10.0).contains(&aspect) {
            return Err(Failure::Invalid(
                "Invalid cover page or aspect ratio".into(),
            ));
        }
        Ok(Self {
            document: document.id.clone(),
            path: document.path.clone(),
            fingerprint: document.fingerprint.clone(),
            page,
            aspect,
        })
    }
    pub fn document(&self) -> &str {
        &self.document
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
    pub fn page(&self) -> u32 {
        self.page
    }
    pub fn aspect(&self) -> f64 {
        self.aspect
    }
}
