use pdfium_render::prelude::*;
use yamantaka_domain::{
    Cancellation, Failure, Result,
    cover_policy::{CoverMetrics, DetectedCover, MAX_COVER_CANDIDATES, choose},
    memory::MemoryGovernor,
};

pub(crate) fn detect<'a>(
    pdfium: &'a Pdfium,
    existing: Option<&PdfDocument<'a>>,
    source: crate::covers::CoverSource<'_>,
    cancel: &Cancellation,
    governor: &MemoryGovernor,
    cache: Option<&dyn yamantaka_domain::cover_cache::CoverImageCache>,
) -> Result<DetectedCover> {
    cancel.check()?;
    let cache = cache.filter(|_| existing.is_none());
    let revision = cache.and_then(|cache| cache.identity(source.path, 0, cancel).ok());
    let opened;
    let document = if let Some(existing) = existing {
        existing
    } else {
        opened = pdfium
            .load_pdf_from_file(source.path, None)
            .map_err(crate::worker::pdf_failure)?;
        &opened
    };
    let count = usize::try_from(document.pages().len())
        .map_err(|_| Failure::Pdf("Invalid PDF page count".into()))?
        .min(MAX_COVER_CANDIDATES);
    let mut candidates = Vec::with_capacity(count);
    for page in 0..count as u32 {
        cancel.check()?;
        let image = crate::covers::render(
            pdfium,
            Some(document),
            crate::covers::CoverSource {
                id: source.id.clone(),
                path: source.path,
                page,
                generation: source.generation,
            },
            cancel,
            governor,
            None,
        )?;
        let text = match crate::text_geometry::read(
            document,
            page,
            yamantaka_domain::pdf_text::MAX_PAGE_TEXT_CHARACTERS,
            governor,
            || cancel.check(),
        ) {
            Ok(text) => Some(text),
            Err(error @ (Failure::Cancelled | Failure::MemoryBudget)) => return Err(error),
            Err(_) => None,
        };
        candidates.push(CoverMetrics::measure(
            page,
            image.width,
            image.height,
            image.pixels(),
            text.as_ref().map_or("", |text| text.text()),
            cancel,
        )?);
        // Drop each governed preview and its text before allocating the next.
    }
    let choice = choose(&candidates)?;
    cancel.check()?;
    let image = crate::covers::render(
        pdfium,
        Some(document),
        crate::covers::CoverSource {
            id: source.id.clone(),
            path: source.path,
            page: choice.page,
            generation: source.generation,
        },
        cancel,
        governor,
        None,
    )?;
    if let (Some(cache), Some(revision)) = (cache, revision)
        && cache.identity(source.path, 0, cancel).ok() == Some(revision)
        && let Ok(identity) = cache.identity(source.path, choice.page, cancel)
        && cache.identity(source.path, 0, cancel).ok() == Some(revision)
    {
        let _ = cache.save(identity, &image, cancel);
    }
    cancel.check()?;
    Ok(DetectedCover { image, choice })
}
