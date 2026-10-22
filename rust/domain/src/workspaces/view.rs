use crate::{Failure, LibraryReadingState, LibrarySort, Result, SortDirection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceGrouping {
    #[default]
    None,
    Author,
    Collection,
    Tag,
    ReadingState,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceDensity {
    #[default]
    Comfortable,
    Compact,
}
#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceView {
    pub query: Value,
    pub collection_id: Option<String>,
    pub grouping: WorkspaceGrouping,
    pub sort: LibrarySort,
    pub density: WorkspaceDensity,
    pub focus: bool,
}
impl Default for WorkspaceView {
    fn default() -> Self {
        Self {
            query: serde_json::json!({}),
            collection_id: None,
            grouping: Default::default(),
            sort: Default::default(),
            density: Default::default(),
            focus: false,
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WorkspaceQuery {
    pub text: String,
    pub favorites_only: bool,
    pub missing_only: bool,
    pub modified_only: bool,
    pub reading_state: LibraryReadingState,
    pub sort_direction: SortDirection,
    pub tag_ids: Vec<String>,
    pub list: bool,
    pub inspector: bool,
    pub smart_view: Option<String>,
    #[serde(rename = "yamantakaRustComparison")]
    pub comparison: Option<super::WorkspaceComparison>,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, Value>,
}
impl WorkspaceView {
    pub fn parsed_query(&self) -> Result<WorkspaceQuery> {
        let query: WorkspaceQuery = serde_json::from_value(self.query.clone())
            .map_err(|error| Failure::Invalid(format!("Invalid saved workspace query: {error}")))?;
        if query.text.len() > 4096
            || query.tag_ids.len() > 80
            || query
                .tag_ids
                .iter()
                .any(|id| id.is_empty() || id.len() > 4096)
        {
            return Err(Failure::Invalid(
                "Saved workspace query exceeds supported limits; stored data is preserved".into(),
            ));
        }
        if query
            .smart_view
            .as_deref()
            .is_some_and(|key| crate::LibrarySmartView::from_key(key).is_none())
        {
            return Err(Failure::Invalid(
                "Unknown saved workspace library view; stored data is preserved".into(),
            ));
        }
        if let Some(comparison) = &query.comparison {
            comparison.validate()?;
        }
        Ok(query)
    }
    pub fn validate(&self) -> Result<()> {
        self.parsed_query()?;
        if self
            .collection_id
            .as_ref()
            .is_some_and(|id| id.is_empty() || id.len() > 4096)
            || serde_json::to_vec(&self.query)
                .map_err(|error| Failure::Invalid(error.to_string()))?
                .len()
                > 4 * 1024 * 1024
        {
            return Err(Failure::Invalid(
                "Saved workspace view exceeds supported limits".into(),
            ));
        }
        Ok(())
    }
}
