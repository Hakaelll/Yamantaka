use super::*;

pub struct TextSelectionSnapshot {
    selection: TextSelection,
    pages: Vec<Arc<PdfPageText>>,
    text: SelectedText,
    _lease: ByteLease,
}

impl TextSelectionSnapshot {
    pub fn new(
        selection: TextSelection,
        pages: impl IntoIterator<Item = Result<Arc<PdfPageText>>>,
        governor: &MemoryGovernor,
    ) -> Result<Self> {
        if selection.is_empty() {
            return Err(Failure::Invalid("No text passage is selected".into()));
        }
        let count = usize::try_from(
            u64::from(selection.end().page()) - u64::from(selection.start().page()) + 1,
        )
        .map_err(|_| Failure::MemoryBudget)?;
        let bytes = count
            .checked_mul(std::mem::size_of::<Arc<PdfPageText>>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        let mut lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                bytes,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let mut snapshots = Vec::new();
        snapshots
            .try_reserve_exact(count)
            .map_err(|_| Failure::MemoryBudget)?;
        for page in pages {
            if snapshots.len() == count {
                return Err(Failure::Invalid(
                    "Selection contains extra source pages".into(),
                ));
            }
            snapshots.push(page?);
        }
        let text = selection.copy_text(&snapshots, governor)?;
        let actual = snapshots
            .capacity()
            .checked_mul(std::mem::size_of::<Arc<PdfPageText>>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        lease
            .try_resize(actual)
            .map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            selection,
            pages: snapshots,
            text,
            _lease: lease,
        })
    }
    pub fn selection(&self) -> TextSelection {
        self.selection
    }
    pub fn pages(&self) -> &[Arc<PdfPageText>] {
        &self.pages
    }
    pub fn text(&self) -> &str {
        self.text.text()
    }
    pub fn into_text(self) -> SelectedText {
        self.text
    }
}
