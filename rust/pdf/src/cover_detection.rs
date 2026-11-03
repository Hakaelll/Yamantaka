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
    source.version.with_source(source.path, || {
        cancel.check()?;
        let cache = cache.filter(|_| existing.is_none());
        let revision = cache.and_then(|cache| cache.identity(source.path, 0, cancel).ok());
        let opened;
        let document = if let Some(existing) = existing {
            existing
        } else {
            opened = source.version.open_pdf(pdfium, source.path, cancel)?;
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
                    version: source.version,
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
                version: source.version,
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
    })
}

/// The importing actor holds a full-content read guard until confirmed close and persistence.
/// Other existing-handle rendering keeps its stricter no-cache policy.
pub(crate) fn capture_import<'a>(
    pdfium: &'a Pdfium,
    loaded: &crate::source::LoadedPdf<'a>,
    id: String,
    expected: &yamantaka_domain::source_revision::SourceRevision,
    cancel: &Cancellation,
    governor: &MemoryGovernor,
    cache: Option<&dyn yamantaka_domain::cover_cache::CoverImageCache>,
) -> Result<DetectedCover> {
    cancel.check()?;
    if loaded.source.snapshot() != *expected {
        return Err(crate::source::source_changed());
    }
    loaded.source.verify(&expected.canonical_path, cancel)?;
    let detected = detect(
        pdfium,
        Some(&loaded.document),
        crate::covers::CoverSource {
            version: &loaded.source,
            id,
            path: &expected.canonical_path,
            page: 0,
            generation: 0,
        },
        cancel,
        governor,
        None,
    )?;
    loaded.source.verify(&expected.canonical_path, cancel)?;
    if let Some(cache) = cache
        && let Ok(identity) = cache.identity(&expected.canonical_path, detected.choice.page, cancel)
    {
        loaded.source.verify(&expected.canonical_path, cancel)?;
        let _ = cache.save(identity, &detected.image, cancel);
        loaded.source.verify(&expected.canonical_path, cancel)?;
    }
    cancel.check()?;
    Ok(detected)
}

pub(crate) fn capture_page<'a>(
    pdfium: &'a Pdfium,
    loaded: &crate::source::LoadedPdf<'a>,
    source: crate::covers::CoverSource<'_>,
    expected: &yamantaka_domain::source_revision::SourceRevision,
    cancel: &Cancellation,
    governor: &MemoryGovernor,
    cache: Option<&dyn yamantaka_domain::cover_cache::CoverImageCache>,
) -> Result<std::sync::Arc<yamantaka_domain::RenderedPage>> {
    cancel.check()?;
    if loaded.source.snapshot() != *expected {
        return Err(crate::source::source_changed());
    }
    loaded.source.verify(source.path, cancel)?;
    let cache = cache.ok_or_else(|| Failure::Unsupported("Cover cache is disabled".into()))?;
    let identity = cache.identity(source.path, source.page, cancel)?;
    let image = crate::covers::render(
        pdfium,
        Some(&loaded.document),
        crate::covers::CoverSource {
            version: &loaded.source,
            id: source.id.clone(),
            path: source.path,
            page: source.page,
            generation: source.generation,
        },
        cancel,
        governor,
        None,
    )?;
    loaded.source.verify(source.path, cancel)?;
    if cache.identity(source.path, source.page, cancel)? != identity {
        return Err(crate::source::source_changed());
    }
    cache.save(identity, &image, cancel)?;
    loaded.source.verify(source.path, cancel)?;
    cancel.check()?;
    Ok(image)
}
