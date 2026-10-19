use crate::{Database, database::storage};
use rusqlite::{Connection, TransactionBehavior, params};
use yamantaka_domain::{
    Cancellation, Failure, MAX_LIBRARY_PAGE, Result, search::IndexAccess, tags::*,
};

fn document_exists(db: &Connection, id: &str) -> Result<()> {
    if !db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE id=?)",
            [id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(storage)?
    {
        return Err(Failure::Invalid(
            "Document is no longer in the library".into(),
        ));
    }
    Ok(())
}
impl Database {
    pub(crate) fn tag_page(&self, query: &TagQuery, cancel: &Cancellation) -> Result<TagPage> {
        cancel.check()?;
        if query.text.len() > 4096 {
            return Err(Failure::Invalid("Tag search is too long".into()));
        }
        if let Some(document) = &query.document {
            document_exists(&self.connection, document)?;
        }
        let pattern = format!(
            "%{}%",
            query
                .text
                .trim()
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        let total=self.connection.query_row("SELECT count(*) FROM tags WHERE (?1 IS NULL OR id=?1) AND name LIKE ?2 ESCAPE '\\'",params![query.id,pattern],|row|row.get::<_,i64>(0)).map_err(storage)? as u64;
        let offset = i64::try_from(query.offset)
            .map_err(|_| Failure::Invalid("Tag page offset is too large".into()))?;
        let mut statement=self.connection.prepare("SELECT t.id,t.name,t.color,(SELECT count(*) FROM document_tags d WHERE d.tag_id=t.id),(SELECT count(*) FROM annotation_tags a WHERE a.tag_id=t.id),EXISTS(SELECT 1 FROM document_tags d WHERE d.tag_id=t.id AND d.document_id=?1) FROM tags t WHERE (?2 IS NULL OR t.id=?2) AND t.name LIKE ?3 ESCAPE '\\' ORDER BY t.name COLLATE NOCASE,t.id LIMIT ?4 OFFSET ?5").map_err(storage)?;
        let rows = statement
            .query_map(
                params![
                    query.document,
                    query.id,
                    pattern,
                    MAX_LIBRARY_PAGE as i64,
                    offset
                ],
                |row| {
                    Ok(Tag {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        color: row.get(2)?,
                        document_count: row.get::<_, i64>(3)? as u64,
                        annotation_count: row.get::<_, i64>(4)? as u64,
                        contains_document: row.get(5)?,
                    })
                },
            )
            .map_err(storage)?;
        let mut entries = Vec::new();
        for row in rows {
            cancel.check()?;
            entries.push(row.map_err(storage)?);
        }
        cancel.check()?;
        Ok(TagPage { entries, total })
    }
    pub(crate) fn apply_tag(
        &mut self,
        change: &TagChange,
        access: IndexAccess,
        cancel: &Cancellation,
    ) -> Result<String> {
        cancel.check()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let id = match change {
            TagChange::Ensure { name } => {
                let clean = name.trim();
                if clean.is_empty() || clean.encode_utf16().count() > 60 {
                    return Err(Failure::Invalid(
                        "Use a tag name between 1 and 60 characters".into(),
                    ));
                }
                crate::annotations::ensure_tag(&tx, clean, &chrono::Utc::now().to_rfc3339())?
            }
            TagChange::SetMembership {
                tag,
                document,
                included,
            } => {
                document_exists(&tx, document)?;
                if !tx
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM tags WHERE id=?)",
                        [tag],
                        |row| row.get::<_, bool>(0),
                    )
                    .map_err(storage)?
                {
                    return Err(Failure::Invalid("Tag no longer exists".into()));
                }
                let changed = if *included {
                    tx.execute(
                        "INSERT OR IGNORE INTO document_tags(document_id,tag_id) VALUES(?,?)",
                        params![document, tag],
                    )
                } else {
                    tx.execute(
                        "DELETE FROM document_tags WHERE document_id=? AND tag_id=?",
                        params![document, tag],
                    )
                }
                .map_err(storage)?;
                if changed > 0 {
                    crate::search::refresh_document_tags(&tx, document, access)?;
                }
                tag.clone()
            }
        };
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(id)
    }
}
