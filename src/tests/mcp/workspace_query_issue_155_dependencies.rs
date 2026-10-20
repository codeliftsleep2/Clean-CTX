//! Supplemental closure/lifecycle coverage; the original RED file is unchanged.
use crate::mcp::McpState;
use serde_json::{Value, json};
use std::path::Path;

fn state(roots: &[&Path]) -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = roots
        .iter()
        .map(|root| root.to_string_lossy().into_owned())
        .collect();
    McpState::new(config)
}

fn call(state: &McpState, tool: &str, root: &Path, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(1550), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    response["result"]["structuredContent"].clone()
}

fn file(state: &McpState, root: &Path) -> Value {
    call(
        state,
        "workspace_query",
        root,
        json!({"type":"entities_in_file", "file_path":"Target.cs", "fidelity":"high"}),
    )
}

fn write(root: &Path, file: &str, source: &str) {
    std::fs::write(root.join(file), source).unwrap();
}

fn has_hub(result: &Value) -> bool {
    result["entities"].as_array().unwrap().iter().any(|entity| {
        entity["domain"] == "dotnet" && entity["entity_type"] == "Hub" && entity["name"] == "Target"
    })
}

fn complete(result: &Value) {
    assert_eq!(result["coverage"]["source_complete"], true, "{result:#}");
    assert_eq!(
        result["coverage"]["dependency_proof"]["status"], "complete",
        "{result:#}"
    );
    assert!(
        result["coverage"]["dependency_proof"]["generation"].is_string(),
        "{result:#}"
    );
}

fn partial(result: &Value, reason: &str) {
    assert!(!has_hub(result), "{result:#}");
    assert_eq!(result["coverage"]["source_complete"], false, "{result:#}");
    assert_eq!(result["coverage"]["omitted_possible"], true, "{result:#}");
    assert_eq!(
        result["coverage"]["result_semantics"], "lower_bound",
        "{result:#}"
    );
    assert_eq!(
        result["coverage"]["dependency_proof"]["status"], "unproven",
        "{result:#}"
    );
    assert!(
        result["coverage"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!(reason)),
        "{result:#}"
    );
    assert!(
        !result["coverage"]["evidence_requirements"]
            .as_array()
            .unwrap()
            .contains(&json!("dependencies_current")),
        "{result:#}"
    );
}

#[test]
fn issue_155_missing_dependency_is_partial_cold_and_cached() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : AbsentBase { public void Send() {} }",
    );
    let state = state(&[]);
    for _ in 0..2 {
        partial(&file(&state, root.path()), "dependency_unresolved");
    }
}

#[test]
fn issue_155_same_name_ambiguity_never_selects_first_candidate() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "using A; using B; public class Target : Bridge { public void Send() {} }",
    );
    write(
        root.path(),
        "A.cs",
        "namespace A; public class Bridge : global::Microsoft.AspNetCore.SignalR.Hub {}",
    );
    write(
        root.path(),
        "B.cs",
        "namespace B; public class Bridge : global::Microsoft.AspNetCore.SignalR.Hub {}",
    );
    for order in [["A.cs", "B.cs"], ["B.cs", "A.cs"]] {
        let state = state(&[]);
        for file in order {
            call(
                &state,
                "workspace_query",
                root.path(),
                json!({"type":"entities_in_file", "file_path":file}),
            );
        }
        partial(&file(&state, root.path()), "dependency_ambiguous");
    }
    partial(&file(&state(&[]), root.path()), "dependency_ambiguous");
}

#[test]
fn issue_155_framework_spelling_is_not_authority() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : PretendHub { public void Send() {} }",
    );
    write(root.path(), "Pretend.cs", "public class PretendHub {}");
    let result = file(&state(&[]), root.path());
    assert!(!has_hub(&result), "{result:#}");
    complete(&result);
    write(
        root.path(),
        "Target.cs",
        "public class Target : global::Microsoft.AspNetCore.SignalR.Hub { public void Send() {} }",
    );
    write(
        root.path(),
        "Fake.cs",
        "namespace Microsoft.AspNetCore.SignalR; public class Hub {}",
    );
    partial(&file(&state(&[]), root.path()), "dependency_ambiguous");
}

#[test]
fn issue_155_cycles_are_unproven_and_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : Bridge { public void Send() {} }",
    );
    write(root.path(), "Bridge.cs", "public class Bridge : Target {}");
    partial(&file(&state(&[]), root.path()), "dependency_cycle");
}

#[test]
fn issue_155_partial_owner_acquires_the_part_that_proves_ancestry() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public partial class Target { public void Send() {} }",
    );
    write(
        root.path(),
        "OtherPart.cs",
        "public partial class Target : Bridge {}",
    );
    write(
        root.path(),
        "Bridge.cs",
        "public class Bridge : global::Microsoft.AspNetCore.SignalR.Hub {}",
    );
    let state = state(&[]);
    let result = file(&state, root.path());
    assert!(has_hub(&result), "{result:#}");
    complete(&result);
}

#[test]
fn issue_155_refresh_retracts_and_restores_ancestry_and_proof() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : Bridge { public void Send() {} }",
    );
    let state = state(&[]);
    for (base, expected) in [
        (
            "public class Bridge : global::Microsoft.AspNetCore.SignalR.Hub {}",
            true,
        ),
        ("public class Bridge {}", false),
        (
            "public class Bridge : global::Microsoft.AspNetCore.SignalR.Hub {}",
            true,
        ),
    ] {
        write(root.path(), "Bridge.cs", base);
        let refreshed = call(&state, "refresh_workspace", root.path(), json!({}));
        assert_eq!(refreshed["refreshed"], true, "{refreshed:#}");
        let result = file(&state, root.path());
        assert_eq!(has_hub(&result), expected, "{result:#}");
        complete(&result);
    }
}

#[test]
fn issue_155_batch_order_and_root_order_share_final_publication() {
    let root = tempfile::tempdir().unwrap();
    let other_a = tempfile::tempdir().unwrap();
    let other_b = tempfile::tempdir().unwrap();
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
    // An identically named declaration in another partition cannot establish or
    // compete for this owner's identity merely because its root was first.
    write(other_a.path(), "Bridge.cs", "public class Bridge {}");
    for roots in [
        [other_a.path(), other_b.path()],
        [other_b.path(), other_a.path()],
    ] {
        for reverse in [false, true] {
            let state = state(&roots);
            let mut queries = vec![
                json!({"id":"target", "type":"entities_in_file", "file_path":"Target.cs", "fidelity":"high"}),
                json!({"id":"base", "type":"entities_in_file", "file_path":"Bridge.cs"}),
            ];
            if reverse {
                queries.reverse();
            }
            let result = call(
                &state,
                "workspace_query",
                root.path(),
                json!({"queries":queries}),
            );
            let target = result["results"]
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["id"] == "target")
                .unwrap();
            assert_eq!(target["status"], "ok", "{result:#}");
            assert!(has_hub(&target["result"]), "{result:#}");
            complete(&target["result"]);
        }
    }
}

#[test]
fn issue_155_warm_closure_reuses_compilation_and_reconciliation() {
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
    let state = state(&[]);
    complete(&file(&state, root.path()));
    state.reset_test_csharp_reconciliation_count();
    state.reset_test_full_semantic_parses();
    for _ in 0..2 {
        complete(&file(&state, root.path()));
    }
    assert_eq!(state.test_csharp_reconciliation_count(), 0);
    assert_eq!(state.test_full_semantic_parses(), 0);
}

#[test]
fn issue_155_edges_expose_dependency_proof_without_claiming_workspace_absence() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "using Parent = Project.Bridge; public class Target : Parent { public void Send() {} }",
    );
    write(
        root.path(),
        "Bridge.cs",
        "namespace Project; public class Bridge : global::Microsoft.AspNetCore.SignalR.Hub {}",
    );
    let state = state(&[]);
    let result = call(
        &state,
        "workspace_query",
        root.path(),
        json!({
            "type":"forward_edges", "domain":"dotnet", "entity_type":"Hub", "name":"Target"
        }),
    );
    assert!(
        result["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| edge["relation"] == "HubMethodTargets"),
        "{result:#}"
    );
    assert_eq!(
        result["coverage"]["dependency_proof"]["status"], "complete",
        "{result:#}"
    );
    // Closing this owner's ancestry does not prove an incoming workspace universe.
    assert_eq!(
        result["coverage"]["result_semantics"], "lower_bound",
        "{result:#}"
    );
    assert_eq!(result["coverage"]["source_complete"], false, "{result:#}");
}

#[test]
fn issue_155_find_entities_consumes_the_same_cold_closure() {
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
    let state = state(&[]);
    let result = call(
        &state,
        "workspace_query",
        root.path(),
        json!({"type":"find_entities", "name":"Target"}),
    );
    assert!(has_hub(&result), "{result:#}");
    assert_eq!(
        result["coverage"]["dependency_proof"]["status"], "complete",
        "{result:#}"
    );
}

struct ReleaseFlight(std::sync::Arc<McpState>);

#[test]
fn issue_155_duplicate_owner_is_not_hidden_by_a_file_local_query() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : global::Microsoft.AspNetCore.SignalR.Hub { public void Send() {} }",
    );
    write(root.path(), "Duplicate.cs", "public class Target {}");
    partial(&file(&state(&[]), root.path()), "dependency_ambiguous");
}

impl Drop for ReleaseFlight {
    fn drop(&mut self) {
        self.0.hydration_flights.test_release();
    }
}

#[test]
fn issue_155_single_flight_followers_revalidate_unresolved_requirements() {
    use crate::mcp::tool_handlers::hydration::acquire_csharp_dependencies;
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "Target.cs",
        "public class Target : Absent { public void Send() {} }",
    );
    let state = std::sync::Arc::new(state(&[]));
    let _release = ReleaseFlight(state.clone());
    let owner = state.semantic_owner_path(&root.path().join("Target.cs").to_string_lossy());
    state.hydration_flights.test_arm();
    let start = || {
        let state = state.clone();
        let owner = owner.clone();
        let root = root.path().to_string_lossy().into_owned();
        std::thread::spawn(move || acquire_csharp_dependencies(&state, &owner, Some(&root)))
    };
    let leader = start();
    state.hydration_flights.test_wait_for_leader();
    let follower = start();
    state.hydration_flights.test_wait_for_followers(1);
    state.hydration_flights.test_release();
    for proof in [
        leader.join().unwrap().unwrap(),
        follower.join().unwrap().unwrap(),
    ] {
        let mut coverage = json!({"source_complete":true, "evidence_requirements":[]});
        proof.apply(&state.workspace_index_read(), &mut coverage);
        assert_eq!(coverage["source_complete"], false, "{coverage:#}");
        assert_eq!(
            coverage["dependency_proof"]["status"], "unproven",
            "{coverage:#}"
        );
    }
}
