use crate::{
    Failure, Result, memory::ByteLease, pdf_destination::NavigationDestination,
    pdf_geometry::NormalizedPageRect,
};

pub const MAX_PAGE_LINKS: usize = 512;
pub const MAX_LINK_REGIONS: usize = 64;

#[derive(Debug)]
pub struct PdfPageLink {
    pub annotation_index: u32,
    pub regions: Vec<NormalizedPageRect>,
    pub destination: NavigationDestination,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
        pdf_destination::DestinationView,
        pdf_geometry::PageCoordinateSpace,
    };

    #[test]
    fn invalid_regions_order_and_limits_release_the_reserved_payload() {
        let governor = MemoryGovernor::default();
        let reserve = || {
            governor
                .try_reserve_one(Cost::new(
                    MemoryCategory::Text,
                    MemoryRepresentation::Cpu,
                    2 * 1024 * 1024,
                ))
                .unwrap()
        };
        let link = |index, count| PdfPageLink {
            annotation_index: index,
            regions: vec![NormalizedPageRect::new([0.1, 0.2], [0.3, 0.4]).unwrap(); count],
            destination: NavigationDestination {
                page: 1,
                view: DestinationView::FitPage,
                coordinates: PageCoordinateSpace::new([0.0, 800.0], [600.0, 800.0], [0.0, 0.0])
                    .unwrap(),
                content_bounds: None,
            },
        };
        for links in [
            vec![link(0, 0)],
            vec![link(0, MAX_LINK_REGIONS + 1)],
            vec![link(2, 1), link(1, 1)],
            vec![link(0, 1), link(0, 1)],
            (0..=MAX_PAGE_LINKS)
                .map(|index| link(index as u32, 1))
                .collect(),
        ] {
            assert!(PdfPageLinks::from_reserved(0, links, 0, reserve()).is_err());
            assert_eq!(governor.usage().text, 0);
        }
        let payload = std::sync::Arc::new(
            PdfPageLinks::from_reserved(3, vec![link(7, 2)], 4, reserve()).unwrap(),
        );
        let retained = payload.clone();
        drop(payload);
        assert_eq!(retained.page(), 3);
        assert_eq!(retained.unsupported_targets(), 4);
        assert!(governor.usage().text > 0 && governor.usage().text < 4096);
        drop(retained);
        assert_eq!(governor.usage().text, 0);
    }
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
                    total.checked_add(
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
