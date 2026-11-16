//! Shared source-backed fixtures and public query helpers for ANG-R2 tests.
use super::angular_bootstrap_inline_controls_tests::call;
use super::typescript_route_guard_field_support::setup;
use crate::mcp::McpState;
use serde_json::{Value, json};
use std::path::Path;

pub(super) fn fixture(form: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = setup(form);
    let (imports, value) = match form {
        "alias" => ("import {Gate as Hidden} from './guard-a';", "Hidden"),
        "barrel" => ("import {A as Hidden} from './guard-api';", "Hidden"),
        "namespace" => ("import * as hidden from './guard-a';", "hidden.Gate"),
        "local" => ("function Gate(){return true;}", "Gate"),
        _ => ("import {Gate} from './guard-a';", "Gate"),
    };
    let source = format!(
        "import type {{Routes}} from '@angular/router'; {imports} export const routes:Routes=[{{path:'dashboard',resolve:{{primary:{value},secondary:{value}}}}}];"
    );
    std::fs::write(root.path().join("one.ts"), source).unwrap();
    std::fs::remove_file(root.path().join("two.ts")).unwrap();
    (root, config)
}

pub(super) fn raw(state: &McpState, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(16279),
        "workspace_query",
        &json!({"arguments":args}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}

pub(super) fn query(state: &McpState, root: &Path, direction: &str) -> Value {
    let selector = if direction == "forward" {
        "route_path"
    } else {
        "resolver_name"
    };
    let mut args =
        json!({"type":"resolver_registrations","direction":direction,"workspaceRoot":root});
    args[selector] = json!(if direction == "forward" {
        "dashboard"
    } else {
        "Gate"
    });
    let response = raw(state, args);
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    response["result"]["structuredContent"].clone()
}

pub(super) fn keys(answer: &Value) -> Vec<String> {
    let mut keys: Vec<_> = answer["registrations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["key"].as_str().unwrap().to_owned())
        .collect();
    keys.sort();
    keys
}

pub(super) fn acquire_session(state: &McpState, root: &Path) {
    // Query hydration deliberately owns index facts, not a session/durable
    // context. Acquire that distinct lifecycle only AFTER the cold query.
    call(
        state,
        root,
        "compress_code_context",
        json!({"filePath":root.join("one.ts"),"fidelity":"high"}),
    );
}
