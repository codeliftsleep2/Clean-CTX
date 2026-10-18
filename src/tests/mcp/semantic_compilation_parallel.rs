use crate::compression::Fidelity;
use crate::mcp::semantic_compilation::executor::{CompilationJob, SemanticCompilerExecutor};
use std::sync::Arc;

fn job(index: usize, source: &str) -> CompilationJob {
    let file_path = format!("parallel-{index}.rs");
    CompilationJob {
        canonical_owner: file_path.clone(),
        file_path: file_path.clone(),
        source: Arc::new(source.to_string()),
        fidelity: Fidelity::High,
        path_alias: file_path,
        previous_version: 0,
        skip_set: None,
    }
}

fn compile_with(worker_count: usize) -> Vec<(String, String)> {
    let executor = SemanticCompilerExecutor::new(worker_count);
    let config = crate::tests::test_config();
    executor
        .compile(
            &config,
            vec![
                job(0, "struct Zero; impl Zero { fn value(&self) -> i32 { 0 } }"),
                job(1, "struct One; impl One { fn value(&self) -> i32 { 1 } }"),
                job(2, "struct Two; impl Two { fn value(&self) -> i32 { 2 } }"),
                job(3, "struct Three; impl Three { fn value(&self) -> i32 { 3 } }"),
            ],
        )
        .into_iter()
        .map(|result| {
            let artifact = result.result.expect("parallel compilation");
            (
                result.file_path,
                format!("{:?}", artifact.compiled().instructions),
            )
        })
        .collect()
}

#[cfg(feature = "rust")]
#[test]
fn semantic_compilation_parallel_is_deterministic_for_pool_sizes_one_two_and_four() {
    let serial = compile_with(1);
    assert_eq!(compile_with(2), serial);
    assert_eq!(compile_with(4), serial);
    assert_eq!(
        serial.iter().map(|entry| entry.0.as_str()).collect::<Vec<_>>(),
        vec![
            "parallel-0.rs",
            "parallel-1.rs",
            "parallel-2.rs",
            "parallel-3.rs"
        ]
    );
}

#[cfg(feature = "rust")]
#[test]
fn semantic_compilation_parallel_reports_one_failed_job_without_losing_neighbors() {
    let _serial = crate::protocol::handler_response_serial();
    const MARKER: &str = "issue_144_parallel_failure_marker";
    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .expect("failure injection lock") = Some(MARKER.to_string());
    let executor = SemanticCompilerExecutor::new(4);
    let results = executor.compile(
        &crate::tests::test_config(),
        vec![
            job(0, "struct Before;"),
            job(1, "fn issue_144_parallel_failure_marker() {}"),
            job(2, "struct After;"),
        ],
    );
    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .expect("failure injection lock") = None;

    assert!(results[0].result.is_ok());
    assert!(results[1].result.is_err());
    assert!(results[2].result.is_ok());
    assert_eq!(
        results
            .iter()
            .map(|result| result.file_path.as_str())
            .collect::<Vec<_>>(),
        vec!["parallel-0.rs", "parallel-1.rs", "parallel-2.rs"]
    );
}

fn workspace() -> (tempfile::TempDir, crate::mcp::McpState, Vec<String>) {
    let root = tempfile::tempdir().expect("workspace");
    let paths = ["First.rs", "Second.rs"]
        .into_iter()
        .map(|name| root.path().join(name).to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    std::fs::write(&paths[0], "struct First;").expect("first source");
    std::fs::write(&paths[1], "struct Second;").expect("second source");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config), paths)
}

fn assert_first_was_not_published(state: &crate::mcp::McpState, path: &str) {
    let source = state.read_source(path).expect("first source snapshot");
    let hash = crate::cache::content_hash(source.as_bytes());
    let canonical = state.semantic_owner_path(path);
    assert!(!state.workspace_index_read().has_current_semantic_projection(
        &canonical,
        crate::workspace::index::SemanticFidelity::High,
        &hash,
    ));
}

#[test]
fn semantic_compilation_parallel_worker_failure_commits_no_partial_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let (_root, state, paths) = workspace();
    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .expect("failure injection lock") = Some("struct Second".to_string());
    let outcomes = super::publication::compile_candidates_for(
        &state,
        &paths,
        None,
        crate::mcp::tool_handlers::hydration::HydrationRequirement::LegacyEdit,
    )
    .expect("batch result");
    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .expect("failure injection lock") = None;

    assert!(outcomes.iter().all(|outcome| matches!(
        outcome,
        super::publication::CandidatePublication::Failed
    )));
    assert_first_was_not_published(&state, &paths[0]);
}

#[test]
fn semantic_compilation_parallel_superseded_job_commits_no_partial_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let (_root, state, paths) = workspace();
    super::publication::supersede_test_batch_once(
        paths[1].clone(),
        "struct NewerSecond;".to_string(),
    );
    let outcomes = super::publication::compile_candidates_for(
        &state,
        &paths,
        None,
        crate::mcp::tool_handlers::hydration::HydrationRequirement::LegacyEdit,
    )
    .expect("batch result");

    assert!(outcomes.iter().all(|outcome| matches!(
        outcome,
        super::publication::CandidatePublication::Failed
    )));
    assert_first_was_not_published(&state, &paths[0]);
}
