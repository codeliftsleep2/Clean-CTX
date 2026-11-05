//! Synthetic non-credential regression for a length-only replacement shortcut.
use serde_json::json;
const INPUT: &str = "password=abcdefghij";
const EXPECTED: &str = "password=[REDACTED]";

#[test]
fn af002_proxy_equal_length_direct_scrub() {
    assert_eq!(
        INPUT.len(),
        EXPECTED.len(),
        "exercise equal-byte-length replacement"
    );
    let actual = crate::scrub::scrub_secrets(INPUT);
    assert_eq!(
        actual.content, EXPECTED,
        "equal-length direct redaction must survive"
    );
    assert_eq!(actual.hits.len(), 1);
}

#[test]
fn af002_proxy_equal_length_request_pipeline() {
    let config = crate::pipeline::PipelineConfig {
        scrub_secrets: true,
        ..Default::default()
    };
    let pipeline = crate::pipeline::Pipeline::build(&config);
    let mut body = json!({"messages":[{"role":"user","content":[{"type":"tool_result","tool_use_id":"synthetic","content":INPUT}]}]});
    let mut stats = crate::transform::TransformStats::default();
    pipeline.run(
        &mut body,
        &mut stats,
        &config,
        &crate::platform::anthropic::AnthropicAdapter,
    );
    assert_eq!(
        body,
        json!({"messages":[{"role":"user","content":[{"type":"tool_result","tool_use_id":"synthetic","content":EXPECTED}]}]}),
        "equal-length pipeline redaction must survive"
    );
    assert_eq!(stats.secrets_scrubbed, 1);
}

#[test]
fn af002_proxy_scrub_changed_and_unchanged_bytes() {
    for input in [
        "password=a",
        "password=abcdefghij",
        "password=abcdefghijklmnopqrs",
        "password=ééééé",
    ] {
        let result = crate::scrub::scrub_secrets(input);
        assert_eq!(result.content.as_bytes(), EXPECTED.as_bytes());
        assert_eq!(result.hits.len(), 1);
        assert_eq!(
            result.hits[0].secret_type,
            crate::scrub::SecretType::SecretAssignment
        );
        assert_eq!(result.hits[0].line, 0);
        assert_eq!(result.hits[0].replacement, "${1}[REDACTED]");
    }
    for input in [
        "",
        "harmless λ text",
        "password=[REDACTED]",
        "Authorization: Bearer [REDACTED]",
    ] {
        let result = crate::scrub::scrub_secrets(input);
        assert_eq!(result.content.as_bytes(), input.as_bytes());
        assert!(
            result.hits.is_empty(),
            "unchanged matches must not become new hits"
        );
    }
}

#[test]
fn af002_proxy_scrub_cancelling_length_changes() {
    let input = "password=a\npassword=abcdefghijklmnopqrs";
    let expected = "password=[REDACTED]\npassword=[REDACTED]";
    assert_eq!(input.len(), expected.len());
    let result = crate::scrub::scrub_secrets(input);
    assert_eq!(result.content.as_bytes(), expected.as_bytes());
    assert_eq!(result.hits.len(), 2);
    let repeated = crate::scrub::scrub_secrets(&result.content);
    assert_eq!(repeated.content, expected);
    assert!(repeated.hits.is_empty());
}

#[test]
fn af002_proxy_scrub_rule_order_and_fail_closed_wrapper() {
    let input = "--password short\nAuthorization: Bearer abcdefghij\npassword=abcdefghijklmnopqrs";
    let expected = "--password [REDACTED]\nAuthorization: Bearer [REDACTED]\npassword=[REDACTED]";
    let result = crate::scrub::scrub_fail_closed(input).unwrap();
    assert_eq!(result.content.as_bytes(), expected.as_bytes());
    let kinds: Vec<_> = result
        .hits
        .iter()
        .map(|hit| hit.secret_type.clone())
        .collect();
    assert_eq!(
        kinds,
        [
            crate::scrub::SecretType::AuthorizationHeader,
            crate::scrub::SecretType::SecretFlagValue,
            crate::scrub::SecretType::SecretAssignment
        ]
    );
}

#[test]
fn af002_proxy_scrub_pipeline_shapes_and_idempotence() {
    let config = crate::pipeline::PipelineConfig {
        scrub_secrets: true,
        ..Default::default()
    };
    let pipeline = crate::pipeline::Pipeline::build(&config);
    let mut body = json!({"messages":[{"role":"user","content":[
        {"type":"text","text":INPUT},
        {"type":"tool_result","tool_use_id":"one","content":INPUT},
        {"type":"tool_result","tool_use_id":"two","content":[{"type":"text","text":INPUT},{"type":"text","text":"unchanged"}]}
    ]}]});
    let expected = json!({"messages":[{"role":"user","content":[
        {"type":"text","text":EXPECTED},
        {"type":"tool_result","tool_use_id":"one","content":EXPECTED},
        {"type":"tool_result","tool_use_id":"two","content":[{"type":"text","text":EXPECTED},{"type":"text","text":"unchanged"}]}
    ]}]});
    let mut stats = crate::transform::TransformStats::default();
    let adapter = crate::platform::anthropic::AnthropicAdapter;
    pipeline.run(&mut body, &mut stats, &config, &adapter);
    assert_eq!(body, expected);
    assert_eq!(stats.secrets_scrubbed, 3);
    let mut repeated_stats = crate::transform::TransformStats::default();
    pipeline.run(&mut body, &mut repeated_stats, &config, &adapter);
    assert_eq!(body, expected);
    assert_eq!(repeated_stats.secrets_scrubbed, 0);
}
