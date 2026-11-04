use super::*;
use unicode_segmentation::UnicodeSegmentation;

pub(super) fn at(
    page: &PdfPageText,
    character: usize,
) -> Option<(SelectionPosition, SelectionPosition)> {
    let byte = page.characters().get(character)?.bytes.start as usize;
    let (start, text) = page
        .text()
        .split_word_bound_indices()
        .find(|(start, text)| *start <= byte && byte < *start + text.len())?;
    let position = |byte| {
        let index = if byte == page.text().len() {
            page.characters().len()
        } else {
            page.characters()
                .binary_search_by_key(&byte, |character| character.bytes.start as usize)
                .ok()?
        };
        SelectionPosition::new(page.page(), index).ok()
    };
    Some((
        keyboard::snap_position(page, position(start)?, false)?,
        keyboard::snap_position(page, position(start + text.len())?, true)?,
    ))
}
