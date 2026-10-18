//! Resolve PDF destinations without changing navigation or preferences.
use crate::{
    Failure, Result,
    pdf_destination::{DestinationRectangle, DestinationView, NavigationDestination},
    reader::ReaderPosition,
    reader_layout::ReaderLayout,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedDestination {
    pub position: ReaderPosition,
    pub zoom: f64,
}

/// The caller supplies the current viewport's top-left PDF point and effective zoom.
/// Null PDF parameters retain those values even when the destination changes page.
pub fn resolve(
    layout: &ReaderLayout,
    destination: NavigationDestination,
    viewport: [f64; 2],
    current_pdf_point: [f64; 2],
    current_zoom: f64,
) -> Result<ResolvedDestination> {
    let available = layout.destination_viewport(viewport)?;
    let size = layout
        .page_size(destination.page)
        .ok_or_else(|| Failure::Invalid("Destination page is outside the reader".into()))?;
    if current_pdf_point.iter().any(|v| !v.is_finite())
        || !current_zoom.is_finite()
        || current_zoom <= 0.0
    {
        return Err(Failure::Invalid(
            "Current destination context must be finite".into(),
        ));
    }
    let rotation = layout.rotation();
    let project = |point| rotation.rotate_point(destination.coordinates.normalized_point(point));
    let rectangle = |rect: DestinationRectangle| {
        let [left, bottom, right, top] = rect.bounds();
        let mut bounds = [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ];
        for corner in [[left, bottom], [left, top], [right, bottom], [right, top]] {
            let [x, y] = project(corner);
            bounds = [
                bounds[0].min(x),
                bounds[1].min(y),
                bounds[2].max(x),
                bounds[3].max(y),
            ];
        }
        bounds
    };
    let whole = [0.0, 0.0, 1.0, 1.0];
    let content = destination.content_bounds.map(rectangle).unwrap_or(whole);
    let top_left_pdf = destination
        .coordinates
        .pdf_point(rotation.unrotate_point([0.0, 0.0]));
    enum FitAxis {
        Both,
        Width,
        Height,
    }
    let fit = |bounds: [f64; 4], axis: FitAxis| {
        let horizontal = available[0] / ((bounds[2] - bounds[0]) * size[0]);
        let vertical = available[1] / ((bounds[3] - bounds[1]) * size[1]);
        match axis {
            FitAxis::Both => horizontal.min(vertical),
            FitAxis::Width => horizontal,
            FitAxis::Height => vertical,
        }
    };
    let fit_rectangle = |bounds: [f64; 4]| {
        let zoom = fit(bounds, FitAxis::Both);
        let left_top = [
            bounds[0] - (viewport[0] / (size[0] * zoom) - (bounds[2] - bounds[0])) / 2.0,
            bounds[1] - (viewport[1] / (size[1] * zoom) - (bounds[3] - bounds[1])) / 2.0,
        ];
        (left_top, zoom)
    };
    let view = if destination.content_bounds.is_none() {
        match destination.view {
            DestinationView::FitBounds => DestinationView::FitPage,
            DestinationView::FitBoundsWidth { top } => DestinationView::FitWidth { top },
            DestinationView::FitBoundsHeight { left } => DestinationView::FitHeight { left },
            view => view,
        }
    } else {
        destination.view
    };
    let (left_top, zoom) = match view {
        DestinationView::Unknown => ([0.0, 0.0], current_zoom),
        DestinationView::Coordinates { x, y, zoom } => {
            let zoom = match zoom {
                None | Some(0.0) => current_zoom,
                Some(value) => f64::from(value),
            };
            (
                project([
                    x.map(f64::from).unwrap_or(current_pdf_point[0]),
                    y.map(f64::from).unwrap_or(current_pdf_point[1]),
                ]),
                zoom,
            )
        }
        DestinationView::FitPage => fit_rectangle(whole),
        DestinationView::FitWidth { top } => (
            project([
                top_left_pdf[0],
                top.map(f64::from).unwrap_or(current_pdf_point[1]),
            ]),
            fit(whole, FitAxis::Width),
        ),
        DestinationView::FitHeight { left } => (
            project([
                left.map(f64::from).unwrap_or(current_pdf_point[0]),
                top_left_pdf[1],
            ]),
            fit(whole, FitAxis::Height),
        ),
        DestinationView::FitRectangle {
            left,
            bottom,
            right,
            top,
        } => {
            let bounds = rectangle(DestinationRectangle::new([
                left.into(),
                bottom.into(),
                right.into(),
                top.into(),
            ])?);
            fit_rectangle(bounds)
        }
        DestinationView::FitBounds => fit_rectangle(content),
        DestinationView::FitBoundsWidth { top } => {
            let raw = destination
                .content_bounds
                .expect("Empty bounds use page fitting")
                .bounds();
            (
                project([raw[0], top.map(f64::from).unwrap_or(current_pdf_point[1])]),
                fit(content, FitAxis::Width),
            )
        }
        DestinationView::FitBoundsHeight { left } => {
            let raw = destination
                .content_bounds
                .expect("Empty bounds use page fitting")
                .bounds();
            (
                project([left.map(f64::from).unwrap_or(current_pdf_point[0]), raw[3]]),
                fit(content, FitAxis::Height),
            )
        }
    };
    if !zoom.is_finite() || zoom <= 0.0 || left_top.iter().any(|v| !v.is_finite()) {
        return Err(Failure::Pdf(
            "Invalid PDF destination coordinates or zoom".into(),
        ));
    }
    let x = left_top[0] + viewport[0] / (2.0 * size[0] * zoom);
    let point = rotation.unrotate_point([x.clamp(0.0, 1.0), left_top[1].clamp(0.0, 1.0)]);
    let position = ReaderPosition::at(destination.page, point)?;
    layout.frame(
        viewport,
        position,
        crate::reader_preferences::ZoomMode::Custom,
        zoom,
        [0.0, 0.0],
        0.0,
    )?;
    Ok(ResolvedDestination { position, zoom })
}
