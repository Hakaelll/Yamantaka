use std::fmt;
use zeroize::{Zeroize, Zeroizing};

/// An owned, transient PDF credential. Deliberately neither Clone nor serde.
/// Moving it through commands transfers ownership without duplicating plaintext.
pub struct PdfPassword(Zeroizing<String>);

impl PdfPassword {
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Used only by the transient password field; never pass to persistence.
    pub fn edit(&mut self) -> &mut String {
        &mut self.0
    }

    pub fn clear(&mut self) {
        self.0.zeroize();
    }
}

impl Default for PdfPassword {
    fn default() -> Self {
        Self(Zeroizing::new(String::with_capacity(4096)))
    }
}

impl From<String> for PdfPassword {
    fn from(value: String) -> Self {
        Self(Zeroizing::new(value))
    }
}

impl From<&str> for PdfPassword {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl fmt::Debug for PdfPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PdfPassword([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_redact_and_explicit_clear_erases_the_live_allocation() {
        let mut password = PdfPassword::from("synthetic ephemeral credential");
        assert_eq!(format!("{password:?}"), "PdfPassword([REDACTED])");
        let bytes = password.0.as_ptr();
        let len = password.0.len();
        password.clear();
        assert!(password.expose().is_empty());
        // SAFETY: Zeroize keeps the String allocation alive and overwrites its
        // previously initialized bytes. This test reads only that original range
        // before the owner is dropped; no reallocation occurs between reads.
        let erased = unsafe { std::slice::from_raw_parts(bytes, len) };
        assert!(erased.iter().all(|byte| *byte == 0));
    }
}
