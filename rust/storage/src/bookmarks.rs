use crate::{Database, database::storage};
use chrono::{SecondsFormat, Utc};
use rusqlite::{OptionalExtension, params};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    bookmarks::{BOOKMARK_PAGE_SIZE, Bookmark, BookmarkChange, BookmarkPage, BookmarkRepository},
};

fn label(value: &Option<String>) -> Result<Option<String>> {
    let value = value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if value.is_some_and(|value| value.chars().count() > 4096) {
        return Err(Failure::Invalid(
            "Bookmark label exceeds 4096 characters".into(),
        ));
    }
    Ok(value.map(str::to_owned))
}

fn read_page(
    connection: &rusqlite::Connection,
    document_id: &str,
    offset: usize,
    cancel: &Cancellation,
) -> Result<BookmarkPage> {
    cancel.check()?;
    let total: i64 = connection
        .query_row(
            "SELECT count(*) FROM bookmarks WHERE document_id=?",
            [document_id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    let total =
        u64::try_from(total).map_err(|_| Failure::Invalid("Negative bookmark count".into()))?;
    let last = total.saturating_sub(1) / BOOKMARK_PAGE_SIZE as u64 * BOOKMARK_PAGE_SIZE as u64;
    let offset = (offset / BOOKMARK_PAGE_SIZE * BOOKMARK_PAGE_SIZE).min(
        usize::try_from(last)
            .map_err(|_| Failure::Invalid("Bookmark count exceeds platform limits".into()))?,
    );
    let sql_offset = i64::try_from(offset)
        .map_err(|_| Failure::Invalid("Bookmark offset exceeds SQLite limits".into()))?;
    let mut statement = connection.prepare("SELECT id,document_id,page_index,label,created_at,updated_at FROM bookmarks WHERE document_id=? ORDER BY page_index,id LIMIT ? OFFSET ?").map_err(storage)?;
    let rows = statement
        .query_map(
            params![document_id, BOOKMARK_PAGE_SIZE as i64, sql_offset],
            |row| {
                Ok(Bookmark {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    page_index: row.get(2)?,
                    label: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .map_err(storage)?;
    let mut entries = Vec::with_capacity(BOOKMARK_PAGE_SIZE);
    for row in rows {
        cancel.check()?;
        entries.push(row.map_err(storage)?);
    }
    cancel.check()?;
    Ok(BookmarkPage {
        entries,
        total,
        offset,
    })
}

impl BookmarkRepository for Database {
    fn bookmarks(
        &self,
        document_id: &str,
        offset: usize,
        cancel: &Cancellation,
    ) -> Result<BookmarkPage> {
        let tx = self.connection.unchecked_transaction().map_err(storage)?;
        let page = read_page(&tx, document_id, offset, cancel)?;
        tx.commit().map_err(storage)?;
        Ok(page)
    }

    fn change_bookmark(
        &mut self,
        document_id: &str,
        change: &BookmarkChange,
        offset: usize,
        access: yamantaka_domain::search::IndexAccess,
        cancel: &Cancellation,
    ) -> Result<BookmarkPage> {
        cancel.check()?;
        let tx = self.connection.transaction().map_err(storage)?;
        let exists: Option<u8> = tx
            .query_row("SELECT 1 FROM documents WHERE id=?", [document_id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(storage)?;
        exists.ok_or_else(|| Failure::Invalid("Bookmark document no longer exists".into()))?;
        let target_page = match change {
            BookmarkChange::Toggle { page }
            | BookmarkChange::Ensure { page }
            | BookmarkChange::Save { page, .. } => *page,
            BookmarkChange::Rename { id, .. } | BookmarkChange::Remove { id } => tx
                .query_row(
                    "SELECT page_index FROM bookmarks WHERE id=? AND document_id=?",
                    params![id, document_id],
                    |row| row.get::<_, u32>(0),
                )
                .optional()
                .map_err(storage)?
                .ok_or_else(|| {
                    Failure::Invalid("Bookmark no longer belongs to this document".into())
                })?,
        };
        let now = Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true);
        match change {
            BookmarkChange::Toggle { page } => {
                let removed = tx
                    .execute(
                        "DELETE FROM bookmarks WHERE document_id=? AND page_index=?",
                        params![document_id, page],
                    )
                    .map_err(storage)?;
                if removed == 0 {
                    tx.execute("INSERT INTO bookmarks(id,document_id,page_index,label,created_at,updated_at) VALUES(?,?,?,NULL,?,?)",params![uuid::Uuid::new_v4().to_string(),document_id,page,now,now]).map_err(storage)?;
                }
            }
            BookmarkChange::Ensure { page } => {
                tx.execute("INSERT INTO bookmarks(id,document_id,page_index,label,created_at,updated_at) VALUES(?,?,?,NULL,?,?) ON CONFLICT(document_id,page_index) DO NOTHING", params![uuid::Uuid::new_v4().to_string(), document_id, page, now, now]).map_err(storage)?;
            }
            BookmarkChange::Save { page, label: value } => {
                // The canonical reader validates actual PDF page bounds.
                // Persisted catalogue page counts can be stale after file changes;
                // Flutter permits bookmarks beyond that metadata count as well.
                let value = label(value)?;
                tx.execute("INSERT INTO bookmarks(id,document_id,page_index,label,created_at,updated_at) VALUES(?,?,?,?,?,?) ON CONFLICT(document_id,page_index) DO UPDATE SET label=excluded.label,updated_at=excluded.updated_at", params![uuid::Uuid::new_v4().to_string(), document_id, page, value, now, now]).map_err(storage)?;
            }
            BookmarkChange::Rename { id, label: value } => {
                let value = label(value)?;
                if tx
                    .execute(
                        "UPDATE bookmarks SET label=?,updated_at=? WHERE id=? AND document_id=?",
                        params![value, now, id, document_id],
                    )
                    .map_err(storage)?
                    == 0
                {
                    return Err(Failure::Invalid(
                        "Bookmark no longer belongs to this document".into(),
                    ));
                }
            }
            BookmarkChange::Remove { id } => {
                if tx
                    .execute(
                        "DELETE FROM bookmarks WHERE id=? AND document_id=?",
                        params![id, document_id],
                    )
                    .map_err(storage)?
                    == 0
                {
                    return Err(Failure::Invalid(
                        "Bookmark no longer belongs to this document".into(),
                    ));
                }
            }
        }
        crate::search::refresh_bookmark_page(&tx, document_id, target_page, access)?;
        let page = read_page(&tx, document_id, offset, cancel)?;
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(page)
    }
}
