use crate::{Database, database::storage};
use rusqlite::OptionalExtension;
use yamantaka_domain::{
    Failure, Result,
    automation::imported::ImportedDocuments,
    memory::{Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
};
const ID: &str = "rust-import-rule-context-v1";
const LIMIT: usize = 8 * 1024 * 1024;
fn bounded_ids<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Vec<String>, D::Error> {
    struct Ids;
    impl<'de> serde::de::Visitor<'de> for Ids {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most 100000 bounded import identities")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut ids = Vec::new();
            while let Some(id) = seq.next_element::<String>()? {
                if ids.len() == 100000 || id.is_empty() || id.len() > 4096 {
                    return Err(serde::de::Error::custom(
                        "Import identity count or size exceeds its bounds",
                    ));
                }
                ids.push(id);
            }
            Ok(ids)
        }
    }
    deserializer.deserialize_seq(Ids)
}
impl Database {
    pub(crate) fn write_import_rule_context(
        &mut self,
        batch: Option<&ImportedDocuments>,
        governor: Option<&MemoryGovernor>,
    ) -> Result<()> {
        if batch.is_none() {
            self.connection.execute("DELETE FROM operation_journal WHERE id=?1 AND operation_kind='rustImportContext.v1' AND compatibility_hash='rust-import-context-v1'",[ID]).map_err(storage)?;
            return Ok(());
        }
        let batch = batch.unwrap();
        if batch.ids().any(|id| id.is_empty() || id.len() > 4096) {
            return Err(Failure::Invalid(
                "Invalid import rule context identity".into(),
            ));
        }
        let bytes = batch
            .ids()
            .try_fold(64usize, |total, id| total.checked_add(id.len() * 6 + 4))
            .ok_or(Failure::MemoryBudget)?;
        if bytes > LIMIT {
            return Err(Failure::Invalid(
                "Import rule context exceeds its snapshot limit".into(),
            ));
        }
        let governor = governor.ok_or(Failure::MemoryBudget)?;
        let _lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Other,
                MemoryRepresentation::Cpu,
                LIMIT * 3,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let snapshot = serde_json::json!({"version":1,"ids":batch.ids().collect::<Vec<_>>()});
        let text = serde_json::to_string(&snapshot).map_err(storage)?;
        let now = chrono::Utc::now().to_rfc3339();
        let changed=self.connection.execute("INSERT INTO operation_journal(id,operation_kind,status,summary_json,total_items,completed_items,started_at,updated_at,phase,resume_capability,compatibility_hash) VALUES(?1,'rustImportContext.v1','completed',?2,?3,?3,?4,?4,'completed','none','rust-import-context-v1') ON CONFLICT(id) DO UPDATE SET summary_json=excluded.summary_json,total_items=excluded.total_items,completed_items=excluded.completed_items,updated_at=excluded.updated_at WHERE operation_journal.operation_kind='rustImportContext.v1' AND operation_journal.compatibility_hash='rust-import-context-v1' AND operation_journal.status='completed'",rusqlite::params![ID,text,batch.len() as i64,now]).map_err(storage)?;
        if changed != 1 {
            return Err(Failure::Invalid(
                "Import rule context conflicts with an unsupported journal".into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn read_import_rule_context(
        &self,
        governor: &MemoryGovernor,
    ) -> Result<Option<std::sync::Arc<ImportedDocuments>>> {
        let _lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Other,
                MemoryRepresentation::Cpu,
                LIMIT * 3,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        let mut statement=self.connection.prepare("SELECT summary_json FROM operation_journal WHERE id=?1 AND operation_kind='rustImportContext.v1' AND status='completed' AND compatibility_hash='rust-import-context-v1'").map_err(storage)?;
        let snapshot: Option<String> = statement
            .query_row([ID], |row| {
                let value = row.get_ref(0)?.as_str()?;
                if value.len() > LIMIT {
                    return Err(rusqlite::Error::InvalidQuery);
                }
                Ok(value.to_owned())
            })
            .optional()
            .map_err(storage)?;
        let Some(text) = snapshot else {
            return Ok(None);
        };
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Snapshot {
            version: u32,
            #[serde(deserialize_with = "bounded_ids")]
            ids: Vec<String>,
        }
        let snapshot: Snapshot = serde_json::from_str(&text).map_err(storage)?;
        if snapshot.version != 1 || snapshot.ids.len() > 100000 {
            return Err(Failure::Invalid("Unsupported import rule context".into()));
        }
        let mut batch = ImportedDocuments::default();
        for id in snapshot.ids {
            if id.is_empty() || id.len() > 4096 || batch.contains(&id) {
                return Err(Failure::Invalid(
                    "Invalid import rule context identity".into(),
                ));
            }
            batch.insert(&id, governor)?;
        }
        Ok(Some(std::sync::Arc::new(batch)))
    }
}
