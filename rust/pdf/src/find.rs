use pdfium_render::prelude::*;
use std::sync::Arc;
use yamantaka_domain::{
    Failure, Result,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    pdf_find::{FIND_PAGE_SIZE, MatchRectangle, PdfFindQuery, PdfMatch, PdfPageMatches},
};

const MAX_PAGE_CHARACTERS: i32 = 128_000;
const MAX_MATCH_VISITS: u32 = 10_000;
const MAX_RECTANGLES: usize = 64;
const WORKING_BYTES: usize = 2 * 1024 * 1024;

pub(crate) fn read(
    document: &PdfDocument<'_>,
    page: u32,
    query: &PdfFindQuery,
    offset: u32,
    governor: &MemoryGovernor,
    check_current: impl Fn() -> Result<()>,
) -> Result<Arc<PdfPageMatches>> {
    check_current()?;
    if offset >= MAX_MATCH_VISITS {
        return Err(Failure::Invalid(
            "PDF search exceeds the per-page match work limit".into(),
        ));
    }
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            WORKING_BYTES,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let page_index = i32::try_from(page)
        .map_err(|_| Failure::Invalid("PDF search page is outside the native range".into()))?;
    let native_page = document
        .pages()
        .get(page_index)
        .map_err(super::worker::pdf_failure)?;
    let coordinates = crate::page_geometry::read(&native_page)?.coordinates();
    let text = native_page.text().map_err(super::worker::pdf_failure)?;
    if !(0..=MAX_PAGE_CHARACTERS).contains(&text.len()) {
        return Err(Failure::Pdf(
            "PDF page search exceeds the character work limit".into(),
        ));
    }
    let options = PdfSearchOptions::new()
        .match_case(query.match_case())
        .match_whole_word(query.whole_words());
    let search = text
        .search(query.text(), &options)
        .map_err(super::worker::pdf_failure)?;
    for _ in 0..offset {
        check_current()?;
        if search.find_next().is_none() {
            return Err(Failure::Invalid(
                "PDF match cursor is outside the page".into(),
            ));
        }
    }
    let mut matches = Vec::with_capacity(FIND_PAGE_SIZE);
    let mut ordinal = offset;
    let mut next_match = None;
    loop {
        check_current()?;
        let Some(segments) = search.find_next() else {
            break;
        };
        if ordinal >= MAX_MATCH_VISITS {
            return Err(Failure::Pdf(
                "PDF page search exceeds the match work limit".into(),
            ));
        }
        if matches.len() == FIND_PAGE_SIZE {
            next_match = Some(ordinal);
            break;
        }
        let count = segments.len();
        if count > MAX_RECTANGLES {
            return Err(Failure::Pdf("PDF match exceeds the rectangle limit".into()));
        }
        let mut rectangles = Vec::with_capacity(count);
        let mut snippet = String::new();
        for index in 0..count {
            check_current()?;
            let segment = segments.get(index).map_err(super::worker::pdf_failure)?;
            let bounds = segment.bounds();
            let left = f64::from(bounds.left().value);
            let right = f64::from(bounds.right().value);
            let bottom = f64::from(bounds.bottom().value);
            let top = f64::from(bounds.top().value);
            let mut min = [1.0f64; 2];
            let mut max = [0.0f64; 2];
            for point in [[left, top], [right, top], [left, bottom], [right, bottom]] {
                let point = coordinates.normalized_point(point);
                if point.into_iter().any(|value| !value.is_finite()) {
                    return Err(Failure::Pdf("PDF match geometry is invalid".into()));
                }
                for axis in 0..2 {
                    min[axis] = min[axis].min(point[axis].clamp(0.0, 1.0));
                    max[axis] = max[axis].max(point[axis].clamp(0.0, 1.0));
                }
            }
            if (0..2).all(|axis| min[axis] < max[axis]) {
                rectangles.push(MatchRectangle::new(min, max)?);
            }
            if snippet.is_empty() {
                snippet = segment.text().chars().take(160).collect();
            }
        }
        if snippet.is_empty() {
            snippet = query.text().into();
        }
        matches.push(PdfMatch {
            ordinal,
            snippet,
            rectangles,
        });
        ordinal += 1;
    }
    check_current()?;
    Ok(Arc::new(PdfPageMatches::from_reserved(
        page, matches, next_match, lease,
    )?))
}
