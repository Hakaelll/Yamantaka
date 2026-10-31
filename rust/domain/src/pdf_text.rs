use crate::{Failure, Result, memory::ByteLease, pdf_geometry::NormalizedPageRect};
use std::ops::Range;

pub const MAX_PAGE_TEXT_CHARACTERS: usize = 128_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextGeometryPurpose {
    Reader,
    FrontMatter,
    Recovery,
}

impl TextGeometryPurpose {
    pub fn character_limit(self) -> usize {
        match self {
            Self::Reader | Self::Recovery => MAX_PAGE_TEXT_CHARACTERS,
            Self::FrontMatter => 16_000,
        }
    }
}

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
