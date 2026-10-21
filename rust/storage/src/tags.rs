use crate::{Database, database::storage};
use rusqlite::{Connection, TransactionBehavior, params, params_from_iter, types::Value};
use yamantaka_domain::{
    Cancellation, Failure, MAX_LIBRARY_PAGE, Result, search::IndexAccess, tags::*,
};
mod edits;
mod search_cache;

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
        if query.ids.len() > MAX_TAG_FILTERS {
            return Err(Failure::Invalid("Use at most eighty tag filters".into()));
        }
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
        let mut predicate = String::from(" WHERE t.name LIKE ? ESCAPE '\\'");
        let mut values = vec![Value::Text(pattern)];
        if !query.ids.is_empty() {
            predicate.push_str(&format!(
                " AND t.id IN ({})",
                vec!["?"; query.ids.len()].join(",")
            ));
            values.extend(query.ids.iter().cloned().map(Value::Text));
        }
        let total = self
            .connection
            .query_row(
                &format!("SELECT count(*) FROM tags t{predicate}"),
                params_from_iter(&values),
                |row| row.get::<_, i64>(0),
            )
            .map_err(storage)? as u64;
        let offset = i64::try_from(query.offset)
            .map_err(|_| Failure::Invalid("Tag page offset is too large".into()))?;
        let mut statement=self.connection.prepare(&format!("SELECT t.id,t.name,t.color,(SELECT count(*) FROM document_tags d WHERE d.tag_id=t.id),(SELECT count(*) FROM annotation_tags a WHERE a.tag_id=t.id),EXISTS(SELECT 1 FROM document_tags d WHERE d.tag_id=t.id AND d.document_id=?),t.updated_at FROM tags t{predicate} ORDER BY t.name COLLATE NOCASE,t.id LIMIT ? OFFSET ?")).map_err(storage)?;
        values.insert(
            0,
            query
                .document
                .clone()
                .map(Value::Text)
                .unwrap_or(Value::Null),
        );
        values.extend([
            Value::Integer(MAX_LIBRARY_PAGE as i64),
            Value::Integer(offset),
        ]);
        let rows = statement
            .query_map(params_from_iter(&values), |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    document_count: row.get::<_, i64>(3)? as u64,
                    annotation_count: row.get::<_, i64>(4)? as u64,
                    contains_document: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
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
            TagChange::Ensure { name } => ensure(&tx, name)?,
            TagChange::EnsureForDocument { name, document } => {
                let id = ensure(&tx, name)?;
                membership(&tx, &id, document, true, access)?;
                id
            }
            TagChange::SetMembership {
                tag,
                document,
                included,
            } => {
                membership(&tx, tag, document, *included, access)?;
                tag.clone()
            }
            TagChange::Rename { .. }
            | TagChange::Color { .. }
            | TagChange::Merge { .. }
            | TagChange::Delete { .. } => edits::apply(&tx, change, access, cancel)?,
        };
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(id)
    }
}

fn ensure(db: &Connection, name: &str) -> Result<String> {
    let clean = name.trim();
    if clean.is_empty() || clean.encode_utf16().count() > 60 {
        return Err(Failure::Invalid(
            "Use a tag name between 1 and 60 characters".into(),
        ));
    }
    crate::annotations::ensure_tag(db, clean, &chrono::Utc::now().to_rfc3339())
}
fn membership(
    db: &Connection,
    tag: &str,
    document: &str,
    included: bool,
    access: IndexAccess,
) -> Result<()> {
    document_exists(db, document)?;
    if !db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM tags WHERE id=?)",
            [tag],
            |row| row.get::<_, bool>(0),
        )
        .map_err(storage)?
    {
        return Err(Failure::Invalid("Tag no longer exists".into()));
    }
    let changed = if included {
        db.execute(
            "INSERT OR IGNORE INTO document_tags(document_id,tag_id) VALUES(?,?)",
            params![document, tag],
        )
    } else {
        db.execute(
            "DELETE FROM document_tags WHERE document_id=? AND tag_id=?",
            params![document, tag],
        )
    }
    .map_err(storage)?;
    if changed > 0 {
        crate::search::refresh_document_tags(db, document, access)?;
    }
    Ok(())
}
