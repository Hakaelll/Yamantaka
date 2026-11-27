use image::{ImageBuffer, Rgba, imageops::sample_bilinear};
use std::sync::Arc;
use yamantaka_domain::{
    Failure, RenderedPage, Result,
    memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
};

// A full 80-book catalogue page uses at most 31.25 MiB of textures, even for square covers.
const SIDE: u32 = 320;

pub(super) fn dimensions(aspect: f64) -> [u32; 2] {
    if aspect > 1.0 {
        [SIDE, (f64::from(SIDE) / aspect).round().max(1.0) as u32]
    } else {
        [(f64::from(SIDE) * aspect).round().max(1.0) as u32, SIDE]
    }
}

pub(super) fn prepare(
    governor: &MemoryGovernor,
    page: &Arc<RenderedPage>,
) -> Result<Arc<RenderedPage>> {
    if page.width.max(page.height) <= SIDE {
        return Ok(page.clone());
    }
    let [width, height] = dimensions(f64::from(page.width) / f64::from(page.height));
    let source = ImageBuffer::<Rgba<u8>, _>::from_raw(page.width, page.height, page.pixels())
        .expect("rendered page has exactly one RGBA pixel per coordinate");
    let mut key = page.key.clone();
    key.physical_width = width;
    key.physical_height = height;
    let bytes = key.byte_size(SIDE, usize::MAX)?;
    let mut pixels = TrackedBytesBuilder::try_zeroed(governor, MemoryCategory::Pages, bytes)
        .map_err(|_| Failure::MemoryBudget)?;
    for (y, row) in pixels
        .as_mut_slice()
        .chunks_exact_mut(width as usize * 4)
        .enumerate()
    {
        let v = (y as f32 + 0.5) / height as f32;
        for (x, pixel) in row.chunks_exact_mut(4).enumerate() {
            let u = (x as f32 + 0.5) / width as f32;
            pixel.copy_from_slice(&sample_bilinear(&source, u, v).unwrap().0);
        }
    }
    Ok(Arc::new(RenderedPage::from_pixels(key, pixels.finish())?))
}
