#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub sort_order: i64,
    pub pinned: bool,
    pub document_count: u64,
    pub contains_document: bool,
    pub updated_at: String,
}
#[derive(Clone, Debug, Default)]
pub struct CollectionQuery {
    pub id: Option<String>,
    pub text: String,
    pub document: Option<String>,
    pub offset: usize,
}
pub struct CollectionPage {
    pub entries: Vec<Collection>,
    pub total: u64,
}
#[derive(Clone, Debug)]
pub struct CollectionRevision {
    pub id: String,
    pub updated_at: String,
}
impl From<&Collection> for CollectionRevision {
    fn from(collection: &Collection) -> Self {
        Self {
            id: collection.id.clone(),
            updated_at: collection.updated_at.clone(),
        }
    }
}
#[derive(Clone)]
pub enum CollectionDirection {
    Earlier,
    Later,
}
pub const COLLECTION_ICONS: [&str; 5] = ["folder", "study", "reference", "archive", "star"];
#[derive(Clone)]
pub enum CollectionChange {
    EnsureForDocument {
        name: String,
        icon: String,
        document: String,
    },
    Create {
        name: String,
        icon: String,
    },
    Rename {
        expected: CollectionRevision,
        name: String,
    },
    Pin {
        expected: CollectionRevision,
        pinned: bool,
    },
    Icon {
        expected: CollectionRevision,
        icon: String,
    },
    Move {
        expected: CollectionRevision,
        direction: CollectionDirection,
    },
    Delete {
        expected: CollectionRevision,
    },
    SetMembership {
        expected: CollectionRevision,
        document: String,
        included: bool,
    },
}
