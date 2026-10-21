use super::*;
use rusqlite::OptionalExtension;

fn verify(db: &Connection, expected: &TagRevision) -> Result<()> {
    let current = db
        .query_row(
            "SELECT updated_at FROM tags WHERE id=?",
            [&expected.id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(storage)?;
    if current.as_deref() != Some(expected.updated_at.as_str()) {
        return Err(Failure::Invalid(
            "Tag changed; refresh before retrying".into(),
        ));
    }
    Ok(())
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
pub(super) fn apply(
    db: &Connection,
    change: &TagChange,
    access: IndexAccess,
    cancel: &Cancellation,
) -> Result<String> {
    let expected = match change {
        TagChange::Rename { expected, .. }
        | TagChange::Color { expected, .. }
        | TagChange::Delete { expected } => expected,
        TagChange::Merge { source, target } => {
            verify(db, target)?;
            source
        }
        _ => unreachable!(),
    };
    verify(db, expected)?;
    if let TagChange::Color { color, .. } = change {
        let hex = color.strip_prefix('#').is_some_and(|value| {
            value.len() == 6 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
        if !hex
            && !["blue", "violet", "purple", "green", "red", "orange", "gray"]
                .contains(&color.as_str())
        {
            return Err(Failure::Invalid(
                "Choose a supported tag color or #RRGGBB".into(),
            ));
        }
        db.execute(
            "UPDATE tags SET color=?,updated_at=? WHERE id=?",
            params![color, revision(&expected.updated_at), expected.id],
        )
        .map_err(storage)?;
        return Ok(expected.id.clone());
    }
    if let TagChange::Merge { source, target } = change
        && source.id == target.id
    {
        return Ok(target.id.clone());
    }
    // SQLite owns the affected-page snapshot; no unbounded Rust membership list is retained.
    db.execute_batch("CREATE TEMP TABLE tag_edit_pages(document_id TEXT NOT NULL,page_index INTEGER NOT NULL,PRIMARY KEY(document_id,page_index)) WITHOUT ROWID;").map_err(storage)?;
    db.execute("INSERT INTO tag_edit_pages SELECT document_id,-1 FROM document_tags WHERE tag_id=?1 UNION SELECT a.document_id,a.page_index FROM annotation_tags t JOIN annotations a ON a.id=t.annotation_id WHERE t.tag_id=?1",[&expected.id]).map_err(storage)?;
    cancel.check()?;
    let id = match change {
        TagChange::Rename { name, .. } => {
            let clean = name.trim();
            if clean.is_empty() || clean.encode_utf16().count() > 60 {
                return Err(Failure::Invalid(
                    "Use a tag name between 1 and 60 characters".into(),
                ));
            }
            db.execute(
                "UPDATE tags SET name=?,normalized_name=?,updated_at=? WHERE id=?",
                params![
                    clean,
                    clean.to_lowercase(),
                    revision(&expected.updated_at),
                    expected.id
                ],
            )
            .map_err(storage)?;
            expected.id.clone()
        }
        TagChange::Merge { source, target } => {
            db.execute("INSERT OR IGNORE INTO document_tags(document_id,tag_id) SELECT document_id,? FROM document_tags WHERE tag_id=?",params![target.id,source.id]).map_err(storage)?;
            cancel.check()?;
            db.execute("INSERT OR IGNORE INTO annotation_tags(annotation_id,tag_id) SELECT annotation_id,? FROM annotation_tags WHERE tag_id=?",params![target.id,source.id]).map_err(storage)?;
            db.execute("DELETE FROM tags WHERE id=?", [&source.id])
                .map_err(storage)?;
            db.execute(
                "UPDATE tags SET updated_at=? WHERE id=?",
                params![revision(&target.updated_at), target.id],
            )
            .map_err(storage)?;
            target.id.clone()
        }
        TagChange::Delete { .. } => {
            db.execute("DELETE FROM tags WHERE id=?", [&expected.id])
                .map_err(storage)?;
            expected.id.clone()
        }
        _ => unreachable!(),
    };
    super::search_cache::refresh(db, access, cancel)?;
    db.execute_batch("DROP TABLE tag_edit_pages")
        .map_err(storage)?;
    Ok(id)
}
