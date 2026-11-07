use super::*;
use yamantaka_domain::annotations::marks::{AnnotationMark, AnnotationMarks};

const VISIBLE_MARKS: &str = "FROM annotations a JOIN documents d ON d.id=a.document_id WHERE a.document_id=? AND a.page_index=? AND a.status='attached' AND (a.type NOT IN ('ink','arrow','rectangle') OR (d.fingerprint=a.document_fingerprint AND d.is_modified=0 AND d.page_count>a.page_index))";

pub(super) fn read(
    db: &Database,
    document: &str,
    page: u32,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<AnnotationMarks> {
    cancel.check()?;
    let tx = db.connection.unchecked_transaction().map_err(storage)?;
    let count: i64 = tx
        .query_row(
            &format!("SELECT count(*) {VISIBLE_MARKS}"),
            params![document, page],
            |row| row.get(0),
        )
        .map_err(storage)?;
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
        let mut statement = tx.prepare(&format!("SELECT a.type,a.color,a.rects_json,a.id,COALESCE(trim(a.note),'')<>'' {VISIBLE_MARKS} ORDER BY a.created_at,a.id")).map_err(storage)?;
        let mut rows = statement.query(params![document, page]).map_err(storage)?;
        while let Some(row) = rows.next().map_err(storage)? {
            cancel.check()?;
            let geometry = field(row, 2)?;
            let id = field(row, 3)?;
            // The JSON rectangle keys exceed each decoded rectangle's size;
            // this covers serde's vector growth before its retained capacity is measured.
            let bytes = geometry
                .len()
                .checked_mul(3)
                .and_then(|bytes| bytes.checked_add(id.len()))
                .and_then(|bytes| bytes.checked_add(std::mem::size_of::<AnnotationMark>()))
                .ok_or(Failure::MemoryBudget)?;
            let mark_lease = governor
                .try_reserve_one(Cost::new(
                    MemoryCategory::Text,
                    MemoryRepresentation::Cpu,
                    bytes,
                ))
                .map_err(|_| Failure::MemoryBudget)?;
            let kind = decode(field(row, 0)?)?;
            let (rects, drawing) = decode_geometry(kind, geometry)?;
            marks.push(AnnotationMark::from_reserved(
                id.into(),
                kind,
                decode_color(field(row, 1)?)?,
                rects,
                drawing,
                row.get(4).map_err(storage)?,
                mark_lease,
            )?);
        }
    }
    cancel.check()?;
    let result = AnnotationMarks::from_reserved(page, marks, lease)?;
    tx.commit().map_err(storage)?;
    Ok(result)
}
