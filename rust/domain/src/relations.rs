use crate::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RelationKind {
    #[default]
    Related,
    Replaces,
    SourceOf,
    AttachmentOf,
}
impl RelationKind {
    pub const ALL: [Self; 4] = [
        Self::Related,
        Self::Replaces,
        Self::SourceOf,
        Self::AttachmentOf,
    ];
    pub fn stored(self) -> &'static str {
        match self {
            Self::Related => "related",
            Self::Replaces => "replaces",
            Self::SourceOf => "sourceOf",
            Self::AttachmentOf => "attachmentOf",
        }
    }
    pub fn parse(value: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.stored() == value)
            .ok_or_else(|| crate::Failure::Invalid("Unknown document relation type".into()))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relation {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: RelationKind,
    pub source_anchor_kind: Option<String>,
    pub source_anchor_id: Option<String>,
    pub target_anchor_kind: Option<String>,
    pub target_anchor_id: Option<String>,
    pub created_at: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationDestination {
    Document,
    Page(u32),
    Unavailable,
}
#[derive(Clone, Debug)]
pub struct RelationEntry {
    pub relation: Relation,
    pub other_document: String,
    pub title: String,
    pub backlink: bool,
    pub destination: RelationDestination,
}
#[derive(Clone, Debug)]
pub struct RelationQuery {
    pub document: String,
    pub offset: usize,
}
pub struct RelationPage {
    pub entries: Vec<RelationEntry>,
    pub total: u64,
}
#[derive(Clone, Debug)]
pub enum RelationChange {
    UpdateLocation {
        document: String,
        expected: Relation,
        page: Option<u32>,
    },
    Create {
        source: String,
        target: String,
        kind: RelationKind,
        source_page: Option<u32>,
        target_page: Option<u32>,
    },
    Remove {
        document: String,
        expected: Relation,
    },
}
