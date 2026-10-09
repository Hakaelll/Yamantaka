use crate::{
    Failure, Result,
    memory::{ByteLease, Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    outline::PageCoordinateSpace,
};
use std::{ops::Deref, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfPageInfo {
    size: [f32; 2],
    coordinates: PageCoordinateSpace,
}
impl PdfPageInfo {
    pub fn new(size: [f32; 2], coordinates: PageCoordinateSpace) -> Result<Self> {
        if size.iter().any(|value| !value.is_finite() || *value <= 0.0) {
            return Err(Failure::Pdf("Invalid PDF page dimensions".into()));
        }
        Ok(Self { size, coordinates })
    }
    pub fn size(self) -> [f32; 2] {
        self.size
    }
    pub fn coordinates(self) -> PageCoordinateSpace {
        self.coordinates
    }
}

/// Read-only shared page records retain their global memory reservation.
#[derive(Debug)]
pub struct PdfPages {
    entries: Vec<PdfPageInfo>,
    _lease: ByteLease,
}
impl Deref for PdfPages {
    type Target = [PdfPageInfo];
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}
pub struct PdfPagesBuilder {
    entries: Vec<PdfPageInfo>,
    expected: usize,
    lease: ByteLease,
}
fn cost(capacity: usize) -> Result<usize> {
    capacity
        .checked_mul(std::mem::size_of::<PdfPageInfo>())
        .and_then(|bytes| {
            bytes.checked_add(std::mem::size_of::<PdfPages>() + 2 * std::mem::size_of::<usize>())
        })
        .ok_or(Failure::MemoryBudget)
}
impl PdfPagesBuilder {
    pub fn new(expected: usize, governor: &MemoryGovernor) -> Result<Self> {
        let mut lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Other,
                MemoryRepresentation::Cpu,
                cost(expected)?,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(expected)
            .map_err(|_| Failure::MemoryBudget)?;
        lease
            .try_resize(cost(entries.capacity())?)
            .map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            entries,
            expected,
            lease,
        })
    }
    pub fn push(&mut self, page: PdfPageInfo) -> Result<()> {
        if self.entries.len() == self.expected {
            return Err(Failure::Pdf(
                "PDF page inspection exceeds its reserved count".into(),
            ));
        }
        self.entries.push(page);
        Ok(())
    }
    pub fn finish(self) -> Result<Arc<PdfPages>> {
        if self.entries.len() != self.expected {
            return Err(Failure::Pdf("PDF page inspection is incomplete".into()));
        }
        Ok(Arc::new(PdfPages {
            entries: self.entries,
            _lease: self.lease,
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn page() -> PdfPageInfo {
        PdfPageInfo::new(
            [200.0, 400.0],
            PageCoordinateSpace::new([20.0, 430.0], [220.0, 430.0], [20.0, 30.0]).unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn page_geometry_and_lease_survive_clones_until_the_final_owner_drops() {
        let governor = MemoryGovernor::default();
        let mut builder = PdfPagesBuilder::new(2, &governor).unwrap();
        builder.push(page()).unwrap();
        builder.push(page()).unwrap();
        assert!(builder.push(page()).is_err());
        let pages = builder.finish().unwrap();
        let retained = governor.usage().other;
        assert!(retained > 0);
        let clone = pages.clone();
        drop(pages);
        assert_eq!(governor.usage().other, retained);
        assert_eq!(clone[1].size(), [200.0, 400.0]);
        assert_eq!(
            clone[1].coordinates().normalized_point([120.0, 330.0]),
            [0.5, 0.25]
        );
        drop(clone);
        assert_eq!(governor.usage().other, 0);
        assert!(PdfPagesBuilder::new(usize::MAX, &governor).is_err());
        let incomplete = PdfPagesBuilder::new(1, &governor).unwrap();
        assert!(incomplete.finish().is_err());
        assert_eq!(governor.usage().other, 0);
    }
    #[test]
    fn denied_or_cancelled_inspection_does_not_retain_page_metadata() {
        let governor = MemoryGovernor::new(crate::memory::GlobalMemoryLimits {
            total: 0,
            ..Default::default()
        });
        assert!(matches!(
            PdfPagesBuilder::new(1, &governor),
            Err(Failure::MemoryBudget)
        ));
        assert_eq!(governor.usage().total, 0);
        governor.set_limits(Default::default());
        let mut builder = PdfPagesBuilder::new(5000, &governor).unwrap();
        builder.push(page()).unwrap();
        assert!(governor.usage().other > 0);
        drop(builder);
        assert_eq!(governor.usage().other, 0);
        assert!(PdfPageInfo::new([0.0, 400.0], page().coordinates()).is_err());
        assert!(PdfPageInfo::new([200.0, f32::NAN], page().coordinates()).is_err());
    }
}
