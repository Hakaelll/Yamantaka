use crate::pdf_geometry::NormalizedPageRect;
use crate::{Failure, Result, memory::ByteLease, reader::ReaderPosition};

pub const FIND_PAGE_SIZE: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentTextPresence {
    Available,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfFindQuery {
    text: String,
    match_case: bool,
    whole_words: bool,
}

impl PdfFindQuery {
    pub fn new(text: &str, match_case: bool, whole_words: bool) -> Result<Self> {
        if text.trim().is_empty() || text.len() > 1024 || text.contains('\0') {
            return Err(Failure::Invalid(
                "PDF search requires 1..=1024 UTF-8 bytes without NUL".into(),
            ));
        }
        Ok(Self {
            text: text.into(),
            match_case,
            whole_words,
        })
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn match_case(&self) -> bool {
        self.match_case
    }
    pub fn whole_words(&self) -> bool {
        self.whole_words
    }
}

#[derive(Debug)]
pub struct PdfMatch {
    pub ordinal: u32,
    pub snippet: String,
    pub rectangles: Vec<NormalizedPageRect>,
}

impl PdfMatch {
    pub fn position(&self, page: u32) -> Option<ReaderPosition> {
        self.rectangles.first().and_then(|rect| {
            ReaderPosition::at(page, [(rect.min()[0] + rect.max()[0]) / 2.0, rect.min()[1]]).ok()
        })
    }
}

#[derive(Debug)]
pub struct PdfPageMatches {
    page: u32,
    text_present: bool,
    matches: Vec<PdfMatch>,
    next_match: Option<u32>,
    _lease: ByteLease,
}

impl PdfPageMatches {
    pub fn page(&self) -> u32 {
        self.page
    }
    pub fn has_text(&self) -> bool {
        self.text_present
    }
    pub fn matches(&self) -> &[PdfMatch] {
        &self.matches
    }
    pub fn next_match(&self) -> Option<u32> {
        self.next_match
    }
    pub fn from_reserved(
        page: u32,
        matches: Vec<PdfMatch>,
        next_match: Option<u32>,
        text_present: bool,
        mut lease: ByteLease,
    ) -> Result<Self> {
        if (!text_present && !matches.is_empty())
            || matches.len() > FIND_PAGE_SIZE
            || matches
                .windows(2)
                .any(|pair| pair[0].ordinal >= pair[1].ordinal)
            || next_match.is_some_and(|next| matches.last().is_none_or(|last| next <= last.ordinal))
        {
            return Err(Failure::Invalid(
                "PDF match continuation is inconsistent".into(),
            ));
        }
        let bytes = matches
            .iter()
            .try_fold(
                matches
                    .capacity()
                    .checked_mul(std::mem::size_of::<PdfMatch>())
                    .ok_or(Failure::MemoryBudget)?,
                |total, item| {
                    total.checked_add(item.snippet.capacity())?.checked_add(
                        item.rectangles
                            .capacity()
                            .checked_mul(std::mem::size_of::<NormalizedPageRect>())?,
                    )
                },
            )
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())
            })
            .ok_or(Failure::MemoryBudget)?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            page,
            text_present,
            matches,
            next_match,
            _lease: lease,
        })
    }
}
