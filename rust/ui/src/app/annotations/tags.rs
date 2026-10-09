use super::*;

impl AnnotationEditor {
    pub(super) fn tag_changes(&self) -> yamantaka_domain::Result<Option<Vec<String>>> {
        if let Some(record) = self.stored()
            && self.tags == record.tags.join(", ")
        {
            return Ok(None);
        }
        if self.tags.encode_utf16().count() > 160 {
            return Err(Failure::Invalid(
                "Use at most 160 UTF-16 units for tags".into(),
            ));
        }
        let mut tags: Vec<String> = Vec::new();
        for tag in self
            .tags
            .split(',')
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
        {
            let normalized = tag.to_lowercase();
            if tags.iter().any(|value| value.to_lowercase() == normalized) {
                continue;
            }
            if tags.len() == 12 {
                return Err(Failure::Invalid("Use at most twelve tags".into()));
            }
            tags.push(tag.into());
        }
        Ok(Some(tags))
    }
}
