use crate::{Database, database::storage};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    metadata_review::{MetadataMutation, MetadataReview},
    search::IndexAccess,
};

pub(crate) fn read(
    db: &Connection,
    document: &str,
    cancel: &Cancellation,
) -> Result<MetadataReview> {
    read_record(db, document, cancel).map(|(review, _)| review)
}
fn read_record(
    db: &Connection,
    document: &str,
    cancel: &Cancellation,
) -> Result<(MetadataReview, serde_json::Value)> {
    cancel.check()?;
    let encoded: Option<Option<String>> = db.query_row(
        "SELECT CASE WHEN length(CAST(record AS BLOB))<=65536 THEN record ELSE NULL END FROM (SELECT json_object('fingerprint',d.fingerprint,'file',d.file_name,'title',d.title,'author',d.author,'sourceTitle',d.source_title,'sourceAuthor',d.source_author,'titleOverride',d.title_override,'authorOverride',d.author_override,'analysisFingerprint',e.fingerprint,'version',e.analyzer_version,'status',e.status,'suggestedTitle',e.suggested_title,'suggestedAuthor',e.suggested_author,'analyzedAt',e.analyzed_at,'error',e.error_code,'evidence',a.value) AS record FROM documents d LEFT JOIN document_enrichment e ON e.document_id=d.id LEFT JOIN app_metadata a ON a.key='rust.frontMatterEvidence.v1:'||d.id WHERE d.id=?)",
        [document], |row| row.get(0)).optional().map_err(storage)?;
    let encoded = encoded
        .ok_or_else(|| Failure::Invalid("Document is absent from the library".into()))?
        .ok_or_else(|| Failure::Invalid("Metadata review exceeds its work limit".into()))?;
    let row: serde_json::Value = serde_json::from_str(&encoded).map_err(storage)?;
    let value = |name: &str| row[name].as_str().map(str::to_owned);
    let title_override = value("titleOverride");
    let author_override = value("authorOverride");
    let current_analysis =
        row["fingerprint"] == row["analysisFingerprint"] && row["status"] == "review";
    let result = MetadataReview {
        document: document.into(),
        revision: format!("{:x}", Sha256::digest(encoded.as_bytes())),
        title: title_override
            .clone()
            .or_else(|| value("title"))
            .or_else(|| value("sourceTitle"))
            .filter(|s| !s.is_empty())
            .or_else(|| value("file"))
            .unwrap_or_default(),
        author: author_override
            .clone()
            .or_else(|| value("author"))
            .or_else(|| value("sourceAuthor"))
            .unwrap_or_default(),
        source_title: value("sourceTitle"),
        source_author: value("sourceAuthor"),
        title_override,
        author_override,
        suggested_title: current_analysis.then(|| value("suggestedTitle")).flatten(),
        suggested_author: current_analysis.then(|| value("suggestedAuthor")).flatten(),
    };
    cancel.check()?;
    Ok((result, row))
}

pub(crate) fn change(
    database: &mut Database,
    expected: &MetadataReview,
    change: &MetadataMutation,
    access: IndexAccess,
    cancel: &Cancellation,
) -> Result<MetadataReview> {
    cancel.check()?;
    let tx = database
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    let (before, mut intended) = read_record(&tx, &expected.document, cancel)?;
    if before != *expected {
        return Err(Failure::Invalid(
            "Metadata changed; reload the review before saving".into(),
        ));
    }
    let (title, author) = match change {
        MetadataMutation::Save(value) => (Some(value.title()), Some(value.author())),
        MetadataMutation::ResetOverrides => (None, None),
        MetadataMutation::DismissSuggestions => (
            before.title_override.as_deref(),
            before.author_override.as_deref(),
        ),
    };
    if tx
        .execute(
            "UPDATE documents SET title_override=?,author_override=? WHERE id=?",
            params![title, author, before.document],
        )
        .map_err(storage)?
        != 1
    {
        return Err(Failure::Storage("Metadata correction was not saved".into()));
    }
    let dismiss = !matches!(change, MetadataMutation::ResetOverrides);
    intended["titleOverride"] = serde_json::json!(title);
    intended["authorOverride"] = serde_json::json!(author);
    if dismiss {
        if intended["status"] == "review" {
            intended["status"] = serde_json::json!("completed");
        }
        intended["suggestedTitle"] = serde_json::Value::Null;
        intended["suggestedAuthor"] = serde_json::Value::Null;
        tx.execute("UPDATE document_enrichment SET status=CASE WHEN status='review' THEN 'completed' ELSE status END,suggested_title=NULL,suggested_author=NULL WHERE document_id=?",[&before.document]).map_err(storage)?;
    }
    if access == IndexAccess::Enabled {
        crate::search::refresh_metadata(&tx, &before.document)?;
    }
    let (after, stored) = read_record(&tx, &before.document, cancel)?;
    if stored != intended {
        return Err(Failure::Storage(
            "Metadata correction changed before commit".into(),
        ));
    }
    cancel.check()?;
    tx.commit().map_err(storage)?;
    Ok(after)
}
