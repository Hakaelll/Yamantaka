use crate::Database;
use std::{fs::File, io::Write, path::Path};
use yamantaka_domain::{Cancellation, Failure, Result};

fn output_error(_: impl std::fmt::Display) -> Failure {
    Failure::Storage("Annotation export file could not be published".into())
}

impl Database {
    /// Publishes a complete export to a new file. Existing destinations are never replaced.
    pub fn create_annotation_export<T>(
        &self,
        destination: &Path,
        cancel: &Cancellation,
        write: impl FnOnce(&mut File) -> Result<T>,
    ) -> Result<T> {
        cancel.check()?;
        let extension = destination.extension().and_then(|value| value.to_str());
        if !extension.is_some_and(|value| {
            value.eq_ignore_ascii_case("md") || value.eq_ignore_ascii_case("json")
        }) {
            return Err(Failure::Invalid(
                "Annotation exports require a Markdown or JSON extension".into(),
            ));
        }
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .canonicalize()
            .map_err(output_error)?;
        let name = destination
            .file_name()
            .ok_or_else(|| Failure::Invalid("An export file name is required".into()))?;
        let destination = parent.join(name);
        let normalized = destination.to_string_lossy().to_lowercase();
        // Flutter stored ordinary paths; Rust canonicalization uses Windows verbatim prefixes.
        let legacy = if let Some(unc) = normalized.strip_prefix(r"\\?\unc\") {
            format!(r"\\{unc}")
        } else {
            normalized
                .strip_prefix(r"\\?\")
                .unwrap_or(&normalized)
                .to_owned()
        };
        let is_source: bool = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM documents WHERE normalized_path IN (?1,?2) OR lower(path) IN (?1,?2))",
                [&normalized, &legacy],
                |row| row.get(0),
            )
            .map_err(output_error)?;
        if is_source {
            return Err(Failure::Invalid(
                "An export cannot use a registered document path".into(),
            ));
        }
        match destination.symlink_metadata() {
            Ok(_) => return Err(Failure::DestinationOccupied),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(output_error(error)),
        }
        let mut temporary = tempfile::NamedTempFile::new_in(&parent).map_err(output_error)?;
        let result = write(temporary.as_file_mut())?;
        cancel.check()?;
        temporary.flush().map_err(output_error)?;
        temporary.as_file().sync_all().map_err(output_error)?;
        cancel.check()?;
        // Same-directory MoveFileExW without REPLACE_EXISTING also rejects a late collision.
        temporary.persist_noclobber(&destination).map_err(|error| {
            if error.error.kind() == std::io::ErrorKind::AlreadyExists {
                Failure::DestinationOccupied
            } else {
                output_error(error)
            }
        })?;
        Ok(result)
    }
}
