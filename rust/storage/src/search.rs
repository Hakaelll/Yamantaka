//! FTS5 operations on the existing schema. No indexing happens on database open.
use crate::{Database, database::storage};
use rusqlite::{Connection, OptionalExtension, params, params_from_iter, types::Value};
use yamantaka_domain::{Cancellation, Failure, Result, search::*};

fn enabled(access: IndexAccess) -> Result<()> {
    if access == IndexAccess::Enabled {
        Ok(())
    } else {
        Err(Failure::Invalid("Global indexing is disabled".into()))
    }
}
fn revision(previous: &str) -> String {
    let mut now = chrono::Utc::now();
    if let Ok(old) = chrono::DateTime::parse_from_rfc3339(previous)
        && now <= old
    {
        now = old.with_timezone(&chrono::Utc) + chrono::Duration::nanoseconds(1);
    }
    now.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}
fn state(db: &Connection, id: &str) -> Result<Option<IndexState>> {
    db.query_row("SELECT document_id,fingerprint,status,indexed_pages,updated_at,error_code FROM search_index_state WHERE document_id=?", [id], |r| Ok(IndexState {
        document_id:r.get(0)?, fingerprint:r.get(1)?, status:IndexStatus::from_name(&r.get::<_,String>(2)?), indexed_pages:r.get(3)?, updated_at:r.get(4)?, error_code:r.get(5)?
    })).optional().map_err(storage)
}
fn document_identity(db: &Connection, id: &str) -> Result<(String, u32)> {
    db.query_row(
        "SELECT fingerprint,page_count FROM documents WHERE id=?",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
    .map_err(storage)?
    .ok_or_else(|| Failure::Invalid("Document is absent from the library".into()))
}
fn verify_session(db: &Connection, session: &IndexSession) -> Result<()> {
    let (fingerprint, pages) = document_identity(db, &session.document_id)?;
    let current = state(db, &session.document_id)?
        .ok_or_else(|| Failure::Invalid("Index session no longer exists".into()))?;
    if fingerprint != session.fingerprint
        || pages != session.page_count
        || current.fingerprint != session.fingerprint
        || current.status != IndexStatus::Indexing
        || current.indexed_pages != session.cursor
        || current.updated_at != session.revision
    {
        return Err(Failure::Invalid(
            "Index session is obsolete; restart or resume indexing".into(),
        ));
    }
    Ok(())
}
const TITLE: &str = "COALESCE(d.title_override,d.title,d.source_title,d.file_name)";
pub(crate) fn refresh_document_tags(db: &Connection, id: &str, access: IndexAccess) -> Result<()> {
    let excluded:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM search_index_state WHERE document_id=? AND status='excluded')",[id],|row|row.get(0)).map_err(storage)?;
    if excluded {
        return Ok(());
    }
    if access == IndexAccess::Disabled {
        let cached: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM library_search WHERE document_id=?)",
                [id],
                |row| row.get(0),
            )
            .map_err(storage)?;
        if !cached {
            return Ok(());
        }
    }
    db.execute(
        "DELETE FROM library_search WHERE document_id=? AND kind='tag'",
        [id],
    )
    .map_err(storage)?;
    metadata(db, id)?;
    db.execute(&format!("INSERT INTO library_search(document_id,page_index,kind,title,body,tags) SELECT d.id,-1,'tag',{TITLE},'',t.name FROM document_tags dt JOIN tags t ON t.id=dt.tag_id JOIN documents d ON d.id=dt.document_id WHERE d.id=?"),[id]).map_err(storage)?;
    Ok(())
}
pub(crate) fn refresh_metadata(db: &Connection, id: &str) -> Result<()> {
    let excluded: bool = db.query_row("SELECT EXISTS(SELECT 1 FROM search_index_state WHERE document_id=? AND status='excluded')",[id],|row|row.get(0)).map_err(storage)?;
    if !excluded {
        metadata(db, id)?;
    }
    Ok(())
}
/// Expire derived text and any active indexing revision after a source move.
/// Existing exclusions survive, and an unindexed document stays opt-in.
pub(crate) fn source_relocated(db: &Connection, id: &str) -> Result<()> {
    let previous = state(db, id)?;
    db.execute("DELETE FROM library_search WHERE document_id=?", [id])
        .map_err(storage)?;
    if let Some(previous) = previous {
        db.execute("UPDATE search_index_state SET status=CASE WHEN status='excluded' THEN 'excluded' ELSE 'pending' END,indexed_pages=0,updated_at=?1,error_code=NULL WHERE document_id=?2",params![revision(&previous.updated_at),id]).map_err(storage)?;
    }
    Ok(())
}
/// Update one auxiliary entry inside the caller's bookmark transaction. This
/// neither refreshes the catalogue nor changes an active PDF index revision.
pub(crate) fn refresh_bookmark_page(
    db: &Connection,
    id: &str,
    page: u32,
    access: IndexAccess,
) -> Result<()> {
    if access == IndexAccess::Disabled {
        return Ok(());
    }
    db.execute(
        "DELETE FROM library_search WHERE document_id=? AND page_index=? AND kind='bookmark'",
        params![id, page],
    )
    .map_err(storage)?;
    db.execute(&format!("INSERT INTO library_search(document_id,page_index,kind,title,body,tags) SELECT b.document_id,b.page_index,'bookmark',{TITLE},COALESCE(b.label,'') || ' página ' || CAST(b.page_index+1 AS TEXT),'' FROM bookmarks b JOIN documents d ON d.id=b.document_id WHERE b.document_id=? AND b.page_index=? AND NOT EXISTS(SELECT 1 FROM search_index_state s WHERE s.document_id=d.id AND s.status='excluded')"),params![id,page]).map_err(storage)?;
    Ok(())
}
const NEEDS_INDEX: &str = "(s.document_id IS NULL OR s.fingerprint<>d.fingerprint OR s.status IN ('pending','indexing','failed') OR s.indexed_pages<>d.page_count OR NOT EXISTS(SELECT 1 FROM library_search m WHERE m.document_id=d.id AND m.kind='metadata' AND m.title=COALESCE(d.title_override,d.title,d.source_title,d.file_name) AND m.body=COALESCE(d.author_override,d.author,d.source_author,'') || ' ' || d.file_name || ' ' || COALESCE((SELECT note FROM document_notes n WHERE n.document_id=d.id),'') AND m.tags=COALESCE((SELECT group_concat(t.name,' ') FROM document_tags dt JOIN tags t ON t.id=dt.tag_id WHERE dt.document_id=d.id),'')))";
fn metadata(db: &Connection, id: &str) -> Result<()> {
    db.execute("UPDATE library_search SET tags=COALESCE((SELECT group_concat(t.name,' ') FROM document_tags dt JOIN tags t ON t.id=dt.tag_id WHERE dt.document_id=?),'') WHERE document_id=? AND kind='pageText'",params![id,id]).map_err(storage)?;
    db.execute(&format!("UPDATE library_search SET title=(SELECT {TITLE} FROM documents d WHERE d.id=?) WHERE document_id=?"), params![id,id]).map_err(storage)?;
    db.execute(
        "DELETE FROM library_search WHERE document_id=? AND kind='metadata'",
        [id],
    )
    .map_err(storage)?;
    db.execute(&format!("INSERT INTO library_search(document_id,page_index,kind,title,body,tags) SELECT d.id,-1,'metadata',{TITLE},COALESCE(d.author_override,d.author,d.source_author,'') || ' ' || d.file_name || ' ' || COALESCE(n.note,''),COALESCE((SELECT group_concat(t.name,' ') FROM tags t JOIN document_tags dt ON dt.tag_id=t.id WHERE dt.document_id=d.id),'') FROM documents d LEFT JOIN document_notes n ON n.document_id=d.id WHERE d.id=?"), [id]).map_err(storage)?;
    Ok(())
}
impl SearchRepository for Database {
    fn index_candidates(
        &self,
        after_id: Option<&str>,
        limit: usize,
        access: IndexAccess,
        cancel: &Cancellation,
    ) -> Result<Vec<String>> {
        enabled(access)?;
        cancel.check()?;
        let sql = format!(
            "SELECT d.id FROM documents d LEFT JOIN search_index_state s ON s.document_id=d.id WHERE d.is_missing=0 AND COALESCE(s.status,'pending')<>'excluded' AND d.id>? AND {NEEDS_INDEX} ORDER BY d.id LIMIT ?"
        );
        let mut statement = self.connection.prepare(&sql).map_err(storage)?;
        let rows = statement
            .query_map(
                params![after_id.unwrap_or(""), limit.clamp(1, 80) as i64],
                |r| r.get::<_, String>(0),
            )
            .map_err(storage)?;
        let mut ids = Vec::new();
        for row in rows {
            cancel.check()?;
            ids.push(row.map_err(storage)?);
        }
        cancel.check()?;
        Ok(ids)
    }
    fn needs_index(&self, id: &str) -> Result<bool> {
        self.connection.query_row(&format!("SELECT COALESCE(s.status,'pending')<>'excluded' AND {NEEDS_INDEX} FROM documents d LEFT JOIN search_index_state s ON s.document_id=d.id WHERE d.id=?"),[id],|r|r.get(0)).optional().map_err(storage)?.ok_or_else(||Failure::Invalid("Document is absent from the library".into()))
    }
    fn search(&self, query: &SearchQuery, cancel: &Cancellation) -> Result<Vec<SearchResult>> {
        cancel.check()?;
        let expression = fts_expression(&query.text);
        if expression.is_empty() {
            return Ok(Vec::new());
        }
        let mut sql = String::from(
            "SELECT library_search.document_id,library_search.page_index,library_search.kind,library_search.title,snippet(library_search,4,'‹','›',' … ',18),bm25(library_search,4.0,1.0,0.5) AS rank,d.is_missing,d.is_modified FROM library_search JOIN documents d ON d.id=library_search.document_id WHERE library_search MATCH ? AND NOT EXISTS(SELECT 1 FROM search_index_state s WHERE s.document_id=d.id AND s.status='excluded')",
        );
        let mut values = vec![Value::Text(expression)];
        let mut kinds = query.kinds.clone();
        kinds.sort();
        kinds.dedup();
        if !kinds.is_empty() {
            sql.push_str(&format!(
                " AND library_search.kind IN ({})",
                vec!["?"; kinds.len()].join(",")
            ));
            values.extend(kinds.iter().map(|k| Value::Text(k.as_str().into())));
        }
        if let Some(id) = &query.document_id {
            sql.push_str(" AND d.id=?");
            values.push(Value::Text(id.clone()));
        }
        for (column, value) in [("is_missing", query.missing), ("is_modified", query.stale)] {
            if let Some(value) = value {
                sql.push_str(&format!(" AND d.{column}=?"));
                values.push(Value::Integer(i64::from(value)));
            }
        }
        sql.push_str(" ORDER BY rank,d.last_opened_at DESC,library_search.document_id,library_search.page_index,library_search.kind,library_search.rowid LIMIT ? OFFSET ?");
        values.push(Value::Integer(query.limit.clamp(1, 200) as i64));
        values.push(Value::Integer(query.offset.min(10000) as i64));
        let mut statement = self.connection.prepare(&sql).map_err(storage)?;
        let rows = statement
            .query_map(params_from_iter(values), |r| {
                let page: i64 = r.get(1)?;
                Ok(SearchResult {
                    document_id: r.get(0)?,
                    page: if page < 0 { None } else { Some(page as u32) },
                    kind: SearchKind::from_name(&r.get::<_, String>(2)?),
                    title: r.get(3)?,
                    snippet: r.get(4)?,
                    rank: r.get(5)?,
                    missing: r.get(6)?,
                    stale: r.get(7)?,
                })
            })
            .map_err(storage)?;
        let mut results = Vec::new();
        for row in rows {
            cancel.check()?;
            results.push(row.map_err(storage)?);
        }
        cancel.check()?;
        Ok(results)
    }
    fn begin_index(
        &mut self,
        id: &str,
        resume: bool,
        access: IndexAccess,
        cancel: &Cancellation,
    ) -> Result<IndexSession> {
        enabled(access)?;
        cancel.check()?;
        let tx = self.connection.transaction().map_err(storage)?;
        let (fingerprint, page_count) = document_identity(&tx, id)?;
        if page_count > 10_000_000 {
            return Err(Failure::Invalid("Page count exceeds index limits".into()));
        }
        let old = state(&tx, id)?;
        if old
            .as_ref()
            .is_some_and(|s| s.status == IndexStatus::Excluded)
        {
            return Err(Failure::Invalid(
                "Document is excluded from indexing".into(),
            ));
        }
        let cursor = old
            .as_ref()
            .filter(|s| {
                resume
                    && s.status == IndexStatus::Indexing
                    && s.fingerprint == fingerprint
                    && s.indexed_pages <= page_count
            })
            .map_or(0, |s| s.indexed_pages);
        let can_resume = old.as_ref().is_some_and(|s| {
            resume
                && s.status == IndexStatus::Indexing
                && s.fingerprint == fingerprint
                && s.indexed_pages <= page_count
        });
        if !can_resume {
            tx.execute("DELETE FROM library_search WHERE document_id=?", [id])
                .map_err(storage)?;
        }
        metadata(&tx, id)?;
        let next = revision(old.as_ref().map_or("", |s| s.updated_at.as_str()));
        tx.execute("INSERT INTO search_index_state(document_id,fingerprint,status,indexed_pages,updated_at,error_code) VALUES(?,?,'indexing',?,?,NULL) ON CONFLICT(document_id) DO UPDATE SET fingerprint=excluded.fingerprint,status='indexing',indexed_pages=excluded.indexed_pages,updated_at=excluded.updated_at,error_code=NULL", params![id,fingerprint,cursor,next]).map_err(storage)?;
        cancel.check()?;
        tx.commit().map_err(storage)?;
        Ok(IndexSession {
            document_id: id.into(),
            fingerprint,
            page_count,
            cursor,
            revision: next,
        })
    }
    fn index_page(
        &mut self,
        session: &mut IndexSession,
        page: u32,
        text: &str,
        access: IndexAccess,
        cancel: &Cancellation,
    ) -> Result<()> {
        enabled(access)?;
        cancel.check()?;
        if page >= session.page_count || page > session.cursor {
            return Err(Failure::Invalid(
                "Index pages must be committed in cursor order".into(),
            ));
        }
        let tx = self.connection.transaction().map_err(storage)?;
        verify_session(&tx, session)?;
        tx.execute(
            "DELETE FROM library_search WHERE document_id=? AND page_index=? AND kind='pageText'",
            params![session.document_id, page],
        )
        .map_err(storage)?;
        if !text.trim().is_empty() {
            tx.execute(&format!("INSERT INTO library_search(document_id,page_index,kind,title,body,tags) SELECT d.id,?,'pageText',{TITLE},?,COALESCE((SELECT group_concat(t.name,' ') FROM document_tags dt JOIN tags t ON t.id=dt.tag_id WHERE dt.document_id=d.id),'') FROM documents d WHERE d.id=?"), params![page,text,session.document_id]).map_err(storage)?;
        }
        let cursor = if page == session.cursor {
            session.cursor + 1
        } else {
            session.cursor
        };
        let next = revision(&session.revision);
        tx.execute(
            "UPDATE search_index_state SET indexed_pages=?,updated_at=? WHERE document_id=?",
            params![cursor, next, session.document_id],
        )
        .map_err(storage)?;
        cancel.check()?;
        tx.commit().map_err(storage)?;
        session.cursor = cursor;
        session.revision = next;
        Ok(())
    }
    fn finish_index(
        &mut self,
        session: &mut IndexSession,
        access: IndexAccess,
        cancel: &Cancellation,
    ) -> Result<()> {
        enabled(access)?;
        cancel.check()?;
        if session.cursor != session.page_count {
            return Err(Failure::Invalid("Indexing has uncommitted pages".into()));
        }
        let tx = self.connection.transaction().map_err(storage)?;
        verify_session(&tx, session)?;
        let has_text:bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM library_search WHERE document_id=? AND kind='pageText')", [&session.document_id], |r| r.get(0)).map_err(storage)?;
        let next = revision(&session.revision);
        tx.execute("UPDATE search_index_state SET status=?,updated_at=?,error_code=NULL WHERE document_id=?", params![if has_text {"indexed"} else {"notIndexable"},next,session.document_id]).map_err(storage)?;
        cancel.check()?;
        tx.commit().map_err(storage)?;
        session.revision = next;
        Ok(())
    }
    fn index_state(&self, id: &str) -> Result<Option<IndexState>> {
        state(&self.connection, id)
    }
    fn fail_index(
        &mut self,
        session: &mut IndexSession,
        code: &str,
        cancel: &Cancellation,
    ) -> Result<()> {
        Database::fail_index(self, session, code, cancel)
    }
    fn index_stats(&self) -> Result<IndexStats> {
        let count = |status: &str| {
            self.connection.query_row("SELECT COUNT(*) FROM search_index_state s JOIN documents d ON d.id=s.document_id WHERE s.status=?",[status],|r| r.get::<_,i64>(0)).map(|n|n as u64).map_err(storage)
        };
        Ok(IndexStats {
            indexed: count("indexed")?,
            indexing: count("indexing")?,
            not_indexable: count("notIndexable")?,
            excluded: count("excluded")?,
            failed: count("failed")?,
            rows: self
                .connection
                .query_row(
                    "SELECT COUNT(*) FROM library_search s JOIN documents d ON d.id=s.document_id",
                    [],
                    |r| r.get::<_, i64>(0),
                )
                .map(|n| n as u64)
                .map_err(storage)?,
        })
    }
    fn exclude_from_index(
        &mut self,
        id: &str,
        excluded: bool,
        cancel: &Cancellation,
    ) -> Result<()> {
        cancel.check()?;
        let tx = self.connection.transaction().map_err(storage)?;
        let (fingerprint, _) = document_identity(&tx, id)?;
        let old = state(&tx, id)?;
        tx.execute("DELETE FROM library_search WHERE document_id=?", [id])
            .map_err(storage)?;
        tx.execute("INSERT INTO search_index_state(document_id,fingerprint,status,indexed_pages,updated_at,error_code) VALUES(?,?,?,0,?,NULL) ON CONFLICT(document_id) DO UPDATE SET fingerprint=excluded.fingerprint,status=excluded.status,indexed_pages=0,updated_at=excluded.updated_at,error_code=NULL",params![id,fingerprint,if excluded {"excluded"} else {"pending"},revision(old.as_ref().map_or("",|s| s.updated_at.as_str()))]).map_err(storage)?;
        cancel.check()?;
        tx.commit().map_err(storage)
    }
    fn refresh_search_entries(&mut self, access: IndexAccess, cancel: &Cancellation) -> Result<()> {
        enabled(access)?;
        cancel.check()?;
        let tx = self.connection.transaction().map_err(storage)?;
        let ids = tx.prepare("SELECT document_id FROM search_index_state WHERE status <> 'excluded' ORDER BY document_id").map_err(storage)?.query_map([],|r|r.get::<_,String>(0)).map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;
        for id in ids {
            cancel.check()?;
            metadata(&tx, &id)?;
        }
        tx.execute("DELETE FROM library_search WHERE kind IN ('annotation','bookmark','collection','tag','relation') OR NOT EXISTS(SELECT 1 FROM documents d WHERE d.id=library_search.document_id)", []).map_err(storage)?;
        let eligible = "JOIN search_index_state s ON s.document_id=d.id AND s.status<>'excluded'";
        for sql in [
            format!(
                "INSERT INTO library_search SELECT a.document_id,a.page_index,'annotation',{TITLE},COALESCE(a.selected_text,'') || ' ' || COALESCE(a.note,''),COALESCE((SELECT group_concat(t.name,' ') FROM annotation_tags at JOIN tags t ON t.id=at.tag_id WHERE at.annotation_id=a.id),'') FROM annotations a JOIN documents d ON d.id=a.document_id {eligible}"
            ),
            format!(
                "INSERT INTO library_search SELECT b.document_id,b.page_index,'bookmark',{TITLE},COALESCE(b.label,'') || ' página ' || CAST(b.page_index+1 AS TEXT),'' FROM bookmarks b JOIN documents d ON d.id=b.document_id WHERE NOT EXISTS(SELECT 1 FROM search_index_state s WHERE s.document_id=d.id AND s.status='excluded')"
            ),
            format!(
                "INSERT INTO library_search SELECT cd.document_id,-1,'collection',{TITLE},c.name,'' FROM collection_documents cd JOIN collections c ON c.id=cd.collection_id JOIN documents d ON d.id=cd.document_id {eligible}"
            ),
            format!(
                "INSERT INTO library_search SELECT dt.document_id,-1,'tag',{TITLE},'',t.name FROM document_tags dt JOIN tags t ON t.id=dt.tag_id JOIN documents d ON d.id=dt.document_id {eligible}"
            ),
            format!(
                "INSERT INTO library_search SELECT d.id,-1,'relation',{TITLE},r.relation_type || ' ' || COALESCE(other.title_override,other.title,other.source_title,other.file_name),'' FROM document_relations r JOIN documents d ON d.id=r.source_document_id JOIN documents other ON other.id=r.target_document_id {eligible} WHERE NOT EXISTS(SELECT 1 FROM search_index_state target_state WHERE target_state.document_id=other.id AND target_state.status='excluded')"
            ),
        ] {
            cancel.check()?;
            tx.execute(&sql, []).map_err(storage)?;
        }
        cancel.check()?;
        tx.commit().map_err(storage)
    }
    fn clear_index(&mut self, cancel: &Cancellation) -> Result<()> {
        cancel.check()?;
        let tx = self.connection.transaction().map_err(storage)?;
        tx.execute("DELETE FROM library_search", [])
            .map_err(storage)?;
        // Privacy exclusions survive a rebuild, so it cannot silently reindex them.
        tx.execute(
            "DELETE FROM search_index_state WHERE status<>'excluded'",
            [],
        )
        .map_err(storage)?;
        cancel.check()?;
        tx.commit().map_err(storage)
    }
}

impl Database {
    /// Failure codes are short machine labels; never persist native errors,
    /// paths, extracted text, or passwords in index diagnostics.
    pub fn fail_index(
        &mut self,
        session: &mut IndexSession,
        code: &str,
        cancel: &Cancellation,
    ) -> Result<()> {
        if code.is_empty()
            || code.len() > 64
            || !code.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(Failure::Invalid("Invalid index failure code".into()));
        }
        cancel.check()?;
        let tx = self.connection.transaction().map_err(storage)?;
        verify_session(&tx, session)?;
        let next = revision(&session.revision);
        tx.execute("UPDATE search_index_state SET status='failed',error_code=?,updated_at=? WHERE document_id=?", params![code,next,session.document_id]).map_err(storage)?;
        cancel.check()?;
        tx.commit().map_err(storage)?;
        session.revision = next;
        Ok(())
    }
}
