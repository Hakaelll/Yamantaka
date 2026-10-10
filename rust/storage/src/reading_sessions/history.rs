use super::*;
pub(super) use yamantaka_domain::reading_sessions::history::{
    ReadingHistoryPage, ReadingHistoryQuery,
};
use yamantaka_domain::{
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    reading_sessions::history::{READING_HISTORY_PAGE_SIZE, ReadingHistoryEntry},
};

fn text<'a>(row: &'a rusqlite::Row<'_>, column: usize) -> Result<&'a str> {
    row.get_ref(column)
        .map_err(storage)?
        .as_str()
        .map_err(storage)
}

pub(super) fn read(
    database: &Database,
    query: &ReadingHistoryQuery,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<ReadingHistoryPage> {
    cancel.check()?;
    let tx = database
        .connection
        .unchecked_transaction()
        .map_err(storage)?;
    let valid: Option<bool> = tx.query_row("SELECT CASE WHEN json_valid(value) THEN json_type(value)='object' ELSE 0 END FROM app_metadata WHERE key=?1", [CONTEXTS], |r| r.get(0)).optional().map_err(storage)?;
    if valid == Some(false) {
        return Err(Failure::Invalid(
            "Stored reading session contexts require recovery".into(),
        ));
    }
    let total: i64 = tx.query_row("SELECT count(*) FROM reading_sessions s JOIN documents d ON d.id=s.document_id WHERE s.ended_at IS NOT NULL AND (?1 IS NULL OR s.document_id=?1)", [&query.document], |r| r.get(0)).map_err(storage)?;
    let total = u64::try_from(total)
        .map_err(|_| Failure::Invalid("Reading history count is invalid".into()))?;
    let offset = query.offset.min(i64::MAX as usize);
    let capacity = total
        .saturating_sub(offset as u64)
        .min(READING_HISTORY_PAGE_SIZE as u64) as usize;
    let mut bytes = capacity
        .checked_mul(std::mem::size_of::<ReadingHistoryEntry>())
        .and_then(|n| n.checked_add(std::mem::size_of::<ReadingHistoryPage>()))
        .ok_or(Failure::MemoryBudget)?;
    let mut lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(capacity)
        .map_err(|_| Failure::MemoryBudget)?;
    let mut statement = tx.prepare("WITH page AS MATERIALIZED(SELECT s.id,s.document_id,COALESCE(d.title_override,d.title,d.source_title,d.file_name) AS title,d.path,s.started_at,s.ended_at,s.start_page,s.end_page,s.active_seconds,d.is_missing FROM reading_sessions s JOIN documents d ON d.id=s.document_id WHERE s.ended_at IS NOT NULL AND (?1 IS NULL OR s.document_id=?1) ORDER BY s.started_at DESC,s.id DESC LIMIT 80 OFFSET ?2) SELECT page.*,context.value FROM page LEFT JOIN json_each((SELECT value FROM app_metadata WHERE key=?3)) context ON context.key=page.id AND context.type='object' ORDER BY page.started_at DESC,page.id DESC").map_err(storage)?;
    let mut rows = statement
        .query(params![query.document, offset as i64, CONTEXTS])
        .map_err(storage)?;
    while let Some(row) = rows.next().map_err(storage)? {
        cancel.check()?;
        let fields = [
            text(row, 0)?,
            text(row, 1)?,
            text(row, 2)?,
            text(row, 3)?,
            text(row, 4)?,
            text(row, 5)?,
        ];
        if entries
            .iter()
            .any(|entry: &ReadingHistoryEntry| entry.id == fields[0])
        {
            return Err(Failure::Invalid(
                "Stored reading session contexts contain duplicate identifiers".into(),
            ));
        }
        let context = match row.get_ref(10).map_err(storage)? {
            rusqlite::types::ValueRef::Null => None,
            value => Some(value.as_str().map_err(storage)?),
        };
        bytes = fields
            .into_iter()
            .try_fold(bytes, |n, s| {
                n.checked_add(s.len()).ok_or(Failure::MemoryBudget)
            })?
            .checked_add(context.map_or(0, str::len))
            .ok_or(Failure::MemoryBudget)?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        let started = timestamp(fields[4])?;
        let ended = timestamp(fields[5])?;
        if ended < started {
            return Err(Failure::Invalid(
                "Stored reading session ends before its start".into(),
            ));
        }
        let seconds: i64 = row.get(8).map_err(storage)?;
        let active_seconds = u64::try_from(seconds)
            .map_err(|_| Failure::Invalid("Stored reading duration is negative".into()))?;
        entries.push(ReadingHistoryEntry {
            id: fields[0].into(),
            document: fields[1].into(),
            title: fields[2].into(),
            path: fields[3].into(),
            started_at: fields[4].into(),
            ended_at: fields[5].into(),
            start_page: row.get(6).map_err(storage)?,
            end_page: row.get(7).map_err(storage)?,
            active_seconds,
            missing: row.get(9).map_err(storage)?,
            context_json: context.map(str::to_owned),
        });
    }
    drop(rows);
    drop(statement);
    let page = ReadingHistoryPage::from_reserved(entries, total, offset, lease)?;
    cancel.check()?;
    tx.commit().map_err(storage)?;
    Ok(page)
}
