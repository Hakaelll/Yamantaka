use crate::database::{Database, storage};
use rusqlite::OptionalExtension;
use yamantaka_domain::Result;
use yamantaka_domain::{Failure, cover_selection::ManualCover};

pub(crate) fn save(database: &mut Database, choice: &ManualCover) -> Result<()> {
    let transaction = database.connection.transaction().map_err(storage)?;
    let current: Option<(String, String, u32, String)> = transaction
        .query_row(
            "SELECT path,fingerprint,page_count,file_modified_at FROM documents WHERE id=?1",
            [choice.document()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(storage)?;
    let Some((path, fingerprint, count, modified)) = current else {
        return Err(Failure::Invalid("Document no longer exists".into()));
    };
    if std::path::Path::new(&path) != choice.path()
        || fingerprint != choice.fingerprint()
        || choice.page() >= count
    {
        return Err(Failure::Invalid(
            "Document changed; reopen the cover picker".into(),
        ));
    }
    let now = chrono::Utc::now().to_rfc3339();
    let affected = transaction.execute(
        "INSERT INTO document_covers(document_id,source_page_index,small_thumbnail_path,large_thumbnail_path,generated_at,source_modified_at,confidence,selection_mode,algorithm_version,crop_mode,page_aspect_ratio)
         VALUES(?1,?2,'','',?3,?4,1,'manual',1,'contain',?5)
         ON CONFLICT(document_id) DO UPDATE SET source_page_index=excluded.source_page_index,small_thumbnail_path='',large_thumbnail_path='',generated_at=excluded.generated_at,source_modified_at=excluded.source_modified_at,confidence=1,selection_mode='manual',algorithm_version=1,crop_mode='contain',page_aspect_ratio=excluded.page_aspect_ratio",
        rusqlite::params![choice.document(),choice.page(),now,modified,choice.aspect()],
    ).map_err(storage)?;
    if affected != 1 {
        return Err(Failure::Storage("Cover selection was not saved".into()));
    }
    let updated = transaction
        .execute(
            "UPDATE documents SET thumbnail_path=NULL WHERE id=?1",
            [choice.document()],
        )
        .map_err(storage)?;
    let actual: (u32,String,f64,String,String) = transaction.query_row(
        "SELECT source_page_index,selection_mode,page_aspect_ratio,small_thumbnail_path,large_thumbnail_path FROM document_covers WHERE document_id=?1",[choice.document()],
        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?)),
    ).map_err(storage)?;
    let thumbnail: Option<String> = transaction
        .query_row(
            "SELECT thumbnail_path FROM documents WHERE id=?1",
            [choice.document()],
            |row| row.get(0),
        )
        .map_err(storage)?;
    if updated != 1
        || actual
            != (
                choice.page(),
                "manual".into(),
                choice.aspect(),
                String::new(),
                String::new(),
            )
        || thumbnail.is_some()
    {
        return Err(Failure::Storage(
            "Cover selection could not be verified".into(),
        ));
    }
    transaction.commit().map_err(storage)
}

pub(crate) fn source_page(database: &Database, id: &str) -> Result<Option<u32>> {
    database
        .connection
        .query_row(
            "SELECT source_page_index FROM document_covers WHERE document_id=?1",
            [id],
            |row| row.get(0),
        )
        .optional()
        .map_err(storage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::LibraryRepository;

    fn editable() -> Database {
        let db = Database::in_memory_at(11).unwrap();
        db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint,thumbnail_path,last_page_index,reading_progress) VALUES('d','synthetic.pdf','synthetic','synthetic.pdf',3,1,'now','now','now','synthetic','document-cache',1,0.5);
            INSERT INTO document_covers VALUES('d',2,'old-small','old-large','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z',1,'manual',4,'contain',0.75);").unwrap();
        db
    }

    #[test]
    fn choosing_a_previewed_page_persists_manual_intent_and_invalidates_only_cache_references() {
        let mut db = editable();
        let before = db.get("d").unwrap().unwrap();
        let choice = ManualCover::new(&before, 1, 4.0 / 3.0).unwrap();
        db.save_manual_cover(&choice).unwrap();
        assert_eq!(db.cover_source_page("d").unwrap(), Some(1));
        let actual: (String,String,String,f64) = db.connection.query_row(
            "SELECT selection_mode,small_thumbnail_path,large_thumbnail_path,page_aspect_ratio FROM document_covers WHERE document_id='d'",[],
            |row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)),
        ).unwrap();
        assert_eq!(
            actual,
            ("manual".into(), String::new(), String::new(), 4.0 / 3.0)
        );
        let after = db.get("d").unwrap().unwrap();
        assert!(after.thumbnail_path.is_none());
        assert_eq!(after.path, before.path);
        assert_eq!(after.fingerprint, before.fingerprint);
        assert_eq!(after.last_page_index, 1);
        assert_eq!(after.reading_progress, 0.5);
    }

    #[test]
    fn changed_documents_and_invalid_choices_preserve_the_previous_cover() {
        let mut db = editable();
        let document = db.get("d").unwrap().unwrap();
        for (page, aspect) in [
            (3, 0.75),
            (0, f64::NAN),
            (0, f64::INFINITY),
            (0, 0.09),
            (0, 10.01),
        ] {
            assert!(ManualCover::new(&document, page, aspect).is_err());
        }
        let choice = ManualCover::new(&document, 1, 0.75).unwrap();
        db.execute_batch("UPDATE documents SET fingerprint='changed' WHERE id='d'")
            .unwrap();
        assert!(db.save_manual_cover(&choice).is_err());
        assert_eq!(db.cover_source_page("d").unwrap(), Some(2));
        assert_eq!(
            db.get("d").unwrap().unwrap().thumbnail_path,
            Some("document-cache".into())
        );
    }

    #[test]
    fn ignored_aborted_and_mutated_sqlite_writes_roll_back_the_cover_and_document_cache_together() {
        for trigger in [
            "CREATE TRIGGER denied_cover BEFORE UPDATE ON document_covers BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER denied_cache BEFORE UPDATE OF thumbnail_path ON documents BEGIN SELECT RAISE(IGNORE); END;",
            "CREATE TRIGGER aborted_cache BEFORE UPDATE OF thumbnail_path ON documents BEGIN SELECT RAISE(ABORT,'synthetic write rejection'); END;",
            "CREATE TRIGGER changed_cover AFTER UPDATE ON document_covers BEGIN UPDATE document_covers SET source_page_index=0 WHERE document_id=NEW.document_id; END;",
        ] {
            let mut db = editable();
            let choice = ManualCover::new(&db.get("d").unwrap().unwrap(), 1, 4.0 / 3.0).unwrap();
            db.execute_batch(trigger).unwrap();
            assert!(db.save_manual_cover(&choice).is_err());
            assert_eq!(db.cover_source_page("d").unwrap(), Some(2));
            let paths: (String,String,u32) = db.connection.query_row(
                "SELECT small_thumbnail_path,large_thumbnail_path,algorithm_version FROM document_covers WHERE document_id='d'",[],
                |row|Ok((row.get(0)?,row.get(1)?,row.get(2)?)),
            ).unwrap();
            assert_eq!(paths, ("old-small".into(), "old-large".into(), 4));
            assert_eq!(
                db.get("d").unwrap().unwrap().thumbnail_path,
                Some("document-cache".into())
            );
        }
    }

    #[test]
    fn saved_cover_page_is_read_without_rewriting_selection_or_cache_metadata() {
        let db = Database::in_memory_at(11).unwrap();
        db.execute_batch("INSERT INTO documents(id,path,normalized_path,file_name,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES('d','synthetic.pdf','synthetic','synthetic.pdf',3,1,'now','now','now','synthetic');
            INSERT INTO document_covers VALUES('d',2,'old-small','old-large','2026-10-01T00:00:00Z','2026-10-01T00:00:00Z',1,'manual',4,'contain',0.75);").unwrap();
        assert_eq!(db.cover_source_page("d").unwrap(), Some(2));
        assert_eq!(db.cover_source_page("missing").unwrap(), None);
        let preserved: (String, String, String, u32) = db.connection.query_row(
            "SELECT selection_mode,small_thumbnail_path,large_thumbnail_path,algorithm_version FROM document_covers WHERE document_id='d'", [],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)),
        ).unwrap();
        assert_eq!(
            preserved,
            ("manual".into(), "old-small".into(), "old-large".into(), 4)
        );
    }
}
