use crate::{Database, database::storage};
use rusqlite::{TransactionBehavior, params};
use yamantaka_domain::{
    Cancellation, Document, Failure, NewDocument, Result, front_matter::ImportAnalysis,
    metadata_review::MetadataReview, search::IndexAccess,
};

pub(crate) fn save(
    database: &mut Database,
    expected: &MetadataReview,
    original: &Document,
    analyzed: &NewDocument,
    access: IndexAccess,
    cancel: &Cancellation,
) -> Result<MetadataReview> {
    cancel.check()?;
    if !matches!(analyzed.analysis, ImportAnalysis::Complete(_)) {
        return Err(Failure::Pdf(
            "Metadata analysis could not complete within its work limits".into(),
        ));
    }
    if original.id != expected.document || original.path != analyzed.path {
        return Err(Failure::Invalid(
            "Metadata analysis belongs to another document".into(),
        ));
    }
    let tx = database
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    let before = crate::metadata_review::read(&tx, &expected.document, cancel)?;
    if before != *expected {
        return Err(Failure::Invalid(
            "Metadata changed during analysis; reload before retrying".into(),
        ));
    }
    let (path,fingerprint,pages,size,modified,missing):(String,String,u32,i64,String,bool)=tx.query_row("SELECT path,fingerprint,page_count,file_size,file_modified_at,is_missing FROM documents WHERE id=?",[&expected.document],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).map_err(storage)?;
    let source_before = (
        path.clone(),
        fingerprint.clone(),
        pages,
        size,
        modified.clone(),
        missing,
    );
    let canonical = std::fs::canonicalize(&path).map_err(storage)?;
    let native_modified: chrono::DateTime<chrono::Utc> = analyzed.source.modified.into();
    let catalogue_modified = chrono::DateTime::parse_from_rfc3339(&modified)
        .ok()
        .map(|t| t.with_timezone(&chrono::Utc));
    if missing
        || original.fingerprint != fingerprint
        || original.path != std::path::Path::new(&path)
        || canonical != analyzed.source.canonical_path
        || pages as usize != analyzed.info.pages.len()
        || u64::try_from(size).ok() != Some(analyzed.source.size)
        || catalogue_modified != Some(native_modified)
    {
        return Err(Failure::Invalid(
            "Revalidate the PDF before metadata analysis".into(),
        ));
    }
    if crate::identity::verify_import_source(&canonical, &analyzed.source)? != fingerprint {
        return Err(Failure::Invalid(
            "PDF fingerprint changed during metadata analysis".into(),
        ));
    }
    let file_name = canonical.file_name().unwrap_or_default().to_string_lossy();
    let (title, author) = crate::import_metadata::effective(analyzed, &file_name);
    for (baseline, analyzed) in [
        (
            yamantaka_domain::import_metadata::title(before.automatic_title.as_deref(), &file_name),
            &title,
        ),
        (
            yamantaka_domain::import_metadata::author(before.automatic_author.as_deref()),
            &author,
        ),
    ] {
        if let yamantaka_domain::import_metadata::MetadataValue::Accepted(value) = baseline
            && analyzed.as_deref() != Some(value.as_str())
        {
            return Err(Failure::Invalid(
                "Analysis does not preserve the accepted catalogue metadata".into(),
            ));
        }
    }
    if tx
        .execute(
            "UPDATE documents SET title=?,author=?,source_title=?,source_author=? WHERE id=?",
            params![
                title,
                author,
                analyzed.info.title,
                analyzed.info.author,
                expected.document
            ],
        )
        .map_err(storage)?
        != 1
    {
        return Err(Failure::Storage("Analyzed metadata was not saved".into()));
    }
    let now = chrono::Utc::now().to_rfc3339();
    crate::import_metadata::save(
        &tx,
        &expected.document,
        &fingerprint,
        &analyzed.analysis,
        [
            before.title_override.is_some(),
            before.author_override.is_some(),
        ],
        &now,
    )?;
    if access == IndexAccess::Enabled {
        crate::search::refresh_metadata(&tx, &expected.document)?;
    }
    let stored:[Option<String>;6]=tx.query_row("SELECT title,author,source_title,source_author,title_override,author_override FROM documents WHERE id=?",[&expected.document],|r|Ok([r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?])).map_err(storage)?;
    let source_after:(String,String,u32,i64,String,bool)=tx.query_row("SELECT path,fingerprint,page_count,file_size,file_modified_at,is_missing FROM documents WHERE id=?",[&expected.document],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).map_err(storage)?;
    if stored
        != [
            title,
            author,
            analyzed.info.title.clone(),
            analyzed.info.author.clone(),
            before.title_override,
            before.author_override,
        ]
        || source_after != source_before
        || crate::identity::verify_import_source(&canonical, &analyzed.source)? != fingerprint
    {
        return Err(Failure::Storage(
            "Analyzed metadata changed before commit".into(),
        ));
    }
    let result = crate::metadata_review::read(&tx, &expected.document, cancel)?;
    cancel.check()?;
    tx.commit().map_err(storage)?;
    Ok(result)
}
