//! Existing rule triggers must survive the fast prefilter; synthetic values only.
#[test]
fn af002_prefilter_existing_rule_triggers() {
    let mut omitted = Vec::new();
    let keys = [
        "password",
        "passwd",
        "pwd",
        "secret",
        "token",
        "apikey",
        "api_key",
        "api-key",
        "accesskey",
        "access_key",
        "access-key",
        "secretkey",
        "secret_key",
        "secret-key",
        "privatekey",
        "private_key",
        "private-key",
        "authtoken",
        "auth_token",
        "auth-token",
        "clientsecret",
        "client_secret",
        "client-secret",
    ];
    for key in keys {
        for uppercase in [false, true] {
            let key = if uppercase {
                key.to_ascii_uppercase()
            } else {
                key.to_owned()
            };
            for (label, input, rule) in [
                (
                    "assignment",
                    format!("{key}=abcdefghij"),
                    crate::scrub_patterns::secret_assignment_re(),
                ),
                (
                    "flag",
                    format!("--{key} abcdefghij"),
                    crate::scrub_patterns::secret_flag_value_re(),
                ),
            ] {
                assert!(
                    rule.is_match(&input),
                    "fixture must match an existing {label} rule: {key}"
                );
                let admitted = crate::scrub::might_contain_secret(&input);
                let scrubbed = crate::scrub::scrub_secrets(&input);
                eprintln!(
                    "AF002_PREFILTER family={label} keyword={key} admitted={admitted} hits={}",
                    scrubbed.hits.len()
                );
                if !admitted {
                    omitted.push(format!("{label}:{key}"));
                }
            }
        }
    }
    for scheme in ["Bearer", "Basic", "Digest", "Custom"] {
        let input = format!("Authorization: {scheme} abcdefghij");
        assert!(crate::scrub_patterns::authorization_header_re().is_match(&input));
        let admitted = crate::scrub::might_contain_secret(&input);
        eprintln!("AF002_PREFILTER family=authorization scheme={scheme} admitted={admitted}");
        if !admitted {
            omitted.push(format!("authorization:{scheme}"));
        }
    }
    assert!(
        omitted.is_empty(),
        "Prefilter excludes existing rule matches: {omitted:?}"
    );
}

#[test]
fn af002_prefilter_pipeline_existing_assignment() {
    use serde_json::json;
    let config = crate::pipeline::PipelineConfig {
        scrub_secrets: true,
        ..Default::default()
    };
    let mut body = json!({"messages":[{"role":"user","content":[{"type":"tool_result","content":"apikey=abcdefghij"}]}]});
    let expected = json!({"messages":[{"role":"user","content":[{"type":"tool_result","content":"apikey=[REDACTED]"}]}]});
    let mut stats = crate::transform::TransformStats::default();
    crate::pipeline::Pipeline::build(&config).run(
        &mut body,
        &mut stats,
        &config,
        &crate::platform::anthropic::AnthropicAdapter,
    );
    assert_eq!(
        body, expected,
        "Prefilter pipeline must retain existing rule redaction"
    );
    assert_eq!(stats.secrets_scrubbed, 1);
}

fn assert_scrub_and_pipeline(input: &str, expected: &str, kind: crate::scrub::SecretType) {
    use serde_json::json;
    let standalone = crate::scrub::scrub_secrets(input);
    assert_eq!(standalone.content.as_bytes(), expected.as_bytes());
    assert_eq!(standalone.hits.len(), 1);
    assert_eq!(standalone.hits[0].secret_type, kind);
    assert_eq!(standalone.hits[0].line, 0);
    assert_eq!(standalone.hits[0].replacement, "${1}[REDACTED]");
    let config = crate::pipeline::PipelineConfig {
        scrub_secrets: true,
        ..Default::default()
    };
    let mut body = json!({"messages":[{"role":"user","content":[{"type":"tool_result","tool_use_id":"fixture","content":input}]}]});
    let mut stats = crate::transform::TransformStats::default();
    crate::pipeline::Pipeline::build(&config).run(
        &mut body,
        &mut stats,
        &config,
        &crate::platform::anthropic::AnthropicAdapter,
    );
    assert_eq!(
        body,
        json!({"messages":[{"role":"user","content":[{"type":"tool_result","tool_use_id":"fixture","content":expected}]}]})
    );
    assert_eq!(stats.secrets_scrubbed, 1);
    let repeated = crate::scrub::scrub_secrets(&standalone.content);
    assert_eq!(repeated.content, expected);
    assert!(repeated.hits.is_empty());
}

#[test]
fn af002_prefilter_exact_96_case_outputs() {
    let keys = [
        "password",
        "passwd",
        "pwd",
        "secret",
        "token",
        "apikey",
        "api_key",
        "api-key",
        "accesskey",
        "access_key",
        "access-key",
        "secretkey",
        "secret_key",
        "secret-key",
        "privatekey",
        "private_key",
        "private-key",
        "authtoken",
        "auth_token",
        "auth-token",
        "clientsecret",
        "client_secret",
        "client-secret",
    ];
    let mut cases = 0;
    for key in keys {
        for uppercase in [false, true] {
            let key = if uppercase {
                key.to_ascii_uppercase()
            } else {
                key.to_owned()
            };
            assert_scrub_and_pipeline(
                &format!("{key}=abcdefghij"),
                &format!("{key}=[REDACTED]"),
                crate::scrub::SecretType::SecretAssignment,
            );
            assert_scrub_and_pipeline(
                &format!("--{key} abcdefghij"),
                &format!("--{key} [REDACTED]"),
                crate::scrub::SecretType::SecretFlagValue,
            );
            cases += 2;
        }
    }
    for scheme in ["Bearer", "Basic", "Digest", "Custom"] {
        assert_scrub_and_pipeline(
            &format!("Authorization: {scheme} abcdefghij"),
            &format!("Authorization: {scheme} [REDACTED]"),
            crate::scrub::SecretType::AuthorizationHeader,
        );
        cases += 1;
    }
    assert_eq!(cases, 96);
    eprintln!("AF002_PREFILTER_EXACT cases={cases} standalone_and_pipeline=true exact_bytes_and_hits=true");
}

#[test]
fn af002_prefilter_false_positives_preserve_bytes() {
    use serde_json::json;
    for root in [
        "pwd",
        "apikey",
        "api-key",
        "accesskey",
        "access-key",
        "privatekey",
        "private-key",
        "authorization",
    ] {
        for keyword in [root.to_owned(), root.to_ascii_uppercase()] {
            let input = format!("Documentation about {keyword} usage.");
            assert!(crate::scrub::might_contain_secret(&input));
            let result = crate::scrub::scrub_secrets(&input);
            assert_eq!(result.content.as_bytes(), input.as_bytes());
            assert!(result.hits.is_empty());
            let config = crate::pipeline::PipelineConfig {
                scrub_secrets: true,
                ..Default::default()
            };
            let mut body =
                json!({"messages":[{"role":"user","content":[{"type":"text","text":input}]}]});
            let original = body.clone();
            let mut stats = crate::transform::TransformStats::default();
            crate::pipeline::Pipeline::build(&config).run(
                &mut body,
                &mut stats,
                &config,
                &crate::platform::anthropic::AnthropicAdapter,
            );
            assert_eq!(body, original);
            assert_eq!(stats.secrets_scrubbed, 0);
        }
    }
}
