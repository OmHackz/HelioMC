//! Logging secret redaction.

/// Mask a value so it never appears in log files or output.
/// Keeps a 4-char prefix and replaces the rest with `***`.
pub fn redact(value: &str) -> String {
    if value.len() <= 4 {
        "***".to_string()
    } else {
        format!("{}***", &value[..4])
    }
}

/// Redact a value that looks like a token or secret.
/// If it is short, mask it entirely; otherwise keep the first 4 chars.
pub fn redact_secret(value: &str) -> String {
    redact(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_values_are_fully_masked() {
        assert_eq!(redact("abc"), "***");
        assert_eq!(redact_secret("xyz"), "***");
    }

    #[test]
    fn long_values_keep_prefix() {
        assert_eq!(redact("abcd1234"), "abcd***");
        assert_eq!(redact_secret("mytoken123"), "myto***");
    }
}
