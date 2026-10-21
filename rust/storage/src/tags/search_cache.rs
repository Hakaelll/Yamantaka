use super::*;

pub(super) fn refresh(db: &Connection, access: IndexAccess, cancel: &Cancellation) -> Result<()> {
    cancel.check()?;
    db.execute_batch("CREATE TEMP TABLE tag_edit_refresh(document_id TEXT NOT NULL,page_index INTEGER NOT NULL,PRIMARY KEY(document_id,page_index)) WITHOUT ROWID;").map_err(storage)?;
    if access == IndexAccess::Enabled {
        db.execute(
            "INSERT INTO tag_edit_refresh SELECT document_id,page_index FROM tag_edit_pages",
            [],
        )
        .map_err(storage)?;
    } else {
        // FTS metadata predicates scan. Keep FTS outermost and probe at most two snapshot keys per row.
        db.execute("INSERT OR IGNORE INTO tag_edit_refresh SELECT p.document_id,p.page_index FROM library_search l CROSS JOIN tag_edit_pages p WHERE p.document_id=l.document_id AND p.page_index IN(-1,CASE WHEN l.kind='annotation' THEN l.page_index ELSE -1 END)",[]).map_err(storage)?;
    }
    cancel.check()?;
    db.execute("UPDATE library_search SET tags=COALESCE((SELECT group_concat(t.name,' ') FROM document_tags dt JOIN tags t ON t.id=dt.tag_id WHERE dt.document_id=library_search.document_id),'') WHERE kind='pageText' AND document_id IN(SELECT document_id FROM tag_edit_refresh WHERE page_index=-1)",[]).map_err(storage)?;
    cancel.check()?;
    db.execute("DELETE FROM library_search WHERE (kind IN('metadata','tag') AND document_id IN(SELECT document_id FROM tag_edit_refresh WHERE page_index=-1)) OR (kind='annotation' AND EXISTS(SELECT 1 FROM tag_edit_refresh p WHERE p.document_id=library_search.document_id AND p.page_index=library_search.page_index))",[]).map_err(storage)?;
    let title = crate::search::TITLE;
    let eligible = "NOT EXISTS(SELECT 1 FROM search_index_state s WHERE s.document_id=d.id AND s.status='excluded')";
    cancel.check()?;
    db.execute(&format!("INSERT INTO library_search(document_id,page_index,kind,title,body,tags) SELECT d.id,-1,'metadata',{title},COALESCE(d.author_override,d.author,d.source_author,'') || ' ' || d.file_name || ' ' || COALESCE(n.note,''),COALESCE((SELECT group_concat(t.name,' ') FROM document_tags dt JOIN tags t ON t.id=dt.tag_id WHERE dt.document_id=d.id),'') FROM tag_edit_refresh p JOIN documents d ON d.id=p.document_id LEFT JOIN document_notes n ON n.document_id=d.id WHERE p.page_index=-1 AND {eligible}"),[]).map_err(storage)?;
    cancel.check()?;
    db.execute(&format!("INSERT INTO library_search(document_id,page_index,kind,title,body,tags) SELECT d.id,-1,'tag',{title},'',t.name FROM tag_edit_refresh p JOIN documents d ON d.id=p.document_id JOIN document_tags dt ON dt.document_id=d.id JOIN tags t ON t.id=dt.tag_id WHERE p.page_index=-1 AND {eligible}"),[]).map_err(storage)?;
    cancel.check()?;
    db.execute(&format!("INSERT INTO library_search(document_id,page_index,kind,title,body,tags) SELECT a.document_id,a.page_index,'annotation',{title},COALESCE(a.selected_text,'') || ' ' || COALESCE(a.note,''),COALESCE((SELECT group_concat(t.name,' ') FROM annotation_tags at JOIN tags t ON t.id=at.tag_id WHERE at.annotation_id=a.id),'') FROM tag_edit_refresh p JOIN annotations a ON a.document_id=p.document_id AND a.page_index=p.page_index JOIN documents d ON d.id=a.document_id WHERE p.page_index>=0 AND {eligible}"),[]).map_err(storage)?;
    cancel.check()?;
    db.execute_batch("DROP TABLE tag_edit_refresh")
        .map_err(storage)?;
    Ok(())
}
