use crate::{Failure, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataReview {
    pub document: String,
    pub revision: String,
    pub title: String,
    pub author: String,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
    pub title_override: Option<String>,
    pub author_override: Option<String>,
    pub suggested_title: Option<String>,
    pub suggested_author: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManualMetadata {
    title: String,
    author: String,
}
impl ManualMetadata {
    pub fn new(title: &str, author: &str) -> Result<Self> {
        fn clean(value: &str) -> Result<String> {
            if value.chars().count() > 512
                || value.chars().any(|c| c.is_control() && !c.is_whitespace())
            {
                return Err(Failure::Invalid(
                    "Metadata requires at most 512 characters without control codes".into(),
                ));
            }
            Ok(value.split_whitespace().collect::<Vec<_>>().join(" "))
        }
        let title = clean(title)?;
        let author = clean(author)?;
        if title.is_empty() {
            return Err(Failure::Invalid(
                "Enter a title before saving metadata".into(),
            ));
        }
        Ok(Self { title, author })
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn author(&self) -> &str {
        &self.author
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataMutation {
    Save(ManualMetadata),
    ResetOverrides,
    DismissSuggestions,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manual_values_preserve_unicode_and_user_words_but_bound_input() {
        let value = ManualMetadata::new("  道\t1984  ", " Anonymous ").unwrap();
        assert_eq!(value.title(), "道 1984");
        assert_eq!(value.author(), "Anonymous");
        assert_eq!(ManualMetadata::new("Unknown", "").unwrap().author(), "");
        for bad in ["".into(), " \t".into(), "a\0b".into(), "a".repeat(513)] {
            assert!(ManualMetadata::new(&bad, "Author").is_err());
        }
    }
}
