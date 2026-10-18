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
