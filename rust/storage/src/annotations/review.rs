use super::*;
use yamantaka_domain::annotations::review::{AnnotationReviewMetadata, AnnotationReviewSnapshot};

pub(super) fn read(
    database: &Database,
    query: &AnnotationQuery,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<AnnotationReviewSnapshot> {
    cancel.check()?;
    let tx = database
        .connection
        .unchecked_transaction()
        .map_err(storage)?;
    let page = read_page(&tx, query, governor, cancel)?;
    let mut bytes = page
        .entries()
        .len()
        .checked_mul(std::mem::size_of::<AnnotationReviewMetadata>())
        .and_then(|n| n.checked_add(std::mem::size_of::<AnnotationReviewSnapshot>()))
        .ok_or(Failure::MemoryBudget)?;
    let mut lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut documents: Vec<AnnotationReviewMetadata> = Vec::new();
    documents
        .try_reserve_exact(page.entries().len())
        .map_err(|_| Failure::MemoryBudget)?;
    let mut statement = tx.prepare("SELECT COALESCE(title_override,title,source_title,file_name),COALESCE(author_override,author,source_author),reading_progress FROM documents WHERE id=?").map_err(storage)?;
    for record in page.entries() {
        cancel.check()?;
        let id = &record.data().values.document_id;
        if documents.iter().any(|m| m.document == *id) {
            continue;
        }
        let mut rows = statement.query([id]).map_err(storage)?;
        let row = rows.next().map_err(storage)?;
        let (title, author, progress) = if let Some(row) = row {
            (
                Some(field(row, 0)?),
                optional_text(row, 1)?,
                row.get::<_, f64>(2).map_err(storage)?,
            )
        } else {
            (None, None, 0.0)
        };
        bytes = bytes
            .checked_add(id.len())
            .and_then(|n| n.checked_add(title.map_or(0, str::len)))
            .and_then(|n| n.checked_add(author.map_or(0, str::len)))
            .ok_or(Failure::MemoryBudget)?;
        lease.try_resize(bytes).map_err(|_| Failure::MemoryBudget)?;
        documents.push(AnnotationReviewMetadata {
            document: id.clone(),
            title: title.map(str::to_owned),
            author: author.map(str::to_owned),
            progress,
        });
    }
    drop(statement);
    let snapshot = AnnotationReviewSnapshot::from_reserved(page, documents, lease)?;
    cancel.check()?;
    tx.commit().map_err(storage)?;
    Ok(snapshot)
}

fn optional_text<'a>(row: &'a Row<'_>, index: usize) -> Result<Option<&'a str>> {
    match row.get_ref(index).map_err(storage)? {
        rusqlite::types::ValueRef::Null => Ok(None),
        value => value.as_str().map(Some).map_err(storage),
    }
}
