//! Bounded bibliographic evidence from native text snapshots, without guessing names.

mod lines;

use crate::{
    Cancellation, Failure, Result,
    import_metadata::{self, MetadataValue},
    memory::{ByteLease, Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    pdf_geometry::{NormalizedPageRect, PageCoordinateSpace},
    pdf_text::PdfPageText,
};

pub const MAX_FRONT_MATTER_PAGES: u32 = 6;
pub const ENRICHMENT_ALGORITHM_VERSION: u32 = 4;

pub enum ImportAnalysis {
    NotRequested,
    Complete(Box<EnrichmentEvidence>),
    Unavailable(AnalysisFailure),
}

#[derive(Clone, Copy)]
pub enum AnalysisFailure {
    Unavailable,
    MemoryBudget,
    TextWorkLimit,
}

impl AnalysisFailure {
    pub fn code(self) -> &'static str {
        match self {
            Self::Unavailable => "front_matter_unavailable",
            Self::MemoryBudget => "front_matter_memory_budget",
            Self::TextWorkLimit => "front_matter_work_limit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceKind {
    ProminentText,
    TitleWithByline,
    RepeatedTitleWithByline,
    ExplicitByline,
    ConflictingBylines,
}

#[derive(Debug)]
pub struct MetadataCandidate {
    pub value: String,
    pub page: u32,
    pub bounds: NormalizedPageRect,
    pub evidence: EvidenceKind,
}

#[derive(Debug)]
pub enum MetadataDecision {
    Unchanged(MetadataValue),
    Corroborated {
        value: String,
        page: u32,
    },
    Autofill(MetadataCandidate),
    Review {
        current: MetadataValue,
        candidate: MetadataCandidate,
    },
}

pub struct EnrichmentEvidence {
    pub title: MetadataDecision,
    pub author: MetadataDecision,
    pub has_text: bool,
    _lease: ByteLease,
}

pub struct FrontMatter {
    title: MetadataValue,
    title_from_filename: bool,
    author: MetadataValue,
    title_match: Option<u32>,
    author_match: Option<u32>,
    candidate_title: Option<(MetadataCandidate, (bool, f64))>,
    candidate_author: Option<MetadataCandidate>,
    ambiguous_author: bool,
    title_pages: u8,
    title_byline: bool,
    last_page: Option<u32>,
    has_text: bool,
    governor: MemoryGovernor,
    lease: ByteLease,
}

impl FrontMatter {
    pub fn new(
        file_name: &str,
        title: Option<&str>,
        author: Option<&str>,
        governor: &MemoryGovernor,
    ) -> Result<Self> {
        // Retained values are bounded metadata plus two short candidate strings.
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                16 * 1024,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let title = import_metadata::title(title, file_name);
        let title_from_filename = matches!(&title, MetadataValue::Accepted(value)
            if import_metadata::is_filename_title(value, file_name));
        Ok(Self {
            title,
            title_from_filename,
            author: import_metadata::author(author),
            title_match: None,
            author_match: None,
            candidate_title: None,
            candidate_author: None,
            ambiguous_author: false,
            title_pages: 0,
            title_byline: false,
            last_page: None,
            has_text: false,
            governor: governor.clone(),
            lease,
        })
    }

    pub fn observe(
        &mut self,
        page: &PdfPageText,
        coordinates: PageCoordinateSpace,
        cancel: &Cancellation,
    ) -> Result<()> {
        cancel.check()?;
        if page.page() >= MAX_FRONT_MATTER_PAGES
            || self.last_page.is_some_and(|last| page.page() <= last)
        {
            return Err(Failure::Invalid(
                "Front matter pages must be unique, increasing and below six".into(),
            ));
        }
        let _work = self
            .governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                192 * 1024,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let analysis = lines::analyze(page, coordinates, cancel)?;
        cancel.check()?;
        self.has_text |= !page.text().trim().is_empty();
        self.last_page = Some(page.page());
        if !analysis.front_matter {
            return Ok(());
        }
        for line in &analysis.lines {
            if line.layout.min()[1] > 0.8 {
                continue;
            }
            if matches_value(&self.title, &line.text) {
                self.title_match.get_or_insert(page.page());
            }
            if matches_value(
                &self.author,
                lines::byline(&line.text).unwrap_or(&line.text),
            ) {
                self.author_match.get_or_insert(page.page());
            }
        }
        let title = analysis.prominent_title();
        let mut has_author = false;
        for line in analysis.explicit_authors(title.as_ref().map(|block| &block.line)) {
            let value = lines::byline(&line.text).unwrap();
            if let MetadataValue::Accepted(value) = import_metadata::author(Some(value)) {
                has_author = true;
                let candidate = MetadataCandidate {
                    value,
                    page: page.page(),
                    bounds: line.bounds,
                    evidence: EvidenceKind::ExplicitByline,
                };
                if let Some(current) = &self.candidate_author {
                    self.ambiguous_author |=
                        normalized(&current.value) != normalized(&candidate.value);
                } else {
                    self.candidate_author = Some(candidate);
                }
            }
        }
        if let Some(block) = title {
            let line = block.line;
            let score = (
                has_author && page.page() < 3,
                block.height * 10.0 + (1.0 - line.layout.min()[1]) * 0.1
                    - f64::from(page.page()) * 0.005,
            );
            if let Some((current, _)) = &self.candidate_title
                && normalized(&current.value) == normalized(&line.text)
            {
                self.title_pages |= 1 << page.page();
                self.title_byline |= has_author;
            } else if self
                .candidate_title
                .as_ref()
                .is_none_or(|(_, old)| score > *old)
            {
                self.candidate_title = Some((
                    MetadataCandidate {
                        value: line.text.clone(),
                        page: page.page(),
                        bounds: line.bounds,
                        evidence: if has_author && page.page() < 3 {
                            EvidenceKind::TitleWithByline
                        } else {
                            EvidenceKind::ProminentText
                        },
                    },
                    score,
                ));
                self.title_pages = 1 << page.page();
                self.title_byline = has_author;
            }
        }
        Ok(())
    }

    pub fn finish(mut self, cancel: &Cancellation) -> Result<EnrichmentEvidence> {
        cancel.check()?;
        if self.ambiguous_author
            && let Some(candidate) = &mut self.candidate_author
        {
            candidate.evidence = EvidenceKind::ConflictingBylines;
        }
        let title = self.candidate_title.take().map(|(mut candidate, _)| {
            if self.ambiguous_author {
                candidate.evidence = EvidenceKind::ProminentText;
            } else if self.title_pages.count_ones() >= 2 && self.title_byline {
                candidate.evidence = EvidenceKind::RepeatedTitleWithByline;
            }
            candidate
        });
        Ok(EnrichmentEvidence {
            title: decision(
                self.title,
                self.title_match,
                title,
                self.title_from_filename,
            ),
            author: decision(self.author, self.author_match, self.candidate_author, false),
            has_text: self.has_text,
            _lease: self.lease,
        })
    }
}

fn decision(
    current: MetadataValue,
    matching_page: Option<u32>,
    candidate: Option<MetadataCandidate>,
    replace_filename: bool,
) -> MetadataDecision {
    let matching_page = matching_page.or_else(|| {
        candidate
            .as_ref()
            .filter(|candidate| matches_value(&current, &candidate.value))
            .map(|candidate| candidate.page)
    });
    let candidate = match candidate {
        Some(candidate) if candidate.evidence == EvidenceKind::ConflictingBylines => {
            return MetadataDecision::Review { current, candidate };
        }
        Some(candidate)
            if replace_filename
                && automatic_evidence(candidate.evidence)
                && !matches_value(&current, &candidate.value) =>
        {
            return MetadataDecision::Autofill(candidate);
        }
        candidate => candidate,
    };
    if let MetadataValue::Accepted(value) = &current
        && let Some(page) = matching_page
    {
        return MetadataDecision::Corroborated {
            value: value.clone(),
            page,
        };
    }
    let Some(candidate) = candidate else {
        return MetadataDecision::Unchanged(current);
    };
    if matches!(current, MetadataValue::Accepted(_)) || !automatic_evidence(candidate.evidence) {
        MetadataDecision::Review { current, candidate }
    } else {
        MetadataDecision::Autofill(candidate)
    }
}

fn automatic_evidence(evidence: EvidenceKind) -> bool {
    matches!(
        evidence,
        EvidenceKind::ExplicitByline
            | EvidenceKind::TitleWithByline
            | EvidenceKind::RepeatedTitleWithByline
    )
}

fn normalized(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn matches_value(value: &MetadataValue, text: &str) -> bool {
    matches!(value, MetadataValue::Accepted(value) if normalized(value) == normalized(text))
}
