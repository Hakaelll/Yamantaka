use std::sync::Arc;
use yamantaka_domain::{
    Document, Failure, LibraryRepository, RenderedPage, Result, automatic_cover::AutomaticCover,
    cover_policy::DetectedCover,
};

pub(crate) enum CoverCompletion {
    Image(Arc<RenderedPage>),
    StoredPage(u32),
}
pub(crate) fn complete(
    repository: &mut impl LibraryRepository,
    document: Document,
    generation: u64,
    detected: DetectedCover,
) -> Result<CoverCompletion> {
    if detected.image.key.generation != generation {
        return Err(Failure::Invalid("Stale automatic cover generation".into()));
    }
    let choice = AutomaticCover::new(document, detected.choice, &detected.image)?;
    if repository.save_automatic_cover(&choice)? {
        return Ok(CoverCompletion::Image(detected.image));
    }
    // A newer manual or automatic choice wins over background detection.
    match repository.cover_source_page(&choice.document().id)? {
        Some(page) if page == choice.page() => Ok(CoverCompletion::Image(detected.image)),
        Some(page) => Ok(CoverCompletion::StoredPage(page)),
        None => Err(Failure::Invalid(
            "Cover choice changed during completion".into(),
        )),
    }
}

#[cfg(test)]
mod tests;
