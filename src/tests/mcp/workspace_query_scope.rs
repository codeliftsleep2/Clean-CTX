// src/tests/mcp/workspace_query_scope.rs
//
// Workspace-scoped `workspace_query` edge results — cross-repository isolation.
//
// Semantic identity is deliberately NOT the boundary here: `EntityKey` remains
// `(domain, entity_type, name)` (Model C), so two repositories may hold the very
// same semantic fact at once. What separates them is occurrence PROVENANCE — the
// canonical file that asserted the occurrence — and a query issued FOR a
// workspace must answer with that workspace's evidence only.
//
// Live defect this closes: `reverse_edges OrderBy` returned the wanted
// repository's real `Calls` facts PLUS three real `OrderBy(argc=2)` callers
// authored by an unrelated repository that merely happened to be indexed in the
// same session. Semantically correct, insufficiently scoped.
//
// Covers:
//   RED-SCOPE1  — reverse_edges isolates two repositories.
//   RED-SCOPE2  — a configured additional root belongs to the workspace.
//   RED-SCOPE3  — an unrelated third repository is excluded.
//   RED-SCOPE4  — identical semantic subject name across repositories.
//   RED-SCOPE5  — identical name AND arity across repositories (no dedup).
//   RED-SCOPE6  — forward_edges is isolated by the same rule.
//   RED-SCOPE7  — a non-Calls relation shares the isolation (generic layer).
//   RED-SCOPE8  — repository-name prefix safety (`repo` vs `repo-old`).
//   RED-SCOPE9  — equivalent root spellings select the same occurrences.
//   RED-SCOPE10 — removal/recompile never changes what a scope selects.
//   plus the object-provenance control (the CALLEE's file never gates
//   admission) and root-less-query preservation (an unscoped caller keeps its
//   previous behaviour).
//
// RED-SCOPE9–10 and the two controls live in the sibling module
// `workspace_query_scope_provenance.rs`, which imports the fixtures below.
//
// Every assertion is made through the REAL production path: MCP dispatch →
// `handle_workspace_query` → hydration → scoped WorkspaceIndex lookup.

use super::*;
use crate::dictionary::path::canonical_identity_key;
use crate::layers::meta::semantic::{CallEvidence, EntityRef, SemanticEdge, SemanticRelation};
use crate::mcp::McpState;
use std::path::{Path, PathBuf};

/// The semantic domain these fixtures use (framework-agnostic builtin facts).
pub(super) const DOMAIN: &str = "builtin";

/// The captured-response sink is process-global, so this module dispatches one
/// query at a time: the sink is cleared, one query is dispatched, one response is
/// popped. Serializing the module's own tests keeps them from consuming each
/// other's response.
pub(super) fn serialize() -> std::sync::MutexGuard<'static, ()> {
    static TEST_SERIALIZE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A repository fixture on disk — real directories, so canonical root identity
/// resolves exactly as it does in production.
pub(super) struct Repo {
    pub(super) root: PathBuf,
}

impl Repo {
    pub(super) fn inside(base: &Path, name: &str) -> Self {
        let root = base.join(name);
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    /// The canonical root identity: the same form production uses for both
    /// configured roots and `asserting_file` keys.
    pub(super) fn key(&self) -> String {
        canonical_identity_key(&self.root.to_string_lossy())
    }

    /// Create `name` inside this repository and return its canonical key.
    ///
    /// Fixture contents deliberately contain none of the queried names, so the
    /// hydration pass discovers no candidates and every assertion observes the
    /// seeded evidence only.
    pub(super) fn file(&self, name: &str) -> String {
        let path = self.root.join(name);
        std::fs::write(&path, "// scope fixture\n").unwrap();
        canonical_identity_key(&path.to_string_lossy())
    }
}

/// A state with CBM disabled, so discovery falls back to the deterministic
/// filesystem provider (candidate PATHS only — never relationships).
pub(super) fn state(additional: &[&Repo]) -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional.iter().map(|repo| repo.key()).collect();
    let state = McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}

/// Add one edge occurrence asserted by `asserting_file`.
pub(super) fn insert(state: &McpState, asserting_file: &str, edge: SemanticEdge) {
    let mut index = state.workspace_index_lock();
    index.add_edges(asserting_file, vec![edge]);
}

/// One call fact: `caller --Calls(argc)--> callee`, asserted by `asserting_file`.
pub(super) fn seed_call(
    state: &McpState,
    asserting_file: &str,
    caller: &str,
    callee: &str,
    argc: usize,
) {
    let edge = call_edge(asserting_file, caller, callee, argc, asserting_file);
    insert(state, asserting_file, edge);
}

/// One call fact whose CALLEE occurrence is declared in `callee_file` while the
/// fact itself is asserted by `asserting_file` (an external / cross-repository
/// callee declaration).
pub(super) fn call_edge(
    asserting_file: &str,
    caller: &str,
    callee: &str,
    argc: usize,
    callee_file: &str,
) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Calls,
        subject: EntityRef::new(DOMAIN, "Method", caller).with_file(asserting_file.to_string()),
        object: EntityRef::new(DOMAIN, "Method", callee).with_file(callee_file.to_string()),
        layer: DOMAIN,
        // These scope fixtures model exact calls only: the spread qualifier is
        // exercised by `src/tests/workspace/index_calls.rs` (RED-SPREAD6) and by
        // the end-to-end language regressions.
        call_evidence: Some(CallEvidence::new(argc, false)),
    }
}

/// One non-call relation occurrence between two `builtin` classes, asserted by
/// `asserting_file` — the genericity control (same shared lookup layer).
pub(super) fn seed_generic(
    state: &McpState,
    asserting_file: &str,
    relation: SemanticRelation,
    subject: &str,
    object: &str,
) {
    insert(
        state,
        asserting_file,
        SemanticEdge {
            relation,
            subject: EntityRef::new(DOMAIN, "Class", subject).with_file(asserting_file.to_string()),
            object: EntityRef::new(DOMAIN, "Class", object).with_file(asserting_file.to_string()),
            layer: DOMAIN,
            call_evidence: None,
        },
    );
}

/// Run an edge query through the REAL MCP dispatch path and return its edges.
///
/// `root` is the query's `workspaceRoot`; `None` omits the argument entirely
/// (a root-less query).
pub(super) fn edges(
    state: &McpState,
    query_type: &str,
    entity_type: &str,
    name: &str,
    root: Option<&str>,
) -> serde_json::Value {
    let mut arguments = json!({
        "type": query_type,
        "domain": DOMAIN,
        "entity_type": entity_type,
        "name": name,
    });
    if let Some(root) = root {
        arguments["workspaceRoot"] = json!(root);
    }
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
        "{query_type} must succeed: {response}"
    );
    response["result"]["structuredContent"]["edges"].clone()
}

/// `(subject, object, asserting-file NAME, argc)` for every returned edge, sorted.
///
/// The asserting file is reduced to its final path component so the assertions
/// stay independent of the platform's canonical path form while still proving
/// WHICH file authored the fact.
pub(super) fn facts(edges: &serde_json::Value) -> Vec<(String, String, String, Option<u64>)> {
    let mut facts: Vec<(String, String, String, Option<u64>)> = edges
        .as_array()
        .map(|edges| {
            edges
                .iter()
                .map(|edge| {
                    let file = edge["subject"]["file"].as_str().unwrap_or_default();
                    let file_name = file
                        .rsplit(['/', '\\'])
                        .next()
                        .unwrap_or_default()
                        .to_string();
                    (
                        edge["subject"]["name"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        edge["object"]["name"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        file_name,
                        edge["call_evidence"]["explicit_arg_count"].as_u64(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    facts.sort();
    facts
}

/// One expected call fact: `(caller, callee, asserting-file NAME, argc)`.
pub(super) fn call_fact(
    caller: &str,
    callee: &str,
    file: &str,
    argc: usize,
) -> (String, String, String, Option<u64>) {
    (
        caller.to_string(),
        callee.to_string(),
        file.to_string(),
        Some(argc as u64),
    )
}

// ── RED-SCOPE1: reverse_edges isolates two repositories ──────────────

#[test]
fn red_scope1_reverse_edges_is_isolated_per_repository() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    seed_call(&state, &repo_a.file("A.cs"), "Process", "OrderBy", 2);
    seed_call(&state, &repo_b.file("B.cs"), "SortData", "OrderBy", 2);

    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo_a.key())
        )),
        vec![call_fact("Process", "OrderBy", "A.cs", 2)],
        "a query scoped to repo A must answer with repo A's caller only"
    );
    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo_b.key())
        )),
        vec![call_fact("SortData", "OrderBy", "B.cs", 2)],
        "a query scoped to repo B must answer with repo B's caller only"
    );
}

// ── RED-SCOPE2: the workspace's additional roots are included ────────

#[test]
fn red_scope2_configured_additional_root_belongs_to_the_workspace() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_a2 = Repo::inside(base.path(), "repo-a2");
    let state = state(&[&repo_a2]);
    seed_call(&state, &repo_a.file("A.cs"), "Process", "OrderBy", 2);
    seed_call(&state, &repo_a2.file("A2.cs"), "Worker", "OrderBy", 0);

    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo_a.key())
        )),
        vec![
            call_fact("Process", "OrderBy", "A.cs", 2),
            call_fact("Worker", "OrderBy", "A2.cs", 0),
        ],
        "a caller in a configured additional root belongs to the primary workspace"
    );
}

// ── RED-SCOPE3: an unrelated third repository is excluded ────────────

#[test]
fn red_scope3_unrelated_third_repository_is_excluded() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_a2 = Repo::inside(base.path(), "repo-a2");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[&repo_a2]);
    seed_call(&state, &repo_a.file("A.cs"), "Process", "OrderBy", 2);
    seed_call(&state, &repo_a2.file("A2.cs"), "Worker", "OrderBy", 2);
    seed_call(&state, &repo_b.file("B.cs"), "SortData", "OrderBy", 2);

    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo_a.key())
        )),
        vec![
            call_fact("Process", "OrderBy", "A.cs", 2),
            call_fact("Worker", "OrderBy", "A2.cs", 2),
        ],
        "workspace A is its primary root plus its additional roots — nothing else"
    );
    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo_b.key())
        )),
        vec![
            call_fact("SortData", "OrderBy", "B.cs", 2),
            call_fact("Worker", "OrderBy", "A2.cs", 2),
        ],
        "repo B is still queryable, and it answers with its own occurrence plus the \
         configured additional root: the session holds ONE flat root configuration \
         (there is no per-primary association), so every configured additional root \
         participates in a scoped query. An UNCONFIGURED repository never does — \
         which is why repo B is absent from repo A's answer above."
    );
}

// ── RED-SCOPE4: identical semantic subject name across repositories ──

#[test]
fn red_scope4_same_subject_identity_is_separated_by_provenance() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    // Both repositories assert the SAME Model C identity, caller and callee.
    seed_call(&state, &repo_a.file("A.cs"), "Process", "Foo", 0);
    seed_call(&state, &repo_b.file("B.cs"), "Process", "Foo", 0);

    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "Foo",
            Some(&repo_a.key())
        )),
        vec![call_fact("Process", "Foo", "A.cs", 0)],
        "repo A returns exactly its own occurrence of the shared identity"
    );
    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "Foo",
            Some(&repo_b.key())
        )),
        vec![call_fact("Process", "Foo", "B.cs", 0)],
        "repo B returns exactly its own occurrence of the shared identity"
    );
}

// ── RED-SCOPE5: identical name AND arity across repositories ─────────

#[test]
fn red_scope5_same_name_and_arity_is_not_deduped_across_repositories() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    seed_call(&state, &repo_a.file("A.cs"), "Process", "OrderBy", 2);
    seed_call(&state, &repo_b.file("B.cs"), "Process", "OrderBy", 2);

    assert_eq!(
        state
            .workspace_index_read()
            .reverse_edges_by_identity(DOMAIN, "Method", "OrderBy")
            .len(),
        2,
        "both occurrences stay stored globally (the scope never deletes evidence)"
    );
    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo_a.key())
        )),
        vec![call_fact("Process", "OrderBy", "A.cs", 2)],
        "each scoped query returns exactly one occurrence"
    );
    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo_b.key())
        )),
        vec![call_fact("Process", "OrderBy", "B.cs", 2)],
        "each scoped query returns exactly one occurrence"
    );
}

// ─ RED-SCOPE6: forward_edges is isolated by the same rule ───────────

#[test]
fn red_scope6_forward_edges_is_isolated_per_repository() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    // One identical caller identity, asserting a DIFFERENT callee per repository.
    seed_call(&state, &repo_a.file("A.cs"), "Process", "Foo", 0);
    seed_call(&state, &repo_b.file("B.cs"), "Process", "Bar", 0);

    assert_eq!(
        facts(&edges(
            &state,
            "forward_edges",
            "Method",
            "Process",
            Some(&repo_a.key())
        )),
        vec![call_fact("Process", "Foo", "A.cs", 0)],
        "a same-named caller elsewhere must not contribute forward facts here"
    );
    assert_eq!(
        facts(&edges(
            &state,
            "forward_edges",
            "Method",
            "Process",
            Some(&repo_b.key())
        )),
        vec![call_fact("Process", "Bar", "B.cs", 0)],
        "repo B sees its own outgoing facts only"
    );
}

// ── RED-SCOPE7: a non-Calls relation shares the isolation ────────────

#[test]
fn red_scope7_non_calls_relation_shares_the_same_isolation() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    // The shared lookup layer is relation-agnostic: the proof uses a plain
    // framework relation, never a call fact.
    seed_generic(
        &state,
        &repo_a.file("A.cs"),
        SemanticRelation::Injects,
        "Client",
        "ApiClient",
    );
    seed_generic(
        &state,
        &repo_b.file("B.cs"),
        SemanticRelation::Injects,
        "Client",
        "ApiClient",
    );

    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Class",
            "ApiClient",
            Some(&repo_a.key())
        )),
        vec![(
            "Client".to_string(),
            "ApiClient".to_string(),
            "A.cs".to_string(),
            None
        )],
        "the isolation is generic, not Calls-specific"
    );
    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Class",
            "ApiClient",
            Some(&repo_b.key())
        )),
        vec![(
            "Client".to_string(),
            "ApiClient".to_string(),
            "B.cs".to_string(),
            None
        )],
        "repo B sees its own Injects occurrence only"
    );
}

// ── RED-SCOPE8: repository-name prefix safety ────────────────────────

#[test]
fn red_scope8_repository_name_prefix_does_not_admit_a_sibling_repository() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let older = Repo::inside(base.path(), "repo-old");
    let state = state(&[]);
    seed_call(&state, &older.file("Old.cs"), "Process", "OrderBy", 2);

    assert!(
        older.key().starts_with(&repo.key()),
        "the fixture must exercise the string-prefix trap: {} starts with {}",
        older.key(),
        repo.key()
    );
    assert!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo.key())
        ))
        .is_empty(),
        "an occurrence under 'repo-old' must never be admitted by root 'repo'"
    );
    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&older.key())
        )),
        vec![call_fact("Process", "OrderBy", "Old.cs", 2)],
        "the sibling repository is still queryable on its own root"
    );
}
