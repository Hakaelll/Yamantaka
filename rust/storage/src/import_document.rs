use crate::{Database, database::storage};
use rusqlite::params;
use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository, NewDocument, Result, source_revision::FullSourceHash,
};
pub(crate) fn save(
    database: &mut Database,
    document: &NewDocument,
    reference: &FullSourceHash,
    cancel: &Cancellation,
    import_job: Option<&str>,
) -> Result<String> {
    let id = uuid::Uuid::new_v4().to_string();
    let result = insert(database, &id, document, reference, cancel, import_job);
    if result.is_err() && matches!(database.get(&id), Ok(None)) {
        // An absent main row cannot own this new UUID's derived reference.
        let _ = crate::file_check::remove_import_reference(database, &id);
    }
    result
}
fn insert(
    database: &mut Database,
    id: &str,
    document: &NewDocument,
    reference: &FullSourceHash,
    cancel: &Cancellation,
    import_job: Option<&str>,
) -> Result<String> {
    cancel.check()?;
    if reference.source != document.source {
        return Err(Failure::Invalid(
            "Import content reference belongs to another source revision".into(),
        ));
    }
    let path = std::fs::canonicalize(&document.path).map_err(storage)?;
    let normalized = path.to_string_lossy().to_lowercase();
    if let Some(id) = crate::identity::registered_document(&database.connection, &path)? {
        return Ok(id);
    }
    let metadata = path.metadata().map_err(storage)?;
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let (title, author) = crate::import_metadata::effective(document, &file_name);

    let now = chrono::Utc::now().to_rfc3339();
    let fingerprint = crate::identity::verify_import_source(&path, &document.source)?;
    let transaction = database.connection.transaction().map_err(storage)?;
    if transaction.execute("INSERT INTO documents(id,path,normalized_path,file_name,title,author,source_title,source_author,page_count,file_size,file_modified_at,added_at,last_opened_at,fingerprint) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?12,?13)", params![id,path.to_string_lossy(),normalized,file_name,title,author,document.info.title,document.info.author,document.info.pages.len() as i64,metadata.len() as i64,chrono::DateTime::<chrono::Utc>::from(metadata.modified().map_err(storage)?).to_rfc3339(),now,fingerprint]).map_err(storage)? !=1 {
            return Err(Failure::Storage("Imported document was not saved".into()));
        }
    crate::import_metadata::save(
        &transaction,
        id,
        &fingerprint,
        &document.analysis,
        [false, false],
        &now,
    )?;
    let saved:serde_json::Value=transaction.query_row("SELECT json_object('path',path,'normalized',normalized_path,'file',file_name,'title',title,'author',author,'sourceTitle',source_title,'sourceAuthor',source_author,'pages',page_count,'size',file_size,'modified',file_modified_at,'fingerprint',fingerprint) FROM documents WHERE id=?",[&id],|row|row.get(0)).map_err(storage)?;
    let expected = serde_json::json!({"path":path.to_string_lossy(),"normalized":normalized,"file":file_name,"title":title,"author":author,"sourceTitle":document.info.title,"sourceAuthor":document.info.author,"pages":document.info.pages.len(),"size":metadata.len(),"modified":chrono::DateTime::<chrono::Utc>::from(metadata.modified().map_err(storage)?).to_rfc3339(),"fingerprint":fingerprint});
    if saved != expected
        || crate::identity::verify_import_source(&path, &document.source)? != fingerprint
    {
        return Err(Failure::Storage(
            "Imported metadata changed before commit".into(),
        ));
    }
    crate::file_check::save_import_reference(
        &transaction,
        &database.file_checks,
        id,
        document,
        &reference.digest,
    )?;
    if let Some(job) = import_job {
        if job.is_empty() || job.len() > 128 {
            return Err(Failure::Invalid("Invalid import job identity".into()));
        }
        let receipt = serde_json::json!({"version":1,"job":job,"document":id,"path":crate::import_recovery::key(&path),"fingerprint":fingerprint,"pages":document.info.pages.len()});
        transaction.execute("INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,started_at,updated_at,phase,resume_capability,compatibility_hash) VALUES(?1,'rustImportMember.v1','completed',?2,1,1,?3,?3,'completed','none','rust-import-member-v1')",params![format!("rust-import-member-{id}"),receipt.to_string(),now]).map_err(storage)?;
    }
    cancel.check()?;
    transaction.commit().map_err(storage)?;
    Ok(id.to_owned())
}
