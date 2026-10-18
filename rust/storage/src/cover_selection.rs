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
