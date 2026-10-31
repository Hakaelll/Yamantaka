use super::*;
pub(super) fn revert(db: &Connection, undo: &Undo, context: &ExecutionContext) -> Result<()> {
    match undo {
        Undo::Targets { entries } => {
            for target in entries.iter().rev() {
                if super::super::plan::record(db, target.kind.table(), &target.id)?
                    != target.revision
                {
                    return Err(invalid(
                        "A created destination was edited; it was preserved",
                    ));
                }
                let used:bool=match target.kind {
                TargetKind::Tag=>db.query_row("SELECT EXISTS(SELECT 1 FROM document_tags WHERE tag_id=?1) OR EXISTS(SELECT 1 FROM annotation_tags WHERE tag_id=?1)",[&target.id],|row|row.get(0)),
                TargetKind::Collection=>db.query_row("SELECT EXISTS(SELECT 1 FROM collection_documents WHERE collection_id=?1) OR EXISTS(SELECT 1 FROM workspaces WHERE collection_id=?1)",[&target.id],|row|row.get(0)),
                TargetKind::Workspace=>return Err(invalid("Unknown created workspace receipt")),
            }.map_err(storage)?;
                if used {
                    return Err(invalid(
                        "A created destination is still in use; it was preserved",
                    ));
                }
                db.execute(
                    &format!("DELETE FROM {} WHERE id=?", target.kind.table()),
                    [&target.id],
                )
                .map_err(storage)?;
            }
        }
        Undo::Document(undo) => {
            if document_identity(db, &undo.document)? != undo.identity {
                return Err(invalid(
                    "The document source changed after execution; it was preserved",
                ));
            }
            if let Some((before, after)) = &undo.favorite {
                let value: bool = db
                    .query_row(
                        "SELECT is_favorite FROM documents WHERE id=?",
                        [&undo.document],
                        |row| row.get(0),
                    )
                    .map_err(storage)?;
                if value != *after {
                    return Err(invalid(
                        "Favorite changed after execution; it was preserved",
                    ));
                }
                db.execute(
                    "UPDATE documents SET is_favorite=? WHERE id=?",
                    params![before, undo.document],
                )
                .map_err(storage)?;
            }
            if let Some((before, after)) = &undo.progress {
                if progress(db, &undo.document)? != *after {
                    return Err(invalid(
                        "Reading progress changed after execution; it was preserved",
                    ));
                }
                db.execute("UPDATE documents SET last_page_index=?,reading_progress=?,last_opened_at=? WHERE id=?",params![before.page,before.value,before.opened,undo.document]).map_err(storage)?;
            }
            for row in undo.rows.iter().rev() {
                if membership_signature(db, row.kind, &row.id, &row.target)? != row.revision {
                    return Err(invalid(
                        "An added membership changed after execution; it was preserved",
                    ));
                }
                let query = match row.kind {
                    TargetKind::Tag => "DELETE FROM document_tags WHERE document_id=? AND tag_id=?",
                    TargetKind::Collection => {
                        "DELETE FROM collection_documents WHERE document_id=? AND collection_id=?"
                    }
                    TargetKind::Workspace => {
                        "DELETE FROM workspace_tabs WHERE id=? AND workspace_id=?"
                    }
                };
                db.execute(query, params![row.id, row.target])
                    .map_err(storage)?;
                if row.kind == TargetKind::Workspace {
                    db.execute("UPDATE workspaces SET active_tab_id=CASE WHEN active_tab_id=? THEN NULL ELSE active_tab_id END WHERE id=?",params![row.id,row.target]).map_err(storage)?;
                    touch_workspace(db, &row.target)?;
                }
            }
            crate::search::refresh_document_tags(db, &undo.document, context.access)?;
        }
    }
    Ok(())
}
