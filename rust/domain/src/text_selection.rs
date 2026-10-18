pub mod snapshot;
use crate::{
    Failure, Result,
    memory::{ByteLease, Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    pdf_geometry::NormalizedPageRect,
    pdf_text::{MAX_PAGE_TEXT_CHARACTERS, PdfPageText},
};
use std::{ops::Range, sync::Arc};

/// A boundary between native characters, not a UTF-8 byte or screen coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectionPosition {
    page: u32,
    character: usize,
}
impl SelectionPosition {
    pub fn new(page: u32, character: usize) -> Result<Self> {
        if character > MAX_PAGE_TEXT_CHARACTERS {
            return Err(Failure::Invalid(
                "PDF selection exceeds the character work limit".into(),
            ));
        }
        Ok(Self { page, character })
    }
    pub fn page(self) -> u32 {
        self.page
    }
    pub fn character(self) -> usize {
        self.character
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSelection {
    start: SelectionPosition,
    end: SelectionPosition,
}
impl TextSelection {
    pub fn new(anchor: SelectionPosition, extent: SelectionPosition) -> Self {
        Self {
            start: anchor.min(extent),
            end: anchor.max(extent),
        }
    }
    pub fn start(self) -> SelectionPosition {
        self.start
    }
    pub fn end(self) -> SelectionPosition {
        self.end
    }
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
    pub fn page<'a>(self, page: &'a PdfPageText) -> Result<Option<SelectedPage<'a>>> {
        if page.page() < self.start.page || page.page() > self.end.page {
            return Ok(None);
        }
        let start = if page.page() == self.start.page {
            self.start.character
        } else {
            0
        };
        let end = if page.page() == self.end.page {
            self.end.character
        } else {
            page.characters().len()
        };
        let text = page.selected_text(start..end)?;
        Ok(Some(SelectedPage {
            page,
            text,
            characters: start..end,
        }))
    }
    /// Complete ordered snapshots are required; pending pages never become a partial copy.
    pub fn copy_text(
        self,
        pages: &[Arc<PdfPageText>],
        governor: &MemoryGovernor,
    ) -> Result<SelectedText> {
        let count = u64::from(self.end.page) - u64::from(self.start.page) + 1;
        if pages.len() as u64 != count {
            return Err(Failure::Invalid(
                "PDF selection is waiting for page text".into(),
            ));
        }
        let mut bytes = 0usize;
        let mut parts = 0usize;
        for (index, page) in pages.iter().enumerate() {
            if u64::from(page.page()) != u64::from(self.start.page) + index as u64 {
                return Err(Failure::Invalid(
                    "PDF selection pages are missing, duplicated or unordered".into(),
                ));
            }
            let selection = self.page(page)?.ok_or_else(|| {
                Failure::Invalid("PDF selection page is outside the range".into())
            })?;
            let text = selection.text();
            if !text.is_empty() {
                bytes = bytes
                    .checked_add(text.len())
                    .and_then(|bytes| bytes.checked_add(usize::from(parts > 0)))
                    .ok_or(Failure::MemoryBudget)?;
                parts += 1;
            }
        }
        let mut lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                bytes
                    .checked_add(std::mem::size_of::<SelectedText>())
                    .ok_or(Failure::MemoryBudget)?,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let mut text = String::new();
        text.try_reserve_exact(bytes)
            .map_err(|_| Failure::MemoryBudget)?;
        lease
            .try_resize(
                text.capacity()
                    .checked_add(std::mem::size_of::<SelectedText>())
                    .ok_or(Failure::MemoryBudget)?,
            )
            .map_err(|_| Failure::MemoryBudget)?;
        for page in pages {
            let selection = self.page(page)?.ok_or_else(|| {
                Failure::Invalid("PDF selection page is outside the range".into())
            })?;
            let part = selection.text();
            if !part.is_empty() {
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str(part);
            }
        }
        Ok(SelectedText {
            text,
            _lease: lease,
        })
    }
}

pub struct SelectedPage<'a> {
    page: &'a PdfPageText,
    text: &'a str,
    characters: Range<usize>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryCoverage {
    Complete,
    Partial,
    Unavailable,
}
impl SelectedPage<'_> {
    pub fn page(&self) -> u32 {
        self.page.page()
    }
    pub fn text(&self) -> &str {
        self.text
    }
    /// Flutter keeps up to 80 UTF-16 units around an annotation passage.
    /// Native scalar boundaries avoid splitting supplementary characters.
    pub fn annotation_context(&self) -> (&str, &str) {
        let characters = self.page.characters();
        let start = characters
            .get(self.characters.start)
            .map_or(self.page.text().len(), |character| {
                character.bytes.start as usize
            });
        let end = characters
            .get(self.characters.end)
            .map_or(self.page.text().len(), |character| {
                character.bytes.start as usize
            });
        let mut before = start;
        let mut units = 0;
        for character in characters[..self.characters.start].iter().rev() {
            let value =
                &self.page.text()[character.bytes.start as usize..character.bytes.end as usize];
            let count = value.encode_utf16().count();
            if units + count > 80 {
                break;
            }
            units += count;
            before = character.bytes.start as usize;
        }
        let mut after = end;
        units = 0;
        for character in &characters[self.characters.end..] {
            let value =
                &self.page.text()[character.bytes.start as usize..character.bytes.end as usize];
            let count = value.encode_utf16().count();
            if units + count > 80 {
                break;
            }
            units += count;
            after = character.bytes.end as usize;
        }
        (
            &self.page.text()[before..start],
            &self.page.text()[end..after],
        )
    }
    pub fn regions(&self) -> impl Iterator<Item = NormalizedPageRect> + '_ {
        self.page.characters()[self.characters.clone()]
            .iter()
            .filter_map(|character| character.region)
    }
    pub fn geometry_coverage(&self) -> GeometryCoverage {
        let mut present = false;
        let mut absent = false;
        for character in &self.page.characters()[self.characters.clone()] {
            let value =
                &self.page.text()[character.bytes.start as usize..character.bytes.end as usize];
            if value.chars().all(char::is_whitespace) {
                continue;
            }
            present |= character.region.is_some();
            absent |= character.region.is_none();
        }
        match (present, absent) {
            (true, false) => GeometryCoverage::Complete,
            (true, true) => GeometryCoverage::Partial,
            (false, _) => GeometryCoverage::Unavailable,
        }
    }
    /// Annotation bounds omit whitespace without removing it from copied text.
    pub fn ink_regions(&self) -> impl Iterator<Item = NormalizedPageRect> + '_ {
        self.page.characters()[self.characters.clone()]
            .iter()
            .filter_map(|character| {
                let value =
                    &self.page.text()[character.bytes.start as usize..character.bytes.end as usize];
                (!value.chars().all(char::is_whitespace))
                    .then_some(character.region)
                    .flatten()
            })
    }
}

/// The materialized clipboard/export text owns its own Text-memory reservation.
pub struct SelectedText {
    text: String,
    _lease: ByteLease,
}
impl SelectedText {
    pub fn text(&self) -> &str {
        &self.text
    }
}
