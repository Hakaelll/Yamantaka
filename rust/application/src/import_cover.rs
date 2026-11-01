//! Map the temporary native owner to the committed catalogue identity without copying pixels.
use std::sync::Arc;
use yamantaka_domain::{
    Failure, LibraryRepository, NewDocument, Result, cover_policy::DetectedCover,
    cover_snapshot::CoverPlan, source_revision::legacy_fingerprint,
};

pub(crate) fn persist(
    repository: &mut impl LibraryRepository,
    id: &str,
    imported: &NewDocument,
    detected: DetectedCover,
) -> Result<()> {
    let document = repository
        .get(id)?
        .ok_or_else(|| Failure::Invalid("Imported cover document is missing".into()))?;
    if document.path != imported.source.canonical_path
        || document.page_count as usize != imported.info.pages.len()
        || document.fingerprint
            != legacy_fingerprint(imported.source.size, imported.source.head_sha256)
    {
        return Err(Failure::Invalid("Imported cover source changed".into()));
    }
    let snapshot = repository.cover_snapshot(id)?;
    if snapshot.plan != CoverPlan::Detect {
        return Ok(());
    }
    let mut image = detected.image.at_generation(0);
    image.key.document_id = id.into();
    crate::automatic_cover::complete(
        repository,
        document,
        0,
        DetectedCover {
            image: Arc::new(image),
            choice: detected.choice,
        },
        snapshot,
    )?;
    Ok(())
}
