use super::*;
use chrono::{SecondsFormat, Utc};
use std::collections::BTreeSet;

pub(super) fn change(
    db: &mut Database,
    change: AnnotationBatchChange<'_>,
    query: &AnnotationQuery,
    access: IndexAccess,
    governor: &MemoryGovernor,
    cancel: &Cancellation,
) -> Result<AnnotationPage> {
    cancel.check()?;
    let targets = match &change {
        AnnotationBatchChange::Review { targets, .. }
        | AnnotationBatchChange::AddTag { targets, .. } => *targets,
    };
    writes::validate_batch(targets.len())?;
    if targets.is_empty() {
        return Err(Failure::Invalid(
            "Select annotations before applying a batch".into(),
        ));
    }
    let tx = db.connection.transaction().map_err(storage)?;
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true);
    let mut pages = BTreeSet::new();
    for target in targets.iter().collect::<BTreeSet<_>>() {
        cancel.check()?;
        let page = match &change {
            AnnotationBatchChange::Review { state, .. } => writes::review_one(
                &tx,
                &target.document_id,
                &target.annotation_id,
                *state,
                &now,
            )?,
            AnnotationBatchChange::AddTag { tag, .. } => {
                writes::tag_one(&tx, &target.document_id, &target.annotation_id, tag, &now)?
            }
        };
        pages.insert((&target.document_id, page));
    }
    for (document, page) in pages {
        cancel.check()?;
        writes::refresh(&tx, document, page, access)?;
    }
    let result = read_page(&tx, query, governor, cancel)?;
    cancel.check()?;
    tx.commit().map_err(storage)?;
    for target in targets {
        db.annotation_history.clear(&target.document_id);
    }
    Ok(result)
}
