use pdfium_render::prelude::*;
use yamantaka_domain::{
    Failure, Result,
    pdf_destination::{DestinationRectangle, DestinationView, NavigationDestination},
};

pub(crate) const MAX_CONTENT_OBJECTS: usize = 100_000;

pub(crate) fn read(
    destination: PdfDestination<'_>,
    document: &PdfDocument<'_>,
    check_current: &impl Fn() -> Result<()>,
    remaining_objects: &mut usize,
) -> Result<NavigationDestination> {
    let native_index = destination
        .page_index()
        .map_err(super::worker::pdf_failure)?;
    let page = u32::try_from(native_index)
        .map_err(|_| Failure::Pdf("PDF destination has a negative page".into()))?;
    if page >= document.pages().len() as u32 {
        return Err(Failure::Pdf(
            "PDF destination is outside the document".into(),
        ));
    }
    let settings = destination
        .view_settings()
        .map_err(super::worker::pdf_failure)?;
    let point = |value: Option<PdfPoints>| value.map(|value| value.value);
    let view = match settings {
        PdfDestinationViewSettings::Unknown => DestinationView::Unknown,
        PdfDestinationViewSettings::SpecificCoordinatesAndZoom(x, y, zoom) => {
            let x = point(x);
            let y = point(y);
            for value in [x, y, zoom].into_iter().flatten() {
                finite(value)?;
            }
            DestinationView::Coordinates { x, y, zoom }
        }
        PdfDestinationViewSettings::FitPageToWindow => DestinationView::FitPage,
        PdfDestinationViewSettings::FitPageHorizontallyToWindow(top) => DestinationView::FitWidth {
            top: optional(point(top))?,
        },
        PdfDestinationViewSettings::FitPageVerticallyToWindow(left) => DestinationView::FitHeight {
            left: optional(point(left))?,
        },
        PdfDestinationViewSettings::FitPageToRectangle(rect) => DestinationView::FitRectangle {
            left: finite(rect.left().value)?,
            bottom: finite(rect.bottom().value)?,
            right: finite(rect.right().value)?,
            top: finite(rect.top().value)?,
        },
        PdfDestinationViewSettings::FitBoundsToWindow => DestinationView::FitBounds,
        PdfDestinationViewSettings::FitBoundsHorizontallyToWindow(top) => {
            DestinationView::FitBoundsWidth {
                top: optional(point(top))?,
            }
        }
        PdfDestinationViewSettings::FitBoundsVerticallyToWindow(left) => {
            DestinationView::FitBoundsHeight {
                left: optional(point(left))?,
            }
        }
    };
    let native_page = document
        .pages()
        .get(native_index)
        .map_err(super::worker::pdf_failure)?;
    let coordinates = super::page_geometry::read(&native_page)?.coordinates();
    let content_bounds = if matches!(
        view,
        DestinationView::FitBounds
            | DestinationView::FitBoundsWidth { .. }
            | DestinationView::FitBoundsHeight { .. }
    ) {
        let objects = native_page.objects();
        *remaining_objects = remaining_objects
            .checked_sub(objects.len())
            .ok_or_else(|| {
                Failure::Pdf("PDF destination content exceeds the object work limit".into())
            })?;
        let crop = native_page
            .boundaries()
            .bounding()
            .map_err(super::worker::pdf_failure)?
            .bounds;
        let mut bounds: Option<[f64; 4]> = None;
        for object in objects.iter() {
            check_current()?;
            let rect = object.bounds().map_err(super::worker::pdf_failure)?;
            let next = [
                f64::from(finite(rect.left().value)?),
                f64::from(finite(rect.bottom().value)?),
                f64::from(finite(rect.right().value)?),
                f64::from(finite(rect.top().value)?),
            ];
            bounds = Some(match bounds {
                None => next,
                Some(old) => [
                    old[0].min(next[0]),
                    old[1].min(next[1]),
                    old[2].max(next[2]),
                    old[3].max(next[3]),
                ],
            });
        }
        let bounds = bounds.map(|b| {
            [
                b[0].max(f64::from(crop.left().value)),
                b[1].max(f64::from(crop.bottom().value)),
                b[2].min(f64::from(crop.right().value)),
                b[3].min(f64::from(crop.top().value)),
            ]
        });
        bounds
            .filter(|b| b[0] < b[2] && b[1] < b[3])
            .map(DestinationRectangle::new)
            .transpose()?
    } else {
        None
    };
    Ok(NavigationDestination {
        page,
        view,
        coordinates,
        content_bounds,
    })
}
fn finite(value: f32) -> Result<f32> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Failure::Pdf(
            "PDF destination contains a non-finite value".into(),
        ))
    }
}
fn optional(value: Option<f32>) -> Result<Option<f32>> {
    value.map(finite).transpose()
}
