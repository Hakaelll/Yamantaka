//! Global search contracts. Index access is opt-in and cancellation never clears progress.
use crate::{Cancellation, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SearchKind {
    Metadata,
    PageText,
    Annotation,
    Bookmark,
    Collection,
    Tag,
    Relation,
}
impl SearchKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::PageText => "pageText",
            Self::Annotation => "annotation",
            Self::Bookmark => "bookmark",
            Self::Collection => "collection",
            Self::Tag => "tag",
            Self::Relation => "relation",
        }
    }
    pub fn from_name(name: &str) -> Self {
        match name {
            "pageText" => Self::PageText,
            "annotation" => Self::Annotation,
            "bookmark" => Self::Bookmark,
            "collection" => Self::Collection,
            "tag" => Self::Tag,
            "relation" => Self::Relation,
            _ => Self::Metadata,
        }
    }
}
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub kinds: Vec<SearchKind>,
    pub document_id: Option<String>,
    pub missing: Option<bool>,
    pub stale: Option<bool>,
    pub limit: usize,
    pub offset: usize,
}
impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            kinds: Vec::new(),
            document_id: None,
            missing: None,
            stale: None,
            limit: 80,
            offset: 0,
        }
    }
}
/// Treat all user tokens as quoted prefixes, never as an FTS expression.
pub fn fts_expression(text: &str) -> String {
    text.split_whitespace()
        .map(|s| s.replace('"', ""))
        .filter(|s| !s.is_empty())
        .take(8)
        .map(|s| format!("\"{s}\"*"))
        .collect::<Vec<_>>()
        .join(" AND ")
}
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub document_id: String,
    pub page: Option<u32>,
    pub kind: SearchKind,
    pub title: String,
    pub snippet: String,
    pub rank: f64,
    pub missing: bool,
    pub stale: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IndexAccess {
    #[default]
    Disabled,
    Enabled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexStatus {
    Pending,
    Indexing,
    Indexed,
    NotIndexable,
    Excluded,
    Failed,
}
impl IndexStatus {
    pub fn from_name(name: &str) -> Self {
        match name {
            "indexing" => Self::Indexing,
            "indexed" => Self::Indexed,
            "notIndexable" => Self::NotIndexable,
            "excluded" => Self::Excluded,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexState {
    pub document_id: String,
    pub fingerprint: String,
    pub status: IndexStatus,
    pub indexed_pages: u32,
    pub updated_at: String,
    pub error_code: Option<String>,
}
/// Storage-issued revision rejects late writes from an earlier job. Commit pages
/// in order; empty pages also advance the durable cursor. Replacement of an
/// already committed page is allowed without advancing the cursor again.
#[derive(Debug, Clone)]
pub struct IndexSession {
    pub document_id: String,
    pub fingerprint: String,
    pub page_count: u32,
    pub cursor: u32,
    pub revision: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexStats {
    pub indexed: u64,
    pub indexing: u64,
    pub not_indexable: u64,
    pub excluded: u64,
    pub failed: u64,
    pub rows: u64,
}
pub trait SearchRepository {
    fn index_candidates(
        &self,
        after_id: Option<&str>,
        limit: usize,
        access: IndexAccess,
        cancel: &Cancellation,
    ) -> Result<Vec<String>>;
    fn needs_index(&self, document_id: &str) -> Result<bool>;
    fn search(&self, query: &SearchQuery, cancel: &Cancellation) -> Result<Vec<SearchResult>>;
    fn begin_index(
        &mut self,
        document_id: &str,
        resume: bool,
        access: IndexAccess,
        cancel: &Cancellation,
    ) -> Result<IndexSession>;
    fn index_page(
        &mut self,
        session: &mut IndexSession,
        page: u32,
        text: &str,
        access: IndexAccess,
        cancel: &Cancellation,
    ) -> Result<()>;
    fn finish_index(
        &mut self,
        session: &mut IndexSession,
        access: IndexAccess,
        cancel: &Cancellation,
    ) -> Result<()>;
    fn fail_index(
        &mut self,
        session: &mut IndexSession,
        code: &str,
        cancel: &Cancellation,
    ) -> Result<()>;
    fn index_state(&self, document_id: &str) -> Result<Option<IndexState>>;
    fn index_stats(&self) -> Result<IndexStats>;
    fn exclude_from_index(
        &mut self,
        document_id: &str,
        excluded: bool,
        cancel: &Cancellation,
    ) -> Result<()>;
    fn refresh_search_entries(&mut self, access: IndexAccess, cancel: &Cancellation) -> Result<()>;
    fn clear_index(&mut self, cancel: &Cancellation) -> Result<()>;
}
