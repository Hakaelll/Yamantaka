use crate::database::storage;
use rusqlite::{Transaction, params};
use serde_json::{Value, json};
use yamantaka_domain::{
    Failure, NewDocument, Result,
    front_matter::{
        ENRICHMENT_ALGORITHM_VERSION, EvidenceKind, ImportAnalysis, MetadataCandidate,
        MetadataDecision,
    },
    import_metadata::{self, MetadataValue},
};

fn value(value: MetadataValue, fallback: &str) -> Option<String> {
    match value {
        MetadataValue::Missing => None,
        MetadataValue::Accepted(value) => Some(value),
        MetadataValue::Rejected => Some(fallback.into()),
    }
}

fn decision_value(decision: &MetadataDecision, fallback: &str) -> Option<String> {
    match decision {
        MetadataDecision::Unchanged(current) | MetadataDecision::Review { current, .. } => {
            match current {
                MetadataValue::Missing => None,
                MetadataValue::Accepted(value) => Some(value.clone()),
                MetadataValue::Rejected => Some(fallback.into()),
            }
        }
        MetadataDecision::Corroborated { value, .. } => Some(value.clone()),
        MetadataDecision::Autofill(candidate) => Some(candidate.value.clone()),
    }
}

pub(crate) fn effective(
    document: &NewDocument,
    file_name: &str,
) -> (Option<String>, Option<String>) {
    if let ImportAnalysis::Complete(evidence) = &document.analysis {
        (
            decision_value(&evidence.title, file_name),
            decision_value(&evidence.author, ""),
        )
    } else {
        (
            value(
                import_metadata::title(document.info.title.as_deref(), file_name),
                file_name,
            ),
            value(import_metadata::author(document.info.author.as_deref()), ""),
        )
    }
}

fn candidate(candidate: &MetadataCandidate) -> Value {
    json!({"value":candidate.value,"page":candidate.page,
    "bounds":[candidate.bounds.min(),candidate.bounds.max()],
    "evidence":match candidate.evidence {
        EvidenceKind::ProminentText=>"prominent_text",EvidenceKind::RepeatedTitleWithByline=>"repeated_title_with_byline",
        EvidenceKind::ExplicitByline=>"explicit_byline",EvidenceKind::ConflictingBylines=>"conflicting_bylines",
    }})
}

fn current(value: &MetadataValue) -> Value {
    match value {
        MetadataValue::Missing => json!({"kind":"missing"}),
        MetadataValue::Rejected => json!({"kind":"rejected"}),
        MetadataValue::Accepted(value) => json!({"kind":"metadata","value":value}),
    }
}

fn decision(decision: &MetadataDecision) -> Value {
    match decision {
        MetadataDecision::Unchanged(value) => json!({"kind":"unchanged","current":current(value)}),
        MetadataDecision::Corroborated { value, page } => {
            json!({"kind":"corroborated","value":value,"page":page})
        }
        MetadataDecision::Autofill(value) => {
            json!({"kind":"autofill","candidate":candidate(value)})
        }
        MetadataDecision::Review {
            current: old,
            candidate: value,
        } => json!({"kind":"review","current":current(old),"candidate":candidate(value)}),
    }
}

fn proposal(decision: &MetadataDecision) -> Option<&str> {
    match decision {
        MetadataDecision::Review { candidate, .. } => Some(&candidate.value),
        _ => None,
    }
}

pub(crate) fn save(
    tx: &Transaction<'_>,
    id: &str,
    fingerprint: &str,
    analysis: &ImportAnalysis,
    now: &str,
) -> Result<()> {
    let (status, title, author, error, record) = match analysis {
        ImportAnalysis::NotRequested => return Ok(()),
        ImportAnalysis::Unavailable(reason) => (
            "failed",
            None,
            None,
            Some(reason.code()),
            json!({"version":1,"analyzerVersion":ENRICHMENT_ALGORITHM_VERSION,"error":reason.code()}),
        ),
        ImportAnalysis::Complete(evidence) => {
            let title = proposal(&evidence.title);
            let author = proposal(&evidence.author);
            let status = if title.is_some() || author.is_some() {
                "review"
            } else if evidence.has_text {
                "completed"
            } else {
                "noText"
            };
            (
                status,
                title,
                author,
                None,
                json!({"version":1,"analyzerVersion":ENRICHMENT_ALGORITHM_VERSION,"hasText":evidence.has_text,"title":decision(&evidence.title),"author":decision(&evidence.author)}),
            )
        }
    };
    if tx.execute("INSERT INTO document_enrichment(document_id,fingerprint,analyzer_version,status,suggested_title,suggested_author,analyzed_at,error_code) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![id,fingerprint,ENRICHMENT_ALGORITHM_VERSION,status,title,author,now,error]).map_err(storage)? !=1 {
        return Err(Failure::Storage("Imported metadata analysis was not saved".into()));
    }
    let key = format!("rust.frontMatterEvidence.v1:{id}");
    let encoded = serde_json::to_string(&record).map_err(storage)?;
    if tx
        .execute(
            "INSERT INTO app_metadata(key,value,updated_at) VALUES(?1,?2,?3)",
            params![key, encoded, now],
        )
        .map_err(storage)?
        != 1
    {
        return Err(Failure::Storage(
            "Imported metadata evidence was not saved".into(),
        ));
    }
    let saved:(String,u32,String,Option<String>,Option<String>,String,Option<String>)=tx.query_row(
        "SELECT fingerprint,analyzer_version,status,suggested_title,suggested_author,analyzed_at,error_code FROM document_enrichment WHERE document_id=?",[id],
        |row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?)),
    ).map_err(storage)?;
    if saved
        != (
            fingerprint.into(),
            ENRICHMENT_ALGORITHM_VERSION,
            status.into(),
            title.map(str::to_owned),
            author.map(str::to_owned),
            now.into(),
            error.map(str::to_owned),
        )
    {
        return Err(Failure::Storage(
            "Imported metadata analysis changed before commit".into(),
        ));
    }
    let saved: String = tx
        .query_row("SELECT value FROM app_metadata WHERE key=?", [key], |row| {
            row.get(0)
        })
        .map_err(storage)?;
    if saved != encoded {
        return Err(Failure::Storage(
            "Imported metadata evidence changed before commit".into(),
        ));
    }
    Ok(())
}
