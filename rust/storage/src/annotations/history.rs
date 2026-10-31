use super::*;
use std::collections::VecDeque;
use yamantaka_domain::memory::ByteLease;

const MAX_BYTES: usize = 8 * 1024 * 1024;
#[derive(Default)]
pub(crate) struct History {
    entries: VecDeque<Entry>,
}
pub(super) struct Entry {
    document: String,
    changes: Vec<(Snapshot, Snapshot)>,
    undone: bool,
    lease: ByteLease,
    bytes: usize,
}
#[derive(PartialEq)]
pub(super) struct Snapshot {
    pub(super) id: String,
    columns: Vec<String>,
    row: Option<Vec<Value>>,
    tags: Vec<String>,
}
impl History {
    pub(super) fn decorate(&self, document: Option<&str>, page: &mut AnnotationPage) {
        page.history = document
            .map(|document| AnnotationHistory {
                undo: self
                    .entries
                    .iter()
                    .filter(|e| e.document == document && !e.undone)
                    .count(),
                redo: self
                    .entries
                    .iter()
                    .filter(|e| e.document == document && e.undone)
                    .count(),
            })
            .unwrap_or_default();
    }
    pub(super) fn clear(&mut self, document: &str) {
        self.entries.retain(|entry| entry.document != document);
    }
    pub(super) fn prepare(&mut self) -> Result<()> {
        self.entries
            .try_reserve(1)
            .map_err(|_| Failure::MemoryBudget)
    }
    pub(super) fn push(&mut self, entry: Entry) {
        if entry.changes.iter().all(|(before, after)| before == after) {
            return;
        }
        self.entries
            .retain(|old| old.document != entry.document || !old.undone);
        while self
            .entries
            .iter()
            .filter(|old| old.document == entry.document)
            .count()
            >= 100
        {
            let index = self
                .entries
                .iter()
                .position(|old| old.document == entry.document)
                .unwrap();
            self.entries.remove(index);
        }
        while self.entries.iter().map(|old| old.bytes).sum::<usize>() + entry.bytes > MAX_BYTES {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }
}
impl Entry {
    pub(super) fn new(document: &str, governor: &MemoryGovernor) -> Result<Self> {
        let bytes = 1024usize
            .checked_add(document.len())
            .ok_or(Failure::MemoryBudget)?;
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Other,
                MemoryRepresentation::Cpu,
                bytes,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            document: document.into(),
            changes: Vec::new(),
            undone: false,
            lease,
            bytes,
        })
    }
    fn reserve(&mut self, bytes: usize) -> Result<()> {
        let total = self
            .bytes
            .checked_add(bytes)
            .filter(|bytes| *bytes <= MAX_BYTES)
            .ok_or(Failure::MemoryBudget)?;
        self.lease
            .try_resize(total)
            .map_err(|_| Failure::MemoryBudget)?;
        self.bytes = total;
        Ok(())
    }
    pub(super) fn capture(
        &mut self,
        db: &Connection,
        id: &str,
        cancel: &Cancellation,
    ) -> Result<Snapshot> {
        cancel.check()?;
        self.reserve(512 + id.len())?;
        let mut statement = db
            .prepare("SELECT * FROM annotations WHERE id=?")
            .map_err(storage)?;
        let mut columns = Vec::new();
        for name in statement.column_names() {
            self.reserve(name.len() + 128)?;
            columns.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
            columns.push(name.to_owned());
        }
        let mut rows = statement.query([id]).map_err(storage)?;
        let row = if let Some(row) = rows.next().map_err(storage)? {
            if row.get::<_, String>("document_id").map_err(storage)? != self.document {
                return Err(Failure::Invalid(
                    "Annotation history belongs to another document".into(),
                ));
            }
            let mut values = Vec::new();
            values
                .try_reserve_exact(columns.len())
                .map_err(|_| Failure::MemoryBudget)?;
            for index in 0..columns.len() {
                let value = row.get_ref(index).map_err(storage)?;
                let bytes = match value {
                    rusqlite::types::ValueRef::Text(bytes)
                    | rusqlite::types::ValueRef::Blob(bytes) => bytes.len(),
                    _ => 8,
                };
                self.reserve(bytes.checked_add(128).ok_or(Failure::MemoryBudget)?)?;
                values.push(row.get::<_, Value>(index).map_err(storage)?);
            }
            Some(values)
        } else {
            None
        };
        let mut statement = db
            .prepare("SELECT tag_id FROM annotation_tags WHERE annotation_id=? ORDER BY tag_id")
            .map_err(storage)?;
        let mut rows = statement.query([id]).map_err(storage)?;
        let mut tags = Vec::new();
        while let Some(row) = rows.next().map_err(storage)? {
            cancel.check()?;
            let tag = field(row, 0)?;
            self.reserve(tag.len() + 128)?;
            tags.try_reserve(1).map_err(|_| Failure::MemoryBudget)?;
            tags.push(tag.into());
        }
        Ok(Snapshot {
            id: id.into(),
            columns,
            row,
            tags,
        })
    }
    pub(super) fn record(&mut self, before: Snapshot, after: Snapshot) -> Result<()> {
        self.reserve(2 * std::mem::size_of::<Snapshot>())?;
        self.changes
            .try_reserve(1)
            .map_err(|_| Failure::MemoryBudget)?;
        self.changes.push((before, after));
        Ok(())
    }
}
fn restore(db: &Connection, snapshot: &Snapshot, cancel: &Cancellation) -> Result<()> {
    cancel.check()?;
    if let Some(values) = &snapshot.row {
        let columns = snapshot
            .columns
            .iter()
            .map(|name| format!("\"{}\"", name.replace('"', "\"\"")))
            .collect::<Vec<_>>();
        let updates = columns
            .iter()
            .map(|column| format!("{column}=excluded.{column}"))
            .collect::<Vec<_>>()
            .join(",");
        let placeholders = vec!["?"; columns.len()].join(",");
        db.execute(&format!("INSERT INTO annotations({}) VALUES({placeholders}) ON CONFLICT(id) DO UPDATE SET {updates}", columns.join(",")), params_from_iter(values)).map_err(storage)?;
    } else {
        db.execute("DELETE FROM annotations WHERE id=?", [&snapshot.id])
            .map_err(storage)?;
    }
    db.execute(
        "DELETE FROM annotation_tags WHERE annotation_id=?",
        [&snapshot.id],
    )
    .map_err(storage)?;
    for tag in &snapshot.tags {
        cancel.check()?;
        db.execute(
            "INSERT INTO annotation_tags(annotation_id,tag_id) VALUES(?,?)",
            params![snapshot.id, tag],
        )
        .map_err(storage)?;
    }
    Ok(())
}
pub(super) fn travel(
    db: &mut Database,
    document: &str,
    redo: bool,
    query: &AnnotationQuery,
    access: IndexAccess,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<AnnotationPage> {
    cancel.check()?;
    let index = if redo {
        db.annotation_history
            .entries
            .iter()
            .position(|entry| entry.document == document && entry.undone)
    } else {
        db.annotation_history
            .entries
            .iter()
            .rposition(|entry| entry.document == document && !entry.undone)
    }
    .ok_or_else(|| {
        Failure::Invalid("No annotation change is available in that direction".into())
    })?;
    let entry = &db.annotation_history.entries[index];
    let tx = db.connection.transaction().map_err(storage)?;
    let mut verification = Entry::new(document, governor)?;
    let mut pages = std::collections::BTreeSet::new();
    for (before, after) in &entry.changes {
        cancel.check()?;
        let (expected, replacement) = if redo {
            (before, after)
        } else {
            (after, before)
        };
        if verification.capture(&tx, &expected.id, cancel)? != *expected {
            return Err(Failure::Invalid(
                "The annotation changed after this history entry; nothing was overwritten".into(),
            ));
        }
        for snapshot in [before, after] {
            if let Some(values) = &snapshot.row {
                let index = snapshot
                    .columns
                    .iter()
                    .position(|name| name == "page_index")
                    .ok_or_else(|| Failure::Invalid("Annotation history schema changed".into()))?;
                if let Value::Integer(page) = values[index] {
                    pages.insert(u32::try_from(page).map_err(storage)?);
                }
            }
        }
        restore(&tx, replacement, cancel)?;
    }
    for page in pages {
        writes::refresh(&tx, document, page, access)?;
    }
    let result = read_page(&tx, query, governor, cancel)?;
    cancel.check()?;
    tx.commit().map_err(storage)?;
    db.annotation_history.entries[index].undone = !redo;
    Ok(result)
}
