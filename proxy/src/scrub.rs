// proxy/src/scrub.rs
//
// Secret scrubbing engine for the Clean-CTX proxy.
// Detects and redacts secrets in tool results before they reach the LLM.
// Adapted from ctx-wire's scrub module with ScrubFailClosed semantics.
//
// The scrubbing runs unconditionally when enabled, and callers persisting
// output use ScrubFailClosed so that a redaction failure withholds the
// output rather than risking a leak.

use crate::scrub_patterns;
use regex::Regex;
use std::sync::OnceLock;
use tracing::debug;

/// Placeholder substituted for a detected secret.
const REDACTED: &str = "[REDACTED]";

/// Types of secrets that can be detected.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SecretType {
    PemPrivateKey,
    Token,
    AuthorizationHeader,
    SecretFlagValue,
    UrlUserInfo,
    SecretAssignment,
    DatabaseUrl,
    Custom(String),
}

impl std::fmt::Display for SecretType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretType::PemPrivateKey => write!(f, "pem_private_key"),
            SecretType::Token => write!(f, "token"),
            SecretType::AuthorizationHeader => write!(f, "authorization_header"),
            SecretType::SecretFlagValue => write!(f, "secret_flag_value"),
            SecretType::UrlUserInfo => write!(f, "url_userinfo"),
            SecretType::SecretAssignment => write!(f, "secret_assignment"),
            SecretType::DatabaseUrl => write!(f, "database_url"),
            SecretType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

/// A single redaction event.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScrubHit {
    pub secret_type: SecretType,
    pub line: usize,
    pub replacement: String,
}

/// Result of secret scrubbing.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScrubResult {
    pub content: String,
    pub hits: Vec<ScrubHit>,
}

/// A compiled scrub rule.
#[allow(dead_code)]
struct ScrubRule {
    secret_type: SecretType,
    re: fn() -> &'static Regex,
    replacement: &'static str,
}

/// All scrub rules, applied in order.
/// PEM (multi-line) runs first; the many single-line token shapes are merged
/// into one alternation so the common case is a single regex pass; the
/// group-capturing rules run last.
fn scrub_rules() -> &'static [ScrubRule] {
    static RULES: OnceLock<Vec<ScrubRule>> = OnceLock::new();
    RULES.get_or_init(|| {
        vec![
            ScrubRule {
                secret_type: SecretType::PemPrivateKey,
                re: scrub_patterns::pem_private_key_re,
                replacement: REDACTED,
            },
            ScrubRule {
                secret_type: SecretType::Token,
                re: scrub_patterns::tokens_re,
                replacement: REDACTED,
            },
            ScrubRule {
                secret_type: SecretType::AuthorizationHeader,
                re: scrub_patterns::authorization_header_re,
                replacement: "${1}[REDACTED]",
            },
            ScrubRule {
                secret_type: SecretType::SecretFlagValue,
                re: scrub_patterns::secret_flag_value_re,
                replacement: "${1}[REDACTED]",
            },
            ScrubRule {
                secret_type: SecretType::UrlUserInfo,
                re: scrub_patterns::url_userinfo_re,
                replacement: "${1}[REDACTED]${2}",
            },
            ScrubRule {
                secret_type: SecretType::SecretAssignment,
                re: scrub_patterns::secret_assignment_re,
                replacement: "${1}[REDACTED]",
            },
            ScrubRule {
                secret_type: SecretType::DatabaseUrl,
                re: scrub_patterns::database_url_re,
                replacement: "[REDACTED:db_credentials]@",
            },
        ]
    })
}

/// Cheap pre-filter: reports whether s contains any marker that some redaction
/// rule could match. When it returns false, the expensive regex passes are
/// skipped entirely. It is a strict superset of the rule triggers, so it never
/// causes a real secret to be skipped.
pub fn might_contain_secret(s: &str) -> bool {
    // Check literal anchors first (case-sensitive)
    for anchor in scrub_patterns::LITERAL_ANCHORS {
        if s.contains(anchor) {
            return true;
        }
    }
    // Check keyword roots (case-insensitive)
    let lower = s.to_ascii_lowercase();
    for kw in scrub_patterns::KEYWORD_ROOTS {
        if lower.contains(kw) {
            return true;
        }
    }
    false
}

/// Scrub known secret shapes from s. Returns the redacted content and a list
/// of hits. Never returns an error and is safe on empty input.
pub fn scrub_secrets(content: &str) -> ScrubResult {
    if content.is_empty() || !might_contain_secret(content) {
        return ScrubResult {
            content: content.to_string(),
            hits: Vec::new(),
        };
    }

    let mut result = content.to_string();
    let mut hits = Vec::new();

    for rule in scrub_rules() {
        let re = (rule.re)();
        let new_result = re.replace_all(&result, rule.replacement);
        // Detect changes by comparing lengths (avoids a full string comparison
        // and eliminates the separate is_match / find_iter passes).
        if new_result.len() != result.len() {
            let count = re.captures_iter(&result).count();
            for _ in 0..count {
                hits.push(ScrubHit {
                    secret_type: rule.secret_type.clone(),
                    line: 0,
                    replacement: rule.replacement.to_string(),
                });
            }
            debug!("[scrub] Applied {} rule, {} hits", rule.secret_type, count);
            result = new_result.into_owned();
        }
    }

    ScrubResult { content: result, hits }
}

/// Scrub secrets with fail-closed semantics: if scrubbing panics for any
/// reason, return an error so the caller can withhold the output rather than
/// risk leaking a secret.
#[allow(dead_code)]
pub fn scrub_fail_closed(content: &str) -> Result<ScrubResult, ScrubError> {
    std::panic::catch_unwind(|| scrub_secrets(content))
        .map_err(|_| ScrubError::Panicked)
}

/// Errors from secret scrubbing.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ScrubError {
    Panicked,
}

impl std::fmt::Display for ScrubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrubError::Panicked => write!(f, "Secret scrubbing panicked"),
        }
    }
}

impl std::error::Error for ScrubError {}

/// Scrub secrets from each argv element, returning a new slice.
#[allow(dead_code)]
pub fn scrub_args(args: &[String]) -> Vec<String> {
    args.iter().map(|a| scrub_secrets(a).content).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrub_empty() {
        let result = scrub_secrets("");
        assert_eq!(result.content, "");
        assert!(result.hits.is_empty());
    }

    #[test]
    fn test_scrub_no_secrets() {
        let input = "Hello world, this is a normal string.";
        let result = scrub_secrets(input);
        assert_eq!(result.content, input);
        assert!(result.hits.is_empty());
    }

    #[test]
    fn test_scrub_aws_key() {
        let input = "AWS access key: AKIAIOSFODNN7EXAMPLE";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_scrub_github_token() {
        let input = "Token: ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef1234";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("ghp_"));
    }

    #[test]
    fn test_scrub_jwt() {
        let input = "JWT: eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("eyJ"));
    }

    #[test]
    fn test_scrub_pem_private_key() {
        let input = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("PRIVATE KEY"));
    }

    #[test]
    fn test_scrub_authorization_header() {
        let input = "Authorization: Bearer secret123";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(result.content.contains("Authorization: Bearer"));
    }

    #[test]
    fn test_scrub_secret_assignment() {
        let input = "password = hunter2";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("hunter2"));
    }

    #[test]
    fn test_scrub_database_url() {
        let input = "postgres://user:pass@localhost:5432/db";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED"));
        assert!(!result.content.contains("pass@"));
    }

    #[test]
    fn test_scrub_url_userinfo() {
        let input = "https://user:password@example.com/path";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(result.content.contains("https://user:"));
        assert!(result.content.contains("@example.com"));
    }

    #[test]
    fn test_scrub_slack_token() {
        let input = "Slack token: xoxb-FAKE_TOKEN_123456789012345678901234567890";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("xoxb-"));
    }

    #[test]
    fn test_scrub_stripe_key() {
        let input = "Stripe key: sk_live_abcdefghijklmnop";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("sk_live_"));
    }

    #[test]
    fn test_scrub_openai_key() {
        let input = "OpenAI key: sk-ant-api03-1234567890abcdefghijklmnop";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("sk-ant-"));
    }

    #[test]
    fn test_scrub_vault_token() {
        let input = "Vault token: hvs.ABCDEFGHIJKLMNOPqrstuv";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("hvs."));
    }

    #[test]
    fn test_scrub_pypi_token() {
        let input = "PyPI token: pypi-ABCDEFGHIJKLMNOPQRSTUVWXYZabcd";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("pypi-"));
    }

    #[test]
    fn test_scrub_google_api_key() {
        let input = "Google key: AIzaSyA1234567890abcdefghijklmnopqrstuv";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("AIza"));
    }

    #[test]
    fn test_scrub_github_pat() {
        let input = "PAT: github_pat_11ABCDEF0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef1234567890abcdef12";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("github_pat_"));
    }

    #[test]
    fn test_scrub_secret_flag_value() {
        let input = "Command: --password hunter2 --token abc123";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("hunter2"));
        assert!(!result.content.contains("abc123"));
    }

    #[test]
    fn test_no_false_positive_normal_code() {
        let input = "let api_key_input = getApiKey();\nconst password_length = 12;\nlet token_count = 0;";
        let result = scrub_secrets(input);
        assert_eq!(result.content, input);
        assert!(result.hits.is_empty());
    }

    #[test]
    fn test_no_false_positive_comments() {
        let input = "// This is a comment about password handling\n// The api_key is stored in .env";
        let result = scrub_secrets(input);
        assert_eq!(result.content, input);
        assert!(result.hits.is_empty());
    }

    #[test]
    fn test_scrub_fail_closed() {
        let result = scrub_fail_closed("Hello world");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello world");
    }

    #[test]
    fn test_scrub_args() {
        let args = vec![
            "git".to_string(),
            "push".to_string(),
            "--token".to_string(),
            "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef1234".to_string(),
        ];
        let result = scrub_args(&args);
        assert_eq!(result[0], "git");
        assert_eq!(result[1], "push");
        assert_eq!(result[2], "--token");
        assert!(result[3].contains("[REDACTED]"));
        assert!(!result[3].contains("ghp_"));
    }

    #[test]
    fn test_might_contain_secret() {
        assert!(might_contain_secret("AKIAIOSFODNN7EXAMPLE"));
        assert!(might_contain_secret("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef1234"));
        assert!(might_contain_secret("eyJhbGciOiJIUzI1NiJ9"));
        assert!(might_contain_secret("password = hunter2"));
        assert!(might_contain_secret("secret_key: abc123"));
        assert!(!might_contain_secret("Hello world"));
        assert!(!might_contain_secret("let x = 5;"));
    }

    #[test]
    fn test_scrub_multiple_secrets() {
        let input = "AWS: AKIAIOSFODNN7EXAMPLE\nGitHub: ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef1234\nJWT: eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let result = scrub_secrets(input);
        assert!(result.content.contains("[REDACTED]"));
        assert!(!result.content.contains("AKIA"));
        assert!(!result.content.contains("ghp_"));
        assert!(!result.content.contains("eyJ"));
        assert!(result.hits.len() >= 3);
    }
}