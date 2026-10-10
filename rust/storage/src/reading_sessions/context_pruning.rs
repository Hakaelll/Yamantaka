use super::*;

pub(super) fn prune(
    tx: &rusqlite::Transaction<'_>,
    removed: &str,
    selection: impl rusqlite::ToSql,
) -> Result<()> {
    let valid: Option<bool> = tx.query_row("SELECT CASE WHEN json_valid(value) THEN json_type(value)='object' ELSE 0 END FROM app_metadata WHERE key=?1", [CONTEXTS], |r| r.get(0)).optional().map_err(storage)?;
    if valid == Some(false) {
        return Err(Failure::Invalid(
            "Stored reading session contexts require recovery".into(),
        ));
    }
    let update = format!(
        "WITH removed AS ({removed}) UPDATE app_metadata SET value=(SELECT json_group_object(key,json(CASE type WHEN 'object' THEN value WHEN 'array' THEN value WHEN 'true' THEN 'true' WHEN 'false' THEN 'false' WHEN 'null' THEN 'null' ELSE json_quote(value) END)) FROM json_each(app_metadata.value) WHERE key NOT IN (SELECT id FROM removed)),updated_at=?3 WHERE app_metadata.key=?2 AND EXISTS(SELECT 1 FROM json_each(app_metadata.value) WHERE key IN (SELECT id FROM removed))"
    );
    tx.execute(
        &update,
        params![
            selection,
            CONTEXTS,
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
        ],
    )
    .map_err(storage)?;
    Ok(())
}
