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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf_text::TextCharacter;
    fn page(governor: &MemoryGovernor, index: u32) -> Arc<PdfPageText> {
        let text = "café";
        let characters = text
            .char_indices()
            .map(|(index, c)| TextCharacter {
                bytes: index as u32..(index + c.len_utf8()) as u32,
                region: None,
            })
            .collect();
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Text,
                MemoryRepresentation::Cpu,
                4096,
            ))
            .unwrap();
        Arc::new(PdfPageText::from_reserved(index, text.into(), characters, lease).unwrap())
    }
    #[test]
    fn snapshot_keeps_complete_ordered_sources_and_copied_text_with_independent_lifetimes() {
        let governor = MemoryGovernor::default();
        let pages = [page(&governor, 0), page(&governor, 1)];
        let selection = TextSelection::new(
            SelectionPosition::new(1, 4).unwrap(),
            SelectionPosition::new(0, 1).unwrap(),
        );
        let snapshot =
            TextSelectionSnapshot::new(selection, pages.iter().cloned().map(Ok), &governor)
                .unwrap();
        assert_eq!(snapshot.text(), "afé\ncafé");
        assert_eq!(snapshot.pages().len(), 2);
        assert_eq!(snapshot.selection(), selection);
        drop(pages);
        let copied = snapshot.into_text();
        assert_eq!(copied.text(), "afé\ncafé");
        assert!(governor.usage().text > 0);
        drop(copied);
        assert_eq!(governor.usage().text, 0);
    }
    #[test]
    fn missing_repeated_extra_failed_or_denied_sources_release_partial_snapshots() {
        let governor = MemoryGovernor::default();
        let a = page(&governor, 0);
        let b = page(&governor, 1);
        let baseline = governor.usage().text;
        let selection = TextSelection::new(
            SelectionPosition::new(0, 0).unwrap(),
            SelectionPosition::new(1, 4).unwrap(),
        );
        for sources in [
            vec![Ok(a.clone())],
            vec![Ok(a.clone()), Ok(a.clone())],
            vec![Ok(b.clone()), Ok(a.clone())],
            vec![Ok(a.clone()), Ok(b.clone()), Ok(b.clone())],
            vec![Ok(a.clone()), Err(Failure::QueueFull)],
        ] {
            assert!(TextSelectionSnapshot::new(selection, sources, &governor).is_err());
            assert_eq!(governor.usage().text, baseline);
        }
        let denied = MemoryGovernor::new(crate::memory::GlobalMemoryLimits {
            text: 1,
            ..Default::default()
        });
        assert!(matches!(
            TextSelectionSnapshot::new(selection, [Ok(a.clone()), Ok(b.clone())], &denied),
            Err(Failure::MemoryBudget)
        ));
        assert_eq!(denied.usage().text, 0);
        drop((a, b));
        assert_eq!(governor.usage().text, 0);
    }
}
