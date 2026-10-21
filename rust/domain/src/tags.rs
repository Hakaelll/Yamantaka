pub const MAX_TAG_FILTERS: usize = 80;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub document_count: u64,
    pub annotation_count: u64,
    pub contains_document: bool,
    pub updated_at: String,
}
#[derive(Clone, Debug, Default)]
pub struct TagQuery {
    /// An empty set queries all tags; a nonempty set resolves at most 80 IDs.
    pub ids: Vec<String>,
    pub text: String,
    pub document: Option<String>,
    pub offset: usize,
}
pub struct TagPage {
    pub entries: Vec<Tag>,
    pub total: u64,
}
#[derive(Clone)]
pub struct TagRevision {
    pub id: String,
    pub updated_at: String,
}
impl From<&Tag> for TagRevision {
    fn from(tag: &Tag) -> Self {
        Self {
            id: tag.id.clone(),
            updated_at: tag.updated_at.clone(),
        }
    }
}
#[derive(Clone)]
pub enum TagChange {
    Rename {
        expected: TagRevision,
        name: String,
    },
    Color {
        expected: TagRevision,
        color: String,
    },
    Merge {
        source: TagRevision,
        target: TagRevision,
    },
    Delete {
        expected: TagRevision,
    },
    Ensure {
        name: String,
    },
    EnsureForDocument {
        name: String,
        document: String,
    },
    SetMembership {
        tag: String,
        document: String,
        included: bool,
    },
}
