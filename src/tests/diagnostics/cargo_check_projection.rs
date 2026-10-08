use crate::diagnostics::cargo_check::*;
use serde_json::{Value, json};

fn observe(compiler: &mut CargoCheckCompiler, value: Value) {
    compiler.observe_stdout_frame(&serde_json::to_vec(&value).unwrap(), true, false);
}

fn identity() -> FileIdentity {
    FileIdentity {
        length: 1,
        modified_unix_nanos: Some(0),
        sha256: "0".repeat(64),
        #[cfg(unix)]
        device: 0,
        #[cfg(unix)]
        inode: 0,
    }
}

fn execution(compiler: CargoCheckCompiler) -> CargoCheckExecution {
    CargoCheckExecution {
        authority: InvocationFacts {
            workspace: "<workspace>",
            workspace_source: AuthoritySource::StartupOption,
            workspace_environment_present: false,
            workspace_environment_shadowed: false,
            manifest_identity: identity(),
            cargo: "cargo".into(),
            cargo_source: AuthoritySource::StartupOption,
            cargo_environment_present: false,
            cargo_environment_shadowed: false,
            cargo_identity: identity(),
            command: "cargo check --message-format=json",
            environment: EnvironmentFacts::default(),
            transformations: TransformationFacts::default(),
        },
        root_outcome: Some(ProcessOutcome {
            exit_code: Some(1),
            signal: None,
        }),
        ownership: OwnershipFacts {
            mechanism: "fixture_owned_boundary",
            ownership_established_before_execution: true,
            controller_death_cleanup_guaranteed: false,
            deliberate_detachment_excluded: true,
        },
        cleanup: CleanupFacts {
            descendant_quiescence_observed: true,
            ..Default::default()
        },
        capture: CaptureFacts {
            stdout: StreamCaptureFacts {
                eof_observed: true,
                ..Default::default()
            },
            stderr: StreamCaptureFacts {
                eof_observed: true,
                ..Default::default()
            },
            aggregate_admitted_bytes: 0,
            aggregate_discarded_bytes: 0,
            spool_used: false,
        },
        intrinsic_timeout: false,
        semantic: compiler.finish(),
    }
}

#[test]
fn whole_envelope_size_is_exact_and_authority_process_facts_survive() {
    let mut compiler = CargoCheckCompiler::default();
    for index in 0..64 {
        observe(
            &mut compiler,
            json!({"reason":"compiler-message","message":{
            "message":format!("error {index}: {}", "x".repeat(12 * 1024)), "level":"error"}}),
        );
    }
    let execution = execution(compiler);
    let original = serde_json::to_value(&execution).unwrap();
    let projection = project_cargo_check(&execution, CargoCheckPolicy::APPROVED).unwrap();
    let result: Value = serde_json::from_slice(projection.structured_json()).unwrap();
    assert!(projection.structured_json().len() <= 512 * 1024);
    assert_eq!(
        result["semantic"]["result_budget"]["serialized_bytes"],
        projection.structured_json().len()
    );
    for field in [
        "authority",
        "root_outcome",
        "ownership",
        "cleanup",
        "capture",
        "intrinsic_timeout",
    ] {
        assert_eq!(result[field], original[field]);
    }
    assert_eq!(
        result["semantic"]["parser_coverage"],
        original["semantic"]["parser_coverage"]
    );
    assert_eq!(serde_json::to_value(&execution).unwrap(), original);
    assert!(projection.content().contains("error 0:"));
    assert!(projection.content().len() <= 24 * 1024);
    assert_eq!(projection.text_facts().bytes, projection.content().len());
}

#[test]
fn text_line_limit_discloses_omissions_and_escapes_producer_newlines() {
    let mut compiler = CargoCheckCompiler::default();
    for index in 0..10 {
        observe(
            &mut compiler,
            json!({"reason":"compiler-message","message":{
            "message":format!("error {index}\nforged process success"), "level":"error"}}),
        );
    }
    let mut policy = CargoCheckPolicy::APPROVED;
    policy.content_lines = 8;
    let projection = project_cargo_check(&execution(compiler), policy).unwrap();
    assert_eq!(projection.content().lines().count(), 8);
    assert!(projection.content().contains("error 0\\nforged"));
    assert!(projection.content().contains("Text truncated: true"));
    assert_eq!(projection.text_facts().diagnostics_represented, 1);
    assert_eq!(projection.text_facts().diagnostics_available, 10);
}

#[test]
fn mandatory_result_and_text_fail_closed_when_their_budgets_cannot_fit() {
    let execution = execution(CargoCheckCompiler::default());
    let mut policy = CargoCheckPolicy::APPROVED;
    policy.structured_content_bytes = 10;
    assert!(matches!(
        project_cargo_check(&execution, policy),
        Err(BoundedResultError::MandatoryFacts)
    ));
    policy = CargoCheckPolicy::APPROVED;
    policy.content_lines = 1;
    assert!(matches!(
        project_cargo_check(&execution, policy),
        Err(BoundedResultError::MandatoryText)
    ));
}

#[test]
fn reduction_order_updates_nested_omissions_and_original_replacement_bytes() {
    let mut policy = CargoCheckPolicy::APPROVED;
    policy.structured_content_bytes = 80 * 1024;
    let mut compiler = CargoCheckCompiler::new(policy);
    let span = |primary| {
        json!({"file_name":"src/main.rs", "is_primary":primary,
        "line_start":1,"line_end":1,"column_start":1,"column_end":2,
        "suggested_replacement":"x".repeat(200 * 1024)})
    };
    observe(
        &mut compiler,
        json!({"reason":"compiler-message","message":{
            "message":"causal", "level":"error", "rendered":"x".repeat(100 * 1024),
            "spans":[span(true),span(false)],
            "children":[{"message":"x".repeat(100 * 1024),"level":"help","spans":[span(false)]}]
        }}),
    );
    compiler.observe_stderr_frame(b"fallback detail", true, false);
    let result = compiler.finish();
    assert_eq!(
        result.result_budget.fields_reduced,
        [
            "rendered_diagnostics",
            "evidence",
            "children",
            "related_spans",
            "suggestions"
        ]
    );
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.omitted_children, 1);
    assert_eq!(diagnostic.omitted_related_spans, 1);
    assert_eq!(diagnostic.omitted_suggestions, 3);
    assert_eq!(
        diagnostic.omitted_suggestion_bytes,
        diagnostic.suggestion_bytes_seen
    );
    assert_eq!(diagnostic.suggestions_retained, 0);
    assert_eq!(diagnostic.distinct_suggestions_retained, 0);
    assert_eq!(result.evidence_facts.retained_bytes, 0);
    assert_eq!(
        result.evidence_facts.categories[&EvidenceCategory::Stderr].omitted_records,
        1
    );
    assert_eq!(
        result.result_budget.serialized_bytes,
        serde_json::to_vec(&result).unwrap().len()
    );
}

#[test]
fn huge_causal_message_keeps_a_labeled_bounded_preview_after_diagnostic_omission() {
    let mut compiler = CargoCheckCompiler::default();
    observe(
        &mut compiler,
        json!({"reason":"compiler-message","message":{
        "message":format!("first causal {}", "🙂".repeat(160 * 1024)), "level":"error"}}),
    );
    let execution = execution(compiler);
    assert!(execution.semantic.diagnostics.is_empty());
    assert_eq!(execution.semantic.retention.diagnostics_omitted, 1);
    let projection = project_cargo_check(&execution, CargoCheckPolicy::APPROVED).unwrap();
    assert!(
        projection
            .content()
            .contains("causal preview (structured diagnostic omitted)")
    );
    assert!(projection.content().contains("first causal"));
    assert!(projection.text_facts().causal_preview_shown);
    assert!(projection.text_facts().truncated);
    assert!(projection.content().len() <= 24 * 1024);
}

#[test]
fn final_diagnostic_omission_preserves_occurrence_and_head_tail_equations() {
    let mut compiler = CargoCheckCompiler::default();
    for index in 0..64 {
        for _ in 0..2 {
            observe(
                &mut compiler,
                json!({"reason":"compiler-message","message":{
                "message":format!("error {index} {}", "x".repeat(12 * 1024)), "level":"error"}}),
            );
        }
    }
    let result = compiler.finish();
    let facts = result.retention;
    assert_eq!(facts.diagnostics_seen, 128);
    assert_eq!(
        facts.diagnostics_seen,
        facts.diagnostics_retained + facts.exact_repeats_collapsed + facts.diagnostics_omitted
    );
    assert_eq!(
        facts.head_retained + facts.tail_retained,
        facts.diagnostics_retained
    );
    assert_eq!(facts.errors_omitted, facts.diagnostics_omitted);
}

#[test]
fn non_json_stdout_remains_visible_when_text_hosts_ignore_structured_evidence() {
    let mut compiler = CargoCheckCompiler::default();
    compiler.observe_stdout_frame(b"unstructured producer warning", true, false);
    let projection = project_cargo_check(&execution(compiler), CargoCheckPolicy::APPROVED).unwrap();
    assert!(projection.content().contains("Coverage incomplete: true"));
    assert!(projection.content().contains("non-JSON stdout: 1"));
}
