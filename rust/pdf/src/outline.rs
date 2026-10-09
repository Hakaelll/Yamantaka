use pdfium_render::prelude::*;
use std::{collections::HashMap, sync::Arc};
use yamantaka_domain::{
    Failure, Result,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    outline::{DestinationView, OutlineDestination, OutlineEntry, PageCoordinateSpace, PdfOutline},
};

const MAX_ENTRIES: usize = 10_000;
const MAX_TITLE_BYTES: usize = 4096;
const MAX_TOTAL_TITLE_BYTES: usize = 1024 * 1024;
const WORKING_BYTES: usize = 4 * 1024 * 1024;

pub(crate) fn read(
    document: &PdfDocument<'_>,
    governor: &MemoryGovernor,
    check_current: impl Fn() -> Result<()>,
) -> Result<Arc<PdfOutline>> {
    check_current()?;
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            WORKING_BYTES,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut entries = Vec::new();
    let mut title_bytes = 0usize;
    {
        let bookmarks = document.bookmarks();
        let mut indices = HashMap::new();
        for bookmark in bookmarks.iter() {
            check_current()?;
            if entries.len() == MAX_ENTRIES {
                return Err(Failure::Pdf("PDF outline exceeds the entry limit".into()));
            }
            let title = bookmark.title().unwrap_or_default();
            title_bytes = title_bytes
                .checked_add(title.capacity())
                .ok_or(Failure::MemoryBudget)?;
            if title.len() > MAX_TITLE_BYTES || title_bytes > MAX_TOTAL_TITLE_BYTES {
                return Err(Failure::Pdf(
                    "PDF outline titles exceed the memory limit".into(),
                ));
            }
            let parent = bookmark
                .parent()
                .and_then(|parent| indices.get(&parent).copied());
            let destination = if let Some(destination) = bookmark.destination() {
                Some(destination_value(destination, document)?)
            } else if let Some(action) = bookmark.action() {
                action
                    .as_local_destination_action()
                    .map(|action| {
                        action
                            .destination()
                            .map_err(super::worker::pdf_failure)
                            .and_then(|destination| destination_value(destination, document))
                    })
                    .transpose()?
            } else {
                None
            };
            indices.insert(bookmark, entries.len());
            entries.push(OutlineEntry {
                title,
                parent,
                destination,
            });
        }
    }
    check_current()?;
    Ok(Arc::new(PdfOutline::from_reserved(entries, lease)?))
}

fn destination_value(
    destination: PdfDestination<'_>,
    document: &PdfDocument<'_>,
) -> Result<OutlineDestination> {
    let native_index = destination
        .page_index()
        .map_err(super::worker::pdf_failure)?;
    let page = u32::try_from(native_index)
        .map_err(|_| Failure::Pdf("PDF outline destination has a negative page".into()))?;
    if page >= document.pages().len() as u32 {
        return Err(Failure::Pdf(
            "PDF outline destination is outside the document".into(),
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
    let config = PdfRenderConfig::new().set_target_size(10_000, 10_000);
    let corner = |x, y| -> Result<[f64; 2]> {
        let (x, y) = native_page
            .pixels_to_points(x, y, &config)
            .map_err(super::worker::pdf_failure)?;
        Ok([f64::from(x.value), f64::from(y.value)])
    };
    let coordinates =
        PageCoordinateSpace::new(corner(0, 0)?, corner(10_000, 0)?, corner(0, 10_000)?)?;
    Ok(OutlineDestination {
        page,
        view,
        coordinates,
    })
}
fn finite(value: f32) -> Result<f32> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Failure::Pdf(
            "PDF outline destination contains a non-finite value".into(),
        ))
    }
}
fn optional(value: Option<f32>) -> Result<Option<f32>> {
    value.map(finite).transpose()
}
