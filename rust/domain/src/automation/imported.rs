use crate::{
    Failure, Result,
    memory::{ByteLease, Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct ImportedDocuments {
    ids: BTreeMap<String, ByteLease>,
}
impl ImportedDocuments {
    pub fn insert(&mut self, id: &str, governor: &MemoryGovernor) -> Result<()> {
        if self.ids.contains_key(id) {
            return Ok(());
        }
        if self.ids.len() >= super::simulation::MAX_SIMULATION_DOCUMENTS as usize {
            return Err(Failure::Invalid(
                "Import rule context exceeds its document limit".into(),
            ));
        }
        let bytes = id
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(256))
            .ok_or(Failure::MemoryBudget)?;
        let lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Other,
                MemoryRepresentation::Cpu,
                bytes,
            ))
            .map_err(|_| Failure::MemoryBudget)?;
        self.ids.insert(id.to_owned(), lease);
        Ok(())
    }
    pub fn contains(&self, id: &str) -> bool {
        self.ids.contains_key(id)
    }
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.ids.keys().map(String::as_str)
    }
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}
