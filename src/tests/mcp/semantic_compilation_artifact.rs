use crate::compression::Fidelity;
use crate::mcp::semantic_compilation::artifact::SemanticCompilationArtifact;

fn compile_artifact(
    state: &crate::mcp::McpState,
    file_path: &str,
    source: &str,
) -> Result<SemanticCompilationArtifact, crate::error::CleanCtxError> {
    crate::mcp::tool_helpers::compile_source_ir_focused_with_identity(
        file_path,
        source,
        Fidelity::High,
        state,
        None,
        file_path,
        0,
        None,
    )
}

#[test]
fn semantic_compilation_helpers_preserve_artifacts_for_enabled_languages() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let cases = [
        #[cfg(feature = "csharp")]
        (
            "Artifact.cs",
            "class Artifact { public string Name { get; set; } public void Run() {} }",
        ),
        #[cfg(feature = "typescript")]
        (
            "artifact.ts",
            "class Artifact { name: string; run(value: number): void { console.log(value); } }",
        ),
        #[cfg(feature = "java")]
        (
            "Artifact.java",
            "class Artifact { String name; void run(int value) { System.out.println(value); } }",
        ),
        #[cfg(feature = "rust")]
        (
            "artifact.rs",
            "struct Artifact { name: String } impl Artifact { fn run(&self, value: i32) -> i32 { value } }",
        ),
    ];

    assert!(!cases.is_empty(), "the test requires one enabled language");
    for (file_path, source) in cases {
        let artifact = compile_artifact(&state, file_path, source).expect("artifact compilation");
        let candidate = crate::mcp::tool_helpers::compile_source_ir_candidate(
            file_path,
            source,
            Fidelity::High,
            &state,
        )
        .expect("candidate artifact compilation");

        assert_eq!(
            artifact.compiled(),
            candidate.compiled(),
            "compiled IR for {file_path}"
        );
        assert_eq!(
            serde_json::to_value(artifact.semantic_edges()).expect("artifact edge serialization"),
            serde_json::to_value(candidate.semantic_edges()).expect("candidate edge serialization"),
            "edges for {file_path}"
        );
        assert_eq!(
            artifact.semantic_dependencies(),
            candidate.semantic_dependencies(),
            "dependencies for {file_path}"
        );
        assert_eq!(
            artifact.source_hash(),
            candidate.source_hash(),
            "hash for {file_path}"
        );
        assert_eq!(
            artifact.source_snapshot(),
            source,
            "snapshot for {file_path}"
        );
        assert_eq!(
            artifact.fidelity(),
            Fidelity::High,
            "fidelity for {file_path}"
        );
        assert!(
            !artifact.canonical_owner().is_empty(),
            "canonical owner for {file_path}"
        );
        assert!(
            artifact.producer_generation_count() >= 4,
            "producer generations for {file_path}"
        );
    }
}

#[cfg(feature = "csharp")]
#[test]
fn semantic_compilation_artifact_retains_csharp_declarations_and_invocations() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let source = r#"
class Target { public void Run() {} }
class Caller { public void Invoke() { new Target().Run(); } }
"#;
    let artifact = compile_artifact(&state, "ArtifactCalls.cs", source).expect("C# artifact");

    assert!(
        artifact
            .csharp_declarations()
            .is_some_and(|declarations| declarations.len() >= 2),
        "the artifact must retain local declaration authority"
    );
    let evidence = artifact
        .invocation_evidence()
        .expect("the artifact must retain C# invocation evidence");
    let view = evidence
        .view(evidence.source_id(), source)
        .expect("evidence must remain bound to the compiled source");
    assert!(
        !view.occurrences().is_empty(),
        "the call occurrence must survive the compiler return boundary"
    );
}

#[test]
fn semantic_compilation_artifact_failure_produces_no_artifact() {
    const MARKER: &str = "issue_144_artifact_failure_marker";
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .expect("failure injection lock") = Some(MARKER.to_string());

    let result = compile_artifact(
        &state,
        "ArtifactFailure.rs",
        "fn issue_144_artifact_failure_marker() {}",
    );

    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .expect("failure injection lock") = None;
    assert!(
        result.is_err(),
        "failed compilation must return no artifact"
    );
}
