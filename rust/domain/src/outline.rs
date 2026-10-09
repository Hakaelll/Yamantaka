use crate::{memory::ByteLease, pdf_destination::NavigationDestination};

#[derive(Debug, PartialEq)]
pub struct OutlineEntry {
    pub title: String,
    /// Parent is an earlier entry in this depth-first list.
    pub parent: Option<usize>,
    /// External actions and headings without a local destination have no target.
    pub destination: Option<NavigationDestination>,
}

/// Shared replies retain their memory reservation until the final owner drops.
#[derive(Debug)]
pub struct PdfOutline {
    entries: Vec<OutlineEntry>,
    _lease: ByteLease,
}
impl PdfOutline {
    pub fn entries(&self) -> &[OutlineEntry] {
        &self.entries
    }
    pub fn from_reserved(entries: Vec<OutlineEntry>, mut lease: ByteLease) -> crate::Result<Self> {
        if entries
            .iter()
            .enumerate()
            .any(|(index, entry)| entry.parent.is_some_and(|parent| parent >= index))
        {
            return Err(crate::Failure::Invalid(
                "Outline parents must precede their children".into(),
            ));
        }
        let bytes = entries
            .capacity()
            .checked_mul(std::mem::size_of::<OutlineEntry>())
            .and_then(|bytes| {
                entries.iter().try_fold(bytes, |bytes, entry| {
                    bytes.checked_add(entry.title.capacity())
                })
            })
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())
            })
            .ok_or(crate::Failure::MemoryBudget)?;
        lease
            .try_resize(bytes)
            .map_err(|_| crate::Failure::MemoryBudget)?;
        Ok(Self {
            entries,
            _lease: lease,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn malformed_parent_indices_are_rejected_and_release_their_reservation() {
        use crate::memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation};
        let governor = MemoryGovernor::default();
        for parent in [0, 1, usize::MAX] {
            let lease = governor
                .try_reserve_one(Cost::new(
                    MemoryCategory::Text,
                    MemoryRepresentation::Cpu,
                    4096,
                ))
                .unwrap();
            let entry = super::OutlineEntry {
                title: "Invalid".into(),
                parent: Some(parent),
                destination: None,
            };
            assert!(super::PdfOutline::from_reserved(vec![entry], lease).is_err());
            assert_eq!(governor.usage().text, 0);
        }
    }
}
