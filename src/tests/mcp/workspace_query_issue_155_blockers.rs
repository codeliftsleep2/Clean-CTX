//! In-scope audit blockers, kept separate from both frozen regression files.
use crate::mcp::McpState;
use crate::mcp::state::durable_semantics::SemanticSnapshot;
use serde_json::{Value, json};
use std::path::Path;

fn state(root: &Path, durable: bool) -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = durable;
    config.persistence.auto_save = durable;
    config.persistence.db_path = root.join("evidence.db").to_string_lossy().into_owned();
    McpState::new(config)
}
fn write(root: &Path, name: &str, source: &str) {
    std::fs::write(root.join(name), source).unwrap();
}
fn call(state: &McpState, root: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(1554), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    response["result"].clone()
}
fn query(state: &McpState, root: &Path, args: Value) -> Value {
    call(state, root, "workspace_query", args)["structuredContent"].clone()
}
fn file(state: &McpState, root: &Path) -> Value {
    query(
        state,
        root,
        json!({"type":"entities_in_file", "file_path":"Target.cs", "fidelity":"high"}),
    )
}
fn provide(state: &McpState, root: &Path) -> Value {
    call(
        state,
        root,
        "provide_code_context",
        json!({"filePath":"Target.cs", "fidelity":"high"}),
    )
}
fn hub(snapshot: &SemanticSnapshot) -> bool {
    snapshot.edges.iter().any(|edge| {
        edge.subject.name == "Target"
            && edge.relation == crate::layers::meta::semantic::SemanticRelation::HubMethodTargets
    })
}
fn snapshot(state: &McpState, root: &Path) -> SemanticSnapshot {
    let owner = state.semantic_owner_path(&root.join("Target.cs").to_string_lossy());
    let alias = state.alias_for_path(&owner).unwrap();
    state.semantic_snapshot(&alias).unwrap()
}

#[test]
fn issue_155_context_transitive_mutation_stabilizes_inventory_without_refresh() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : Bridge { public void Send() {} }",
    );
    write(
        root.path(),
        "Bridge.cs",
        "public class Bridge : Foundation {}",
    );
    let external = "public class Foundation : global::Microsoft.AspNetCore.SignalR.Hub {}";
    write(root.path(), "Foundation.cs", external);
    let state = state(root.path(), true);
    for (source, expected) in [
        (external, true),
        ("public class Foundation {}", false),
        (external, true),
        ("public class Foundation {}", false),
        (external, true),
    ] {
        write(root.path(), "Foundation.cs", source);
        // Context is the first consumer after each mutation. A later query must
        // not be needed to repair an inventory invalidated by frontier reads.
        let result = provide(&state, root.path());
        assert_eq!(
            result["_meta"]["semantic_coverage"]["dependency_proof"]["status"], "complete",
            "{result:#}"
        );
        assert_eq!(hub(&snapshot(&state, root.path())), expected, "{result:#}");
    }
}

#[test]
fn issue_155_edge_coverage_does_not_insert_null_evidence_requirements() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : Bridge { [global::Microsoft.AspNetCore.Mvc.HttpGetAttribute] public int Get() { return 1; } }",
    );
    write(
        root.path(),
        "Bridge.cs",
        "public class Bridge : global::Microsoft.AspNetCore.Mvc.ControllerBase {}",
    );
    let result = query(
        &state(root.path(), false),
        root.path(),
        json!({"type":"forward_edges", "domain":"dotnet", "entity_type":"Controller", "name":"Target"}),
    );
    assert_eq!(
        result["coverage"]["dependency_proof"]["status"], "complete",
        "{result:#}"
    );
    let tools = crate::mcp::tools::tool_list();
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "workspace_query")
        .unwrap();
    assert_eq!(
        tool["outputSchema"]["properties"]["coverage"]["properties"]["evidence_requirements"]["type"],
        "array"
    );
    if let Some(requirements) = result["coverage"].get("evidence_requirements") {
        assert!(requirements.is_array(), "{result:#}");
        assert!(
            requirements
                .as_array()
                .unwrap()
                .iter()
                .all(Value::is_string),
            "{result:#}"
        );
    }
}

#[test]
fn issue_155_attribute_source_shadow_converges_cold_and_warm() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "using Microsoft.AspNetCore.Mvc; namespace Demo; public class Target : ControllerBase { [HttpGet] public int Get() { return 1; } }",
    );
    write(
        root.path(),
        "Shadow.cs",
        "namespace Demo; public class HttpGetAttribute {}",
    );
    for warm in [false, true] {
        let state = state(root.path(), false);
        if warm {
            query(
                &state,
                root.path(),
                json!({"type":"find_entities", "name":"HttpGetAttribute"}),
            );
        }
        let result = query(
            &state,
            root.path(),
            json!({"type":"forward_edges", "domain":"dotnet", "entity_type":"Controller", "name":"Target"}),
        );
        assert!(
            !result["edges"]
                .as_array()
                .unwrap()
                .iter()
                .any(|edge| edge["relation"] == "ControllerAction"),
            "{result:#}"
        );
        assert_eq!(
            result["coverage"]["dependency_proof"]["status"], "complete",
            "{result:#}"
        );
    }
}

#[test]
fn issue_155_attribute_ambiguity_is_partial_and_matches_registered_schema() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "using Microsoft.AspNetCore.Mvc; public class Target : ControllerBase { [HttpGet] public int Get() { return 1; } }",
    );
    write(
        root.path(),
        "Shadow.cs",
        "namespace Microsoft.AspNetCore.Mvc; public class HttpGetAttribute {}",
    );
    let result = file(&state(root.path(), false), root.path());
    let coverage = &result["coverage"];
    assert_eq!(
        coverage["status"], "semantic_dependencies_unresolved",
        "{result:#}"
    );
    assert_eq!(coverage["source_complete"], false);
    assert_eq!(coverage["dependency_proof"]["status"], "unproven");
    let tools = crate::mcp::tools::tool_list();
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "workspace_query")
        .unwrap();
    let schema = &tool["outputSchema"]["properties"]["coverage"]["properties"];
    assert!(
        schema["status"]["enum"]
            .as_array()
            .unwrap()
            .contains(&coverage["status"])
    );
    let reason_schema = &schema["reasons"]["items"]["oneOf"];
    assert!(
        reason_schema
            .as_array()
            .unwrap()
            .iter()
            .any(|schema| schema["type"] == "object")
    );
    let allowed = reason_schema
        .as_array()
        .unwrap()
        .iter()
        .find(|schema| schema["type"] == "string")
        .unwrap()["enum"]
        .as_array()
        .unwrap();
    for reason in coverage["reasons"].as_array().unwrap() {
        assert!(allowed.contains(reason), "{reason:#}");
    }
    assert_eq!(
        schema["dependency_proof"]["oneOf"][0]["properties"]["generation"]["type"],
        "string"
    );
}

#[test]
fn issue_155_class_attribute_shadow_retracts_framework_authority() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "using Microsoft.AspNetCore.Mvc; namespace Demo; [ApiController] public class Target { [HttpGet] public int Get() { return 1; } }",
    );
    write(
        root.path(),
        "Shadow.cs",
        "namespace Demo; public class ApiControllerAttribute {}",
    );
    let result = file(&state(root.path(), false), root.path());
    assert!(
        !result["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entity| entity["domain"] == "dotnet" && entity["entity_type"] == "Controller"),
        "{result:#}"
    );
}

#[test]
fn issue_155_context_snapshot_and_durable_footprint_retract_and_restore() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : Bridge { public void Send() {} }",
    );
    write(
        root.path(),
        "Bridge.cs",
        "public class Bridge : global::Microsoft.AspNetCore.SignalR.Hub {}",
    );
    let state = state(root.path(), true);
    let cold = provide(&state, root.path());
    assert_eq!(
        cold["_meta"]["semantic_coverage"]["dependency_proof"]["status"], "complete",
        "{cold:#}"
    );
    let original = snapshot(&state, root.path());
    assert!(hub(&original));
    assert!(
        original
            .dependencies
            .iter()
            .any(|dependency| dependency.file_path.ends_with("Bridge.cs"))
    );
    provide(&state, root.path());
    assert!(hub(&snapshot(&state, root.path())));
    let owner = state.durable_owner_path(&root.path().join("Target.cs").to_string_lossy());
    let durable = state
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .load_durable_context(&owner, None)
        .unwrap()
        .unwrap();
    assert!(
        durable
            .semantic_dependencies
            .iter()
            .any(|dependency| dependency.file_path.ends_with("Bridge.cs"))
    );
    write(root.path(), "Bridge.cs", "public class Bridge {}");
    assert!(
        !original
            .dependencies
            .iter()
            .all(|dependency| dependency.is_current())
    );
    assert!(
        !durable
            .semantic_dependencies
            .iter()
            .all(|dependency| dependency.is_current())
    );
    let current = crate::mcp::compatibility::derive_identities(
        &std::fs::read_to_string(root.path().join("Target.cs")).unwrap(),
        &root.path().join("Target.cs"),
        &state.config,
    )
    .unwrap();
    assert!(matches!(
        crate::mcp::compatibility::validator::validate_semantic(&durable, &current),
        Err(crate::mcp::compatibility::validator::CompatibilityFailure::SemanticDependencyStale)
    ));
    let alias = state
        .alias_for_path(&root.path().join("Target.cs").to_string_lossy())
        .unwrap();
    assert!(state.semantic_snapshot(&alias).is_none());
    // No refresh or query prewarming: the context consumer must acquire current evidence itself.
    provide(&state, root.path());
    assert!(!hub(&snapshot(&state, root.path())));
    write(
        root.path(),
        "Bridge.cs",
        "public class Bridge : global::Microsoft.AspNetCore.SignalR.Hub {}",
    );
    provide(&state, root.path());
    assert!(hub(&snapshot(&state, root.path())));
}

#[test]
fn issue_155_context_batch_revalidates_earlier_items_after_later_acquisition() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : Bridge { public void Send() {} }",
    );
    write(
        root.path(),
        "Bridge.cs",
        "public class Bridge : global::Microsoft.AspNetCore.SignalR.Hub {}",
    );
    write(root.path(), "Other.cs", "public class Other : PlainBase {}");
    write(root.path(), "Plain.cs", "public class PlainBase {}");
    for reverse in [false, true] {
        let state = state(root.path(), false);
        let mut items = vec![
            json!({"id":"target", "filePath":"Target.cs", "fidelity":"high"}),
            json!({"id":"other", "filePath":"Other.cs", "fidelity":"high"}),
        ];
        if reverse {
            items.reverse();
        }
        let result = call(
            &state,
            root.path(),
            "provide_code_context",
            json!({"files":items, "responseMode":"structured"}),
        );
        for item in result["structuredContent"]["results"].as_array().unwrap() {
            assert_eq!(item["status"], "ok", "{result:#}");
            assert_eq!(
                item["meta"]["semantic_coverage"]["dependency_proof"]["status"], "complete",
                "{result:#}"
            );
        }
        assert!(hub(&snapshot(&state, root.path())));
    }
}

#[test]
fn issue_155_mixed_batch_stabilizes_after_late_publication_without_reordering() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : Bridge { public void Send() {} }",
    );
    write(
        root.path(),
        "Bridge.cs",
        "public class Bridge : global::Microsoft.AspNetCore.SignalR.Hub {}",
    );
    write(root.path(), "Other.cs", "public class Other : PlainBase {}");
    write(root.path(), "Plain.cs", "public class PlainBase {}");
    write(root.path(), "Unrelated.ts", "export class Unrelated {}");
    for reverse in [false, true] {
        let state = state(root.path(), false);
        let workspace = root.path().to_string_lossy().into_owned();
        let other = state.semantic_owner_path(&root.path().join("Other.cs").to_string_lossy());
        super::publication_test_support::arm(move |state| {
            state
                .prepare_csharp_semantic_owner(&other, Some(&workspace))
                .unwrap();
        });
        let mut items = vec![
            json!({"id":"edges", "type":"forward_edges", "domain":"dotnet", "entity_type":"Hub", "name":"Target"}),
            json!({"id":"file", "type":"entities_in_file", "file_path":"Target.cs", "fidelity":"high"}),
            json!({"id":"unrelated", "type":"find_entities", "name":"Unrelated"}),
        ];
        if reverse {
            items.reverse();
        }
        let result = query(&state, root.path(), json!({"queries":items}));
        let results = result["results"].as_array().unwrap();
        for item in results {
            assert_eq!(item["status"], "ok", "{result:#}");
        }
        let target = &results.iter().find(|item| item["id"] == "file").unwrap()["result"];
        assert_eq!(target["coverage"]["source_complete"], true, "{result:#}");
        assert_eq!(
            target["coverage"]["dependency_proof"]["status"], "complete",
            "{result:#}"
        );
        let owner = state.semantic_owner_path(&root.path().join("Target.cs").to_string_lossy());
        let workspace = root.path().to_string_lossy().into_owned();
        let proof = state
            .prepare_csharp_semantic_owner(&owner, Some(&workspace))
            .unwrap();
        let mut expected = json!({});
        proof.apply(&state.workspace_index_read(), &mut expected);
        assert_eq!(
            target["coverage"]["dependency_proof"],
            expected["dependency_proof"]
        );
        assert_eq!(
            results[0]["id"],
            if reverse { "unrelated" } else { "edges" }
        );
    }
}
