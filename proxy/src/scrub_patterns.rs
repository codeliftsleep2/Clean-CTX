// proxy/src/scrub_patterns.rs
//
// Compiled regex patterns for secret detection. Each pattern is a OnceLock<Regex>
// static that compiles once and is reused across all requests.
//
// Adapted from ctx-wire's scrub module with additional patterns.

use regex::Regex;
use std::sync::OnceLock;

/// A compiled scrub rule: name, regex, and replacement template.
#[allow(dead_code)]
pub(crate) struct ScrubRule {
    pub name: &'static str,
    pub re: fn() -> &'static Regex,
    pub replacement: &'static str,
}

/// Lazy-initialized regex for PEM private keys.
pub(crate) fn pem_private_key_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----")
            .expect("Invalid PEM private key regex")
    })
}

/// Lazy-initialized regex for high-confidence token shapes (JWT, cloud keys, provider tokens).
pub(crate) fn tokens_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(concat!(
            r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+",  // JWT
            r"|\b(?:AKIA|ASIA)[0-9A-Z]{16}\b",                          // AWS access key
            r"|\bAIza[0-9A-Za-z_\-]{35}\b",                              // Google API key
            r"|\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,}\b",           // GitHub token
            r"|\bgithub_pat_[A-Za-z0-9_]{22,}\b",                        // GitHub fine-grained PAT
            r"|\bxox[baprs]-[A-Za-z0-9-]{10,}\b",                        // Slack token
            r"|\b(?:sk|rk)_(?:live|test)_[A-Za-z0-9]{16,}\b",           // Stripe key
            r"|\bsk-(?:ant-)?[A-Za-z0-9_\-]{20,}\b",                    // OpenAI / Anthropic key
            r"|\bhv[sbr]\.[A-Za-z0-9_-]{20,}\b",                        // HashiCorp Vault token
            r"|\bpypi-[A-Za-z0-9_-]{16,}\b",                             // PyPI API token
        ))
        .expect("Invalid tokens regex")
    })
}

/// Lazy-initialized regex for Authorization headers.
pub(crate) fn authorization_header_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(authorization\s*[:=]\s*[A-Za-z][A-Za-z0-9._-]*\s+)\S+")
            .expect("Invalid authorization header regex")
    })
}

/// Lazy-initialized regex for secret flag values (e.g., --password secret).
pub(crate) fn secret_flag_value_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?i)(--(?:password|passwd|pwd|secret|token|auth[_-]?token|access[_-]?token|api[_-]?key|access[_-]?key|secret[_-]?key|private[_-]?key|client[_-]?secret|credential|credentials)\s+)('[^']*'|"(?:[^"\\]|\\.)*"|[^\s]+)"#
        )
        .expect("Invalid secret flag value regex")
    })
}

/// Lazy-initialized regex for URL userinfo (scheme://user:password@host).
pub(crate) fn url_userinfo_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"([a-zA-Z][a-zA-Z0-9+.\-]*://[^\s:/@]+:)[^\s@/]+(@)")
            .expect("Invalid URL userinfo regex")
    })
}

/// Lazy-initialized regex for secret-ish key = value assignments.
pub(crate) fn secret_assignment_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?i)((?:password|passwd|pwd|secret|token|api[_-]?key|access[_-]?key|secret[_-]?key|private[_-]?key|auth[_-]?token|client[_-]?secret)\s*[:=]\s*)('[^']*'|"(?:[^"\\]|\\.)*"|[^\s]+)"#
        )
        .expect("Invalid secret assignment regex")
    })
}

/// Lazy-initialized regex for database URLs with credentials.
pub(crate) fn database_url_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(postgres|mysql|mongodb)://([^@\s]+)@")
            .expect("Invalid database URL regex")
    })
}

/// All token/PEM/URL literal anchors for the pre-filter.
/// These are case-sensitive substrings that every token/PEM/URL rule requires.
pub(crate) const LITERAL_ANCHORS: &[&str] = &[
    "eyJ", "AKIA", "ASIA", "AIza", "ghp_", "gho_", "ghu_", "ghs_", "ghr_",
    "github_pat_", "xox", "sk_", "rk_", "sk-", "-----BEGIN", "://",
    "hvs.", "hvb.", "hvr.", "pypi-",
];

/// Keyword roots for the pre-filter (case-insensitive).
/// These are substrings the assignment and authorization rules require.
pub(crate) const KEYWORD_ROOTS: &[&str] = &[
    "password", "passwd", "secret", "token", "api_key", "access_key",
    "private_key", "auth_token", "client_secret", "credential",
    "bearer",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pem_private_key_pattern() {
        let re = pem_private_key_re();
        assert!(re.is_match("-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----"));
        assert!(!re.is_match("-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkq...\n-----END PUBLIC KEY-----"));
    }

    #[test]
    fn test_tokens_pattern() {
        let re = tokens_re();
        // JWT
        assert!(re.is_match("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"));
        // AWS
        assert!(re.is_match("AKIAIOSFODNN7EXAMPLE"));
        assert!(re.is_match("ASIAIOSFODNN7EXAMPLE"));
        // Google
        assert!(re.is_match("AIzaSyA1234567890abcdefghijklmnopqrstuv"));
        // GitHub
        assert!(re.is_match("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef1234"));
        assert!(re.is_match("github_pat_11ABCDEF0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef1234567890abcdef12"));
        // Slack
        assert!(re.is_match("xoxb-FAKETOKEN123456789012345678901234567890"));
        // Stripe
        assert!(re.is_match("sk_live_abcdefghijklmnop"));
        assert!(re.is_match("rk_test_abcdefghijklmnop"));
        // OpenAI/Anthropic
        assert!(re.is_match("sk-ant-api03-1234567890abcdefghijklmnop"));
        assert!(re.is_match("sk-1234567890abcdefghijklmnop"));
        // Vault
        assert!(re.is_match("hvs.ABCDEFGHIJKLMNOPqrstuv"));
        // PyPI
        assert!(re.is_match("pypi-ABCDEFGHIJKLMNOPQRSTUVWXYZabcd"));
    }

    #[test]
    fn test_tokens_no_false_positive() {
        let re = tokens_re();
        assert!(!re.is_match("let api_key = 'not_a_real_key';"));
        assert!(!re.is_match("const token = getAuthToken();"));
    }

    #[test]
    fn test_authorization_header_pattern() {
        let re = authorization_header_re();
        assert!(re.is_match("Authorization: Bearer secret123"));
        assert!(re.is_match("authorization: Basic dXNlcjpwYXNz"));
        assert!(re.is_match("Authorization=Bearer eyJhbGciOiJIUzI1NiJ9"));
        // Should preserve the scheme prefix
        let result = re.replace("Authorization: Bearer secret123", "${1}[REDACTED]");
        assert_eq!(result, "Authorization: Bearer [REDACTED]");
    }

    #[test]
    fn test_secret_flag_value_pattern() {
        let re = secret_flag_value_re();
        assert!(re.is_match("--password hunter2"));
        assert!(re.is_match("--secret 'mysecret'"));
        assert!(re.is_match("--token \"abc123\""));
        assert!(re.is_match("--api-key sk-12345"));
        // Should not match inline forms
        assert!(!re.is_match("--password=hunter2"));
        // Should not match short flags
        assert!(!re.is_match("-p hunter2"));
    }

    #[test]
    fn test_url_userinfo_pattern() {
        let re = url_userinfo_re();
        assert!(re.is_match("https://user:password@example.com/path"));
        assert!(re.is_match("postgres://admin:secret@localhost:5432/db"));
        // Should preserve scheme, user, and host
        let result = re.replace("https://user:password@example.com/path", "${1}[REDACTED]${2}");
        assert_eq!(result, "https://user:[REDACTED]@example.com/path");
    }

    #[test]
    fn test_secret_assignment_pattern() {
        let re = secret_assignment_re();
        assert!(re.is_match("password = hunter2"));
        assert!(re.is_match("SECRET_KEY: 'abc123'"));
        assert!(re.is_match("api_key=\"sk-12345\""));
        assert!(re.is_match("token: eyJhbGciOiJIUzI1NiJ9"));
        // Should not match normal code
        assert!(!re.is_match("let password_length = 12;"));
    }

    #[test]
    fn test_database_url_pattern() {
        let re = database_url_re();
        assert!(re.is_match("postgres://user:pass@localhost:5432/db"));
        assert!(re.is_match("mysql://admin:secret@db.example.com/mydb"));
        assert!(re.is_match("mongodb://root:password@mongo:27017/admin"));
        // Should not match URLs without credentials
        assert!(!re.is_match("postgres://localhost:5432/db"));
    }

    #[test]
    fn test_literal_anchors() {
        // Verify all anchors are present
        for anchor in LITERAL_ANCHORS {
            assert!(!anchor.is_empty(), "Anchor should not be empty");
        }
    }

    #[test]
    fn test_keyword_roots() {
        // Verify all roots are present
        for root in KEYWORD_ROOTS {
            assert!(!root.is_empty(), "Root should not be empty");
        }
    }
}