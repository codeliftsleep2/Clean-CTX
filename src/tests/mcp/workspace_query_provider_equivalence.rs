//! Issue #148 provider-state equivalence at the shared hydration boundary.

use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::{GraphBridge, GraphNode};
use crate::layers::meta::semantic::SemanticRelation;
use crate::mcp::tool_handlers::hydration::{
    TestProjectReadiness, clear_test_project_search_results, discovery_calls,
    reset_test_scan_calls, set_test_inbound_project_search_results, set_test_project_readiness,
    set_test_project_search_results, test_scan_calls,
};
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

static TEST_SERIALIZE: &std::sync::Mutex<()> =
    &crate::mcp::tool_handlers::hydration::TEST_PROJECT_HYDRATION_SERIALIZE;

#[derive(Clone, Copy)]
enum ProviderCase {
    Unavailable,
    ReadyEmpty,
    ReadySubset,
    ReadyUseful,
}

impl ProviderCase {
    fn ready(self) -> bool {
        !matches!(self, Self::Unavailable)
    }
}

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

fn state(
    root: &Path,
    case: ProviderCase,
    declaration: &[&str],
    inbound: &[&str],
) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    if case.ready() {
        let bridge_config = crate::cbm::config::CbmConfig {
            enabled: false,
            ..Default::default()
        };
        *state.graph_bridge_lock() = Some(GraphBridge::try_create(&bridge_config, root));
        let project = cbm_project_slug(&root.canonicalize().expect("canonical root"));
        set_test_project_search_results(HashMap::from([(
            project.clone(),
            Ok(declaration.iter().copied().map(node).collect()),
        )]));
        set_test_inbound_project_search_results(HashMap::from([(
            project.clone(),
            Ok(inbound.iter().copied().map(node).collect()),
        )]));
        set_test_project_readiness(HashMap::from([(project, TestProjectReadiness::Ready)]));
    } else {
        *state.graph_bridge_lock() = None;
    }
    state
}

fn response() -> Value {
    crate::protocol::captured_responses()
        .pop()
        .expect("workspace_query response")
}

fn reverse_service(state: &crate::mcp::McpState, root: &Path) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(148),
        "workspace_query",
        &json!({ "arguments": {
            "workspaceRoot": root.to_string_lossy(),
            "type": "reverse_edges",
            "domain": "angular",
            "entity_type": "Service",
            "name": "ServiceX"
        }}),
        state,
    );
    response()
}

fn find_context(state: &crate::mcp::McpState, root: &Path) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(149),
        "workspace_query",
        &json!({ "arguments": {
            "workspaceRoot": root.to_string_lossy(),
            "type": "find_entities",
            "name": "ContextA"
        }}),
        state,
    );
    response()
}

fn write_angular_fixture(root: &Path) {
    std::fs::create_dir(root.join(".git")).expect("Git root marker");
    std::fs::write(
        root.join("service-x.ts"),
        "import { Injectable } from '@angular/core';\n@Injectable() export class ServiceX {}\n",
    )
    .expect("service fixture");
    for consumer in ["ConsumerA", "ConsumerB"] {
        std::fs::write(
            root.join(format!("{consumer}.ts")),
            format!(
                "import {{ Component }} from '@angular/core';\nimport {{ ServiceX }} from './service-x';\n@Component({{selector:'x-{consumer}'}}) export class {consumer} {{ constructor(private service: ServiceX) {{}} }}\n"
            ),
        )
        .expect("consumer fixture");
    }
}

fn angular_edges(response: &Value) -> BTreeSet<(String, String, String)> {
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"]["edges"]
        .as_array()
        .expect("edge array")
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .map(|edge| {
            let source_file = Path::new(edge["subject"]["file"].as_str().expect("source file"))
                .file_name()
                .and_then(|name| name.to_str())
                .expect("source filename");
            (
                edge["subject"]["name"].as_str().expect("subject").into(),
                source_file.into(),
                edge["object"]["name"].as_str().expect("object").into(),
            )
        })
        .collect()
}

fn assert_forward_reverse_generation(state: &crate::mcp::McpState) {
    let index = state.workspace_index_read();
    let forward: BTreeSet<_> = ["ConsumerA", "ConsumerB"]
        .into_iter()
        .flat_map(|consumer| {
            index
                .forward_edges_by_identity("angular", "Component", consumer)
                .into_iter()
                .filter(|edge| edge.relation == SemanticRelation::Injects)
                .map(|edge| (edge.subject.name.clone(), edge.object.name.clone()))
        })
        .collect();
    let reverse: BTreeSet<_> = index
        .reverse_edges_by_identity("angular", "Service", "ServiceX")
        .into_iter()
        .filter(|edge| edge.relation == SemanticRelation::Injects)
        .map(|edge| (edge.subject.name.clone(), edge.object.name.clone()))
        .collect();
    assert_eq!(
        forward,
        BTreeSet::from([
            ("ConsumerA".into(), "ServiceX".into()),
            ("ConsumerB".into(), "ServiceX".into()),
        ])
    );
    assert_eq!(reverse, forward, "forward and reverse must share one generation");
}

#[test]
fn issue_148_angular_provider_states_converge_and_completion_requires_clean_acquisition() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let _reset = ResetProvider;
    let cases = [
        (ProviderCase::Unavailable, Vec::new()),
        (ProviderCase::ReadyEmpty, Vec::new()),
        (ProviderCase::ReadySubset, vec!["ConsumerA.ts"]),
        (
            ProviderCase::ReadyUseful,
            vec!["service-x.ts", "ConsumerA.ts", "ConsumerB.ts"],
        ),
    ];
    let mut baseline = None;
    for (case, inbound) in cases {
        clear_test_project_search_results();
        reset_test_scan_calls();
        let root = tempfile::tempdir().expect("workspace");
        write_angular_fixture(root.path());
        let state = state(root.path(), case, &[], &inbound);

        let cold = reverse_service(&state, root.path());
        let warm = reverse_service(&state, root.path());
        let cold_edges = angular_edges(&cold);
        assert_eq!(cold_edges, angular_edges(&warm), "cold/warm divergence");
        assert_eq!(
            cold_edges,
            BTreeSet::from([
                ("ConsumerA".into(), "ConsumerA.ts".into(), "ServiceX".into()),
                ("ConsumerB".into(), "ConsumerB.ts".into(), "ServiceX".into()),
            ]),
            "provider state changed injector ownership: {cold}"
        );
        assert_eq!(cold["result"]["structuredContent"]["discovery"]["compiled"], 3);
        assert!(warm["result"]["structuredContent"]["discovery"]["compiled"].is_null());
        assert_eq!(
            cold["result"]["structuredContent"]["coverage"]["result_semantics"],
            "lower_bound",
            "unsupported proof families must remain honest: {cold}"
        );
        assert_eq!(
            test_scan_calls(),
            1,
            "provider success, including useful-looking candidates, cannot satisfy the root"
        );
        if case.ready() {
            assert_eq!(discovery_calls().len(), 1, "warm provider search must be reused");
            assert_eq!(
                state.hydration_discovery_lock().completed_entries(),
                2,
                "advisory provider reuse and authoritative Clean completion stay distinct"
            );
        } else {
            assert_eq!(state.hydration_discovery_lock().completed_entries(), 1);
        }
        assert_forward_reverse_generation(&state);
        if let Some(expected) = &baseline {
            assert_eq!(expected, &cold_edges, "provider-state semantic divergence");
        } else {
            baseline = Some(cold_edges);
        }
    }
}

fn write_csharp_fixture(root: &Path) {
    std::fs::create_dir(root.join(".git")).expect("Git root marker");
    std::fs::write(root.join("ContextA.cs"), "public class ContextA {}\n")
        .expect("declaration fixture");
    for consumer in ["ConsumerOne", "ConsumerTwo"] {
        std::fs::write(
            root.join(format!("{consumer}.cs")),
            format!(
                "public class {consumer} {{ public {consumer}(ContextA value) {{}} }}\n"
            ),
        )
        .expect("consumer fixture");
    }
}

fn context_occurrences(response: &Value) -> BTreeSet<(String, String, String)> {
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"]["entities"]
        .as_array()
        .expect("entity array")
        .iter()
        .map(|entity| {
            let source_file = Path::new(entity["file"].as_str().expect("source file"))
                .file_name()
                .and_then(|name| name.to_str())
                .expect("source filename");
            (
                entity["domain"].as_str().expect("domain").into(),
                entity["entity_type"].as_str().expect("entity type").into(),
                source_file.into(),
            )
        })
        .collect()
}

#[test]
fn issue_148_declaration_provider_states_cannot_shrink_occurrence_universe() {
    let _hydration_serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _response_serial = crate::protocol::handler_response_serial();
    let _reset = ResetProvider;
    let cases = [
        (ProviderCase::Unavailable, Vec::new()),
        (ProviderCase::ReadyEmpty, Vec::new()),
        (ProviderCase::ReadySubset, vec!["ContextA.cs"]),
        (
            ProviderCase::ReadyUseful,
            vec!["ContextA.cs", "ConsumerOne.cs", "ConsumerTwo.cs"],
        ),
    ];
    let mut baseline = None;
    for (case, declaration) in cases {
        clear_test_project_search_results();
        reset_test_scan_calls();
        let root = tempfile::tempdir().expect("workspace");
        write_csharp_fixture(root.path());
        let state = state(root.path(), case, &declaration, &[]);

        let cold = find_context(&state, root.path());
        let warm = find_context(&state, root.path());
        let cold_occurrences = context_occurrences(&cold);
        assert_eq!(cold_occurrences, context_occurrences(&warm));
        assert_eq!(cold["result"]["structuredContent"]["discovery"]["compiled"], 3);
        assert!(warm["result"]["structuredContent"]["discovery"]["compiled"].is_null());
        assert_eq!(
            test_scan_calls(),
            1,
            "declaration hints cannot become authoritative root completion"
        );
        if case.ready() {
            assert_eq!(discovery_calls().len(), 1, "warm provider search must be reused");
            assert_eq!(state.hydration_discovery_lock().completed_entries(), 2);
        } else {
            assert_eq!(state.hydration_discovery_lock().completed_entries(), 1);
        }
        if let Some(expected) = &baseline {
            assert_eq!(expected, &cold_occurrences, "provider-state occurrence divergence");
        } else {
            baseline = Some(cold_occurrences);
        }
    }
}
