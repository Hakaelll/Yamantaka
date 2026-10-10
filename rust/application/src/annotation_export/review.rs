use super::*;
use yamantaka_domain::{annotations::repository::ANNOTATION_PAGE_SIZE, memory::MemoryGovernor};

pub fn write_review_snapshot<W: Write>(
    writer: W,
    format: AnnotationExportFormat,
    snapshot: &yamantaka_domain::annotations::review::AnnotationReviewSnapshot,
    exported_at: &str,
    governor: &MemoryGovernor,
    cancel: Cancellation,
) -> Result<(W, u64)> {
    use yamantaka_domain::memory::{Cost, MemoryCategory, MemoryRepresentation};
    cancel.check()?;
    let Some(first) = snapshot.page().entries().first() else {
        return write_annotation_review(writer, format, &[], exported_at, governor, cancel);
    };
    let mut rows = [first; ANNOTATION_PAGE_SIZE];
    let mut lengths = [0usize; ANNOTATION_PAGE_SIZE];
    let mut offset = 0;
    for (index, metadata) in snapshot.documents().iter().enumerate() {
        for record in snapshot
            .page()
            .entries()
            .iter()
            .filter(|r| r.data().values.document_id == metadata.document)
        {
            rows[offset] = record;
            lengths[index] += 1;
            offset += 1;
        }
    }
    let bytes = snapshot
        .documents()
        .len()
        .checked_mul(std::mem::size_of::<AnnotationReviewDocument<'_>>())
        .ok_or(Failure::MemoryBudget)?;
    let mut lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Other,
            MemoryRepresentation::Cpu,
            bytes,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut documents = Vec::new();
    documents
        .try_reserve_exact(snapshot.documents().len())
        .map_err(|_| Failure::MemoryBudget)?;
    lease
        .try_resize(
            documents
                .capacity()
                .checked_mul(std::mem::size_of::<AnnotationReviewDocument<'_>>())
                .ok_or(Failure::MemoryBudget)?,
        )
        .map_err(|_| Failure::MemoryBudget)?;
    offset = 0;
    for (index, metadata) in snapshot.documents().iter().enumerate() {
        documents.push(AnnotationReviewDocument {
            metadata: AnnotationExportMetadata {
                document: &metadata.document,
                title: metadata.title.as_deref().unwrap_or("Unavailable document"),
                author: metadata.author.as_deref(),
                progress: metadata.progress,
                exported_at,
            },
            annotations: &rows[offset..offset + lengths[index]],
        });
        offset += lengths[index];
    }
    write_annotation_review(writer, format, &documents, exported_at, governor, cancel)
}

pub struct AnnotationReviewDocument<'a> {
    pub metadata: AnnotationExportMetadata<'a>,
    pub annotations: &'a [&'a AnnotationRecord],
}

/// Writes the bounded review page using the legacy global envelope and document formatter.
pub fn write_annotation_review<W: Write>(
    writer: W,
    format: AnnotationExportFormat,
    documents: &[AnnotationReviewDocument<'_>],
    exported_at: &str,
    governor: &MemoryGovernor,
    cancel: Cancellation,
) -> Result<(W, u64)> {
    let result = (|| {
        cancel.check()?;
        let date = utc(exported_at)?;
        if documents.len() > ANNOTATION_PAGE_SIZE {
            return Err(Failure::Invalid(
                "Review export exceeds one annotation page".into(),
            ));
        }
        let mut total = 0usize;
        for (index, document) in documents.iter().enumerate() {
            let id = document.metadata.document;
            if id.is_empty()
                || document.annotations.is_empty()
                || !document.metadata.progress.is_finite()
                || documents[..index].iter().any(|d| d.metadata.document == id)
            {
                return Err(Failure::Invalid("Review export document is invalid".into()));
            }
            total = total
                .checked_add(document.annotations.len())
                .ok_or(Failure::MemoryBudget)?;
            if total > ANNOTATION_PAGE_SIZE {
                return Err(Failure::Invalid(
                    "Review export exceeds one annotation page".into(),
                ));
            }
            for (position, record) in document.annotations.iter().enumerate() {
                if record.data().values.document_id != id
                    || document.annotations[..position]
                        .iter()
                        .any(|r| r.data().id == record.data().id)
                {
                    return Err(Failure::Invalid(
                        "Review export annotation identity is invalid".into(),
                    ));
                }
            }
        }
        let mut writer = output::ExportOutput::new(writer, governor, cancel.clone())?;
        if matches!(format, AnnotationExportFormat::Json) {
            writer
                .write_all(
                    b"{\"format\":\"yamantaka-global-annotations\",\"version\":1,\"exportedAt\":",
                )
                .map_err(write_error)?;
            serde_json::to_writer(&mut writer, &date).map_err(write_error)?;
            writer.write_all(b",\"documents\":[").map_err(write_error)?;
        }
        for (index, document) in documents.iter().enumerate() {
            cancel.check()?;
            if index > 0 {
                writer
                    .write_all(match format {
                        AnnotationExportFormat::Json => b",",
                        AnnotationExportFormat::Markdown => b"\n---\n\n",
                    })
                    .map_err(write_error)?;
            }
            let mut ordered: [usize; ANNOTATION_PAGE_SIZE] = std::array::from_fn(|index| index);
            let ordered = &mut ordered[..document.annotations.len()];
            ordered.sort_unstable_by_key(|index| {
                (
                    document.annotations[*index].data().values.page_index,
                    *index,
                )
            });
            let mut exporter = AnnotationExporter::new(
                &mut writer,
                format,
                AnnotationExportMetadata {
                    document: document.metadata.document,
                    title: document.metadata.title,
                    author: document.metadata.author,
                    progress: document.metadata.progress,
                    exported_at: &date,
                },
                cancel.clone(),
            )?;
            for index in ordered.iter() {
                exporter.push(document.annotations[*index])?;
            }
            exporter.finish()?;
        }
        if matches!(format, AnnotationExportFormat::Json) {
            writer.write_all(b"]}\n").map_err(write_error)?;
        }
        writer.flush().map_err(write_error)?;
        Ok((writer.into_inner(), total as u64))
    })();
    result.or_else(|error| {
        cancel.check()?;
        Err(error)
    })
}
