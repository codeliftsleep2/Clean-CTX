use crate::dictionary::path::canonical_identity_key;
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn dispatch(state: &crate::mcp::McpState, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(151),
        "workspace_query",
        &json!({ "arguments": arguments }),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("workspace_query response");
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"].clone()
}

fn declaration_files(state: &crate::mcp::McpState, name: &str) -> BTreeSet<String> {
    state
        .workspace_index_read()
        .declarations_by_identity("builtin", "Method", name)
        .into_iter()
        .filter_map(|entity| entity.file.clone())
        .collect()
}

#[test]
fn within_path_projects_answers_without_narrowing_authorized_acquisition() {
    let primary = tempfile::TempDir::new().expect("primary root");
    let additional = tempfile::TempDir::new().expect("additional root");
    let primary_file = primary.path().join("shared.rs");
    let additional_file = additional.path().join("shared.rs");
    let source = "pub struct Probe; impl Probe { pub fn issue_151_target(&self) {} }\n";
    std::fs::write(&primary_file, source).expect("primary fixture");
    std::fs::write(&additional_file, source).expect("additional fixture");

    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = vec![additional.path().to_string_lossy().into_owned()];
    let state = crate::mcp::McpState::new(config);
    let result = dispatch(
        &state,
        json!({
            "type": "find_entities",
            "name": "issue_151_target",
            "workspaceRoot": primary.path(),
            "withinPath": primary.path(),
            "declarations_only": true,
        }),
    );

    let returned_files: BTreeSet<_> = result["entities"]
        .as_array()
        .expect("entities")
        .iter()
        .filter_map(|entity| entity["file"].as_str().map(str::to_owned))
        .collect();
    assert_eq!(
        returned_files,
        BTreeSet::from([canonical_identity_key(&primary_file.to_string_lossy())]),
        "withinPath projects the answer by occurrence provenance"
    );
    assert_eq!(
        declaration_files(&state, "issue_151_target"),
        BTreeSet::from([
            canonical_identity_key(&primary_file.to_string_lossy()),
            canonical_identity_key(&additional_file.to_string_lossy()),
        ]),
        "acquisition still covers every authorized root before answer projection"
    );
}

#[test]
fn find_entities_declarations_require_proof_after_ordinary_discovery_and_on_warm_reuse() {
    let root = tempfile::TempDir::new().expect("workspace root");
    let file = root.path().join("entity.rs");
    std::fs::write(
        &file,
        "pub struct Probe; impl Probe { pub fn issue_151_entity(&self) {} }\n",
    )
    .expect("fixture");
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let mut declaration_coverage: Option<Value> = None;

    for (declarations_only, fact_semantics) in [
        (false, "authoritative_indexed_occurrences"),
        (true, "source_proven_declarations"),
        (true, "source_proven_declarations"),
    ] {
        let result = dispatch(
            &state,
            json!({
                "type": "find_entities",
                "name": "issue_151_entity",
                "workspaceRoot": root.path(),
                "declarations_only": declarations_only,
            }),
        );
        assert_eq!(result["coverage"]["fact_semantics"], fact_semantics);
        assert_eq!(result["coverage"]["source_complete"], declarations_only);
        assert_eq!(
            result["coverage"]["result_semantics"],
            if declarations_only {
                "exact"
            } else {
                "lower_bound"
            }
        );
        assert_eq!(result["coverage"]["omitted_possible"], !declarations_only);
        if declarations_only {
            assert_eq!(result["coverage"]["status"], "authoritative_complete");
            assert!(result["count"].as_u64().is_some_and(|count| count > 0));
            assert!(result["coverage"]["source_generation"].as_str().is_some());
            if let Some(previous) = declaration_coverage.as_ref() {
                assert_eq!(
                    previous, &result["coverage"],
                    "cached proof preserves truth"
                );
            }
            declaration_coverage = Some(result["coverage"].clone());
        }
    }
}

#[test]
fn issue_151_schema_states_projection_and_lower_bound_contracts() {
    let tool = crate::mcp::tools::tool_list()
        .into_iter()
        .find(|tool| tool["name"] == "workspace_query")
        .expect("workspace_query tool");
    let input = &tool["inputSchema"]["properties"];
    let within = input["withinPath"]["description"]
        .as_str()
        .expect("withinPath description");
    for phrase in [
        "answer projection",
        "asserting file",
        "all authorized roots",
    ] {
        assert!(within.contains(phrase), "missing contract phrase: {phrase}");
    }
    let declarations = input["declarations_only"]["description"]
        .as_str()
        .expect("declarations_only description");
    assert!(declarations.contains("not source completeness"));
    assert!(declarations.contains("lower bounds"));

    let fact_semantics =
        &tool["outputSchema"]["properties"]["coverage"]["properties"]["fact_semantics"];
    assert_eq!(
        fact_semantics["enum"],
        json!([
            "authoritative_indexed_occurrences",
            "source_proven_declarations"
        ])
    );
}
