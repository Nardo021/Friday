use regex::Regex;
use std::sync::LazyLock;

static PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (Regex::new(r"(?i)(OPENAI_API_KEY\s*=\s*)\S+").unwrap(), "$1[REDACTED]"),
        (Regex::new(r"(?i)(ANTHROPIC_API_KEY\s*=\s*)\S+").unwrap(), "$1[REDACTED]"),
        (Regex::new(r"(?i)(CURSOR_TOKEN\s*=\s*)\S+").unwrap(), "$1[REDACTED]"),
        (Regex::new(r"(?i)(GITHUB_TOKEN\s*=\s*)\S+").unwrap(), "$1[REDACTED]"),
        (Regex::new(r"(?i)(DATABASE_URL\s*=\s*)\S+").unwrap(), "$1[REDACTED]"),
        (Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(), "sk-[REDACTED]"),
        (Regex::new(r"ghp_[a-zA-Z0-9]{20,}").unwrap(), "ghp_[REDACTED]"),
    ]
});

pub struct SecretRedactor;

impl SecretRedactor {
    pub fn redact(input: &str) -> String {
        let mut result = input.to_string();
        for (pattern, replacement) in PATTERNS.iter() {
            result = pattern.replace_all(&result, *replacement).to_string();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_openai_key() {
        let out = SecretRedactor::redact("OPENAI_API_KEY=sk-abc123secretkey");
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains("abc123secretkey"));
    }
}
