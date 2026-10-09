use pdfium_render::prelude::*;
use std::{collections::HashMap, sync::Arc};
use yamantaka_domain::{
    Failure, Result,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
    outline::{OutlineEntry, PdfOutline},
};

const MAX_ENTRIES: usize = 10_000;
const MAX_TITLE_BYTES: usize = 4096;
const MAX_TOTAL_TITLE_BYTES: usize = 1024 * 1024;
const WORKING_BYTES: usize = 6 * 1024 * 1024;

pub(crate) fn read(
    document: &PdfDocument<'_>,
    governor: &MemoryGovernor,
    check_current: impl Fn() -> Result<()>,
) -> Result<Arc<PdfOutline>> {
    check_current()?;
    let lease = governor
        .try_reserve_one(Cost::new(
            MemoryCategory::Text,
            MemoryRepresentation::Cpu,
            WORKING_BYTES,
        ))
        .map_err(|_| Failure::MemoryBudget)?;
    let mut entries = Vec::new();
    let mut title_bytes = 0usize;
    let mut remaining_objects = crate::destination::MAX_CONTENT_OBJECTS;
    {
        let bookmarks = document.bookmarks();
        let mut indices = HashMap::new();
        for bookmark in bookmarks.iter() {
            check_current()?;
            if entries.len() == MAX_ENTRIES {
                return Err(Failure::Pdf("PDF outline exceeds the entry limit".into()));
            }
            let title = bookmark.title().unwrap_or_default();
            title_bytes = title_bytes
                .checked_add(title.capacity())
                .ok_or(Failure::MemoryBudget)?;
            if title.len() > MAX_TITLE_BYTES || title_bytes > MAX_TOTAL_TITLE_BYTES {
                return Err(Failure::Pdf(
                    "PDF outline titles exceed the memory limit".into(),
                ));
            }
            let parent = bookmark
                .parent()
                .and_then(|parent| indices.get(&parent).copied());
            let destination = if let Some(destination) = bookmark.destination() {
                Some(crate::destination::read(
                    destination,
                    document,
                    &check_current,
                    &mut remaining_objects,
                )?)
            } else if let Some(action) = bookmark.action() {
                action
                    .as_local_destination_action()
                    .map(|action| {
                        action
                            .destination()
                            .map_err(super::worker::pdf_failure)
                            .and_then(|destination| {
                                crate::destination::read(
                                    destination,
                                    document,
                                    &check_current,
                                    &mut remaining_objects,
                                )
                            })
                    })
                    .transpose()?
            } else {
                None
            };
            indices.insert(bookmark, entries.len());
            entries.push(OutlineEntry {
                title,
                parent,
                destination,
            });
        }
    }
    check_current()?;
    Ok(Arc::new(PdfOutline::from_reserved(entries, lease)?))
}
