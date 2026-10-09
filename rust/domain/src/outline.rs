use crate::memory::ByteLease;

/// PDF coordinates are preserved in points until canonical navigation applies
/// the destination using the actual page geometry and current reader settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DestinationView {
    Unknown,
    Coordinates {
        x: Option<f32>,
        y: Option<f32>,
        zoom: Option<f32>,
    },
    FitPage,
    FitWidth {
        top: Option<f32>,
    },
    FitHeight {
        left: Option<f32>,
    },
    FitRectangle {
        left: f32,
        bottom: f32,
        right: f32,
        top: f32,
    },
    FitBounds,
    FitBoundsWidth {
        top: Option<f32>,
    },
    FitBoundsHeight {
        left: Option<f32>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutlineDestination {
    pub page: u32,
    pub view: DestinationView,
}

#[derive(Debug, PartialEq)]
pub struct OutlineEntry {
    pub title: String,
    /// Parent is an earlier entry in this depth-first list.
    pub parent: Option<usize>,
    /// External actions and headings without a local destination have no target.
    pub destination: Option<OutlineDestination>,
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
