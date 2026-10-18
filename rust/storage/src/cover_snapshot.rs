use crate::database::storage;
use rusqlite::{Connection, OptionalExtension};
use yamantaka_domain::{
    Failure, Result,
    cover_policy::COVER_ALGORITHM_VERSION,
    cover_snapshot::{CoverPlan, CoverSnapshot},
};

pub(crate) fn read(connection: &Connection, id: &str) -> Result<CoverSnapshot> {
    let row = connection.query_row(
        "SELECT c.document_id,c.source_page_index,c.small_thumbnail_path,c.large_thumbnail_path,c.generated_at,c.source_modified_at,c.confidence,c.selection_mode,c.algorithm_version,c.crop_mode,c.page_aspect_ratio,d.file_modified_at,d.page_count
         FROM documents d LEFT JOIN document_covers c ON c.document_id=d.id WHERE d.id=?1", [id], |row| {
            let modified: String = row.get(11)?;
            let count: u32 = row.get(12)?;
            let decision = if row.get::<_, Option<String>>(0)?.is_some() {
                Some((crate::cover_revision::from_row(row)?, row.get::<_, u32>(1)?, row.get::<_, String>(5)?, row.get::<_, String>(7)?, row.get::<_, u32>(8)?))
            } else { None };
            Ok((modified, count, decision))
        }).optional().map_err(storage)?;
    let Some((source_modified, count, decision)) = row else {
        return Err(Failure::Invalid("Cover document no longer exists".into()));
    };
    let Some((revision, page, generated_source, mode, version)) = decision else {
        return Ok(CoverSnapshot {
            plan: CoverPlan::Detect,
            revision: None,
            source_modified,
        });
    };
    let plan = match mode.as_str() {
        "manual" if count > 0 => CoverPlan::Render(page.min(count - 1)),
        "automatic"
            if page < count
                && version == COVER_ALGORITHM_VERSION
                && same_modified(&generated_source, &source_modified) =>
        {
            CoverPlan::Render(page)
        }
        "automatic" => CoverPlan::Detect,
        "manual" => return Err(Failure::Invalid("The document has no cover pages".into())),
        _ => return Err(Failure::Invalid("Unknown cover selection mode".into())),
    };
    Ok(CoverSnapshot {
        plan,
        revision: Some(revision),
        source_modified,
    })
}

fn same_modified(left: &str, right: &str) -> bool {
    left == right
        || chrono::DateTime::parse_from_rfc3339(left)
            .ok()
            .zip(chrono::DateTime::parse_from_rfc3339(right).ok())
            .is_some_and(|(left, right)| left == right)
}
