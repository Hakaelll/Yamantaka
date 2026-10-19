#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub document_count: u64,
    pub annotation_count: u64,
    pub contains_document: bool,
}
#[derive(Clone, Debug, Default)]
pub struct TagQuery {
    pub id: Option<String>,
    pub text: String,
    pub document: Option<String>,
    pub offset: usize,
}
pub struct TagPage {
    pub entries: Vec<Tag>,
    pub total: u64,
}
#[derive(Clone)]
pub enum TagChange {
    Ensure {
        name: String,
    },
    SetMembership {
        tag: String,
        document: String,
        included: bool,
    },
}
