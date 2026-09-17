// src/tests/mcp/workspace_query_scope_entities.rs
//
// WSC-004 — workspace-scoped `find_entities`, and the `entities_in_file` contract.
//
// Sibling of `workspace_query_scope.rs` (cross-repository EDGE isolation,
// RED-SCOPE1–10). This file covers the occurrence-bearing ENTITY surface:
//
//   RED-WSC4-1 — find_entities isolates two repositories that declare the same
//                entity name; the root-less query keeps the global view.
//   RED-WSC4-2 — an entity declared in a configured additional root belongs to
//                the primary workspace's answer.
//   RED-WSC4-3 — an identical Model C identity stays globally retained while each
//                workspace answers with its own occurrence.
//   RED-WSC4-4 — the `entities_in_file` contract: the explicit file path is
//                already validated through the trusted-path/root boundary, so it
//                IS the scope and needs no extra filtering.
//
// Entity provenance is the occurrence's own file (`EntityRef.file`); edge
// provenance is `StoredEdge::asserting_file`. Both are decided by the ONE shared
// `WorkspaceScope::admits` rule, so no query type grows its own root logic.
//
// The traversal surfaces (`transitive_dependencies`, `has_cycle`) are covered by
// the sibling `workspace_query_scope_traversal.rs`, which reuses the query
// helpers defined here.
//
// Every assertion goes through the REAL production path: MCP dispatch →
// `handle_workspace_query` → hydration → scoped WorkspaceIndex lookup.

use super::workspace_query_scope::{DOMAIN, Repo, seed_generic, serialize, state};
use super::*;
use crate::layers::meta::semantic::SemanticRelation;
use crate::mcp::McpState;

// ── Query helpers (shared with the traversal sibling) ────────────────

/// Dispatch one `workspace_query` call through the real MCP path and return its
/// `structuredContent`.
pub(super) fn structured(state: &McpState, arguments: serde_json::Value) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({ "arguments": arguments }),
        state,
    );
    let response = pop_response();
    assert!(
        response.get("result").is_some(),
        "query must succeed: {response}"
    );
    response["result"]["structuredContent"].clone()
}

/// `(entity name, owning-file NAME)` for every returned entity occurrence, sorted.
///
/// The file is reduced to its final path component so the assertions stay
/// independent of the platform's canonical spelling while still proving WHICH
/// repository's file the occurrence came from.
pub(super) fn entity_occurrences(sc: &serde_json::Value) -> Vec<(String, String)> {
    let mut occurrences: Vec<(String, String)> = sc["entities"]
        .as_array()
        .map(|entities| {
            entities
                .iter()
                .map(|entity| {
                    let file = entity["file"].as_str().unwrap_or_default();
                    let owning_file = file
                        .rsplit(['/', '\\'])
                        .next()
                        .unwrap_or_default()
                        .to_string();
                    (
                        entity["name"].as_str().unwrap_or_default().to_string(),
                        owning_file,
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    occurrences.sort();
    occurrences
}

/// Attach `workspaceRoot` only when the caller declares one: a root-less query
/// must omit the argument entirely rather than pass a placeholder.
pub(super) fn with_root(mut arguments: serde_json::Value, root: Option<&str>) -> serde_json::Value {
    if let Some(root) = root {
        arguments["workspaceRoot"] = json!(root);
    }
    arguments
}

/// `find_entities` by name → `(name, owning-file)` occurrences, sorted.
pub(super) fn found_entities(
    state: &McpState,
    name: &str,
    root: Option<&str>,
) -> Vec<(String, String)> {
    let arguments = with_root(json!({ "type": "find_entities", "name": name }), root);
    entity_occurrences(&structured(state, arguments))
}

/// `entities_in_file` → the entity occurrences of that file, sorted.
pub(super) fn entities_in_file(
    state: &McpState,
    file_path: &str,
    root: Option<&str>,
) -> Vec<(String, String)> {
    let arguments = with_root(
        json!({ "type": "entities_in_file", "file_path": file_path }),
        root,
    );
    let sc = structured(state, arguments);
    // A path outside the declared roots is answered with the handler's minimal
    // zero-result shape; either way, `entities_in_file` is not
    // hydration-eligible, so it carries no discovery diagnostic at all.
    assert!(
        sc.get("discovery").is_none() && sc.get("hydration_attempted").is_none(),
        "entities_in_file must never hydrate: {sc:?}"
    );
    entity_occurrences(&sc)
}

/// `transitive_dependencies` → the dependency entity NAMES (third element of each
/// serialized `EntityKey`), sorted.
pub(super) fn dependencies(
    state: &McpState,
    name: &str,
    depth: i32,
    root: Option<&str>,
) -> Vec<String> {
    let arguments = with_root(
        json!({
            "type": "transitive_dependencies",
            "domain": DOMAIN,
            "entity_type": "Class",
            "name": name,
            "depth": depth,
        }),
        root,
    );
    let sc = structured(state, arguments);
    let mut names: Vec<String> = sc["dependencies"]
        .as_array()
        .map(|dependencies| {
            dependencies
                .iter()
                .map(|dependency| dependency[2].as_str().unwrap_or_default().to_string())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// `has_cycle` → the boolean the handler reported.
pub(super) fn cycle(state: &McpState, root: Option<&str>) -> bool {
    let arguments = with_root(json!({ "type": "has_cycle" }), root);
    let sc = structured(state, arguments);
    assert!(
        sc.get("discovery").is_none() && sc.get("hydration_attempted").is_none(),
        "has_cycle must stay non-hydration-eligible: {sc:?}"
    );
    sc["has_cycle"].as_bool().expect("has_cycle boolean")
}

// ── RED-WSC4-1: find_entities isolates two repositories ──────────────

#[test]
fn red_wsc4_1_find_entities_isolates_two_repositories() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    // The SAME entity name, declared in two repositories, each in its own file.
    seed_generic(
        &state,
        &repo_a.file("A.cs"),
        SemanticRelation::Injects,
        "LoadingComponent",
        "Router",
    );
    seed_generic(
        &state,
        &repo_b.file("B.cs"),
        SemanticRelation::Injects,
        "LoadingComponent",
        "Router",
    );

    assert_eq!(
        found_entities(&state, "LoadingComponent", Some(&repo_a.key())),
        vec![("LoadingComponent".to_string(), "A.cs".to_string())],
        "a workspace-scoped find_entities answers with that workspace's occurrence only"
    );
    assert_eq!(
        found_entities(&state, "LoadingComponent", Some(&repo_b.key())),
        vec![("LoadingComponent".to_string(), "B.cs".to_string())],
        "repo B answers with its own occurrence only"
    );
    assert_eq!(
        found_entities(&state, "LoadingComponent", None),
        vec![
            ("LoadingComponent".to_string(), "A.cs".to_string()),
            ("LoadingComponent".to_string(), "B.cs".to_string()),
        ],
        "a root-less query keeps the global session view"
    );
}

// ── RED-WSC4-2: an additional root belongs to the workspace ──────────

#[test]
fn red_wsc4_2_entity_in_an_additional_root_is_part_of_the_workspace() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_a2 = Repo::inside(base.path(), "repo-a-additional");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[&repo_a2]);
    seed_generic(
        &state,
        &repo_a2.file("A2.cs"),
        SemanticRelation::Injects,
        "SharedService",
        "Clock",
    );
    seed_generic(
        &state,
        &repo_b.file("B.cs"),
        SemanticRelation::Injects,
        "SharedService",
        "Clock",
    );

    assert_eq!(
        found_entities(&state, "SharedService", Some(&repo_a.key())),
        vec![("SharedService".to_string(), "A2.cs".to_string())],
        "an entity declared in a configured additional root is the workspace's own"
    );
    assert_eq!(
        found_entities(&state, "SharedService", Some(&repo_b.key())),
        vec![
            ("SharedService".to_string(), "A2.cs".to_string()),
            ("SharedService".to_string(), "B.cs".to_string()),
        ],
        "the session holds ONE flat root configuration (no per-primary association), \
         so a configured additional root participates in every scoped query while an \
         unconfigured repository never does — which is why repo B is absent from \
         repo A's answer above"
    );
}

// ── RED-WSC4-3: one identity, two occurrences, both retained ─────────

#[test]
fn red_wsc4_3_same_identity_is_retained_and_scoped_by_occurrence() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    seed_generic(
        &state,
        &repo_a.file("A.cs"),
        SemanticRelation::Injects,
        "LoadingComponent",
        "Router",
    );
    seed_generic(
        &state,
        &repo_b.file("B.cs"),
        SemanticRelation::Injects,
        "LoadingComponent",
        "Router",
    );

    // The scope is a VIEW: one Model C identity, two occurrences, nothing dropped.
    let index = state.workspace_index_read();
    assert_eq!(
        index.find_entities_by_name("LoadingComponent").len(),
        2,
        "both occurrences of the shared identity stay in the index"
    );
    drop(index);

    // The same lookup, scoped, answers with the workspace's own occurrence only,
    // and the semantic identity it reports is unchanged (Model C).
    let scope = crate::workspace::scope::WorkspaceScope::new(Some(&repo_a.key()), &[])
        .expect("a declared root yields a scope");
    let index = state.workspace_index_read();
    let scoped = index.find_entities_by_name_in_scope("LoadingComponent", &scope);
    assert_eq!(scoped.len(), 1, "one in-scope occurrence of the identity");
    assert_eq!(scoped[0].name, "LoadingComponent");
    assert_eq!(scoped[0].entity_type.to_string(), "Class");
    assert_eq!(scoped[0].domain.to_string(), DOMAIN);
    assert!(
        scoped[0]
            .file
            .as_deref()
            .unwrap_or_default()
            .ends_with("A.cs"),
        "the surviving occurrence is the one repo A's file declared: {:?}",
        scoped[0].file
    );
}

// ── RED-WSC4-4: entities_in_file's explicit path IS its scope ────────

#[test]
fn red_wsc4_4_entities_in_file_is_already_bounded_by_its_explicit_path() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let file_a = repo_a.file("A.cs");
    let file_b = repo_b.file("B.cs");
    let state = state(&[]);
    seed_generic(&state, &file_a, SemanticRelation::Injects, "InA", "TargetA");
    seed_generic(&state, &file_b, SemanticRelation::Injects, "InB", "TargetB");

    // (a) The workspace's own declared file answers normally.
    assert_eq!(
        entities_in_file(&state, &file_a, Some(&repo_a.key())),
        vec![
            ("InA".to_string(), "A.cs".to_string()),
            ("TargetA".to_string(), "A.cs".to_string()),
        ],
        "the declared file of the queried workspace answers with its entities"
    );

    // (b) An unrelated repository's file is rejected even though it EXISTS and is
    // indexed in the session: the handler validates the explicit path through the
    // trusted-path/root boundary BEFORE consulting the index, so the explicit file
    // path already is the scope and no extra filtering is needed or added.
    assert!(
        entities_in_file(&state, &file_b, Some(&repo_a.key())).is_empty(),
        "a file outside the declared workspace roots must not be answerable, \
         even while its evidence is retained in the index"
    );

    // (c) The same file becomes answerable exactly when its repository is part of
    // the configured workspace (additional root) — i.e. the accepted set is
    // primary root plus configured additional roots, the same set `WorkspaceScope`
    // uses for occurrence provenance.
    let configured = super::workspace_query_scope::state(&[&repo_b]);
    seed_generic(
        &configured,
        &file_b,
        SemanticRelation::Injects,
        "InB",
        "TargetB",
    );
    assert_eq!(
        entities_in_file(&configured, &file_b, Some(&repo_a.key())),
        vec![
            ("InB".to_string(), "B.cs".to_string()),
            ("TargetB".to_string(), "B.cs".to_string()),
        ],
        "a configured additional root's file is inside the workspace boundary"
    );
}
