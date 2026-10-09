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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{memory::GlobalMemoryLimits, pdf_text::TextCharacter};
    #[test]
    fn annotation_context_keeps_eighty_utf16_units_without_split_surrogates() {
        let governor = MemoryGovernor::default();
        let surrounding = "📖".repeat(40);
        let text = format!("a{surrounding}é{surrounding}z");
        let snapshot = page(&governor, 0, &text, &[]);
        let selection = TextSelection::new(
            SelectionPosition::new(0, 41).unwrap(),
            SelectionPosition::new(0, 42).unwrap(),
        );
        let selected = selection.page(&snapshot).unwrap().unwrap();
        assert_eq!(selected.text(), "é");
        let (before, after) = selected.annotation_context();
        assert_eq!(before, surrounding);
        assert_eq!(after, surrounding);
        assert_eq!(before.encode_utf16().count(), 80);
        assert_eq!(after.encode_utf16().count(), 80);
    }

    fn page(
        governor: &MemoryGovernor,
        index: u32,
        text: &str,
        missing: &[usize],
    ) -> Arc<PdfPageText> {
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                4096,
            ))
            .unwrap();
        let region = NormalizedPageRect::new([0.1, 0.2], [0.3, 0.4]).unwrap();
        let characters = text
            .char_indices()
            .enumerate()
            .map(|(index, (byte, value))| TextCharacter {
                bytes: byte as u32..(byte + value.len_utf8()) as u32,
                region: (!missing.contains(&index)).then_some(region),
            })
            .collect();
        Arc::new(PdfPageText::from_reserved(index, text.into(), characters, lease).unwrap())
    }
    fn select(start: (u32, usize), end: (u32, usize)) -> TextSelection {
        TextSelection::new(
            SelectionPosition::new(start.0, start.1).unwrap(),
            SelectionPosition::new(end.0, end.1).unwrap(),
        )
    }

    #[test]
    fn reverse_cross_page_selection_copies_unicode_in_document_order() {
        let governor = MemoryGovernor::default();
        let pages = [
            page(&governor, 4, "aé📖", &[]),
            page(&governor, 5, "", &[]),
            page(&governor, 6, "בend", &[]),
        ];
        let selection = select((6, 1), (4, 1));
        assert_eq!(selection.start(), SelectionPosition::new(4, 1).unwrap());
        assert_eq!(selection.end(), SelectionPosition::new(6, 1).unwrap());
        assert_eq!(
            selection.copy_text(&pages, &governor).unwrap().text(),
            "é📖\nב"
        );
        assert_eq!(
            selection
                .page(&pages[0])
                .unwrap()
                .unwrap()
                .regions()
                .count(),
            2
        );
        assert!(
            selection
                .page(&page(&governor, 3, "outside", &[]))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn incomplete_duplicate_unordered_and_invalid_endpoint_pages_are_rejected() {
        let governor = MemoryGovernor::default();
        let pages = [
            page(&governor, 0, "first", &[]),
            page(&governor, 1, "last", &[]),
        ];
        let selection = select((0, 2), (1, 3));
        assert!(selection.copy_text(&pages[..1], &governor).is_err());
        assert!(
            selection
                .copy_text(&[pages[1].clone(), pages[0].clone()], &governor)
                .is_err()
        );
        assert!(
            selection
                .copy_text(&[pages[0].clone(), pages[0].clone()], &governor)
                .is_err()
        );
        assert!(select((0, 8), (1, 3)).copy_text(&pages, &governor).is_err());
        assert!(SelectionPosition::new(0, MAX_PAGE_TEXT_CHARACTERS + 1).is_err());
        assert_eq!(
            selection.copy_text(&pages, &governor).unwrap().text(),
            "rst\nlas"
        );
    }

    #[test]
    fn whitespace_missing_geometry_and_empty_selection_are_explicit() {
        let governor = MemoryGovernor::default();
        let page = page(&governor, 0, "a b", &[1, 2]);
        let complete = select((0, 0), (0, 2)).page(&page).unwrap().unwrap();
        assert_eq!(complete.geometry_coverage(), GeometryCoverage::Complete);
        let partial = select((0, 0), (0, 3)).page(&page).unwrap().unwrap();
        assert_eq!(partial.geometry_coverage(), GeometryCoverage::Partial);
        assert_eq!(partial.text(), "a b");
        let absent = select((0, 2), (0, 3)).page(&page).unwrap().unwrap();
        assert_eq!(absent.geometry_coverage(), GeometryCoverage::Unavailable);
        assert_eq!(absent.regions().count(), 0);
        let empty = select((0, 3), (0, 3));
        assert!(empty.is_empty());
        assert_eq!(empty.copy_text(&[page], &governor).unwrap().text(), "");
    }

    #[test]
    fn materialized_copy_is_reserved_separately_and_budget_failure_keeps_source_text() {
        let governor = MemoryGovernor::default();
        let pages = [page(&governor, 0, "café", &[])];
        let baseline = governor.usage().text;
        let selection = select((0, 0), (0, 4));
        governor.set_limits(GlobalMemoryLimits {
            text: baseline,
            ..GlobalMemoryLimits::default()
        });
        assert!(matches!(
            selection.copy_text(&pages, &governor),
            Err(Failure::MemoryBudget)
        ));
        assert_eq!(governor.usage().text, baseline);
        assert_eq!(pages[0].text(), "café");
        governor.set_limits(GlobalMemoryLimits::default());
        let copy = selection.copy_text(&pages, &governor).unwrap();
        assert!(governor.usage().text > baseline);
        drop(pages);
        assert_eq!(copy.text(), "café");
        assert!(governor.usage().text > 0);
        drop(copy);
        assert_eq!(governor.usage().text, 0);
    }
}
