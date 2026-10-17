//! Issue #144 RED regressions for provider-neutral incoming-edge acquisition.
//!
//! A Ready provider may contribute candidate hints, but search success cannot
//! prove that unreturned source owners are irrelevant. These tests intentionally
//! exercise the registered `workspace_query` production path.

use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::{GraphBridge, GraphNode};
use crate::mcp::tool_handlers::hydration::{
    TestProjectReadiness, clear_test_project_search_results, fail_test_publication_once,
    reset_test_scan_calls, set_test_inbound_project_search_results, set_test_project_readiness,
    set_test_project_search_results, test_scan_calls,
};
use crate::mcp::tools::dispatch_tools_call;
use crate::tests::assert_valid_mcp_envelope;
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

static TEST_SERIALIZE: &std::sync::Mutex<()> =
    &crate::mcp::tool_handlers::hydration::TEST_PROJECT_HYDRATION_SERIALIZE;

const TARGET: &str = "ContextA";

struct ResetProvider;

impl Drop for ResetProvider {
    fn drop(&mut self) {
        clear_test_project_search_results();
    }
}

fn node(file: impl Into<String>) -> GraphNode {
    GraphNode {
        id: "candidate".into(),
        label: "Class".into(),
        name: "candidate".into(),
        file: file.into(),
        properties: HashMap::new(),
    }
}

fn write_fixture(root: &Path) {
    std::fs::create_dir(root.join(".git")).expect("Git root marker");
    std::fs::write(
        root.join("ContextA.cs"),
        "public sealed class ContextA {}\n",
    )
    .expect("target source");
    std::fs::write(
        root.join("KnownConsumer.cs"),
        "public sealed class KnownConsumer { public KnownConsumer(ContextA value) {} }\n",
    )
    .expect("known consumer source");
    std::fs::write(
        root.join("ProviderOmittedConsumer.cs"),
        "public sealed class ProviderOmittedConsumer { public ProviderOmittedConsumer(ContextA value) {} }\n",
    )
    .expect("provider-omitted consumer source");
}

fn state_with_provider(root: &Path) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create(&bridge_config, root));
    state
}

fn configure_ready_provider(root: &Path, inbound_files: &[&str]) {
    let project = cbm_project_slug(&root.canonicalize().expect("canonical root"));
    set_test_project_search_results(HashMap::from([(project.clone(), Ok(Vec::new()))]));
    set_test_inbound_project_search_results(HashMap::from([(
        project.clone(),
        Ok(inbound_files.iter().copied().map(node).collect()),
    )]));
    set_test_project_readiness(HashMap::from([(project, TestProjectReadiness::Ready)]));
}

fn dispatch_reverse(state: &crate::mcp::McpState, root: &Path) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(144),
        "workspace_query",
        &json!({
            "arguments": {
                "workspaceRoot": root.to_string_lossy(),
                "type": "reverse_edges",
                "domain": "builtin",
                "entity_type": "TypeRef",
                "name": TARGET
            }
        }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("workspace_query response")
}

fn structured(response: &Value) -> &Value {
    let result = response["result"]
        .as_object()
        .unwrap_or_else(|| panic!("workspace_query must succeed: {response}"));
    assert_valid_mcp_envelope(result);
    &response["result"]["structuredContent"]
}

fn constructor_subjects(response: &Value) -> BTreeSet<String> {
    structured(response)["edges"]
        .as_array()
        .expect("reverse edge array")
        .iter()
        .filter(|edge| edge["relation"] == "HasConstructorParameterType")
        .map(|edge| {
            edge["subject"]["name"]
                .as_str()
                .expect("consumer name")
                .to_string()
        })
        .collect()
}

fn expected_subjects() -> BTreeSet<String> {
    ["KnownConsumer", "ProviderOmittedConsumer"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn assert_exact_coverage(response: &Value) {
    let coverage = &structured(response)["coverage"];
    assert_eq!(coverage["status"], "authoritative_complete", "{coverage}");
    assert_eq!(coverage["source_complete"], true, "{coverage}");
    assert_eq!(coverage["result_semantics"], "exact", "{coverage}");
    assert_eq!(coverage["omitted_possible"], false, "{coverage}");
}

#[test]
fn issue_144_ready_provider_without_proof_cannot_omit_a_consumer() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let _reset = ResetProvider;
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let state = state_with_provider(root.path());
    configure_ready_provider(root.path(), &["KnownConsumer.cs"]);
    reset_test_scan_calls();

    let response = dispatch_reverse(&state, root.path());

    assert_eq!(constructor_subjects(&response), expected_subjects());
    assert_eq!(
        test_scan_calls(),
        1,
        "Ready search success without a coverage proof must retain Clean acquisition"
    );
}

#[test]
fn issue_144_ready_empty_provider_result_has_no_negative_authority() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let _reset = ResetProvider;
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let state = state_with_provider(root.path());
    configure_ready_provider(root.path(), &[]);
    reset_test_scan_calls();

    let response = dispatch_reverse(&state, root.path());

    assert_eq!(constructor_subjects(&response), expected_subjects());
    assert_eq!(
        test_scan_calls(),
        1,
        "Ready Ok(empty) is not an authoritative empty source universe"
    );
}

#[test]
fn issue_144_warm_request_cannot_cache_an_unproved_provider_subset() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let _reset = ResetProvider;
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let state = state_with_provider(root.path());
    configure_ready_provider(root.path(), &["KnownConsumer.cs"]);
    reset_test_scan_calls();

    let cold = dispatch_reverse(&state, root.path());
    let warm = dispatch_reverse(&state, root.path());

    assert_eq!(constructor_subjects(&cold), expected_subjects());
    assert_eq!(constructor_subjects(&warm), expected_subjects());
    assert_eq!(
        constructor_subjects(&cold),
        constructor_subjects(&warm),
        "equivalent cold and warm requests must have identical semantic truth"
    );
    assert!(
        test_scan_calls() >= 1,
        "an unproved provider subset must not suppress all Clean acquisition"
    );
}

#[test]
fn issue_144_proof_backed_reverse_type_ref_earns_exact_coverage() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let _reset = ResetProvider;
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let state = state_with_provider(root.path());
    configure_ready_provider(root.path(), &["KnownConsumer.cs"]);

    let response = dispatch_reverse(&state, root.path());

    assert_eq!(constructor_subjects(&response), expected_subjects());
    assert_exact_coverage(&response);
}

#[test]
fn issue_144_exact_coverage_is_stable_across_cold_and_warm_requests() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let _reset = ResetProvider;
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let state = state_with_provider(root.path());
    configure_ready_provider(root.path(), &["KnownConsumer.cs"]);

    let cold = dispatch_reverse(&state, root.path());
    let warm = dispatch_reverse(&state, root.path());

    assert_exact_coverage(&cold);
    assert_exact_coverage(&warm);
    assert_eq!(structured(&cold)["edges"], structured(&warm)["edges"]);
    assert_eq!(structured(&cold)["coverage"], structured(&warm)["coverage"]);
}

#[test]
fn issue_144_complete_zero_result_is_exact() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let _reset = ResetProvider;
    let root = tempfile::tempdir().expect("workspace");
    std::fs::create_dir(root.path().join(".git")).expect("Git root marker");
    std::fs::write(
        root.path().join("ContextA.cs"),
        "public sealed class ContextA {}\n",
    )
    .expect("target source");
    let state = state_with_provider(root.path());
    configure_ready_provider(root.path(), &[]);

    let response = dispatch_reverse(&state, root.path());

    assert_eq!(structured(&response)["count"], 0);
    assert_exact_coverage(&response);
}

#[test]
fn issue_144_unproved_language_unit_remains_lower_bound() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let _reset = ResetProvider;
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    std::fs::write(
        root.path().join("Unproved.ts"),
        "export class Unproved {}\n",
    )
    .expect("unproved TypeScript source");
    let state = state_with_provider(root.path());
    configure_ready_provider(root.path(), &["KnownConsumer.cs"]);

    let response = dispatch_reverse(&state, root.path());
    let coverage = &structured(&response)["coverage"];

    assert_eq!(constructor_subjects(&response), expected_subjects());
    assert_eq!(coverage["source_complete"], false, "{coverage}");
    assert_eq!(coverage["result_semantics"], "lower_bound", "{coverage}");
    assert_eq!(coverage["omitted_possible"], true, "{coverage}");
}

#[test]
fn issue_144_publication_failure_remains_lower_bound() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let _reset = ResetProvider;
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let state = state_with_provider(root.path());
    configure_ready_provider(root.path(), &["KnownConsumer.cs"]);
    fail_test_publication_once(&root.path().join("KnownConsumer.cs").to_string_lossy());

    let response = dispatch_reverse(&state, root.path());
    let coverage = &structured(&response)["coverage"];

    assert_eq!(coverage["source_complete"], false, "{coverage}");
    assert_eq!(coverage["result_semantics"], "lower_bound", "{coverage}");
    assert_eq!(coverage["omitted_possible"], true, "{coverage}");
}

#[test]
fn issue_144_unscoped_session_view_cannot_claim_workspace_completeness() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    write_fixture(root.path());
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    crate::protocol::captured_responses().clear();

    dispatch_tools_call(
        &json!(144),
        "workspace_query",
        &json!({
            "arguments": {
                "type": "reverse_edges",
                "domain": "builtin",
                "entity_type": "TypeRef",
                "name": TARGET
            }
        }),
        &state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("workspace_query response");
    let coverage = &structured(&response)["coverage"];

    assert_eq!(coverage["source_complete"], false, "{coverage}");
    assert_eq!(coverage["result_semantics"], "lower_bound", "{coverage}");
    assert_eq!(coverage["omitted_possible"], true, "{coverage}");
}
