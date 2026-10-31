use super::*;
pub(super) fn document(
    db: &Connection,
    undo: &DocumentUndo,
    context: &ExecutionContext,
) -> Result<()> {
    if document_identity(db, &undo.document)? != undo.identity {
        return Err(invalid(
            "The recovered document source changed; current data was preserved",
        ));
    }
    for workspace in &undo.workspaces {
        let current = rows::workspace(db, &workspace.id, &context.cancel)?;
        if current.stable != workspace.after.stable || current.queue != workspace.after.queue {
            return Err(invalid(
                "A recovered workspace or queue changed; current data was preserved",
            ));
        }
    }
    if let Some((before, after)) = &undo.favorite {
        let current: bool = db
            .query_row(
                "SELECT is_favorite FROM documents WHERE id=?",
                [&undo.document],
                |row| row.get(0),
            )
            .map_err(storage)?;
        if current != *after {
            return Err(invalid(
                "Favorite changed after recovery; current data was preserved",
            ));
        }
        db.execute(
            "UPDATE documents SET is_favorite=? WHERE id=?",
            params![before, undo.document],
        )
        .map_err(storage)?;
    }
    if let Some((before, after)) = &undo.reading {
        if apply::reading(db, &undo.document)? != *after {
            return Err(invalid(
                "Reading state changed after recovery; current data was preserved",
            ));
        }
        db.execute("UPDATE documents SET last_page_index=?,reading_progress=?,preferred_view_mode=?,preferred_theme=?,last_opened_at=? WHERE id=?",params![before.page,before.progress,before.mode,before.theme,before.opened,undo.document]).map_err(storage)?;
    }
    for row in undo.removed.iter().rev() {
        context.cancel.check()?;
        rows::restore(db, row)?;
    }
    for workspace in undo.workspaces.iter().rev() {
        db.execute(
            "UPDATE workspaces SET active_tab_id=?,split_json=? WHERE id=?",
            params![
                workspace.before.active,
                workspace.before.split,
                workspace.id
            ],
        )
        .map_err(storage)?;
        touch_workspace(db, &workspace.id)?;
        let restored = rows::workspace(db, &workspace.id, &context.cancel)?;
        if restored.stable != workspace.before.stable || restored.queue != workspace.before.queue {
            return Err(invalid(
                "The queue could not be restored to its recorded state",
            ));
        }
    }
    crate::search::refresh_document_tags(db, &undo.document, context.access)?;
    Ok(())
}
pub(super) fn journal(db: &Connection, undo: &JournalUndo) -> Result<()> {
    let Some(expected) = undo.after else {
        return Ok(());
    };
    if record(db, "operation_journal", &undo.id)? != Some(expected) {
        return Err(invalid(
            "The inherited journal changed after recovery; it was preserved",
        ));
    }
    let updated: String = db
        .query_row(
            "SELECT updated_at FROM operation_journal WHERE id=? AND operation_kind=?",
            params![undo.id, journal::LEGACY],
            |row| row.get(0),
        )
        .map_err(storage)?;
    db.execute("UPDATE operation_journal SET status=?,phase=?,resume_capability=?,error_code=?,interruption_reason=?,updated_at=? WHERE id=? AND operation_kind=?",params![undo.before.status,undo.before.phase,undo.before.capability,undo.before.error,undo.before.interruption,stamp(&updated),undo.id,journal::LEGACY]).map_err(storage)?;
    Ok(())
}
