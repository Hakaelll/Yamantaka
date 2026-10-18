use crate::{Failure, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "paths")]
pub enum ImportSource {
    Files(Vec<PathBuf>),
    Folder(PathBuf),
}
impl ImportSource {
    pub fn absolute(self) -> Result<Self> {
        let absolute = |path: PathBuf| {
            std::path::absolute(path)
                .map_err(|_| Failure::Invalid("Import source path is invalid".into()))
        };
        let source = match self {
            Self::Files(paths) => {
                Self::Files(paths.into_iter().map(absolute).collect::<Result<_>>()?)
            }
            Self::Folder(path) => Self::Folder(absolute(path)?),
        };
        source.validate()?;
        Ok(source)
    }
    pub fn validate(&self) -> Result<()> {
        let paths = match self {
            Self::Files(paths) => paths.as_slice(),
            Self::Folder(path) => std::slice::from_ref(path),
        };
        let mut bytes = 0usize;
        if paths.is_empty() || paths.len() > 100000 {
            return Err(Failure::Invalid(
                "Select between one and 100000 import paths".into(),
            ));
        }
        for path in paths {
            let text = path
                .to_str()
                .ok_or_else(|| Failure::Invalid("Import paths must be Unicode".into()))?;
            bytes = bytes.checked_add(text.len()).ok_or(Failure::MemoryBudget)?;
            if !path.is_absolute() || text.chars().any(char::is_control) || bytes > 4 * 1024 * 1024
            {
                return Err(Failure::Invalid(
                    "Import paths are invalid or exceed the recovery limit".into(),
                ));
            }
        }
        Ok(())
    }
    pub fn label(&self) -> String {
        match self {
            Self::Files(paths) if paths.len() != 1 => format!("{} PDFs", paths.len()),
            Self::Files(paths) => paths
                .first()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().chars().take(128).collect())
                .unwrap_or_else(|| "PDF".into()),
            Self::Folder(path) => path
                .file_name()
                .map(|s| s.to_string_lossy().chars().take(128).collect())
                .unwrap_or_else(|| "PDFs".into()),
        }
    }
}
#[derive(Debug, Clone)]
pub struct PendingImport {
    pub id: String,
    pub label: String,
    pub completed: u64,
}
