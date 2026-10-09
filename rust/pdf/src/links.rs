use pdfium_render::prelude::*;
use std::sync::Arc;
use yamantaka_domain::{
    Failure, Result,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    pdf_links::{MAX_LINK_REGIONS, MAX_PAGE_LINKS, PdfLinkTarget, PdfPageLink, PdfPageLinks},
};

pub(crate) fn read(
    document: &PdfDocument<'_>,
    page: u32,
    governor: &MemoryGovernor,
    check_current: impl Fn() -> Result<()>,
) -> Result<Arc<PdfPageLinks>> {
    check_current()?;
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            2 * 1024 * 1024,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let page_index = i32::try_from(page)
        .map_err(|_| Failure::Invalid("PDF link page is outside the native range".into()))?;
    let native_page = document
        .pages()
        .get(page_index)
        .map_err(crate::worker::pdf_failure)?;
    let coordinates = crate::page_geometry::read(&native_page)?.coordinates();
    let annotations = native_page.annotations();
    if annotations.len() > 10_000 {
        return Err(Failure::Pdf(
            "PDF page exceeds the annotation work limit".into(),
        ));
    }
    let mut links = Vec::new();
    let mut uri_bytes = 0usize;
    let mut unsupported_targets = 0;
    let mut link_count = 0;
    let mut remaining_objects = crate::destination::MAX_CONTENT_OBJECTS;
    for index in 0..annotations.len() {
        check_current()?;
        let annotation = annotations.get(index).map_err(crate::worker::pdf_failure)?;
        let Some(annotation) = annotation.as_link_annotation() else {
            continue;
        };
        link_count += 1;
        if link_count > MAX_PAGE_LINKS {
            return Err(Failure::Pdf("PDF page exceeds the link work limit".into()));
        }
        let link = annotation.link().map_err(crate::worker::pdf_failure)?;
        let target = if let Some(destination) = link.destination() {
            Some(PdfLinkTarget::Internal(crate::destination::read(
                destination,
                document,
                &check_current,
                &mut remaining_objects,
            )?))
        } else if let Some(action) = link.action() {
            if let Some(local) = action.as_local_destination_action() {
                Some(PdfLinkTarget::Internal(crate::destination::read(
                    local.destination().map_err(crate::worker::pdf_failure)?,
                    document,
                    &check_current,
                    &mut remaining_objects,
                )?))
            } else if let Some(uri) = action.as_uri_action() {
                check_current()?;
                let value = match uri
                    .uri_with_max_bytes(yamantaka_domain::external_uri::MAX_EXTERNAL_URI_BYTES)
                {
                    Ok(value) => value,
                    Err(PdfiumError::UriLengthExceedsLimit | PdfiumError::NoUriForAction) => {
                        unsupported_targets += 1;
                        continue;
                    }
                    Err(error) => return Err(crate::worker::pdf_failure(error)),
                };
                let Ok(uri) = yamantaka_domain::external_uri::ExternalUri::new(&value) else {
                    unsupported_targets += 1;
                    continue;
                };
                uri_bytes = uri_bytes
                    .checked_add(uri.retained_bytes())
                    .ok_or(Failure::MemoryBudget)?;
                if uri_bytes > 512 * 1024 {
                    return Err(Failure::Pdf(
                        "PDF page exceeds the URI memory work limit".into(),
                    ));
                }
                Some(PdfLinkTarget::External(uri))
            } else {
                None
            }
        } else {
            None
        };
        let Some(target) = target else {
            unsupported_targets += 1;
            continue;
        };
        let points = annotation.attachment_points();
        if points.len() > MAX_LINK_REGIONS {
            return Err(Failure::Pdf(
                "PDF link exceeds the region work limit".into(),
            ));
        }
        let mut regions = Vec::with_capacity(points.len().max(1));
        for point_index in 0..points.len().max(1) {
            check_current()?;
            let rect = if points.is_empty() {
                link.rect().map_err(crate::worker::pdf_failure)?
            } else {
                let quad = points
                    .get(point_index)
                    .map_err(crate::worker::pdf_failure)?;
                if [
                    quad.x1(),
                    quad.y1(),
                    quad.x2(),
                    quad.y2(),
                    quad.x3(),
                    quad.y3(),
                    quad.x4(),
                    quad.y4(),
                ]
                .iter()
                .any(|point| !point.value.is_finite())
                {
                    return Err(Failure::Invalid(
                        "PDF link quadrilateral contains non-finite coordinates".into(),
                    ));
                }
                quad.to_rect()
            };
            if let Some(region) = coordinates.normalized_rectangle([
                f64::from(rect.left().value),
                f64::from(rect.bottom().value),
                f64::from(rect.right().value),
                f64::from(rect.top().value),
            ])? {
                regions.push(region);
            }
        }
        if !regions.is_empty() {
            links.push(PdfPageLink {
                annotation_index: index as u32,
                regions,
                target,
            });
        }
    }
    check_current()?;
    Ok(Arc::new(PdfPageLinks::from_reserved(
        page,
        links,
        unsupported_targets,
        lease,
    )?))
}
