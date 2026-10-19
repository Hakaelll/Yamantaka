//! Presentation commands dispatch only through use cases.
mod cover_job;
use yamantaka_domain::{Document, LibraryQuery, LibraryRepository, Result};
pub mod annotation_export;
pub mod annotations;
mod automatic_cover;
pub mod cache_maintenance;
pub mod cover_source;
pub mod document_find;
pub mod file_check;
pub mod file_comparison;
mod file_digest;
mod folder_import;
mod import_enrichment;
mod import_hash;
pub mod library_check;
pub mod manual_cover;
pub mod metadata_review;
pub mod metadata_source;
pub mod reader_close;
pub mod reader_continuity;
pub mod reading_sessions;
pub mod relocation;
pub mod runtime;
pub mod search_index;
mod temporary_document;

pub enum Command {
    Query(LibraryQuery),
    Favorite { id: String, value: bool },
    Navigate { id: String, page: u32 },
}
pub enum Event {
    Library(Vec<Document>, u64),
    Saved,
}
pub struct LibraryService<R: LibraryRepository> {
    repository: R,
}
impl<R: LibraryRepository> LibraryService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    pub fn execute(&mut self, command: Command) -> Result<Event> {
        match command {
            Command::Query(query) => Ok(Event::Library(
                self.repository.query(&query)?,
                self.repository.count(&query)?,
            )),
            Command::Favorite { id, value } => {
                self.repository.set_favorite(&id, value)?;
                Ok(Event::Saved)
            }
            Command::Navigate { id, page } => {
                self.repository.save_position(&id, page)?;
                Ok(Event::Saved)
            }
        }
    }
}
