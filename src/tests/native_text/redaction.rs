use super::*;

#[test]
fn redacts_every_supported_class_deterministically() {
    let cases = [
        (
            "-----BEGIN PRIVATE KEY-----\nabc\n-----END PRIVATE KEY-----",
            SecretClass::PemPrivateKey,
        ),
        ("AKIAIOSFODNN7EXAMPLE", SecretClass::Token),
        (
            "Authorization: Bearer hunter2",
            SecretClass::AuthorizationHeader,
        ),
        ("--password hunter2", SecretClass::SecretFlagValue),
        ("https://user:hunter2@example.com", SecretClass::UrlUserInfo),
        ("password=hunter2", SecretClass::SecretAssignment),
        (
            "postgres://user:hunter2@localhost/db",
            SecretClass::DatabaseUrl,
        ),
    ];
    for (input, class) in cases {
        let first = redact_recognized_secrets(input);
        assert!(!first.text.contains("hunter2"));
        assert!(first.text.contains("[REDACTED]"));
        assert!(first.facts.counts.contains_key(&class));
        assert_eq!(redact_recognized_secrets(&first.text).text, first.text);
    }
}

#[test]
fn benign_near_matches_are_preserved_and_facts_exclude_source() {
    let input = "password_length=12 token_factory=getAuthToken";
    let result = redact_recognized_secrets(input);
    assert_eq!(result.text, input);
    assert!(result.facts.counts.is_empty());

    let secret = "Authorization: Bearer sentinel-secret";
    let result = redact_recognized_secrets(secret);
    let facts = serde_json::to_string(&result.facts).unwrap();
    assert!(!facts.contains("sentinel-secret"));
    assert!(!facts.contains("Authorization"));
}
