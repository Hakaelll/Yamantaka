use pdfium_render::prelude::*;
use yamantaka_domain::{Result, outline::PageCoordinateSpace, pdf_pages::PdfPageInfo};

pub(crate) fn read(page: &PdfPage<'_>) -> Result<PdfPageInfo> {
    // DeviceToPage derives crop and intrinsic-rotation geometry without a bitmap.
    let config = PdfRenderConfig::new().set_target_size(10_000, 10_000);
    let corner = |x, y| -> Result<[f64; 2]> {
        let (x, y) = page
            .pixels_to_points(x, y, &config)
            .map_err(super::worker::pdf_failure)?;
        Ok([f64::from(x.value), f64::from(y.value)])
    };
    let coordinates =
        PageCoordinateSpace::new(corner(0, 0)?, corner(10_000, 0)?, corner(0, 10_000)?)?;
    PdfPageInfo::new([page.width().value, page.height().value], coordinates)
}
