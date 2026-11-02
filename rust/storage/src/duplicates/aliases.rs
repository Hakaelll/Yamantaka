use super::*;
use crate::database::storage;
pub(crate) fn resolve(db: &Database, id: &str) -> Result<Option<String>> {
    if id.is_empty() || id.len() > 4096 {
        return Err(Failure::Invalid("Invalid reading recovery identity".into()));
    }
    let exists: bool = db
        .connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE id=?)",
            [id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    if exists {
        return Ok(Some(id.into()));
    }
    let (visited,found,target,variants):(i64,i64,Option<String>,i64)=db.connection.query_row(r#"
        WITH RECURSIVE aliases AS MATERIALIZED (
            SELECT h.incoming_document_id incoming,min(h.kept_document_id) kept,count(DISTINCT h.kept_document_id) variants
            FROM duplicate_merge_history h JOIN operation_journal j ON j.id=h.id
            WHERE h.status='completed' AND j.status='completed' AND j.operation_kind='rustDuplicateMerge.v1' AND j.compatibility_hash='rust-duplicate-merge-v1'
                AND json_extract(CASE WHEN json_valid(j.summary_json) THEN j.summary_json ELSE '{}' END,'$.version')=1
                AND json_extract(CASE WHEN json_valid(j.summary_json) THEN j.summary_json ELSE '{}' END,'$.keptDocument')=h.kept_document_id
                AND json_extract(CASE WHEN json_valid(j.summary_json) THEN j.summary_json ELSE '{}' END,'$.incomingDocument')=h.incoming_document_id
            GROUP BY h.incoming_document_id
        ), path(id) AS (
            VALUES(?1) UNION SELECT a.kept FROM path p JOIN aliases a ON a.incoming=p.id
            WHERE NOT EXISTS(SELECT 1 FROM documents WHERE id=p.id) LIMIT 100001
        ) SELECT count(*),sum(EXISTS(SELECT 1 FROM documents d WHERE d.id=p.id)),min(CASE WHEN EXISTS(SELECT 1 FROM documents d WHERE d.id=p.id) THEN p.id END),max(COALESCE(a.variants,0)) FROM path p LEFT JOIN aliases a ON a.incoming=p.id
    "#,[id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).map_err(storage)?;
    let recorded:bool=db.connection.query_row("SELECT EXISTS(SELECT 1 FROM duplicate_merge_history h JOIN operation_journal j ON j.id=h.id WHERE h.incoming_document_id=? AND j.operation_kind='rustDuplicateMerge.v1')",[id],|row|row.get(0)).map_err(storage)?;
    if visited > 100000 || variants > 1 || found > 1 || (found == 0 && (variants > 0 || recorded)) {
        return Err(Failure::Invalid(
            "Consolidated reading history requires recovery; continuity is preserved".into(),
        ));
    }
    Ok(target)
}
