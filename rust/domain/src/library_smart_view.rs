#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibrarySmartView {
    InProgress,
    NotStarted,
    Finished,
    Annotated,
    Bookmarked,
    Missing,
    RecentlyOpened,
    RecentlyAdded,
}
impl LibrarySmartView {
    pub const ALL: [Self; 8] = [
        Self::InProgress,
        Self::NotStarted,
        Self::Finished,
        Self::Annotated,
        Self::Bookmarked,
        Self::Missing,
        Self::RecentlyOpened,
        Self::RecentlyAdded,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::InProgress => "inProgress",
            Self::NotStarted => "notStarted",
            Self::Finished => "finished",
            Self::Annotated => "annotated",
            Self::Bookmarked => "bookmarked",
            Self::Missing => "missing",
            Self::RecentlyOpened => "recentlyOpened",
            Self::RecentlyAdded => "recentlyAdded",
        }
    }
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|view| view.key() == key)
    }
}
