use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn edge(
    domain: &'static str,
    subject_type: &'static str,
    subject: &str,
    object_type: &'static str,
    object: &str,
    file: &str,
) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new(domain, subject_type, subject).with_file(file.to_string()),
        object: EntityRef::new(domain, object_type, object).with_file(file.to_string()),
        layer: "identity-resolution-test",
        call_evidence: None,
    }
}

fn dispatch(state: &crate::mcp::McpState, arguments: Value) -> Value {
    let _serial = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({ "arguments": arguments }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn state() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().expect("temporary workspace");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    (root, state)
}

fn args(root: &tempfile::TempDir, query_type: &str, name: &str) -> Value {
    json!({
        "type": query_type,
        "name": name,
        "workspaceRoot": root.path().to_string_lossy()
    })
}

#[test]
fn name_only_forward_edges_resolves_one_semantic_identity() {
    let (root, state) = state();
    let file = root
        .path()
        .join("consumer.ts")
        .to_string_lossy()
        .into_owned();
    state.workspace_index_lock().add_edges(
        &file,
        vec![edge(
            "angular",
            "Component",
            "Consumer",
            "Service",
            "Orders",
            &file,
        )],
    );

    let response = dispatch(&state, args(&root, "forward_edges", "Consumer"));

    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"]["structuredContent"]["count"], 1);
    assert_eq!(
        response["result"]["structuredContent"]["resolved_identity"],
        json!({ "domain": "angular", "entity_type": "Component", "name": "Consumer" })
    );
}

#[test]
fn repeated_occurrences_of_one_identity_are_not_ambiguous() {
    let (root, state) = state();
    for name in ["one.ts", "two.ts"] {
        let file = root.path().join(name).to_string_lossy().into_owned();
        state.workspace_index_lock().add_edges(
            &file,
            vec![edge(
                "angular",
                "Component",
                "Consumer",
                "Service",
                "Orders",
                &file,
            )],
        );
    }

    let response = dispatch(&state, args(&root, "forward_edges", "Consumer"));

    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"]["structuredContent"]["count"], 2);
}

#[test]
fn name_only_reverse_edges_resolves_the_target_identity() {
    let (root, state) = state();
    let file = root
        .path()
        .join("consumer.ts")
        .to_string_lossy()
        .into_owned();
    state.workspace_index_lock().add_edges(
        &file,
        vec![edge(
            "angular",
            "Component",
            "Consumer",
            "Service",
            "Orders",
            &file,
        )],
    );

    let response = dispatch(&state, args(&root, "reverse_edges", "Orders"));

    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"]["structuredContent"]["count"], 1);
    assert_eq!(
        response["result"]["structuredContent"]["resolved_identity"]["entity_type"],
        "Service"
    );
}

#[test]
fn ambiguous_name_returns_distinct_identity_candidates() {
    let (root, state) = state();
    let first = root
        .path()
        .join("component.ts")
        .to_string_lossy()
        .into_owned();
    let second = root
        .path()
        .join("service.ts")
        .to_string_lossy()
        .into_owned();
    {
        let mut index = state.workspace_index_lock();
        index.add_edges(
            &first,
            vec![edge(
                "angular",
                "Component",
                "Shared",
                "Service",
                "One",
                &first,
            )],
        );
        index.add_edges(
            &second,
            vec![edge(
                "angular", "Service", "Shared", "Service", "Two", &second,
            )],
        );
    }

    let response = dispatch(&state, args(&root, "forward_edges", "Shared"));

    assert_eq!(response["error"]["code"], -32602, "{response}");
    let candidates = response["error"]["data"]["candidates"]
        .as_array()
        .expect("disambiguation candidates");
    assert_eq!(candidates.len(), 2, "{response}");
}

#[test]
fn one_partial_identity_field_filters_before_disambiguation() {
    let (root, state) = state();
    let first = root
        .path()
        .join("angular.ts")
        .to_string_lossy()
        .into_owned();
    let second = root.path().join("spring.rs").to_string_lossy().into_owned();
    {
        let mut index = state.workspace_index_lock();
        index.add_edges(
            &first,
            vec![edge(
                "angular",
                "Component",
                "Shared",
                "Service",
                "One",
                &first,
            )],
        );
        index.add_edges(
            &second,
            vec![edge(
                "spring",
                "Controller",
                "Shared",
                "Service",
                "Two",
                &second,
            )],
        );
    }
    let mut arguments = args(&root, "forward_edges", "Shared");
    arguments["domain"] = json!("spring");

    let response = dispatch(&state, arguments);

    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"]["structuredContent"]["count"], 1);
    assert_eq!(
        response["result"]["structuredContent"]["resolved_identity"]["entity_type"],
        "Controller"
    );
}

#[test]
fn name_only_transitive_dependencies_uses_the_resolved_start_identity() {
    let (root, state) = state();
    let file = root.path().join("chain.ts").to_string_lossy().into_owned();
    state.workspace_index_lock().add_edges(
        &file,
        vec![
            edge("angular", "Component", "Start", "Service", "Middle", &file),
            edge("angular", "Service", "Middle", "Service", "End", &file),
        ],
    );
    let mut arguments = args(&root, "transitive_dependencies", "Start");
    arguments["depth"] = json!(0);

    let response = dispatch(&state, arguments);

    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"]["structuredContent"]["count"], 2);
}

#[test]
fn name_only_query_reports_not_found_instead_of_an_empty_graph_answer() {
    let (root, state) = state();

    let response = dispatch(&state, args(&root, "forward_edges", "Missing"));

    assert_eq!(response["error"]["code"], -32602, "{response}");
    assert_eq!(response["error"]["data"]["candidates"], json!([]));
}
