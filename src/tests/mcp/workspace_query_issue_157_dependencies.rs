//! Supplemental Issue #157 lifecycle receipts; frozen RED remains untouched.
use crate::mcp::McpState;
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

fn call(state: &McpState, root: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(1571), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    response["result"].clone()
}

fn query(state: &McpState, root: &Path, args: Value) -> Value {
    call(state, root, "workspace_query", args)["structuredContent"].clone()
}

fn target() -> Value {
    json!({"type":"forward_edges", "domain":"angular", "entity_type":"Module", "name":"AppModule"})
}

fn members(result: &Value) -> Vec<(String, String)> {
    let mut members: Vec<_> = result["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "DeclaresInModule")
        .map(|edge| {
            (
                edge["object"]["name"].as_str().unwrap().to_string(),
                edge["object"]["entity_type"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    members.sort();
    members.dedup();
    members
}

fn expect(state: &McpState, root: &Path, expected: &[(&str, &str)]) {
    let result = query(state, root, target());
    let mut expected: Vec<_> = expected
        .iter()
        .map(|(name, kind)| (name.to_string(), kind.to_string()))
        .collect();
    expected.sort();
    assert_eq!(members(&result), expected, "{result:#}");
}

fn write(root: &Path, file: &str, source: &str) {
    std::fs::write(root.join(file), source).unwrap();
}

fn leaf(kind: &str) -> String {
    format!(
        "import {{ {kind} }} from '@angular/core'; @{kind}({{selector:'[shared]',name:'shared'}}) export class Shared {{}}"
    )
}

fn fixture(root: &Path) {
    write(root, "leaf.ts", &leaf("Directive"));
    write(
        root,
        "shared.ts",
        "import { Shared } from './leaf'; export const SHARED = [Shared];",
    );
    write(
        root,
        "app.module.ts",
        "import { NgModule } from '@angular/core'; import { SHARED } from './shared'; @NgModule({declarations:[...SHARED]}) export class AppModule {}",
    );
}

#[test]
fn issue_157_array_membership_removal_and_restoration_are_independent() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let state = state(root.path(), false);
    expect(&state, root.path(), &[("Shared", "Directive")]);
    write(root.path(), "shared.ts", "export const SHARED = [];");
    expect(&state, root.path(), &[]);
    write(
        root.path(),
        "shared.ts",
        "import { Shared } from './leaf'; export const SHARED = [Shared];",
    );
    expect(&state, root.path(), &[("Shared", "Directive")]);
}

#[test]
fn issue_157_missing_leaf_and_array_appearance_restore_truth() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    std::fs::remove_file(root.path().join("leaf.ts")).unwrap();
    let state = state(root.path(), false);
    expect(&state, root.path(), &[]);
    write(root.path(), "leaf.ts", &leaf("Pipe"));
    expect(&state, root.path(), &[("Shared", "Pipe")]);
    std::fs::remove_file(root.path().join("shared.ts")).unwrap();
    expect(&state, root.path(), &[]);
    write(
        root.path(),
        "shared.ts",
        "import { Shared } from './leaf'; export const SHARED = [Shared];",
    );
    expect(&state, root.path(), &[("Shared", "Pipe")]);
}

#[test]
fn issue_157_array_retargets_membership_by_existing_import_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    write(
        root.path(),
        "other.ts",
        "import { Pipe } from '@angular/core'; @Pipe({name:'other'}) export class Other {}",
    );
    let state = state(root.path(), false);
    expect(&state, root.path(), &[("Shared", "Directive")]);
    write(
        root.path(),
        "shared.ts",
        "import { Other as Current } from './other'; export const SHARED = [Current];",
    );
    expect(&state, root.path(), &[("Current", "Pipe")]);
}

#[test]
fn issue_157_exact_file_precedence_is_a_resolution_dependency() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let state = state(root.path(), false);
    expect(&state, root.path(), &[("Shared", "Directive")]);
    write(root.path(), "leaf", &leaf("Pipe"));
    expect(&state, root.path(), &[("Shared", "Pipe")]);
    std::fs::remove_file(root.path().join("leaf")).unwrap();
    expect(&state, root.path(), &[("Shared", "Directive")]);
}

#[test]
fn issue_157_both_batch_orders_replace_and_retract_membership() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    for reverse in [true, false] {
        write(root.path(), "leaf.ts", &leaf("Directive"));
        let state = state(root.path(), false);
        expect(&state, root.path(), &[("Shared", "Directive")]);
        write(root.path(), "leaf.ts", &leaf("Pipe"));
        let mut module = target();
        module["id"] = json!("module");
        let contributor = json!({"id":"contributor", "type":"entities_in_file", "file_path":"leaf.ts", "fidelity":"high"});
        let queries = if reverse {
            vec![contributor, module]
        } else {
            vec![module, contributor]
        };
        let result = query(&state, root.path(), json!({"queries":queries}));
        let module = result["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == "module")
            .unwrap();
        assert_eq!(module["status"], "ok", "{result:#}");
        assert_eq!(
            members(&module["result"]),
            vec![("Shared".into(), "Pipe".into())]
        );
        expect(&state, root.path(), &[("Shared", "Pipe")]);
    }
}

#[test]
fn issue_157_warm_query_reuses_compilation_and_ignores_unrelated_content() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let state = state(root.path(), false);
    expect(&state, root.path(), &[("Shared", "Directive")]);
    state.reset_test_full_semantic_parses();
    write(root.path(), "unrelated.ts", "export class Unrelated {}");
    expect(&state, root.path(), &[("Shared", "Directive")]);
    assert_eq!(state.test_full_semantic_parses(), 0);
}

#[cfg(feature = "rust")]
#[test]
fn issue_157_contributor_superseded_before_commit_rejects_candidate() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let state = state(root.path(), false);
    let target = root
        .path()
        .join("app.module.ts")
        .to_string_lossy()
        .into_owned();
    crate::mcp::tool_handlers::hydration::supersede_test_batch_once(
        root.path().join("leaf.ts").to_string_lossy().into_owned(),
        leaf("Pipe"),
    );
    assert!(!crate::mcp::tool_handlers::hydration::compile_candidate(
        &state,
        &target,
        root.path().to_str()
    ));
    assert!(
        state
            .workspace_index_read()
            .entities_in_file(&state.semantic_owner_path(&target))
            .is_empty()
    );
    expect(&state, root.path(), &[("Shared", "Pipe")]);
}

#[test]
fn issue_157_durable_projection_retains_and_validates_contributors() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let state = state(root.path(), true);
    let provide = || {
        call(
            &state,
            root.path(),
            "provide_code_context",
            json!({"filePath":"app.module.ts", "fidelity":"high"}),
        )
    };
    provide();
    let path = root
        .path()
        .join("app.module.ts")
        .to_string_lossy()
        .into_owned();
    let owner = state.durable_owner_path(&path);
    let durable = state
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .load_durable_context(&owner, None)
        .unwrap()
        .unwrap();
    for file in ["shared.ts", "leaf.ts"] {
        assert!(
            durable
                .semantic_dependencies
                .iter()
                .any(|dependency| dependency.file_path.ends_with(file)),
            "{durable:#?}"
        );
    }
    let current = crate::mcp::compatibility::derive_identities(
        &std::fs::read_to_string(&path).unwrap(),
        Path::new(&path),
        &state.config,
    )
    .unwrap();
    crate::mcp::compatibility::validator::validate_semantic(&durable, &current).unwrap();
    let mut old = durable.clone();
    old.compatibility
        .semantic_producers
        .as_mut()
        .unwrap()
        .relevant_producers
        .insert(
            crate::mcp::compatibility::identity::ProducerKey::AngularSemantic,
            26,
        );
    assert!(matches!(crate::mcp::compatibility::validator::validate_semantic(&old, &current),
        Err(crate::mcp::compatibility::validator::CompatibilityFailure::SemanticProducerIncompatible)));
    write(root.path(), "leaf.ts", &leaf("Pipe"));
    assert!(matches!(
        crate::mcp::compatibility::validator::validate_semantic(&durable, &current),
        Err(crate::mcp::compatibility::validator::CompatibilityFailure::SemanticDependencyStale)
    ));
    provide();
    expect(&state, root.path(), &[("Shared", "Pipe")]);
}

#[cfg(all(feature = "dotnet", feature = "csharp"))]
#[test]
fn issue_157_batch_reprepares_after_late_contributor_supersession() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let state = state(root.path(), false);
    expect(&state, root.path(), &[("Shared", "Directive")]);
    let captured = root.path().to_path_buf();
    super::publication_test_support::arm(move |state| {
        write(&captured, "leaf.ts", &leaf("Pipe"));
        query(
            state,
            &captured,
            json!({"type":"entities_in_file", "file_path":"leaf.ts", "fidelity":"high"}),
        );
    });
    let mut module = target();
    module["id"] = json!("module");
    let result = query(&state, root.path(), json!({"queries":[module]}));
    assert_eq!(result["results"][0]["status"], "ok", "{result:#}");
    assert_eq!(
        members(&result["results"][0]["result"]),
        vec![("Shared".into(), "Pipe".into())]
    );
}
