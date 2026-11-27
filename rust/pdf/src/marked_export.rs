use pdfium_render::prelude::*;
use std::{io::Write, path::Path};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    annotations::{AnnotationColor, AnnotationKind},
    study_library::{PdfMark, PdfMarks},
};

pub(crate) fn prepare(
    pdfium: &Pdfium,
    source: &PdfDocument,
    request: &PdfMarks,
    mut marks: Vec<PdfMark>,
    cancel: &Cancellation,
) -> Result<tempfile::NamedTempFile> {
    let range = &request.pages;
    if range.is_empty() || range.end > source.pages().len() as u32 {
        return Err(Failure::Invalid(
            "The export page range is outside the PDF".into(),
        ));
    }
    let extension = request.destination.extension().and_then(|ext| ext.to_str());
    if !extension.is_some_and(|ext| ext.eq_ignore_ascii_case("pdf")) {
        return Err(Failure::Invalid(
            "The copy requires a .pdf extension".into(),
        ));
    }
    if request.destination.symlink_metadata().is_ok() {
        return Err(Failure::DestinationOccupied);
    }
    let folder = request
        .destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut target = pdfium
        .create_new_pdf()
        .map_err(super::worker::pdf_failure)?;
    // Import all selected pages together so PDFium can preserve shared resources.
    let pages = format!("{}-{}", range.start + 1, range.end);
    target
        .pages_mut()
        .copy_pages_from_document(source, &pages, 0)
        .map_err(super::worker::pdf_failure)?;
    marks.sort_by_key(|mark| mark.page);
    let mut marks = marks.iter().peekable();
    while let Some(first) = marks.peek() {
        cancel.check()?;
        let page_number = first.page;
        if !range.contains(&page_number) {
            return Err(Failure::Invalid("Mark outside export range".into()));
        }
        let mut page = target
            .pages()
            .get((page_number - range.start) as i32)
            .map_err(super::worker::pdf_failure)?;
        let geometry = crate::page_geometry::read(&page)?.coordinates();
        page.set_content_regeneration_strategy(PdfPageContentRegenerationStrategy::Manual);
        while let Some(mark) = marks.next_if(|mark| mark.page == page_number) {
            cancel.check()?;
            let point = |p: [f64; 2]| geometry.pdf_point(p).map(|value| value as f32);
            let objects = page.objects_mut();
            let color = color(mark.color);
            let stroke = PdfPoints::new(
                mark.drawing
                    .as_ref()
                    .map_or(1.4, |shape| shape.weight().factor() * 2.0),
            );
            let rect = |a: [f64; 2], b: [f64; 2]| {
                let [a, b] = [point(a), point(b)];
                PdfRect::new_from_values(
                    a[1].min(b[1]),
                    a[0].min(b[0]),
                    a[1].max(b[1]),
                    a[0].max(b[0]),
                )
            };
            if let Some(shape) = &mark.drawing {
                let points = shape.points();
                match mark.kind {
                    AnnotationKind::Rectangle => {
                        objects
                            .create_path_object_rect(
                                rect(points[0], points[1]),
                                Some(color),
                                Some(stroke),
                                None,
                            )
                            .map_err(super::worker::pdf_failure)?;
                    }
                    AnnotationKind::Ellipse => {
                        objects
                            .create_path_object_ellipse(
                                rect(points[0], points[1]),
                                Some(color),
                                Some(stroke),
                                None,
                            )
                            .map_err(super::worker::pdf_failure)?;
                    }
                    AnnotationKind::Ink | AnnotationKind::Line | AnnotationKind::Arrow => {
                        let a = point(points[0]);
                        let b = point(points[1]);
                        let [ax, ay, bx, by] = [a[0], a[1], b[0], b[1]].map(PdfPoints::new);
                        let mut path =
                            PdfPagePathObject::new_line(&target, ax, ay, bx, by, color, stroke)
                                .map_err(super::worker::pdf_failure)?;
                        for next in points.iter().skip(2) {
                            let [x, y] = point(*next).map(PdfPoints::new);
                            path.line_to(x, y).map_err(super::worker::pdf_failure)?;
                        }
                        if mark.kind == AnnotationKind::Arrow {
                            let angle = (b[1] - a[1]).atan2(b[0] - a[0]);
                            let size = ((b[0] - a[0]).hypot(b[1] - a[1]) * 0.25).min(11.0);
                            for direction in [angle - 0.55, angle + 0.55] {
                                path.move_to(bx, by).map_err(super::worker::pdf_failure)?;
                                path.line_to(
                                    PdfPoints::new(b[0] - size * direction.cos()),
                                    PdfPoints::new(b[1] - size * direction.sin()),
                                )
                                .map_err(super::worker::pdf_failure)?;
                            }
                        }
                        objects
                            .add_path_object(path)
                            .map_err(super::worker::pdf_failure)?;
                    }
                    _ => {}
                }
            } else {
                for bounds in &mark.rects {
                    let bounds = bounds.region();
                    if mark.kind == AnnotationKind::Highlight {
                        objects
                            .create_path_object_rect(
                                rect(bounds.min(), bounds.max()),
                                None,
                                None,
                                Some(color.with_alpha(90)),
                            )
                            .map_err(super::worker::pdf_failure)?;
                    } else if mark.kind == AnnotationKind::Underline {
                        let [x, y] = bounds.max();
                        let [a, b] = [point([bounds.min()[0], y]), point([x, y])];
                        let [ax, ay, bx, by] = [a[0], a[1], b[0], b[1]].map(PdfPoints::new);
                        objects
                            .create_path_object_line(ax, ay, bx, by, color, stroke)
                            .map_err(super::worker::pdf_failure)?;
                    }
                }
            }
        }
        page.regenerate_content()
            .map_err(super::worker::pdf_failure)?;
    }
    cancel.check()?;
    let mut temporary = tempfile::NamedTempFile::new_in(folder).map_err(io_failure)?;
    target
        .save_to_writer(temporary.as_file_mut())
        .map_err(super::worker::pdf_failure)?;
    temporary.as_file_mut().flush().map_err(io_failure)?;
    temporary.as_file_mut().sync_all().map_err(io_failure)?;
    Ok(temporary)
}

pub(crate) fn publish(temporary: tempfile::NamedTempFile, target: &Path) -> Result<()> {
    temporary
        .persist_noclobber(target)
        .map_err(|error| match error.error.kind() {
            std::io::ErrorKind::AlreadyExists => Failure::DestinationOccupied,
            _ => io_failure(error),
        })?;
    Ok(())
}

fn io_failure(_: impl std::fmt::Display) -> Failure {
    Failure::Storage("The marked PDF could not be saved".into())
}

fn color(color: AnnotationColor) -> PdfColor {
    let [r, g, b] = match color {
        AnnotationColor::Yellow => [235, 182, 42],
        AnnotationColor::Green => [42, 159, 106],
        AnnotationColor::Blue => [48, 122, 213],
        AnnotationColor::Rose => [213, 79, 131],
        AnnotationColor::Violet => [127, 95, 200],
    };
    PdfColor::new(r, g, b, 255)
}
