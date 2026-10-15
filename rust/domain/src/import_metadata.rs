//! Conservative metadata hygiene, independent of PDFium and persistence.

#[derive(Debug, PartialEq, Eq)]
pub enum MetadataValue {
    Missing,
    Accepted(String),
    Rejected,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn technical_and_placeholder_values_are_rejected_without_substring_blacklists() {
        for text in [
            "Untitled",
            "document_042",
            "Sin título",
            "Microsoft Word - Thesis.docx",
            "Acrobat Distiller",
            "https://example.invalid/book",
        ] {
            assert_eq!(
                title(Some(text), "book.pdf"),
                MetadataValue::Rejected,
                "{text}"
            );
        }
        for text in [
            "Unknown",
            "N/A",
            "Sin autor",
            "Adobe Acrobat",
            "Microsoft Word",
        ] {
            assert_eq!(author(Some(text)), MetadataValue::Rejected, "{text}");
        }
    }

    #[test]
    fn real_titles_names_and_organizations_keep_unicode_punctuation_and_short_values() {
        for text in [
            "1984",
            "道",
            "Editorial Theory",
            "Contents and Context",
            "Microsoft Word for Beginners",
            "ISBN: A History",
            "Second Edition",
        ] {
            assert_eq!(
                title(Some(text), "book.pdf"),
                MetadataValue::Accepted(text.into())
            );
        }
        for text in [
            "Anonymous",
            "李",
            "Na",
            "Instituto de Óptica",
            "Microsoft Research",
            "N. A. Smith",
            "أحمد أمين",
        ] {
            assert_eq!(author(Some(text)), MetadataValue::Accepted(text.into()));
        }
        assert_eq!(
            title(Some("  Óptica\t y\n señales  "), "book.pdf"),
            MetadataValue::Accepted("Óptica y señales".into())
        );
    }

    #[test]
    fn missing_invalid_and_generated_values_do_not_become_guesses() {
        assert_eq!(title(None, "book.pdf"), MetadataValue::Missing);
        assert_eq!(author(None), MetadataValue::Missing);
        for value in ["", "   ", "Author\0Name"] {
            assert_eq!(author(Some(value)), MetadataValue::Rejected);
        }
        assert_eq!(author(Some(&"x".repeat(513))), MetadataValue::Rejected);
        assert_eq!(
            title(Some("a1b2c3d4e5f67890"), "a1b2c3d4e5f67890.pdf"),
            MetadataValue::Rejected
        );
        assert_eq!(
            title(Some("12345678"), "12345678.pdf"),
            MetadataValue::Rejected
        );
        assert_eq!(
            title(Some("1984"), "1984.pdf"),
            MetadataValue::Accepted("1984".into())
        );
    }
}
