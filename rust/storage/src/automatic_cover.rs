use crate::database::{Database, storage};
use rusqlite::OptionalExtension;
use yamantaka_domain::{
    Failure, Result, automatic_cover::AutomaticCover, cover_policy::COVER_ALGORITHM_VERSION,
};

pub(crate) fn save(database: &mut Database, choice: &AutomaticCover) -> Result<bool> {
    persist(database, choice, Write::Missing)
}
pub(crate) fn replace(
    database: &mut Database,
    choice: &AutomaticCover,
    expected: Option<yamantaka_domain::cover_revision::CoverRevision>,
) -> Result<bool> {
    persist(database, choice, Write::Unchanged(expected))
}
enum Write {
    Missing,
    Unchanged(Option<yamantaka_domain::cover_revision::CoverRevision>),
}
fn persist(database: &mut Database, choice: &AutomaticCover, write: Write) -> Result<bool> {
    let transaction = database.connection.transaction().map_err(storage)?;
    let document = choice.document();
    let current: Option<(String, String, u32, String)> = transaction
        .query_row(
            "SELECT path,fingerprint,page_count,file_modified_at FROM documents WHERE id=?1",
            [&document.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(storage)?;
    let Some((path, fingerprint, count, modified)) = current else {
        return Err(Failure::Invalid("Document no longer exists".into()));
    };
    if std::path::Path::new(&path) != document.path
        || fingerprint != document.fingerprint
        || count != document.page_count
        || modified != choice.source_modified()
        || choice.page() >= count
    {
        return Err(Failure::Invalid(
            "Document changed during automatic cover selection".into(),
        ));
    }
    let revision = crate::cover_revision::read(&transaction, &document.id)?;
    if match write {
        Write::Missing => revision.is_some(),
        Write::Unchanged(expected) => revision != expected,
    } {
        return Ok(false);
    }
    let now = chrono::Utc::now().to_rfc3339();
    let inserted=transaction.execute(
        "INSERT INTO document_covers(document_id,source_page_index,small_thumbnail_path,large_thumbnail_path,generated_at,source_modified_at,confidence,selection_mode,algorithm_version,crop_mode,page_aspect_ratio)
         VALUES(?1,?2,'','',?3,?4,?5,'automatic',?6,'contain',?7)
         ON CONFLICT(document_id) DO UPDATE SET source_page_index=excluded.source_page_index,small_thumbnail_path='',large_thumbnail_path='',generated_at=excluded.generated_at,source_modified_at=excluded.source_modified_at,confidence=excluded.confidence,selection_mode='automatic',algorithm_version=excluded.algorithm_version,crop_mode='contain',page_aspect_ratio=excluded.page_aspect_ratio",
        rusqlite::params![document.id,choice.page(),now,modified,choice.confidence(),COVER_ALGORITHM_VERSION,choice.aspect()]).map_err(storage)?;
    if inserted != 1 {
        return Err(Failure::Storage("Automatic cover was not saved".into()));
    }
    let updated = transaction
        .execute(
            "UPDATE documents SET thumbnail_path=NULL WHERE id=?1",
            [&document.id],
        )
        .map_err(storage)?;
    let actual: (u32,String,u32,f64,f64,String,String,String,String,String)=transaction.query_row(
        "SELECT source_page_index,selection_mode,algorithm_version,confidence,page_aspect_ratio,small_thumbnail_path,large_thumbnail_path,crop_mode,generated_at,source_modified_at FROM document_covers WHERE document_id=?1",[&document.id],
        |row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?,row.get(8)?,row.get(9)?))).map_err(storage)?;
    let thumbnail: Option<String> = transaction
        .query_row(
            "SELECT thumbnail_path FROM documents WHERE id=?1",
            [&document.id],
            |row| row.get(0),
        )
        .map_err(storage)?;
    if updated != 1
        || thumbnail.is_some()
        || actual
            != (
                choice.page(),
                "automatic".into(),
                COVER_ALGORITHM_VERSION,
                choice.confidence(),
                choice.aspect(),
                String::new(),
                String::new(),
                "contain".into(),
                now,
                modified,
            )
    {
        return Err(Failure::Storage(
            "Automatic cover could not be verified".into(),
        ));
    }
    transaction.commit().map_err(storage)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::{
        LibraryRepository, RenderKey, RenderedPage,
        cover_policy::CoverChoice,
        memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
    };
    fn database() -> Database {
        let db = Database::in_memory_at(11).unwrap();
        db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,thumbnail_path,last_page_index,reading_progress) VALUES('d','synthetic.pdf','synthetic','synthetic.pdf',3,1,'2026-10-13','now','now','original','old-cache',1,0.5);").unwrap();
        db
    }
    fn choice(db: &Database) -> AutomaticCover {
        let pixels =
            TrackedBytesBuilder::try_zeroed(&MemoryGovernor::default(), MemoryCategory::Pages, 48)
                .unwrap()
                .finish();
        let image = RenderedPage::from_pixels(
            RenderKey {
                document_id: "d".into(),
                page: 1,
                region: None,
                physical_width: 3,
                physical_height: 4,
                rotation: 0,
                generation: 9,
            },
            pixels,
        )
        .unwrap();
        AutomaticCover::new(
            db.get("d").unwrap().unwrap(),
            "2026-10-13".into(),
            CoverChoice {
                page: 1,
                score: 59.0,
                confidence: 0.79,
            },
            &image,
        )
        .unwrap()
    }
    #[test]
    fn automatic_choice_persists_verified_metadata_and_preserves_progress() {
        let mut db = database();
        let choice = choice(&db);
        assert!(db.save_automatic_cover(&choice).unwrap());
        let actual:(u32,String,u32,f64,f64,String,String,String)=db.connection.query_row("SELECT source_page_index,selection_mode,algorithm_version,confidence,page_aspect_ratio,small_thumbnail_path,large_thumbnail_path,crop_mode FROM document_covers WHERE document_id='d'",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?))).unwrap();
        assert_eq!(
            actual,
            (
                1,
                "automatic".into(),
                3,
                0.79,
                0.75,
                String::new(),
                String::new(),
                "contain".into()
            )
        );
        let document = db.get("d").unwrap().unwrap();
        assert_eq!(
            (
                document.last_page_index,
                document.reading_progress,
                document.fingerprint.as_str()
            ),
            (1, 0.5, "original")
        );
        assert!(document.thumbnail_path.is_none());
        assert!(!db.save_automatic_cover(&choice).unwrap());
    }
    #[test]
    fn existing_manual_or_automatic_choices_are_never_replaced() {
        for mode in ["manual", "automatic"] {
            let mut db = database();
            let choice = choice(&db);
            db.execute_batch(&format!("INSERT INTO document_covers VALUES('d',2,'keep-small','keep-large','old','old',1,'{mode}',4,'contain',0.75);")).unwrap();
            assert!(!db.save_automatic_cover(&choice).unwrap());
            let actual:(u32,String,String,u32)=db.connection.query_row("SELECT source_page_index,selection_mode,small_thumbnail_path,algorithm_version FROM document_covers WHERE document_id='d'",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).unwrap();
            assert_eq!(actual, (2, mode.into(), "keep-small".into(), 4));
            assert_eq!(
                db.get("d").unwrap().unwrap().thumbnail_path,
                Some("old-cache".into())
            );
        }
    }
    #[test]
    fn changed_catalogue_identity_and_rejected_or_mutated_writes_leave_no_partial_selection() {
        for sql in [
            "UPDATE documents SET fingerprint='changed' WHERE id='d'",
            "UPDATE documents SET path='changed.pdf' WHERE id='d'",
            "UPDATE documents SET page_count=2 WHERE id='d'",
            "UPDATE documents SET file_modified_at='changed' WHERE id='d'",
            "CREATE TRIGGER rejected BEFORE INSERT ON document_covers BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER ignored_cache BEFORE UPDATE OF thumbnail_path ON documents BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER aborted_cache BEFORE UPDATE OF thumbnail_path ON documents BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;",
            "CREATE TRIGGER altered AFTER INSERT ON document_covers BEGIN UPDATE document_covers SET algorithm_version=4 WHERE document_id=NEW.document_id; END;",
        ] {
            let mut db = database();
            let choice = choice(&db);
            db.execute_batch(sql).unwrap();
            assert!(db.save_automatic_cover(&choice).is_err(), "{sql}");
            assert_eq!(db.cover_source_page("d").unwrap(), None);
            assert_eq!(
                db.get("d").unwrap().unwrap().thumbnail_path,
                Some("old-cache".into())
            );
        }
    }

    #[test]
    fn restore_replaces_only_the_exact_captured_decision_and_cannot_replay_it() {
        let mut db = database();
        let choice = choice(&db);
        db.execute_batch("INSERT INTO document_covers VALUES('d',0,'old-small','old-large','old','old',1,'manual',4,'contain',0.75);").unwrap();
        let expected = db.cover_revision("d").unwrap();
        assert!(db.replace_automatic_cover(&choice, expected).unwrap());
        assert_eq!(db.cover_source_page("d").unwrap(), Some(1));
        assert_ne!(db.cover_revision("d").unwrap(), expected);
        assert!(!db.replace_automatic_cover(&choice, expected).unwrap());
    }
    #[test]
    fn changed_metadata_and_failed_restore_writes_preserve_the_manual_decision() {
        for sql in [
            "UPDATE document_covers SET small_thumbnail_path='new-small'",
            "UPDATE document_covers SET large_thumbnail_path='new-large'",
            "UPDATE document_covers SET generated_at='new'",
            "UPDATE document_covers SET source_modified_at='new'",
            "UPDATE document_covers SET confidence=0.7900000000000001",
            "UPDATE document_covers SET page_aspect_ratio=0.76",
            "UPDATE document_covers SET algorithm_version=5",
            "UPDATE document_covers SET source_page_index=2",
            "CREATE TRIGGER ignored BEFORE UPDATE ON document_covers BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER aborted BEFORE UPDATE OF thumbnail_path ON documents BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;",
            "CREATE TRIGGER mutated AFTER UPDATE ON document_covers BEGIN UPDATE document_covers SET algorithm_version=5 WHERE document_id=NEW.document_id; END;",
        ] {
            let mut db = database();
            let choice = choice(&db);
            db.execute_batch("INSERT INTO document_covers VALUES('d',0,'old-small','old-large','old','old',0.79,'manual',4,'contain',0.75);").unwrap();
            let expected = db.cover_revision("d").unwrap();
            db.execute_batch(sql).unwrap();
            let current = db.cover_revision("d").unwrap();
            if sql.starts_with("UPDATE") {
                assert_ne!(current, expected, "{sql}");
                assert!(!db.replace_automatic_cover(&choice, expected).unwrap());
            } else {
                assert!(db.replace_automatic_cover(&choice, expected).is_err());
            }
            assert_eq!(db.cover_revision("d").unwrap(), current, "{sql}");
            assert_eq!(
                db.get("d").unwrap().unwrap().thumbnail_path,
                Some("old-cache".into())
            );
        }
    }
}
