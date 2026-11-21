//! Shared retirement controls, separate from the byte-frozen behavioral RED.
use super::angular_bootstrap_inline_controls_tests::call;
use super::typescript_route_guard_field_support::setup;
use crate::mcp::McpState;
use serde_json::{Value, json};
use std::io::{Error, ErrorKind};
use std::path::Path;

const OWNER: &str = "import type {Routes} from '@angular/router'; import {Gate} from './guard-a'; import {Screen} from './screen-a'; export const routes:Routes=[{path:'dashboard',component:Screen,resolve:{primary:Gate,secondary:Gate}}];";

fn fixture() -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = setup("direct");
    std::fs::write(
        root.path().join("screen-a.ts"),
        "import {Component} from '@angular/core'; @Component({}) export class Screen {}",
    )
    .unwrap();
    for name in ["one.ts", "two.ts"] {
        std::fs::write(root.path().join(name), OWNER).unwrap();
    }
    (root, config)
}

fn edges(state: &McpState, root: &Path, reverse: bool) -> Value {
    call(
        state,
        root,
        "workspace_query",
        json!({
            "type":if reverse {"reverse_edges"} else {"forward_edges"},
            "domain":"angular", "entity_type":if reverse {"Resolver"} else {"Route"},
            "name":if reverse {"Gate"} else {"dashboard"}
        }),
    )["edges"]
        .clone()
}

fn raw(state: &McpState, root: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(16202), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}

#[test]
fn issue_162_retirement_shared_supports_siblings_and_relationship_families() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture();
    let p = root.path();
    let state = McpState::new(config);
    let initial = edges(&state, p, false);
    assert_eq!(initial.as_array().unwrap().len(), 4);
    std::fs::remove_file(p.join("one.ts")).unwrap();
    let surviving = edges(&state, p, false);
    let rows = surviving.as_array().unwrap();
    assert_eq!(rows.len(), 2, "{surviving:#}");
    assert!(
        rows.iter()
            .all(|row| row["subject"]["file"].as_str().unwrap().ends_with("two.ts"))
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["relation"] == "ResolvedBy")
            .count(),
        1
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["relation"] == "RouteMapsTo")
            .count(),
        1
    );
    assert_eq!(edges(&state, p, true).as_array().unwrap().len(), 1);
    let registrations = call(
        &state,
        p,
        "workspace_query",
        json!({
            "type":"resolver_registrations", "direction":"forward", "route_path":"dashboard"
        }),
    );
    assert_eq!(registrations["count"], 2);
    assert!(
        registrations["registrations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["evidence"]["source"]
                .as_str()
                .unwrap()
                .ends_with("two.ts"))
    );
}

#[test]
fn issue_162_retirement_acquisition_error_is_not_deletion() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture();
    let state = McpState::new(config);
    let before = edges(&state, root.path(), false);
    let owners = state.workspace_index_read().semantic_projection_owners();
    let failed = state.reconcile_semantic_sources_for_test(|_| {
        Err(Error::new(
            ErrorKind::PermissionDenied,
            "controlled acquisition denial",
        ))
    });
    assert!(failed.is_err());
    assert_eq!(
        state
            .workspace_index_read()
            .semantic_projection_owners()
            .len(),
        owners.len()
    );
    assert_eq!(
        state.workspace_index_read().semantic_projection_owners(),
        owners
    );
    assert_eq!(edges(&state, root.path(), false), before);
}

#[test]
fn issue_162_retirement_reappearance_before_commit_preserves_owner() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture();
    let state = McpState::new(config);
    let before = edges(&state, root.path(), false);
    let owners = state.workspace_index_read().semantic_projection_owners();
    let mut probes = 0;
    let changed = state.reconcile_semantic_sources_for_test(|_| {
        probes += 1;
        Ok(probes != 1)
    });
    assert!(changed.is_err());
    assert_eq!(probes, 2, "one missing observation plus commit recheck");
    assert_eq!(
        state.workspace_index_read().semantic_projection_owners(),
        owners
    );
    assert_eq!(edges(&state, root.path(), false), before);
}

#[test]
fn issue_162_retirement_wrong_kind_refuses_public_currentness_without_deletion() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture();
    let p = root.path();
    let state = McpState::new(config);
    edges(&state, p, false);
    let one = state.semantic_owner_path(&p.join("one.ts").to_string_lossy());
    std::fs::remove_file(p.join("one.ts")).unwrap();
    std::fs::create_dir(p.join("one.ts")).unwrap();
    let failed = raw(
        &state,
        p,
        "workspace_query",
        json!({
            "type":"forward_edges", "domain":"angular", "entity_type":"Route", "name":"dashboard"
        }),
    );
    assert_eq!(failed["error"]["code"], -32603, "{failed:#}");
    assert!(failed["result"].is_null());
    assert!(
        state
            .workspace_index_read()
            .semantic_projection_source_hash(&one)
            .is_some(),
        "unknown is not deletion"
    );
    std::fs::remove_dir(p.join("one.ts")).unwrap();
    std::fs::write(p.join("one.ts"), OWNER).unwrap();
    assert_eq!(edges(&state, p, false).as_array().unwrap().len(), 4);
}

#[test]
fn issue_162_retirement_history_and_owner_ordering_survive_deletion() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture();
    let p = root.path();
    let state = McpState::new(config.clone());
    let file = p.join("one.ts");
    call(
        &state,
        p,
        "compress_code_context",
        json!({"filePath":file,"fidelity":"high"}),
    );
    call(&state, p, "save_context", json!({"filePath":file}));
    let alias = state.alias_for_path(&file.to_string_lossy()).unwrap();
    let old = state.begin_semantic_publication(&file.to_string_lossy());
    std::fs::remove_file(&file).unwrap();
    edges(&state, p, false);
    assert!(state.semantic_snapshot(&alias).is_none());
    assert!(
        old.commit(|| Ok::<_, String>(true), || Ok::<_, String>(()))
            .unwrap()
            .is_none(),
        "older publication cannot revive retired facts"
    );
    let rejected = raw(&state, p, "restore_context", json!({"filePath":file}));
    assert!(
        rejected.get("error").is_some() || rejected["result"]["isError"] == true,
        "{rejected:#}"
    );
    std::fs::write(&file, OWNER).unwrap();
    drop(state);
    let restarted = McpState::new(config);
    call(&restarted, p, "restore_context", json!({"filePath":file}));
    assert_eq!(
        edges(&restarted, p, false).as_array().unwrap().len(),
        4,
        "compatible historical baseline remains available after recreation"
    );
}

#[test]
fn issue_162_retirement_work_is_once_per_batch_and_owner() {
    let _serial = crate::protocol::handler_response_serial();
    for count in [2, 16, 64] {
        let root = tempfile::tempdir().unwrap();
        let state = McpState::new(crate::tests::test_config());
        for i in 0..count {
            let path = root.path().join(format!("owner-{i}.rs"));
            std::fs::write(&path, "pub struct Owner;").unwrap();
            let owner = state.semantic_owner_path(&path.to_string_lossy());
            state
                .workspace_index_lock()
                .replace_semantic_projection_with_dependencies(
                    &owner,
                    Vec::new(),
                    crate::workspace::index::SemanticFidelity::High,
                    crate::cache::content_hash(b"pub struct Owner;"),
                    Vec::new(),
                );
        }
        crate::mcp::tool_handlers::query::work_metrics::reset();
        let answer = raw(
            &state,
            root.path(),
            "workspace_query",
            json!({"queries":[
                {"id":"one","type":"has_cycle"},{"id":"two","type":"has_cycle"}
            ]}),
        );
        assert!(answer.get("error").is_none(), "{answer:#}");
        let work = crate::mcp::tool_handlers::query::work_metrics::snapshot();
        assert_eq!(work.retirement_owner_passes, 1);
        assert_eq!(work.retirement_owner_probes, count);
    }
}

#[cfg(any(windows, unix))]
#[test]
fn issue_162_retirement_physical_alias_preserves_live_owner_and_rebinds_deleted_path() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first.ts");
    let second = root.path().join("second.ts");
    std::fs::write(&first, "export const value=1;").unwrap();
    std::fs::hard_link(&first, &second).unwrap();
    let state = McpState::new(crate::tests::test_config());
    let first_owner = state.semantic_owner_path(&first.to_string_lossy());
    let first_durable = state.durable_owner_path(&first.to_string_lossy());
    assert_eq!(
        state.semantic_owner_path(&second.to_string_lossy()),
        first_owner
    );
    assert_eq!(
        state.durable_owner_path(&second.to_string_lossy()),
        first_durable
    );
    std::fs::remove_file(&first).unwrap();
    let second_owner = state.semantic_owner_path(&second.to_string_lossy());
    assert_ne!(
        second_owner, first_owner,
        "absent path cannot remain current owner"
    );
    assert_eq!(
        second_owner,
        crate::dictionary::path::canonical_identity_key(&second.to_string_lossy())
    );
    assert_eq!(
        state.durable_owner_path(&second.to_string_lossy()),
        second.to_string_lossy().into_owned()
    );
    assert_eq!(
        std::fs::read_to_string(&second).unwrap(),
        "export const value=1;"
    );
    std::fs::write(&first, "export const value=2;").unwrap();
    assert_ne!(
        state.semantic_owner_path(&first.to_string_lossy()),
        second_owner
    );
    assert_eq!(
        state.semantic_owner_path(&second.to_string_lossy()),
        second_owner
    );
}

#[test]
fn issue_162_retirement_known_unindexed_paths_are_checked_once_per_batch() {
    let _serial = crate::protocol::handler_response_serial();
    for count in [2, 16, 64] {
        let root = tempfile::tempdir().unwrap();
        let state = McpState::new(crate::tests::test_config());
        for i in 0..count {
            let path = root.path().join(format!("known-{i}.rs"));
            std::fs::write(&path, "pub struct Known;").unwrap();
            state.get_or_create_alias(path.to_string_lossy().into_owned());
        }
        crate::mcp::tool_handlers::query::work_metrics::reset();
        let answer = raw(
            &state,
            root.path(),
            "workspace_query",
            json!({"queries":[
                {"id":"one","type":"has_cycle"},{"id":"two","type":"has_cycle"}
            ]}),
        );
        assert!(answer.get("error").is_none(), "{answer:#}");
        let work = crate::mcp::tool_handlers::query::work_metrics::snapshot();
        assert_eq!(work.retirement_owner_passes, 1);
        assert_eq!(work.retirement_owner_probes, count);
        assert!(
            state
                .workspace_index_read()
                .semantic_projection_owners()
                .is_empty(),
            "path history never manufactures semantic proof"
        );
    }
}
