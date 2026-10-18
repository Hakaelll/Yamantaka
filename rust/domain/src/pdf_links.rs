use crate::{
    Failure, Result, memory::ByteLease, pdf_destination::NavigationDestination,
    pdf_geometry::NormalizedPageRect,
};

pub const MAX_PAGE_LINKS: usize = 512;
pub const MAX_LINK_REGIONS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalLinkRejection {
    InvalidTarget,
    TooLong,
}

#[derive(Debug)]
pub enum PdfLinkTarget {
    Internal(NavigationDestination),
    External(crate::external_uri::ExternalUri),
    Rejected(ExternalLinkRejection),
}

#[derive(Debug)]
pub struct PdfPageLink {
    pub annotation_index: u32,
    pub regions: Vec<NormalizedPageRect>,
    pub target: PdfLinkTarget,
}

#[derive(Debug)]
pub struct PdfPageLinks {
    page: u32,
    links: Vec<PdfPageLink>,
    unsupported_targets: u32,
    _lease: ByteLease,
}

impl PdfPageLinks {
    pub fn page(&self) -> u32 {
        self.page
    }
    pub fn links(&self) -> &[PdfPageLink] {
        &self.links
    }
    pub fn unsupported_targets(&self) -> u32 {
        self.unsupported_targets
    }
    pub fn from_reserved(
        page: u32,
        links: Vec<PdfPageLink>,
        unsupported_targets: u32,
        mut lease: ByteLease,
    ) -> Result<Self> {
        if links.len() > MAX_PAGE_LINKS
            || links
                .iter()
                .any(|link| link.regions.is_empty() || link.regions.len() > MAX_LINK_REGIONS)
            || links
                .windows(2)
                .any(|pair| pair[0].annotation_index >= pair[1].annotation_index)
        {
            return Err(Failure::Invalid(
                "PDF link regions or annotation order are invalid".into(),
            ));
        }
        let bytes = links
            .iter()
            .try_fold(
                links
                    .capacity()
                    .checked_mul(std::mem::size_of::<PdfPageLink>())
                    .ok_or(Failure::MemoryBudget)?,
                |total, link| {
                    total
                        .checked_add(match &link.target {
                            PdfLinkTarget::Internal(_) | PdfLinkTarget::Rejected(_) => 0,
                            PdfLinkTarget::External(uri) => uri.retained_bytes(),
                        })?
                        .checked_add(
                            link.regions
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
            links,
            unsupported_targets,
            _lease: lease,
        })
    }
}
