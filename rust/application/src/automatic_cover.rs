use std::sync::Arc;
use yamantaka_domain::{
    Document, Failure, LibraryRepository, RenderedPage, Result, automatic_cover::AutomaticCover,
    cover_policy::DetectedCover,
};

pub(crate) enum CoverCompletion {
    Image(Arc<RenderedPage>),
    StoredPage(u32),
}
pub(crate) fn complete(
    repository: &mut impl LibraryRepository,
    document: Document,
    generation: u64,
    detected: DetectedCover,
    snapshot: yamantaka_domain::cover_snapshot::CoverSnapshot,
) -> Result<CoverCompletion> {
    if detected.image.key.generation != generation {
        return Err(Failure::Invalid("Stale automatic cover generation".into()));
    }
    let choice = AutomaticCover::new(
        document,
        snapshot.source_modified,
        detected.choice,
        &detected.image,
    )?;
    let saved = match snapshot.revision {
        Some(revision) => repository.replace_automatic_cover(&choice, Some(revision))?,
        None => repository.save_automatic_cover(&choice)?,
    };
    if saved {
        return Ok(CoverCompletion::Image(detected.image));
    }
    // A newer manual or automatic choice wins over background detection.
    match repository.cover_source_page(&choice.document().id)? {
        Some(page) if page == choice.page() => Ok(CoverCompletion::Image(detected.image)),
        Some(page) => Ok(CoverCompletion::StoredPage(page)),
        None => Err(Failure::Invalid(
            "Cover choice changed during completion".into(),
        )),
    }
}

pub(crate) fn restore(
    repository: &mut impl LibraryRepository,
    document: Document,
    generation: u64,
    detected: DetectedCover,
    expected: yamantaka_domain::cover_snapshot::CoverSnapshot,
) -> Result<Arc<RenderedPage>> {
    if detected.image.key.generation != generation {
        return Err(Failure::Invalid("Stale automatic cover generation".into()));
    }
    let choice = AutomaticCover::new(
        document,
        expected.source_modified,
        detected.choice,
        &detected.image,
    )?;
    if !repository.replace_automatic_cover(&choice, expected.revision)? {
        return Err(Failure::Invalid(
            "Cover changed while restoring; reopen the picker".into(),
        ));
    }
    Ok(detected.image)
}

#[cfg(test)]
mod freshness_tests;
#[cfg(test)]
mod restore_tests;
#[cfg(test)]
mod tests;
