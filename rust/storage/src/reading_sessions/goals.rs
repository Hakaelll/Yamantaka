use super::*;
use yamantaka_domain::reading_sessions::goals::ReadingGoal;
const GOALS: &str = "reading_goals_v1";

fn invalid() -> Failure {
    Failure::Invalid("Stored reading goals require recovery; original data is preserved".into())
}
fn target_count(connection: &rusqlite::Connection, document: &str) -> Result<i64> {
    connection.query_row("SELECT count(*) FROM json_each((SELECT value FROM app_metadata WHERE key=?1)) WHERE key=?2",params![GOALS,document],|row|row.get(0)).map_err(storage)
}
fn read(connection: &rusqlite::Connection, document: &str) -> Result<Option<ReadingGoal>> {
    let valid: Option<bool> = connection.query_row("SELECT CASE WHEN json_valid(value) THEN json_type(value)='object' ELSE 0 END FROM app_metadata WHERE key=?1",[GOALS],|row|row.get(0)).optional().map_err(storage)?;
    if valid == Some(false) {
        return Err(invalid());
    }
    let count = target_count(connection, document)?;
    if count == 0 {
        return Ok(None);
    }
    if count != 1 {
        return Err(invalid());
    }
    let kind:String=connection.query_row("SELECT type FROM json_each((SELECT value FROM app_metadata WHERE key=?1)) WHERE key=?2",params![GOALS,document],|row|row.get(0)).map_err(storage)?;
    if kind != "object" {
        return Err(invalid());
    }
    let mut query=connection.prepare("SELECT key,type,value FROM json_each((SELECT value FROM json_each((SELECT value FROM app_metadata WHERE key=?1)) WHERE key=?2)) WHERE key IN ('pages','minutes')").map_err(storage)?;
    let mut rows = query.query(params![GOALS, document]).map_err(storage)?;
    let mut values = [None, None];
    let mut seen = [false, false];
    while let Some(row) = rows.next().map_err(storage)? {
        let key = row
            .get_ref(0)
            .map_err(storage)?
            .as_str()
            .map_err(|_| invalid())?;
        let index = usize::from(key == "minutes");
        if seen[index] {
            return Err(invalid());
        }
        seen[index] = true;
        let kind = row
            .get_ref(1)
            .map_err(storage)?
            .as_str()
            .map_err(|_| invalid())?;
        values[index] = match kind {
            "null" => None,
            "integer" => Some(
                row.get_ref(2)
                    .map_err(storage)?
                    .as_i64()
                    .map_err(|_| invalid())?,
            ),
            _ => return Err(invalid()),
        };
    }
    let goal = ReadingGoal::from_legacy(values[0], values[1]);
    Ok((!goal.is_empty()).then_some(goal))
}
pub(super) fn load(
    db: &Database,
    document: &str,
    cancel: &Cancellation,
) -> Result<Option<ReadingGoal>> {
    cancel.check()?;
    let tx = db.connection.unchecked_transaction().map_err(storage)?;
    let goal = read(&tx, document)?;
    cancel.check()?;
    tx.commit().map_err(storage)?;
    Ok(goal)
}
pub(super) fn save(
    db: &mut Database,
    document: &str,
    goal: Option<ReadingGoal>,
    cancel: &Cancellation,
) -> Result<Option<ReadingGoal>> {
    cancel.check()?;
    let goal = goal.filter(|goal| !goal.is_empty());
    let tx = db.connection.transaction().map_err(storage)?;
    let exists: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE id=?1)",
            [document],
            |row| row.get(0),
        )
        .map_err(storage)?;
    if !exists {
        return Err(Failure::Invalid(
            "Reading goal document is unavailable".into(),
        ));
    }
    read(&tx, document)?;
    let count = target_count(&tx, document)?;
    if goal.is_none() && count == 0 {
        cancel.check()?;
        return Ok(None);
    }
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    if let Some(goal) = goal {
        let changed=tx.execute("WITH existing AS (SELECT key,type,value FROM json_each(COALESCE((SELECT value FROM app_metadata WHERE key=?1),'{}'))),updated AS (SELECT key,CASE WHEN key=?2 THEN json_patch(value,json_object('pages',?3,'minutes',?4)) ELSE CASE type WHEN 'object' THEN value WHEN 'array' THEN value WHEN 'true' THEN 'true' WHEN 'false' THEN 'false' WHEN 'null' THEN 'null' ELSE json_quote(value) END END AS value FROM existing UNION ALL SELECT ?2,json_object('pages',?3,'minutes',?4) WHERE NOT EXISTS(SELECT 1 FROM existing WHERE key=?2)) INSERT INTO app_metadata(key,value,updated_at) SELECT ?1,json_group_object(key,json(value)),?5 FROM updated WHERE 1 ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at",params![GOALS,document,goal.pages(),goal.minutes(),now]).map_err(storage)?;
        if changed != 1 {
            return Err(Failure::Invalid("Reading goal was not saved".into()));
        }
    } else {
        let changed=tx.execute("UPDATE app_metadata SET value=(SELECT json_group_object(key,json(CASE type WHEN 'object' THEN value WHEN 'array' THEN value WHEN 'true' THEN 'true' WHEN 'false' THEN 'false' WHEN 'null' THEN 'null' ELSE json_quote(value) END)) FROM json_each(app_metadata.value) WHERE key<>?2),updated_at=?3 WHERE key=?1",params![GOALS,document,now]).map_err(storage)?;
        if changed != 1 {
            return Err(Failure::Invalid("Reading goal was not cleared".into()));
        }
        tx.execute(
            "DELETE FROM app_metadata WHERE key=?1 AND NOT EXISTS(SELECT 1 FROM json_each(app_metadata.value))",
            [GOALS],
        )
        .map_err(storage)?;
        let empty: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM app_metadata WHERE key=?1 AND NOT EXISTS(SELECT 1 FROM json_each(app_metadata.value)))", [GOALS], |row| row.get(0)).map_err(storage)?;
        if empty {
            return Err(Failure::Invalid(
                "Empty reading goals were not removed".into(),
            ));
        }
    }
    if read(&tx, document)? != goal {
        return Err(Failure::Invalid(
            "Reading goal storage did not match the requested change".into(),
        ));
    }
    cancel.check()?;
    tx.commit().map_err(storage)?;
    Ok(goal)
}
