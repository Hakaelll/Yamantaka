use crate::{Cancellation, Failure, Result};
use serde::{Deserialize, Serialize};

pub const BOOKMARK_PAGE_SIZE: usize = 80;

/// Schema v11 bookmark values, including the original opaque ID and timestamps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub document_id: String,
    pub page_index: u32,
    pub label: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkPage {
    pub entries: Vec<Bookmark>,
    pub total: u64,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookmarkChange {
    /// Add an unnamed bookmark without overwriting an existing page's label.
    Ensure {
        page: u32,
    },
    Save {
        page: u32,
        label: Option<String>,
    },
    Rename {
        id: String,
        label: Option<String>,
    },
    Remove {
        id: String,
    },
}

pub trait BookmarkRepository {
    fn bookmarks(
        &self,
        _document_id: &str,
        _offset: usize,
        _cancel: &Cancellation,
    ) -> Result<BookmarkPage> {
        Err(Failure::Unsupported("Reading bookmarks".into()))
    }
    fn change_bookmark(
        &mut self,
        _document_id: &str,
        _change: &BookmarkChange,
        _offset: usize,
        _cancel: &Cancellation,
    ) -> Result<BookmarkPage> {
        Err(Failure::Unsupported("Changing bookmarks".into()))
    }
}
