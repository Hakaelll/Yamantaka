use super::*;
use chrono::{SecondsFormat, Utc};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

fn target(db: &Connection, document: &str, id: &str) -> Result<u32> {
    db.query_row(
        "SELECT page_index FROM annotations WHERE id=? AND document_id=?",
        params![id, document],
        |row| row.get(0),
    )
    .optional()
    .map_err(storage)?
    .ok_or_else(|| Failure::Invalid("Annotation no longer belongs to this document".into()))
}
fn ensure_tag(db: &Connection, tag: &str, now: &str) -> Result<String> {
    let tag = tag.trim();
    if tag.is_empty() || tag.len() > 4096 {
        return Err(Failure::Invalid(
            "Annotation tag is empty or too long".into(),
        ));
    }
    let normalized = tag.to_lowercase();
    if let Some(id) = db
        .query_row(
            "SELECT id FROM tags WHERE normalized_name=?",
            [&normalized],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(storage)?
    {
        return Ok(id);
    }
    let id = format!("{:x}", Sha256::digest(format!("tag:{normalized}")));
    db.execute("INSERT INTO tags(id,name,normalized_name,color,created_at,updated_at) VALUES(?,?,?,'blue',?,?)",params![id,tag,normalized,now,now]).map_err(storage)?;
    Ok(id)
}
fn tags(
    db: &Connection,
    id: &str,
    values: &[String],
    now: &str,
    cancel: &Cancellation,
) -> Result<()> {
    if values.len() > 12 {
        return Err(Failure::Invalid(
            "Annotations support at most twelve tags".into(),
        ));
    }
    db.execute("DELETE FROM annotation_tags WHERE annotation_id=?", [id])
        .map_err(storage)?;
    for value in values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        cancel.check()?;
        let tag = ensure_tag(db, value, now)?;
        db.execute(
            "INSERT OR IGNORE INTO annotation_tags(annotation_id,tag_id) VALUES(?,?)",
            params![id, tag],
        )
        .map_err(storage)?;
    }
    Ok(())
}
struct SnapshotFields<'a> {
    created: &'a str,
    updated: &'a str,
    origin_document_id: Option<&'a str>,
    origin_fingerprint: Option<&'a str>,
    extensions: &'a [(String, AnnotationColumnValue)],
}
fn column_value(value: &AnnotationColumnValue) -> rusqlite::types::ToSqlOutput<'_> {
    rusqlite::types::ToSqlOutput::Borrowed(match value {
        AnnotationColumnValue::Null => rusqlite::types::ValueRef::Null,
        AnnotationColumnValue::Integer(value) => rusqlite::types::ValueRef::Integer(*value),
        AnnotationColumnValue::Real(value) => rusqlite::types::ValueRef::Real(*value),
        AnnotationColumnValue::Text(value) => rusqlite::types::ValueRef::Text(value.as_bytes()),
        AnnotationColumnValue::Blob(value) => rusqlite::types::ValueRef::Blob(value),
    })
}
fn save(
    db: &Connection,
    id: &str,
    values: &AnnotationDraftValues,
    snapshot: SnapshotFields<'_>,
    tag_values: &[String],
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<()> {
    if values
        .note
        .as_ref()
        .is_some_and(|note| note.encode_utf16().count() > MAX_ANNOTATION_NOTE_UTF16)
    {
        return Err(Failure::Invalid(
            "Annotation note exceeds 4000 UTF-16 units".into(),
        ));
    }
    PageRotation::from_degrees(i64::from(values.page_rotation))?;
    let bytes = values
        .rects
        .len()
        .checked_mul(160)
        .and_then(|bytes| bytes.checked_add(2))
        .ok_or(Failure::MemoryBudget)?;
    let mut lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut json = Vec::new();
    json.try_reserve_exact(bytes)
        .map_err(|_| Failure::MemoryBudget)?;
    serde_json::to_writer(&mut json, &values.rects).map_err(storage)?;
    lease
        .try_resize(json.capacity())
        .map_err(|_| Failure::MemoryBudget)?;
    let rects = std::str::from_utf8(&json)
        .map_err(|_| Failure::Invalid("Annotation geometry encoding failed".into()))?;
    let existing: Option<(String, u32)> = db
        .query_row(
            "SELECT document_id,page_index FROM annotations WHERE id=?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(storage)?;
    if existing
        .is_some_and(|(document, page)| document != values.document_id || page != values.page_index)
    {
        return Err(Failure::Invalid(
            "Annotation identity and page are immutable".into(),
        ));
    }
    let metadata_bytes = snapshot
        .extensions
        .iter()
        .try_fold(4096usize, |bytes, (name, _)| {
            bytes
                .checked_add(name.len().checked_mul(16)?)?
                .checked_add(128)
        })
        .ok_or(Failure::MemoryBudget)?;
    let _metadata = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            metadata_bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut columns = String::from(
        "id,document_id,page_index,type,color,note,selected_text,context_before,context_after,rects_json,page_rotation,document_fingerprint,status,created_at,updated_at,review_state,origin_document_id,origin_fingerprint",
    );
    let mut updates = String::from(
        "type=excluded.type,color=excluded.color,note=excluded.note,selected_text=excluded.selected_text,context_before=excluded.context_before,context_after=excluded.context_after,rects_json=excluded.rects_json,page_rotation=excluded.page_rotation,document_fingerprint=excluded.document_fingerprint,status=excluded.status,updated_at=excluded.updated_at,review_state=excluded.review_state,origin_document_id=excluded.origin_document_id,origin_fingerprint=excluded.origin_fingerprint",
    );
    let mut extra_values = Vec::new();
    extra_values
        .try_reserve_exact(snapshot.extensions.len())
        .map_err(|_| Failure::MemoryBudget)?;
    for (index, (column, value)) in snapshot.extensions.iter().enumerate() {
        cancel.check()?;
        if known_column(column)
            || snapshot.extensions[..index]
                .iter()
                .any(|(name, _)| name == column)
            || db
                .query_row(
                    "SELECT 1 FROM pragma_table_info('annotations') WHERE name=?",
                    [column],
                    |row| row.get::<_, u8>(0),
                )
                .optional()
                .map_err(storage)?
                .is_none()
        {
            return Err(Failure::Invalid(
                "Annotation extension column no longer matches its schema".into(),
            ));
        }
        let identifier = column.replace('"', "\"\"");
        columns.push_str(&format!(",\"{identifier}\""));
        updates.push_str(&format!(",\"{identifier}\"=excluded.\"{identifier}\""));
        extra_values.push(column_value(value));
    }
    let kind = name(values.kind)?;
    let color = name(values.color)?;
    let status = name(values.status)?;
    let review = name(values.review_state)?;
    let base = params![
        id,
        values.document_id,
        values.page_index,
        kind,
        color,
        values.note,
        values.selected_text,
        values.context_before,
        values.context_after,
        rects,
        values.page_rotation,
        values.document_fingerprint,
        status,
        snapshot.created,
        snapshot.updated,
        review,
        snapshot.origin_document_id,
        snapshot.origin_fingerprint
    ];
    let mut parameters: Vec<&dyn rusqlite::ToSql> = Vec::new();
    parameters
        .try_reserve_exact(18 + extra_values.len())
        .map_err(|_| Failure::MemoryBudget)?;
    parameters.extend_from_slice(base);
    parameters.extend(
        extra_values
            .iter()
            .map(|value| value as &dyn rusqlite::ToSql),
    );
    let placeholders = vec!["?"; parameters.len()].join(",");
    db.execute(&format!("INSERT INTO annotations({columns}) VALUES({placeholders}) ON CONFLICT(id) DO UPDATE SET {updates}"),params_from_iter(parameters)).map_err(storage)?;
    tags(db, id, tag_values, snapshot.updated, cancel)
}
pub(super) fn refresh(
    db: &Connection,
    document: &str,
    page: u32,
    access: IndexAccess,
) -> Result<()> {
    if access == IndexAccess::Disabled {
        return Ok(());
    }
    db.execute(
        "DELETE FROM library_search WHERE document_id=? AND page_index=? AND kind='annotation'",
        params![document, page],
    )
    .map_err(storage)?;
    db.execute("INSERT INTO library_search(document_id,page_index,kind,title,body,tags) SELECT a.document_id,a.page_index,'annotation',COALESCE(d.title_override,d.source_title,d.title,d.file_name),COALESCE(a.selected_text,'') || ' ' || COALESCE(a.note,''),COALESCE((SELECT group_concat(t.name,' ') FROM annotation_tags at JOIN tags t ON t.id=at.tag_id WHERE at.annotation_id=a.id),'') FROM annotations a JOIN documents d ON d.id=a.document_id WHERE a.document_id=? AND a.page_index=? AND NOT EXISTS(SELECT 1 FROM search_index_state s WHERE s.document_id=d.id AND s.status='excluded')",params![document,page]).map_err(storage)?;
    Ok(())
}
struct CreationContext<'a> {
    document: &'a str,
    now: &'a str,
    governor: &'a MemoryGovernor,
    cancel: &'a Cancellation,
}
fn create_draft(
    db: &Connection,
    draft: &AnnotationDraft,
    confirm_page_note: bool,
    tag_values: &[String],
    context: &CreationContext<'_>,
) -> Result<u32> {
    let CreationContext {
        document,
        now,
        governor,
        cancel,
    } = *context;
    cancel.check()?;
    if draft.values().document_id != document {
        return Err(Failure::Invalid(
            "Annotation draft belongs to another document".into(),
        ));
    }
    if matches!(
        draft.placement(),
        AnnotationPlacement::PageNoteFallback { .. }
    ) && !confirm_page_note
    {
        return Err(Failure::Invalid(
            "Confirm the page-note fallback before saving".into(),
        ));
    }
    let id = uuid::Uuid::new_v4().to_string();
    save(
        db,
        &id,
        draft.values(),
        SnapshotFields {
            created: now,
            updated: now,
            origin_document_id: None,
            origin_fingerprint: None,
            extensions: &[],
        },
        tag_values,
        governor,
        cancel,
    )?;
    Ok(draft.values().page_index)
}
pub(super) fn change(
    db: &mut Database,
    document: &str,
    change: AnnotationChange<'_>,
    query: &AnnotationQuery,
    access: IndexAccess,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<AnnotationPage> {
    cancel.check()?;
    let tx = db.connection.transaction().map_err(storage)?;
    if tx
        .query_row("SELECT 1 FROM documents WHERE id=?", [document], |row| {
            row.get::<_, u8>(0)
        })
        .optional()
        .map_err(storage)?
        .is_none()
    {
        return Err(Failure::Invalid(
            "Annotation document no longer exists".into(),
        ));
    }
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true);
    let mut pages = BTreeSet::new();
    let creation = CreationContext {
        document,
        now: &now,
        governor,
        cancel,
    };
    match change {
        AnnotationChange::CreateBatch {
            batch,
            confirm_page_note,
            tags,
        } => {
            for draft in batch.drafts() {
                pages.insert(create_draft(
                    &tx,
                    draft,
                    confirm_page_note,
                    tags,
                    &creation,
                )?);
            }
        }
        AnnotationChange::Create {
            draft,
            confirm_page_note,
            tags: tag_values,
        } => {
            pages.insert(create_draft(
                &tx,
                draft,
                confirm_page_note,
                tag_values,
                &creation,
            )?);
        }
        AnnotationChange::Save { record } => {
            let record = record.data();
            if record.values.document_id != document {
                return Err(Failure::Invalid(
                    "Annotation snapshot belongs to another document".into(),
                ));
            }
            save(
                &tx,
                &record.id,
                &record.values,
                SnapshotFields {
                    created: &record.created_at,
                    updated: &record.updated_at,
                    origin_document_id: record.origin_document_id.as_deref(),
                    origin_fingerprint: record.origin_fingerprint.as_deref(),
                    extensions: &record.extensions,
                },
                &record.tags,
                governor,
                cancel,
            )?;
            pages.insert(record.values.page_index);
        }
        AnnotationChange::Remove { id } => {
            pages.insert(target(&tx, document, id)?);
            tx.execute(
                "DELETE FROM annotations WHERE id=? AND document_id=?",
                params![id, document],
            )
            .map_err(storage)?;
        }
        AnnotationChange::Edit {
            id,
            kind,
            color,
            note,
            review,
            tags: tag_values,
        } => {
            if note.is_some_and(|note| note.encode_utf16().count() > MAX_ANNOTATION_NOTE_UTF16) {
                return Err(Failure::Invalid(
                    "Annotation note exceeds 4000 UTF-16 units".into(),
                ));
            }
            pages.insert(target(&tx, document, id)?);
            let note = note.map(str::trim).filter(|note| !note.is_empty());
            tx.execute("UPDATE annotations SET type=?,color=?,note=?,review_state=?,updated_at=? WHERE id=? AND document_id=?",params![name(kind)?,name(color)?,note,name(review)?,now,id,document]).map_err(storage)?;
            if let Some(values) = tag_values {
                tags(&tx, id, values, &now, cancel)?;
            }
        }
        AnnotationChange::Review { ids, state } => {
            validate_batch(ids.len())?;
            for id in ids.iter().collect::<BTreeSet<_>>() {
                cancel.check()?;
                pages.insert(review_one(&tx, document, id, state, &now)?);
            }
        }
        AnnotationChange::AddTag { ids, tag } => {
            validate_batch(ids.len())?;
            for id in ids.iter().collect::<BTreeSet<_>>() {
                cancel.check()?;
                pages.insert(tag_one(&tx, document, id, tag, &now)?);
            }
        }
        AnnotationChange::CheckAttachment { fingerprint } => {
            tx.execute("UPDATE annotations SET status=CASE WHEN document_fingerprint=? THEN 'attached' ELSE 'reviewRequired' END WHERE document_id=?",params![fingerprint,document]).map_err(storage)?;
        }
    }
    for page in pages {
        cancel.check()?;
        refresh(&tx, document, page, access)?;
    }
    let result = read_page(&tx, query, governor, cancel)?;
    cancel.check()?;
    tx.commit().map_err(storage)?;
    Ok(result)
}

pub(super) fn validate_batch(count: usize) -> Result<()> {
    if count > 80 {
        return Err(Failure::Invalid(
            "Annotation batches support at most eighty targets".into(),
        ));
    }
    Ok(())
}
pub(super) fn review_one(
    db: &Connection,
    document: &str,
    id: &str,
    state: AnnotationReviewState,
    now: &str,
) -> Result<u32> {
    let page = target(db, document, id)?;
    db.execute(
        "UPDATE annotations SET review_state=?,updated_at=? WHERE id=? AND document_id=?",
        params![name(state)?, now, id, document],
    )
    .map_err(storage)?;
    Ok(page)
}
pub(super) fn tag_one(
    db: &Connection,
    document: &str,
    id: &str,
    tag: &str,
    now: &str,
) -> Result<u32> {
    let page = target(db, document, id)?;
    let tag = ensure_tag(db, tag, now)?;
    let (count, present): (i64, bool) = db
        .query_row(
            "SELECT count(*),COALESCE(max(tag_id=?),0) FROM annotation_tags WHERE annotation_id=?",
            params![tag, id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(storage)?;
    if count >= 12 && !present {
        return Err(Failure::Invalid(
            "Annotations support at most twelve tags".into(),
        ));
    }
    db.execute(
        "INSERT OR IGNORE INTO annotation_tags(annotation_id,tag_id) VALUES(?,?)",
        params![id, tag],
    )
    .map_err(storage)?;
    Ok(page)
}
