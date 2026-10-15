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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use yamantaka_domain::LibraryRepository;

    #[test]
    fn plans_use_a_consistent_source_and_decision_without_changing_stored_metadata() {
        let db = Database::in_memory_at(11).unwrap();
        db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,thumbnail_path) VALUES('d','fixture.pdf','fixture','fixture.pdf',7,1,'2026-10-15T12:00:00Z','now','now','original','keep-cache');").unwrap();
        let absent = db.cover_snapshot("d").unwrap();
        assert_eq!(absent.plan, CoverPlan::Detect);
        assert!(absent.revision.is_none());
        assert_eq!(absent.source_modified, "2026-10-15T12:00:00Z");
        assert!(db.cover_snapshot("missing").is_err());
        db.execute_batch("INSERT INTO document_covers VALUES('d',1,'small','large','generated','2026-10-15T12:00:00Z',0.79,'automatic',3,'contain',0.75);").unwrap();
        for (sql, plan) in [
            ("SELECT 1", CoverPlan::Render(1)),
            (
                "UPDATE document_covers SET source_modified_at='2026-10-15T14:00:00+02:00'",
                CoverPlan::Render(1),
            ),
            (
                "UPDATE document_covers SET algorithm_version=2",
                CoverPlan::Detect,
            ),
            (
                "UPDATE document_covers SET algorithm_version=3,source_modified_at='2026-10-14T12:00:00Z'",
                CoverPlan::Detect,
            ),
            (
                "UPDATE document_covers SET source_modified_at='2026-10-15T12:00:00Z',source_page_index=7",
                CoverPlan::Detect,
            ),
            (
                "UPDATE document_covers SET source_page_index=0,selection_mode='manual',algorithm_version=1,source_modified_at='old'",
                CoverPlan::Render(0),
            ),
            (
                "UPDATE document_covers SET source_page_index=99",
                CoverPlan::Render(6),
            ),
        ] {
            db.execute_batch(sql).unwrap();
            let before = db.cover_revision("d").unwrap();
            let snapshot = db.cover_snapshot("d").unwrap();
            assert_eq!(snapshot.plan, plan, "{sql}");
            assert_eq!(snapshot.revision, before);
            assert_eq!(db.cover_revision("d").unwrap(), before);
            assert_eq!(
                db.get("d").unwrap().unwrap().thumbnail_path,
                Some("keep-cache".into())
            );
        }
        db.execute_batch("PRAGMA ignore_check_constraints=ON; UPDATE document_covers SET selection_mode='future-mode'")
            .unwrap();
        assert!(db.cover_snapshot("d").is_err());
    }
}
