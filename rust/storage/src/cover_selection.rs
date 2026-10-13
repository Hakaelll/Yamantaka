use crate::database::{Database, storage};
use rusqlite::OptionalExtension;
use yamantaka_domain::Result;

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
