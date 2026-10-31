use super::*;
use serde::{
    Deserialize,
    de::{DeserializeSeed, Error, MapAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};
#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(super) enum Receipt {
    RemoveTag {
        document_id: String,
        tag_id: String,
        #[serde(default)]
        changed: Option<bool>,
    },
    RemoveCollection {
        document_id: String,
        collection_id: String,
        #[serde(default)]
        changed: Option<bool>,
    },
    RemoveQueue {
        document_id: String,
        workspace_id: String,
        #[serde(default)]
        changed: Option<bool>,
    },
    RestoreFavorite {
        document_id: String,
        value: bool,
        #[serde(default)]
        changed: Option<bool>,
    },
    RestoreProgress {
        document_id: String,
        page_index: u32,
        page_count: u32,
        view_mode: String,
        theme: String,
        #[serde(default)]
        changed: Option<bool>,
    },
}
impl Receipt {
    pub(super) fn document(&self) -> &str {
        match self {
            Self::RemoveTag { document_id, .. }
            | Self::RemoveCollection { document_id, .. }
            | Self::RemoveQueue { document_id, .. }
            | Self::RestoreFavorite { document_id, .. }
            | Self::RestoreProgress { document_id, .. } => document_id,
        }
    }
    fn changed(&self) -> bool {
        match self {
            Self::RemoveTag { changed, .. }
            | Self::RemoveCollection { changed, .. }
            | Self::RemoveQueue { changed, .. }
            | Self::RestoreFavorite { changed, .. }
            | Self::RestoreProgress { changed, .. } => *changed != Some(false),
        }
    }
}
struct Ordered<'a>(&'a Cancellation);
impl<'de> DeserializeSeed<'de> for Ordered<'_> {
    type Value = Vec<Receipt>;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        de: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        de.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for Ordered<'_> {
    type Value = Vec<Receipt>;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an ordered inherited receipt object")
    }
    fn visit_map<M: MapAccess<'de>>(
        self,
        mut map: M,
    ) -> std::result::Result<Self::Value, M::Error> {
        let mut seen = BTreeSet::new();
        let mut receipts = Vec::new();
        while let Some((key, value)) = map.next_entry::<String, Receipt>()? {
            if self.0.is_cancelled() {
                return Err(M::Error::custom("cancelled"));
            }
            if receipts.len() >= 800_000 || !seen.insert(key) {
                return Err(M::Error::custom(
                    "duplicate or excessive inherited receipts",
                ));
            }
            receipts.push(value);
        }
        Ok(receipts)
    }
}
pub(super) struct Group {
    pub id: String,
    pub receipts: Vec<Receipt>,
}
pub(super) fn parse(json: &str, cancel: &Cancellation) -> Result<(Vec<Group>, usize)> {
    let mut deserializer = serde_json::Deserializer::from_str(json);
    let parsed=Ordered(cancel).deserialize(&mut deserializer).map_err(|_|if cancel.is_cancelled(){Failure::Cancelled}else{invalid("Inherited receipts contain an unsupported or invalid format; original data was preserved")})?;
    deserializer
        .end()
        .map_err(|_| invalid("Inherited receipts have trailing data"))?;
    let mut groups: Vec<Group> = Vec::new();
    let mut index = BTreeMap::new();
    let mut skipped = 0;
    for receipt in parsed.into_iter().rev() {
        cancel.check()?;
        if !receipt.changed() {
            skipped += 1;
            continue;
        }
        if let Receipt::RestoreProgress {
            page_index,
            page_count,
            view_mode,
            theme,
            ..
        } = &receipt
        {
            yamantaka_domain::reader_preferences::ReaderMode::from_persisted(view_mode)?;
            yamantaka_domain::reader_preferences::ReaderTheme::from_persisted(theme)?;
            if *page_index > page_count.saturating_sub(1) {
                return Err(invalid(
                    "An inherited page index is outside its recorded document",
                ));
            }
        }
        let id = receipt.document();
        if id.is_empty() {
            return Err(invalid("An inherited receipt has no document identity"));
        }
        let group = if let Some(index) = index.get(id) {
            *index
        } else {
            if groups.len() >= 100_000 {
                return Err(invalid("Inherited recovery exceeds the document limit"));
            }
            let next = groups.len();
            index.insert(id.to_owned(), next);
            groups.push(Group {
                id: id.into(),
                receipts: Vec::with_capacity(1),
            });
            next
        };
        if groups[group].receipts.len() >= 8 {
            return Err(invalid(
                "Inherited recovery exceeds eight actions per document",
            ));
        }
        groups[group].receipts.push(receipt);
    }
    Ok((groups, skipped))
}
