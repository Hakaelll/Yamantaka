use super::*;
use yamantaka_domain::annotations::marks::{AnnotationMark, AnnotationMarks};

pub(super) fn read(
    db: &Database,
    document: &str,
    page: u32,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<AnnotationMarks> {
    cancel.check()?;
    let tx = db.connection.unchecked_transaction().map_err(storage)?;
    let count: i64 = tx.query_row("SELECT count(*) FROM annotations WHERE document_id=? AND page_index=? AND status='attached'", params![document,page], |row| row.get(0)).map_err(storage)?;
    let count = usize::try_from(count).map_err(|_| Failure::MemoryBudget)?;
    let bytes = count
        .checked_mul(std::mem::size_of::<AnnotationMark>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<AnnotationMarks>()))
        .ok_or(Failure::MemoryBudget)?;
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut marks = Vec::new();
    marks
        .try_reserve_exact(count)
        .map_err(|_| Failure::MemoryBudget)?;
    {
        let mut statement = tx.prepare("SELECT type,color,rects_json FROM annotations WHERE document_id=? AND page_index=? AND status='attached' ORDER BY created_at,id").map_err(storage)?;
        let mut rows = statement.query(params![document, page]).map_err(storage)?;
        while let Some(row) = rows.next().map_err(storage)? {
            cancel.check()?;
            let geometry = field(row, 2)?;
            // The JSON rectangle keys exceed each decoded rectangle's size;
            // this covers serde's vector growth before its retained capacity is measured.
            let bytes = geometry
                .len()
                .checked_mul(3)
                .and_then(|bytes| bytes.checked_add(std::mem::size_of::<AnnotationMark>()))
                .ok_or(Failure::MemoryBudget)?;
            let mark_lease = governor
                .try_reserve_one(Cost::new(
                    MemoryCategory::Text,
                    MemoryRepresentation::Cpu,
                    bytes,
                ))
                .map_err(|_| Failure::MemoryBudget)?;
            let rects = serde_json::from_str(geometry).map_err(|_| {
                Failure::Invalid("Annotation geometry cannot be decoded safely".into())
            })?;
            marks.push(AnnotationMark::from_reserved(
                decode(field(row, 0)?)?,
                decode(field(row, 1)?)?,
                rects,
                mark_lease,
            )?);
        }
    }
    cancel.check()?;
    let result = AnnotationMarks::from_reserved(page, marks, lease)?;
    tx.commit().map_err(storage)?;
    Ok(result)
}
