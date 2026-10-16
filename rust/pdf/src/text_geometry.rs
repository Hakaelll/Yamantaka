use pdfium_render::prelude::*;
use std::sync::Arc;
use yamantaka_domain::{
    Failure, Result,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    pdf_text::{MAX_PAGE_TEXT_CHARACTERS, PdfPageText, TextCharacter},
};

pub(crate) fn read(
    document: &PdfDocument<'_>,
    page: u32,
    character_limit: usize,
    governor: &MemoryGovernor,
    check_current: impl Fn() -> Result<()>,
) -> Result<Arc<PdfPageText>> {
    check_current()?;
    let mut lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            4096,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let page_index = i32::try_from(page)
        .map_err(|_| Failure::Invalid("PDF text page is outside the native range".into()))?;
    let native_page = document
        .pages()
        .get(page_index)
        .map_err(crate::worker::pdf_failure)?;
    let coordinates = crate::page_geometry::read(&native_page)?.coordinates();
    let native_text = native_page.text().map_err(crate::worker::pdf_failure)?;
    let count = usize::try_from(native_text.len())
        .map_err(|_| Failure::Pdf("PDF character count is invalid".into()))?;
    if count > character_limit.min(MAX_PAGE_TEXT_CHARACTERS) {
        return Err(Failure::TextWorkLimit);
    }
    // The SDK's character collection also allocates one i32 index per character.
    let working_bytes = count
        .checked_mul(std::mem::size_of::<TextCharacter>() + std::mem::size_of::<i32>() + 4)
        .and_then(|bytes| bytes.checked_add(4096))
        .ok_or(Failure::MemoryBudget)?;
    lease
        .try_resize(working_bytes)
        .map_err(|_| Failure::MemoryBudget)?;
    check_current()?;
    let native_characters = native_text.chars();
    let mut text = String::with_capacity(count * 4);
    let mut characters = Vec::with_capacity(count);
    for index in 0..count {
        check_current()?;
        let native = native_characters
            .get(index)
            .map_err(crate::worker::pdf_failure)?;
        let start = text.len() as u32;
        text.push(
            native
                .unicode_char()
                .filter(|value| *value != '\0')
                .unwrap_or('\u{fffd}'),
        );
        let region = native.tight_bounds().ok().and_then(|bounds| {
            coordinates
                .normalized_rectangle([
                    f64::from(bounds.left().value),
                    f64::from(bounds.bottom().value),
                    f64::from(bounds.right().value),
                    f64::from(bounds.top().value),
                ])
                .ok()
                .flatten()
        });
        characters.push(TextCharacter {
            bytes: start..text.len() as u32,
            region,
        });
    }
    drop(native_characters);
    check_current()?;
    Ok(Arc::new(PdfPageText::from_reserved(
        page, text, characters, lease,
    )?))
}
