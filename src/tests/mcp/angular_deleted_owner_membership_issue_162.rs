//! Deleted-owner field finding: legacy membership, without registration queries.
use super::angular_bootstrap_inline_controls_tests::call;
use super::typescript_route_guard_field_support::setup;
use crate::mcp::McpState;
use serde_json::json;
use std::path::Path;

fn memberships(state: &McpState, root: &Path) -> Vec<serde_json::Value> {
    let answer = call(
        state,
        root,
        "workspace_query",
        json!({
            "type":"forward_edges", "domain":"angular",
            "entity_type":"Route", "name":"dashboard"
        }),
    );
    answer["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "ResolvedBy")
        .cloned()
        .collect()
}

#[test]
fn issue_162_deleted_owner_legacy_membership_is_retired() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let root = root.path();
    let source = "import type {Routes} from '@angular/router'; import {Gate} from './guard-a'; export const routes:Routes=[{path:'dashboard',resolve:{primary:Gate}}];";
    std::fs::write(root.join("one.ts"), source).unwrap();
    std::fs::remove_file(root.join("two.ts")).unwrap();
    let state = McpState::new(config);
    // This control never invokes resolver_registrations. It separates the old
    // query lifecycle from the new operation's explicit nominated-owner set.
    assert_eq!(memberships(&state, root).len(), 1, "initial acquisition");
    std::fs::rename(root.join("one.ts"), root.join("moved.ts")).unwrap();
    let moved = memberships(&state, root);
    assert_eq!(
        moved.len(),
        1,
        "deleted original owner must not survive relocation: {moved:#?}"
    );
    std::fs::remove_file(root.join("moved.ts")).unwrap();
    let deleted = memberships(&state, root);
    assert!(
        deleted.is_empty(),
        "deleted asserting owners must not answer: {deleted:#?}"
    );
    std::fs::write(root.join("one.ts"), source).unwrap();
    assert_eq!(
        memberships(&state, root).len(),
        1,
        "recreation must not revive deleted owner facts"
    );
}
