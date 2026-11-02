use super::*;
fn copy_missing(db: &Connection, table: &str, kept: &str, incoming: &str) -> Result<()> {
    let columns = db
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(storage)?
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(storage)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(storage)?;
    let names = columns
        .iter()
        .map(|name| format!("\"{}\"", name.replace('"', "\"\"")))
        .collect::<Vec<_>>();
    let values = columns
        .iter()
        .zip(&names)
        .map(|(name, quoted)| {
            if name == "document_id" {
                "?1"
            } else {
                quoted.as_str()
            }
        })
        .collect::<Vec<_>>();
    db.execute(&format!("INSERT INTO {table}({}) SELECT {} FROM {table} WHERE document_id=?2 AND NOT EXISTS(SELECT 1 FROM {table} WHERE document_id=?1)",names.join(","),values.join(",")),params![kept,incoming]).map_err(storage)?;
    Ok(())
}
pub(super) fn merge(
    db: &Connection,
    plan: &Plan,
    choices: &Choices,
    now: &str,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<()> {
    let kept = &plan.selection.kept.document;
    let incoming = &plan.selection.incoming.document;
    db.execute("UPDATE documents SET title_override=?3,author_override=?4,is_favorite=max(is_favorite,(SELECT is_favorite FROM documents WHERE id=?2)),last_page_index=CASE WHEN reading_progress<(SELECT reading_progress FROM documents WHERE id=?2) THEN (SELECT last_page_index FROM documents WHERE id=?2) ELSE last_page_index END,reading_progress=max(reading_progress,(SELECT reading_progress FROM documents WHERE id=?2)),last_opened_at=max(last_opened_at,(SELECT last_opened_at FROM documents WHERE id=?2)),thumbnail_path=NULL WHERE id=?1",params![kept,incoming,plan.title.resolve(choices.title),plan.author.resolve(choices.author)]).map_err(storage)?;
    if let Some(note) = plan.note.resolve(choices.note) {
        db.execute("INSERT INTO document_notes(document_id,note,updated_at) VALUES(?1,?2,?3) ON CONFLICT(document_id) DO UPDATE SET note=excluded.note,updated_at=excluded.updated_at",params![kept,note,now]).map_err(storage)?;
    }
    copy_missing(db, "document_preferences", kept, incoming)?;
    copy_missing(db, "document_enrichment", kept, incoming)?;
    for sql in [
        "UPDATE document_enrichment SET topics_manual=max(topics_manual,COALESCE((SELECT topics_manual FROM document_enrichment WHERE document_id=?2),0)) WHERE document_id=?1",
        "INSERT INTO document_topics(document_id,topic_key,confidence,origin,analyzer_version,fingerprint,updated_at) SELECT ?1,topic_key,confidence,origin,analyzer_version,fingerprint,updated_at FROM document_topics WHERE document_id=?2 ON CONFLICT(document_id,topic_key) DO UPDATE SET confidence=max(confidence,excluded.confidence),origin=CASE WHEN origin='manual' OR excluded.origin='manual' THEN 'manual' ELSE origin END",
        "UPDATE bookmarks SET label=CASE WHEN label IS NULL OR trim(label)='' THEN (SELECT label FROM bookmarks b WHERE b.document_id=?2 AND b.page_index=bookmarks.page_index) ELSE label END,updated_at=max(updated_at,(SELECT updated_at FROM bookmarks b WHERE b.document_id=?2 AND b.page_index=bookmarks.page_index)) WHERE document_id=?1 AND EXISTS(SELECT 1 FROM bookmarks b WHERE b.document_id=?2 AND b.page_index=bookmarks.page_index)",
        "DELETE FROM bookmarks WHERE document_id=?2 AND page_index IN (SELECT page_index FROM bookmarks WHERE document_id=?1)",
        "UPDATE bookmarks SET document_id=?1 WHERE document_id=?2",
        "UPDATE reading_sessions SET document_id=?1 WHERE document_id=?2",
        "UPDATE annotations SET origin_document_id=COALESCE(origin_document_id,document_id),origin_fingerprint=COALESCE(origin_fingerprint,document_fingerprint),document_id=?1 WHERE document_id=?2",
        "INSERT INTO document_tags(document_id,tag_id) SELECT ?1,tag_id FROM document_tags WHERE document_id=?2 ON CONFLICT(document_id,tag_id) DO NOTHING",
        "INSERT INTO collection_documents(collection_id,document_id,added_at) SELECT collection_id,?1,added_at FROM collection_documents WHERE document_id=?2 ON CONFLICT(collection_id,document_id) DO NOTHING",
        "UPDATE workspaces SET active_tab_id=(SELECT k.id FROM workspace_tabs k WHERE k.workspace_id=workspaces.id AND k.document_id=?1) WHERE active_tab_id IN (SELECT i.id FROM workspace_tabs i WHERE i.document_id=?2 AND i.workspace_id=workspaces.id) AND EXISTS(SELECT 1 FROM workspace_tabs k WHERE k.document_id=?1 AND k.workspace_id=workspaces.id)",
        "DELETE FROM workspace_tabs WHERE document_id=?2 AND workspace_id IN (SELECT workspace_id FROM workspace_tabs WHERE document_id=?1)",
        "UPDATE workspace_tabs SET document_id=?1 WHERE document_id=?2",
        "DELETE FROM document_relations WHERE source_document_id IN (?1,?2) AND target_document_id IN (?1,?2)",
        "DELETE FROM document_relations AS r WHERE (r.source_document_id=?2 OR r.target_document_id=?2) AND EXISTS(SELECT 1 FROM document_relations k WHERE k.id<>r.id AND k.source_document_id=CASE WHEN r.source_document_id=?2 THEN ?1 ELSE r.source_document_id END AND k.target_document_id=CASE WHEN r.target_document_id=?2 THEN ?1 ELSE r.target_document_id END AND k.relation_type=r.relation_type)",
        "UPDATE document_relations SET source_document_id=CASE WHEN source_document_id=?2 THEN ?1 ELSE source_document_id END,target_document_id=CASE WHEN target_document_id=?2 THEN ?1 ELSE target_document_id END WHERE source_document_id=?2 OR target_document_id=?2",
        "INSERT INTO document_revision_members(group_id,document_id,added_at) SELECT group_id,?1,added_at FROM document_revision_members WHERE document_id=?2 ON CONFLICT(group_id,document_id) DO NOTHING",
        "UPDATE document_revision_groups SET current_document_id=?1 WHERE current_document_id=?2",
        "DELETE FROM document_covers WHERE document_id=?1 AND selection_mode='automatic' AND EXISTS(SELECT 1 FROM document_covers WHERE document_id=?2 AND selection_mode='manual')",
        "UPDATE OR IGNORE document_covers SET document_id=?1 WHERE document_id=?2",
        "UPDATE document_covers SET small_thumbnail_path='',large_thumbnail_path='',source_modified_at=(SELECT file_modified_at FROM documents WHERE id=?1) WHERE document_id=?1 AND ?2 IS NOT NULL",
    ] {
        cancel.check()?;
        db.execute(sql, params![kept, incoming]).map_err(storage)?;
    }
    goals(db, kept, incoming, now, governor)?;
    let excluded:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM search_index_state WHERE document_id IN (?1,?2) AND status='excluded')",params![kept,incoming],|row|row.get(0)).map_err(storage)?;
    db.execute(
        "DELETE FROM library_search WHERE document_id IN (?1,?2)",
        params![kept, incoming],
    )
    .map_err(storage)?;
    db.execute(
        "DELETE FROM search_index_state WHERE document_id IN (?1,?2)",
        params![kept, incoming],
    )
    .map_err(storage)?;
    if excluded {
        db.execute("INSERT INTO search_index_state(document_id,fingerprint,status,indexed_pages,updated_at) SELECT id,fingerprint,'excluded',0,?2 FROM documents WHERE id=?1",params![kept,now]).map_err(storage)?;
    }
    if db
        .execute("DELETE FROM documents WHERE id=?1", [incoming])
        .map_err(storage)?
        != 1
    {
        return Err(invalid("Incoming duplicate disappeared"));
    }
    Ok(())
}
fn goals(
    db: &Connection,
    kept: &str,
    incoming: &str,
    now: &str,
    governor: &MemoryGovernor,
) -> Result<()> {
    let bytes: Option<i64> = db
        .query_row(
            "SELECT length(CAST(value AS BLOB)) FROM app_metadata WHERE key='reading_goals_v1'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(storage)?;
    let Some(bytes) = bytes else {
        return Ok(());
    };
    if !(0..=4 * 1024 * 1024).contains(&bytes) {
        return Err(invalid("Reading goals exceed consolidation bounds"));
    }
    let bytes = bytes as usize;
    let _lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Other,
            MemoryRepresentation::Cpu,
            bytes * 8 + 1024,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let valid:bool=db.query_row("SELECT CASE WHEN json_valid(value) THEN json_type(value)='object' AND NOT EXISTS(SELECT 1 FROM json_each(value) GROUP BY key HAVING count(*)<>1) ELSE 0 END FROM app_metadata WHERE key='reading_goals_v1'",[],|row|row.get(0)).map_err(storage)?;
    if !valid {
        return Err(invalid(
            "Reading goals require recovery before consolidation",
        ));
    }
    let text: String = db
        .query_row(
            "SELECT value FROM app_metadata WHERE key='reading_goals_v1'",
            [],
            |row| row.get(0),
        )
        .map_err(storage)?;
    let mut value: serde_json::Value = serde_json::from_str(&text).map_err(storage)?;
    let map = value
        .as_object_mut()
        .ok_or_else(|| invalid("Invalid reading goals"))?;
    for id in [kept, incoming] {
        if map.get(id).is_some_and(|goal| !goal.is_object()) {
            return Err(invalid("Invalid document reading goal"));
        }
    }
    if !map.contains_key(kept)
        && let Some(goal) = map.get(incoming).cloned()
    {
        map.insert(kept.into(), goal);
        db.execute(
            "UPDATE app_metadata SET value=?1,updated_at=?2 WHERE key='reading_goals_v1'",
            params![value.to_string(), now],
        )
        .map_err(storage)?;
    }
    Ok(())
}
