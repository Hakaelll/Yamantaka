#[derive(Clone, Debug)]
pub struct RevisionGroup {
    pub id: String,
    pub name: String,
    pub current: Option<String>,
    pub current_title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub member_count: u64,
}
#[derive(Clone, Debug)]
pub struct RevisionStamp {
    pub id: String,
    pub updated_at: String,
    pub current: Option<String>,
}
impl From<&RevisionGroup> for RevisionStamp {
    fn from(group: &RevisionGroup) -> Self {
        Self {
            id: group.id.clone(),
            updated_at: group.updated_at.clone(),
            current: group.current.clone(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct RevisionQuery {
    pub document: String,
    pub offset: usize,
}
pub struct RevisionPage {
    pub entries: Vec<RevisionGroup>,
    pub total: u64,
}
#[derive(Clone, Debug)]
pub struct RevisionMemberQuery {
    pub group: String,
    pub offset: usize,
}
#[derive(Clone, Debug)]
pub struct RevisionMember {
    pub document: String,
    pub title: String,
    pub missing: bool,
}
pub struct RevisionMemberPage {
    pub group: String,
    pub updated_at: String,
    pub current: Option<String>,
    pub entries: Vec<RevisionMember>,
    pub total: u64,
}
#[derive(Clone, Debug)]
pub enum RevisionChange {
    Create {
        name: String,
        members: Vec<String>,
        current: Option<String>,
    },
    SetCurrent {
        expected: RevisionStamp,
        document: String,
    },
}
