use crate::{Failure, Result, memory::ByteLease};

pub const READING_HISTORY_PAGE_SIZE: usize = 80;

#[derive(Default, Clone)]
pub struct ReadingHistoryQuery {
    pub document: Option<String>,
    pub offset: usize,
}

pub struct ReadingHistoryEntry {
    pub id: String,
    pub document: String,
    pub title: String,
    pub path: String,
    pub missing: bool,
    pub started_at: String,
    pub ended_at: String,
    pub start_page: u32,
    pub end_page: u32,
    pub active_seconds: u64,
    /// Only this session's object, preserving unknown extension fields for restoration.
    pub context_json: Option<String>,
}

pub struct ReadingHistoryPage {
    entries: Vec<ReadingHistoryEntry>,
    total: u64,
    offset: usize,
    _lease: ByteLease,
}
impl ReadingHistoryPage {
    pub fn entries(&self) -> &[ReadingHistoryEntry] {
        &self.entries
    }
    pub fn total(&self) -> u64 {
        self.total
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn from_reserved(
        entries: Vec<ReadingHistoryEntry>,
        total: u64,
        offset: usize,
        mut lease: ByteLease,
    ) -> Result<Self> {
        if entries.len() > READING_HISTORY_PAGE_SIZE
            || entries
                .iter()
                .enumerate()
                .any(|(i, entry)| entries[..i].iter().any(|earlier| earlier.id == entry.id))
        {
            return Err(Failure::Invalid("Reading history exceeds one page".into()));
        }
        let base = entries
            .capacity()
            .checked_mul(std::mem::size_of::<ReadingHistoryEntry>())
            .and_then(|n| n.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Failure::MemoryBudget)?;
        let bytes = entries.iter().try_fold(base, |bytes, entry| {
            [
                &entry.id,
                &entry.document,
                &entry.title,
                &entry.path,
                &entry.started_at,
                &entry.ended_at,
            ]
            .into_iter()
            .try_fold(bytes, |n, s| {
                n.checked_add(s.capacity()).ok_or(Failure::MemoryBudget)
            })?
            .checked_add(entry.context_json.as_ref().map_or(0, String::capacity))
            .ok_or(Failure::MemoryBudget)
        })?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            entries,
            total,
            offset,
            _lease: lease,
        })
    }
}
