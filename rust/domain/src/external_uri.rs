use crate::{Failure, Result};

pub const MAX_EXTERNAL_URI_BYTES: usize = 4096;

pub struct ExternalUri(String);
impl std::fmt::Debug for ExternalUri {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ExternalUri([redacted])")
    }
}
impl ExternalUri {
    pub fn new(value: &str) -> Result<Self> {
        if value.is_empty()
            || value.len() > MAX_EXTERNAL_URI_BYTES
            || value.chars().any(char::is_control)
        {
            return Err(Failure::Invalid(
                "External link text is invalid or exceeds the size limit".into(),
            ));
        }
        let parsed = url::Url::parse(value)
            .map_err(|_| Failure::Invalid("External link is not a valid URI".into()))?;
        let allowed = match parsed.scheme() {
            "http" | "https" => {
                parsed.has_host()
                    && value.split_once(':').is_some_and(|(_, rest)| {
                        rest.strip_prefix("//").is_some_and(|authority| {
                            !authority
                                .split(['/', '?', '#'])
                                .next()
                                .unwrap_or_default()
                                .is_empty()
                        }) && !value.contains('\\')
                    })
            }
            "mailto" => true,
            _ => false,
        };
        if !allowed {
            return Err(Failure::Invalid(
                "External link scheme or host is not permitted".into(),
            ));
        }
        let normalized = parsed.to_string();
        if normalized.len() > MAX_EXTERNAL_URI_BYTES {
            return Err(Failure::Invalid(
                "External link exceeds the size limit after normalization".into(),
            ));
        }
        Ok(Self(normalized))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn retained_bytes(&self) -> usize {
        self.0.capacity()
    }
}
