use crate::{Failure, Result, memory::ByteLease, pdf_geometry::NormalizedPageRect};
use std::ops::Range;

pub const MAX_PAGE_TEXT_CHARACTERS: usize = 128_000;

#[derive(Debug)]
pub struct TextCharacter {
    pub bytes: Range<u32>,
    pub region: Option<NormalizedPageRect>,
}

/// Native character order with explicit UTF-8 boundaries; no SDK handles escape.
#[derive(Debug)]
pub struct PdfPageText {
    page: u32,
    text: String,
    characters: Vec<TextCharacter>,
    _lease: ByteLease,
}

impl PdfPageText {
    pub fn page(&self) -> u32 {
        self.page
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn characters(&self) -> &[TextCharacter] {
        &self.characters
    }
    /// Half-open native character indices, independent of UTF-8 byte length.
    pub fn selected_text(&self, range: Range<usize>) -> Result<&str> {
        if range.start > range.end || range.end > self.characters.len() {
            return Err(Failure::Invalid(
                "PDF text selection is outside the page".into(),
            ));
        }
        if range.is_empty() {
            return Ok("");
        }
        let start = self.characters[range.start].bytes.start as usize;
        let end = self.characters[range.end - 1].bytes.end as usize;
        Ok(&self.text[start..end])
    }
    pub fn from_reserved(
        page: u32,
        text: String,
        characters: Vec<TextCharacter>,
        mut lease: ByteLease,
    ) -> Result<Self> {
        if characters.len() > MAX_PAGE_TEXT_CHARACTERS || text.len() > MAX_PAGE_TEXT_CHARACTERS * 4
        {
            return Err(Failure::Invalid(
                "PDF text exceeds the character work limit".into(),
            ));
        }
        let mut end = 0;
        for character in &characters {
            let range = character.bytes.start as usize..character.bytes.end as usize;
            if range.start != end
                || text
                    .get(range.clone())
                    .is_none_or(|value| value.chars().count() != 1)
            {
                return Err(Failure::Invalid(
                    "PDF character boundaries are invalid".into(),
                ));
            }
            end = range.end;
        }
        if end != text.len() {
            return Err(Failure::Invalid(
                "PDF character boundaries do not cover the text".into(),
            ));
        }
        let bytes = characters
            .capacity()
            .checked_mul(std::mem::size_of::<TextCharacter>())
            .and_then(|bytes| bytes.checked_add(text.capacity()))
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())
            })
            .ok_or(Failure::MemoryBudget)?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            page,
            text,
            characters,
            _lease: lease,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation};
    use std::sync::Arc;

    fn make(governor: &MemoryGovernor, text: &str, ranges: &[Range<u32>]) -> Result<PdfPageText> {
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                4096,
            ))
            .unwrap();
        PdfPageText::from_reserved(
            3,
            text.into(),
            ranges
                .iter()
                .cloned()
                .map(|bytes| TextCharacter {
                    bytes,
                    region: None,
                })
                .collect(),
            lease,
        )
    }

    #[test]
    fn selection_uses_character_indices_for_multibyte_unicode_and_missing_geometry() {
        let governor = MemoryGovernor::default();
        let page = Arc::new(make(&governor, "aé📖ב", &[0..1, 1..3, 3..7, 7..9]).unwrap());
        assert_eq!(page.selected_text(1..3).unwrap(), "é📖");
        assert_eq!(page.selected_text(3..4).unwrap(), "ב");
        assert_eq!(page.selected_text(4..4).unwrap(), "");
        assert!(page.selected_text(0..5).is_err());
        assert!(
            page.selected_text(std::ops::Range { start: 3, end: 2 })
                .is_err()
        );
        assert!(
            page.characters()
                .iter()
                .all(|character| character.region.is_none())
        );
        let retained = page.clone();
        drop(page);
        assert!(governor.usage().text > 0);
        drop(retained);
        assert_eq!(governor.usage().text, 0);
    }

    #[test]
    fn invalid_utf8_gaps_overlaps_and_uncovered_text_release_the_lease() {
        let governor = MemoryGovernor::default();
        for ranges in [
            Vec::from_iter(std::iter::once(0..1)),
            vec![0..2, 2..3],
            vec![0..1, 2..3],
            vec![0..1, 0..3],
            vec![0..0, 0..3],
        ] {
            assert!(make(&governor, "aé", &ranges).is_err());
            assert_eq!(governor.usage().text, 0);
        }
        assert_eq!(
            make(&governor, "", &[])
                .unwrap()
                .selected_text(0..0)
                .unwrap(),
            ""
        );
    }
}
