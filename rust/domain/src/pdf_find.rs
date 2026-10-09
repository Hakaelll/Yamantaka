use crate::{Failure, Result, memory::ByteLease, reader::ReaderPosition};

pub const FIND_PAGE_SIZE: usize = 80;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchRectangle {
    min: [f64; 2],
    max: [f64; 2],
}

impl MatchRectangle {
    pub fn new(min: [f64; 2], max: [f64; 2]) -> Result<Self> {
        if min
            .into_iter()
            .chain(max)
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
            || (0..2).any(|axis| min[axis] >= max[axis])
        {
            return Err(Failure::Invalid(
                "Search rectangles must be finite normalized areas".into(),
            ));
        }
        Ok(Self { min, max })
    }
    pub fn min(&self) -> [f64; 2] {
        self.min
    }
    pub fn max(&self) -> [f64; 2] {
        self.max
    }
}

#[derive(Debug)]
pub struct PdfMatch {
    pub ordinal: u32,
    pub snippet: String,
    pub rectangles: Vec<MatchRectangle>,
}

impl PdfMatch {
    pub fn position(&self, page: u32) -> Option<ReaderPosition> {
        self.rectangles.first().and_then(|rect| {
            ReaderPosition::at(page, [(rect.min[0] + rect.max[0]) / 2.0, rect.min[1]]).ok()
        })
    }
}

#[derive(Debug)]
pub struct PdfPageMatches {
    page: u32,
    matches: Vec<PdfMatch>,
    next_match: Option<u32>,
    _lease: ByteLease,
}

impl PdfPageMatches {
    pub fn page(&self) -> u32 {
        self.page
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
        mut lease: ByteLease,
    ) -> Result<Self> {
        if matches.len() > FIND_PAGE_SIZE
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
                            .checked_mul(std::mem::size_of::<MatchRectangle>())?,
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
            matches,
            next_match,
            _lease: lease,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_match_payload_releases_its_lease_and_invalid_continuations_are_rejected() {
        use crate::memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation};
        let governor = MemoryGovernor::default();
        let reserve = || {
            governor
                .try_reserve_one(Cost::new(
                    MemoryCategory::Text,
                    MemoryRepresentation::Cpu,
                    4096,
                ))
                .unwrap()
        };
        let matched = || PdfMatch {
            ordinal: 2,
            snippet: "needle".into(),
            rectangles: Vec::new(),
        };
        assert!(PdfPageMatches::from_reserved(0, vec![matched()], Some(2), reserve()).is_err());
        assert!(
            PdfPageMatches::from_reserved(0, vec![matched(), matched()], None, reserve()).is_err()
        );
        assert_eq!(governor.usage().text, 0);
        let result = std::sync::Arc::new(
            PdfPageMatches::from_reserved(3, vec![matched()], Some(3), reserve()).unwrap(),
        );
        assert!(result.matches()[0].position(3).is_none());
        let clone = result.clone();
        drop(result);
        assert_eq!(clone.page(), 3);
        assert!(governor.usage().text > 0);
        drop(clone);
        assert_eq!(governor.usage().text, 0);
    }
    #[test]
    fn queries_and_match_anchors_preserve_unicode_and_normalized_geometry() {
        let query = PdfFindQuery::new("Résumé 宇宙", true, true).unwrap();
        assert_eq!(query.text(), "Résumé 宇宙");
        assert!(query.match_case() && query.whole_words());
        for text in ["", " \n", "nul\0suffix", &"a".repeat(1025)] {
            assert!(PdfFindQuery::new(text, false, false).is_err());
        }
        let matched = PdfMatch {
            ordinal: 0,
            snippet: "passage".into(),
            rectangles: vec![MatchRectangle::new([0.2, 0.3], [0.4, 0.5]).unwrap()],
        };
        assert_eq!(matched.position(7).unwrap().page(), 7);
        let point = matched.position(7).unwrap().point();
        assert!((point[0] - 0.3).abs() < 1e-12 && point[1] == 0.3);
        assert!(MatchRectangle::new([0.0, f64::NAN], [1.0, 1.0]).is_err());
        assert!(MatchRectangle::new([0.5, 0.0], [0.5, 1.0]).is_err());
        assert!(MatchRectangle::new([-0.1, 0.0], [1.0, 1.0]).is_err());
    }
}
