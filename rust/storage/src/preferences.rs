use crate::database::storage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, fs, io::Write, path::Path};
use yamantaka_domain::{Failure, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub version: u32,
    /// Preserve every legacy setting, including unknown future keys.
    pub legacy: BTreeMap<String, Value>,
    #[serde(default)]
    pub rust: BTreeMap<String, Value>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            version: 1,
            legacy: BTreeMap::new(),
            rust: BTreeMap::new(),
        }
    }
}
impl Preferences {
    pub fn record_reading_sessions(&self) -> bool {
        match self.get("recordReadingSessions") {
            None => true,
            Some(value) => value.as_bool().unwrap_or(false),
        }
    }
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let result: Self =
            serde_json::from_slice(&fs::read(path).map_err(storage)?).map_err(storage)?;
        if result.version != 1 {
            return Err(Failure::Invalid("Unsupported preferences version".into()));
        }
        Ok(result)
    }
    pub fn from_flutter(path: &Path) -> Result<Self> {
        let raw: BTreeMap<String, Value> =
            serde_json::from_slice(&fs::read(path).map_err(storage)?).map_err(storage)?;
        let mut result = Self::default();
        for (key, value) in raw {
            let key = key.strip_prefix("flutter.").unwrap_or(&key).to_owned();
            if result.legacy.insert(key, value).is_some() {
                return Err(Failure::Invalid(
                    "Conflicting Flutter preference keys".into(),
                ));
            }
        }
        Ok(result)
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        atomic_json(path, self)
    }
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.rust.get(key).or_else(|| self.legacy.get(key))
    }
}
pub(crate) fn atomic_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| Failure::Invalid("A parent directory is required".into()))?;
    fs::create_dir_all(parent).map_err(storage)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(storage)?;
    serde_json::to_writer_pretty(&mut temporary, value).map_err(storage)?;
    temporary.write_all(b"\n").map_err(storage)?;
    temporary.as_file().sync_all().map_err(storage)?;
    // NamedTempFile uses atomic replacement (MoveFileExW on Windows), no delete gap.
    temporary.persist(path).map_err(storage)?;
    Ok(())
}
