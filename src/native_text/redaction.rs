use super::secret_patterns;
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;

const REDACTED: &str = "[REDACTED]";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretClass {
    PemPrivateKey,
    Token,
    AuthorizationHeader,
    SecretFlagValue,
    UrlUserInfo,
    SecretAssignment,
    DatabaseUrl,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RedactionFacts {
    pub counts: BTreeMap<SecretClass, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactedText {
    pub text: String,
    pub facts: RedactionFacts,
}

pub fn initialize_redactor() {
    secret_patterns::initialize();
}

pub fn redact_recognized_secrets(text: &str) -> RedactedText {
    initialize_redactor();
    let rules: [(SecretClass, &Regex, &str); 7] = [
        (SecretClass::PemPrivateKey, secret_patterns::pem(), REDACTED),
        (SecretClass::Token, secret_patterns::tokens(), REDACTED),
        (
            SecretClass::AuthorizationHeader,
            secret_patterns::authorization(),
            "${1}[REDACTED]",
        ),
        (
            SecretClass::SecretFlagValue,
            secret_patterns::secret_flag(),
            "${1}[REDACTED]",
        ),
        (
            SecretClass::UrlUserInfo,
            secret_patterns::url_userinfo(),
            "${1}[REDACTED]${2}",
        ),
        (
            SecretClass::SecretAssignment,
            secret_patterns::secret_assignment(),
            "${1}[REDACTED]",
        ),
        (
            SecretClass::DatabaseUrl,
            secret_patterns::database_url(),
            "${1}[REDACTED]@",
        ),
    ];
    let mut output = text.to_owned();
    let mut counts = BTreeMap::new();
    for (class, regex, replacement) in rules {
        let count = regex.find_iter(&output).count();
        if count > 0 {
            output = regex.replace_all(&output, replacement).into_owned();
            counts.insert(class, count);
        }
    }
    RedactedText {
        text: output,
        facts: RedactionFacts { counts },
    }
}

#[cfg(test)]
#[path = "../tests/native_text/redaction.rs"]
mod tests;
