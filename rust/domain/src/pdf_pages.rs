use crate::{
    Failure, Result,
    memory::{ByteLease, Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    pdf_geometry::PageCoordinateSpace,
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
