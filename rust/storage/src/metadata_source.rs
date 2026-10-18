use crate::{Database, database::storage};
use yamantaka_domain::{
    Cancellation, Failure, LibraryRepository, Result,
    front_matter::{ENRICHMENT_ALGORITHM_VERSION, MAX_FRONT_MATTER_PAGES},
    metadata_review::{
        MetadataEvidence, MetadataEvidenceKind, MetadataField, MetadataFile, MetadataLocation,
        MetadataReview,
    },
};

pub(crate) fn evidence(
    record: &serde_json::Value,
    field: &str,
    value: &str,
    file: &MetadataFile,
) -> Option<MetadataEvidence> {
    if record["version"] != 1
        || record["analyzerVersion"] != ENRICHMENT_ALGORITHM_VERSION
        || record["hasText"] != true
    {
        return None;
    }
    if record["source"]
        != serde_json::json!({"fingerprint":file.fingerprint,"size":file.size,"modified":file.modified,"pages":file.page_count})
    {
        return None;
    }
    let decision = &record[field];
    let (candidate, kind) = match decision["kind"].as_str()? {
        "corroborated" => (decision, MetadataEvidenceKind::Corroborated),
        "review" | "autofill" => {
            let candidate = &decision["candidate"];
            let kind = match candidate["evidence"].as_str()? {
                "prominent_text" => MetadataEvidenceKind::ProminentText,
                "repeated_title_with_byline" => MetadataEvidenceKind::RepeatedTitleWithByline,
                "explicit_byline" => MetadataEvidenceKind::ExplicitByline,
                "conflicting_bylines" => MetadataEvidenceKind::ConflictingBylines,
                _ => return None,
            };
            (candidate, kind)
        }
        _ => return None,
    };
    let text = candidate["value"].as_str()?;
    let page = u32::try_from(candidate["page"].as_u64()?).ok()?;
    if text != value
        || text.is_empty()
        || text.chars().count() > 512
        || text.chars().any(char::is_control)
        || page >= file.page_count.min(MAX_FRONT_MATTER_PAGES)
    {
        return None;
    }
    Some(MetadataEvidence {
        value: text.into(),
        page,
        kind,
    })
}

fn verify_file(file: &MetadataFile, cancel: &Cancellation) -> Result<()> {
    cancel.check()?;
    let path = std::fs::canonicalize(&file.path).map_err(storage)?;
    let before = path.metadata().map_err(storage)?;
    let modified = before.modified().map_err(storage)?;
    let expected = chrono::DateTime::parse_from_rfc3339(&file.modified)
        .ok()
        .map(|t| t.with_timezone(&chrono::Utc));
    if before.len() != file.size
        || expected != Some(chrono::DateTime::<chrono::Utc>::from(modified))
    {
        return Err(Failure::Invalid(
            "PDF changed; revalidate it before opening metadata evidence".into(),
        ));
    }
    let fingerprint = crate::identity::legacy_fingerprint(&path)?;
    cancel.check()?;
    let after = path.metadata().map_err(storage)?;
    if fingerprint != file.fingerprint
        || before.len() != after.len()
        || modified != after.modified().map_err(storage)?
        || std::fs::canonicalize(&file.path).map_err(storage)? != path
    {
        return Err(Failure::Invalid(
            "PDF changed while checking metadata evidence".into(),
        ));
    }
    Ok(())
}

pub(crate) fn open(
    database: &Database,
    expected: &MetadataReview,
    field: MetadataField,
    cancel: &Cancellation,
) -> Result<MetadataLocation> {
    cancel.check()?;
    // One SQLite snapshot binds the displayed decision, source and document row.
    let tx = database
        .connection
        .unchecked_transaction()
        .map_err(storage)?;
    let (current, _) = crate::metadata_review::read_record(&tx, &expected.document, cancel)?;
    if current != *expected {
        return Err(Failure::Invalid(
            "Metadata changed; reload the review before opening its source".into(),
        ));
    }
    let evidence = current.evidence(field).ok_or_else(|| {
        Failure::Invalid("This metadata value has no current page evidence".into())
    })?;
    let document = database
        .get(&current.document)?
        .ok_or_else(|| Failure::Invalid("Metadata document no longer exists".into()))?;
    if document.missing || evidence.page >= document.page_count {
        return Err(Failure::Invalid("Metadata source is unavailable".into()));
    }
    verify_file(&current.file, cancel)?;
    let location = MetadataLocation {
        document,
        page: evidence.page,
    };
    cancel.check()?;
    tx.commit().map_err(storage)?;
    Ok(location)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn evidence_requires_matching_values_versions_pages_and_source_stamps() {
        let file = MetadataFile {
            path: "a.pdf".into(),
            fingerprint: "fa".into(),
            page_count: 2,
            missing: false,
            size: 10,
            modified: "2026-10-18".into(),
        };
        let valid = serde_json::json!({"version":1,"analyzerVersion":2,"hasText":true,"source":{"fingerprint":"fa","size":10,"modified":"2026-10-18","pages":2},"title":{"kind":"review","candidate":{"value":"Geometry","page":1,"evidence":"prominent_text"}}});
        let actual = evidence(&valid, "title", "Geometry", &file).unwrap();
        assert_eq!((actual.value.as_str(), actual.page), ("Geometry", 1));
        for (pointer, value) in [
            ("/version", serde_json::json!(2)),
            ("/analyzerVersion", serde_json::json!(1)),
            ("/hasText", serde_json::json!(false)),
            ("/source/modified", serde_json::json!("changed")),
            ("/source/fingerprint", serde_json::json!("different")),
            ("/title/candidate/value", serde_json::json!("Other")),
            ("/title/candidate/page", serde_json::json!(2)),
            ("/title/candidate/page", serde_json::json!(-1)),
            ("/title/candidate/evidence", serde_json::json!("unknown")),
        ] {
            let mut bad = valid.clone();
            *bad.pointer_mut(pointer).unwrap() = value;
            assert!(
                evidence(&bad, "title", "Geometry", &file).is_none(),
                "{pointer}"
            );
        }
        let mut legacy = valid;
        legacy.as_object_mut().unwrap().remove("source");
        assert!(evidence(&legacy, "title", "Geometry", &file).is_none());
    }
}
