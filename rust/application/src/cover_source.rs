use std::{path::PathBuf, sync::Arc};
use yamantaka_domain::{
    Cancellation, Document, Failure, LibraryRepository, RenderedPage, Result,
    cover_snapshot::CoverSnapshot,
};

#[derive(Clone)]
pub struct CoverSource {
    id: String,
    path: PathBuf,
    fingerprint: String,
    pages: u32,
    missing: bool,
}
impl CoverSource {
    pub fn capture(document: &Document) -> Self {
        Self {
            id: document.id.clone(),
            path: document.path.clone(),
            fingerprint: document.fingerprint.clone(),
            pages: document.page_count,
            missing: document.missing,
        }
    }
    pub(crate) fn id(&self) -> &str {
        &self.id
    }
    pub(crate) fn path(&self) -> PathBuf {
        self.path.clone()
    }
    pub(crate) fn read(&self, repository: &impl LibraryRepository) -> Result<Document> {
        let document = repository
            .get(&self.id)?
            .ok_or_else(|| Failure::Invalid("Cover document no longer exists".into()))?;
        if self.missing
            || document.missing
            || document.id != self.id
            || document.path != self.path
            || document.fingerprint != self.fingerprint
            || document.page_count != self.pages
        {
            return Err(Failure::Invalid(
                "Cover source changed; refresh the document before retrying".into(),
            ));
        }
        Ok(document)
    }
}

pub(crate) struct CoverFrame {
    source: CoverSource,
    page: u32,
    generation: u64,
    selection: Option<CoverSnapshot>,
}
impl CoverFrame {
    pub fn new(
        source: CoverSource,
        page: u32,
        generation: u64,
        selection: Option<CoverSnapshot>,
    ) -> Self {
        Self {
            source,
            page,
            generation,
            selection,
        }
    }
    pub fn id(&self) -> &str {
        self.source.id()
    }
    pub fn finish(
        self,
        repository: &impl LibraryRepository,
        image: Arc<RenderedPage>,
        cancel: &Cancellation,
    ) -> Result<Arc<RenderedPage>> {
        cancel.check()?;
        self.source.read(repository)?;
        if let Some(snapshot) = &self.selection
            && repository.cover_snapshot(self.id())? != *snapshot
        {
            return Err(Failure::Invalid(
                "Cover selection changed while rendering".into(),
            ));
        }
        if self.page >= self.source.pages
            || image.key.document_id != self.source.id
            || image.key.generation != self.generation
            || image.key.page != self.page
            || image.key.rotation != 0
            || image.key.region.is_some()
            || image.width == 0
            || image.height == 0
            || image.width > 512
            || image.height > 512
            || image.key.output_dimensions()? != [image.width, image.height]
            || image.pixels().len() != image.width as usize * image.height as usize * 4
        {
            return Err(Failure::Invalid(
                "Cover image does not match its requested source, page or generation".into(),
            ));
        }
        cancel.check()?;
        Ok(image)
    }
}
