use std::path::PathBuf;
use yamantaka_domain::{Failure, Result};

/// Selecting a path never writes a file. The application publishes successful output later.
pub fn pick_annotation_export(title: &str, extension: &str) -> Result<Option<PathBuf>> {
    let label = match extension {
        "md" => "Markdown",
        "json" => "JSON",
        _ => {
            return Err(Failure::Invalid(
                "Unsupported annotation export format".into(),
            ));
        }
    };
    Ok(rfd::FileDialog::new()
        .set_title(title)
        .set_file_name(format!("Yamantaka notes.{extension}"))
        .add_filter(label, &[extension])
        .save_file())
}
