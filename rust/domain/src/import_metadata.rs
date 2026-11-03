//! Conservative metadata hygiene, independent of PDFium and persistence.

#[derive(Debug, PartialEq, Eq)]
pub enum MetadataValue {
    Missing,
    Accepted(String),
    Rejected,
}

/// Filename metadata remains usable as a fallback, but is not strong title evidence.
pub fn is_filename_title(value: &str, file_name: &str) -> bool {
    let value = key(value);
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem);
    !value.is_empty() && (value == key(file_name) || value == key(stem))
}

pub fn title(value: Option<&str>, file_name: &str) -> MetadataValue {
    let cleaned = clean(value);
    let MetadataValue::Accepted(text) = &cleaned else {
        return cleaned;
    };
    let lower = text.to_lowercase();
    let key = key(text);
    let base = key.trim_end_matches(|character: char| character.is_ascii_digit());
    if matches!(
        base,
        "untitled"
            | "untitleddocument"
            | "sintitulo"
            | "sintítulo"
            | "document"
            | "documento"
            | "scan"
            | "scannedimage"
            | "archivo"
            | "file"
    ) || (lower.starts_with("microsoft word - ")
        && (lower.ends_with(".doc") || lower.ends_with(".docx")))
        || matches!(
            key.as_str(),
            "microsoftworddocument" | "acrobatdistiller" | "pdfcreator"
        )
    {
        return MetadataValue::Rejected;
    }
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem);
    if text.eq_ignore_ascii_case(stem)
        && ((text.len() >= 16 && text.bytes().all(|byte| byte.is_ascii_hexdigit()))
            || (text.len() >= 8 && text.bytes().all(|byte| byte.is_ascii_digit())))
    {
        return MetadataValue::Rejected;
    }
    cleaned
}

pub fn author(value: Option<&str>) -> MetadataValue {
    let cleaned = clean(value);
    let MetadataValue::Accepted(text) = &cleaned else {
        return cleaned;
    };
    if text.eq_ignore_ascii_case("n/a")
        || matches!(
            key(text).as_str(),
            "unknown"
                | "notspecified"
                | "unspecified"
                | "none"
                | "sinautor"
                | "desconocido"
                | "pdfcreator"
                | "acrobatdistiller"
                | "adobeacrobat"
                | "microsoftword"
        )
    {
        return MetadataValue::Rejected;
    }
    cleaned
}

/// A stored filename generated for rejected metadata is not an accepted book title.
pub fn catalogue_title(
    value: Option<&str>,
    source: Option<&str>,
    file_name: &str,
) -> MetadataValue {
    let original = title(source, file_name);
    if value == Some(file_name) && !matches!(original, MetadataValue::Accepted(_)) {
        original
    } else {
        title(value, file_name)
    }
}

fn clean(value: Option<&str>) -> MetadataValue {
    let Some(value) = value else {
        return MetadataValue::Missing;
    };
    if value.chars().take(513).count() > 512
        || value
            .chars()
            .any(|character| character.is_control() && !character.is_whitespace())
    {
        return MetadataValue::Rejected;
    }
    let text = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let lower = text.to_lowercase();
    if text.is_empty() || lower.starts_with("http://") || lower.starts_with("https://") {
        return MetadataValue::Rejected;
    }
    MetadataValue::Accepted(text)
}

fn key(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric())
        .collect()
}
