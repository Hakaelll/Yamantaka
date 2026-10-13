use pdfium_render::prelude::*;
use std::path::Path;
use yamantaka_domain::{
    Cancellation, Failure, Result,
    cover_policy::{CoverMetrics, DetectedCover, MAX_COVER_CANDIDATES, choose},
    memory::MemoryGovernor,
};

pub(crate) fn detect<'a>(
    pdfium: &'a Pdfium,
    existing: Option<&PdfDocument<'a>>,
    id: &str,
    path: &Path,
    generation: u64,
    cancel: &Cancellation,
    governor: &MemoryGovernor,
) -> Result<DetectedCover> {
    cancel.check()?;
    let opened;
    let document = if let Some(existing) = existing {
        existing
    } else {
        opened = pdfium
            .load_pdf_from_file(path, None)
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
                id: id.into(),
                path,
                page,
                generation,
            },
            cancel,
            governor,
            None,
        )?;
        let text = match crate::text_geometry::read(document, page, governor, || cancel.check()) {
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
            id: id.into(),
            path,
            page: choice.page,
            generation,
        },
        cancel,
        governor,
        None,
    )?;
    Ok(DetectedCover { image, choice })
}
