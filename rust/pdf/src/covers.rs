use pdfium_render::prelude::*;
use std::{path::Path, sync::Arc};
use yamantaka_domain::memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder};
use yamantaka_domain::{Cancellation, Failure, RenderKey, RenderedPage, Result};

pub(crate) struct CoverSource<'a> {
    pub id: String,
    pub path: &'a Path,
    pub page: u32,
    pub generation: u64,
}

pub(crate) fn render<'a>(
    pdfium: &'a Pdfium,
    existing: Option<&PdfDocument<'a>>,
    source: CoverSource<'_>,
    cancel: &Cancellation,
    governor: &MemoryGovernor,
) -> Result<Arc<RenderedPage>> {
    cancel.check()?;
    // Temporary handles stay on the native owner and are dropped on every exit.
    let opened;
    let document = if let Some(existing) = existing {
        existing
    } else {
        opened = pdfium
            .load_pdf_from_file(source.path, None)
            .map_err(super::worker::pdf_failure)?;
        &opened
    };
    cancel.check()?;
    let page = document
        .pages()
        .get(
            source
                .page
                .try_into()
                .map_err(|_| Failure::Invalid("Cover page index out of range".into()))?,
        )
        .map_err(super::worker::pdf_failure)?;
    let size = super::page_geometry::read(&page)?.size();
    let longest = size[0].max(size[1]);
    let dimension = |value: f32| ((value / longest * 512.0).round() as u32).clamp(1, 512);
    let key = RenderKey {
        document_id: source.id,
        page: source.page,
        region: None,
        physical_width: dimension(size[0]),
        physical_height: dimension(size[1]),
        rotation: 0,
        generation: source.generation,
    };
    let bytes = key.byte_size(512, 512 * 512 * 4)?;
    let mut pixels = TrackedBytesBuilder::try_zeroed(governor, MemoryCategory::Pages, bytes)
        .map_err(|_| Failure::MemoryBudget)?;
    cancel.check()?;
    {
        let mut bitmap = PdfBitmap::from_bytes(
            key.physical_width as i32,
            key.physical_height as i32,
            PdfBitmapFormat::BGRA,
            pixels.as_mut_slice(),
        )
        .map_err(super::worker::pdf_failure)?;
        let config = PdfRenderConfig::new()
            .set_fixed_size(key.physical_width as i32, key.physical_height as i32)
            .set_reverse_byte_order(false);
        page.render_into_bitmap_with_config(&mut bitmap, &config)
            .map_err(super::worker::pdf_failure)?;
    }
    cancel.check()?;
    for pixel in pixels.as_mut_slice().chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Ok(Arc::new(RenderedPage::from_pixels(key, pixels.finish())?))
}
