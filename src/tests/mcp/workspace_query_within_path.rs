// src/tests/mcp/workspace_query_within_path.rs
//
// `withinPath` — the OPTIONAL second scope layer of `workspace_query`.
//
// WSC-004 authorizes a workspace (`workspaceRoot` + configured
// `additional_roots`); `withinPath` then narrows that ALREADY authorized set to
// one file or directory subtree. The two layers compose:
//
//   effective scope = WorkspaceScope ∩ withinPath
//
// This file covers the occurrence-bearing ENTITY surfaces and owns the fixtures
// the sibling files import:
//
//   RED-WITHIN1  — entity narrowing across additional roots.
//   RED-WITHIN2  — same repository, two feature directories (so the feature is
//                  not merely another repository-root filter).
//   RED-WITHIN13 — equivalent canonical spellings select the same occurrences.
//   RED-WITHIN14 — an exact file narrows to that file alone.
//   plus `entities_in_file` against the narrowing.
//
// The edge-occurrence surfaces (RED-WITHIN3/4/11/12/15 and the root-less rule)
// live in the sibling `workspace_query_within_path_edges.rs`; the graph surfaces
// in `workspace_query_within_path_traversal.rs`; the TypeScript property-arrow
// case in `workspace_query_within_path_arrows.rs`.
//
// Every assertion goes through the REAL production path: MCP dispatch →
// `handle_workspace_query` → hydration → composed-scope WorkspaceIndex lookup.

use super::workspace_query_scope::{
    DOMAIN, Repo, call_fact, facts, seed_call, seed_generic, serialize, state,
};
use super::workspace_query_scope_entities::{entity_occurrences, structured};
use super::*;
use crate::dictionary::path::canonical_identity_key;
use crate::layers::meta::semantic::SemanticRelation;
use crate::mcp::McpState;

/// Create `name` (parent directories included) inside `repo` and return its
/// canonical file key — the form production uses for `asserting_file`.
pub(super) fn file(repo: &Repo, name: &str) -> String {
    let path = repo.root.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, "// withinPath fixture\n").unwrap();
    canonical_identity_key(&path.to_string_lossy())
}

/// The arguments of one query with the two scope arguments attached — each only
/// when the test declares it, so `None` means "argument absent".
pub(super) fn scoped(
    arguments: serde_json::Value,
    root: Option<&str>,
    within: Option<&str>,
) -> serde_json::Value {
    let mut arguments = arguments;
    if let Some(root) = root {
        arguments["workspaceRoot"] = json!(root);
    }
    if let Some(within) = within {
        arguments["withinPath"] = json!(within);
    }
    arguments
}

/// `find_entities` through the real dispatch path → `(name, owning-file)`.
pub(super) fn narrowed_entities(
    state: &McpState,
    name: &str,
    root: Option<&str>,
    within: Option<&str>,
) -> Vec<(String, String)> {
    let arguments = scoped(
        json!({ "type": "find_entities", "name": name }),
        root,
        within,
    );
    entity_occurrences(&structured(state, arguments))
}

/// `forward_edges` / `reverse_edges` through the real dispatch path →
/// `(subject, object, asserting-file NAME, argc)`.
pub(super) fn narrowed_edges(
    state: &McpState,
    query_type: &str,
    entity_type: &str,
    name: &str,
    root: Option<&str>,
    within: Option<&str>,
) -> Vec<(String, String, String, Option<u64>)> {
    let arguments = scoped(
        json!({
            "type": query_type,
            "domain": DOMAIN,
            "entity_type": entity_type,
            "name": name,
        }),
        root,
        within,
    );
    let sc = structured(state, arguments);
    facts(&sc["edges"])
}

/// `entities_in_file` through the real dispatch path → `(name, owning-file)`.
fn file_entities(
    state: &McpState,
    file_path: &str,
    root: Option<&str>,
    within: Option<&str>,
) -> Vec<(String, String)> {
    let arguments = scoped(
        json!({ "type": "entities_in_file", "file_path": file_path }),
        root,
        within,
    );
    entity_occurrences(&structured(state, arguments))
}

/// Dispatch one query and return the RAW response, so a refused argument can be
/// asserted (no `result`, a `-32602` error).
fn raw_response(state: &McpState, arguments: serde_json::Value) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({ "arguments": arguments }),
        state,
    );
    pop_response()
}

/// Assert that one query was REFUSED for its `withinPath`, and return the
/// message: an invalid parameter envelope, never a partial answer about the
/// workspace.
pub(super) fn refused(state: &McpState, arguments: serde_json::Value) -> String {
    let response = raw_response(state, arguments);
    assert!(
        response.get("result").is_none(),
        "a refused withinPath must not produce a result: {response}"
    );
    assert_eq!(
        response["error"]["code"].as_i64(),
        Some(-32602),
        "a refused withinPath is an invalid-parameter error: {response}"
    );
    let message = response["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(
        message.contains("withinPath"),
        "the refusal must name the offending argument: {message}"
    );
    message
}

// ── RED-WITHIN1: entity narrowing across additional roots ────────────

#[test]
fn red_within1_entity_narrowing_across_additional_roots() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    // Repo B is a CONFIGURED additional root, so both repositories are one
    // authorized workspace before any narrowing happens.
    let state = state(&[&repo_b]);
    seed_generic(
        &state,
        &repo_a.file("A.cs"),
        SemanticRelation::Injects,
        "Service",
        "ClockA",
    );
    seed_generic(
        &state,
        &repo_b.file("B.cs"),
        SemanticRelation::Injects,
        "Service",
        "ClockB",
    );

    // Unfiltered: the authorized workspace is primary + additional root.
    assert_eq!(
        narrowed_entities(&state, "Service", Some(&repo_a.key()), None),
        vec![
            ("Service".to_string(), "A.cs".to_string()),
            ("Service".to_string(), "B.cs".to_string()),
        ],
        "the authorized workspace contains both repositories before narrowing"
    );

    // Narrowing selects exactly one of them, in either direction.
    assert_eq!(
        narrowed_entities(&state, "Service", Some(&repo_a.key()), Some(&repo_a.key())),
        vec![("Service".to_string(), "A.cs".to_string())],
        "withinPath = repo A selects repo A's occurrence only"
    );
    assert_eq!(
        narrowed_entities(&state, "Service", Some(&repo_a.key()), Some(&repo_b.key())),
        vec![("Service".to_string(), "B.cs".to_string())],
        "withinPath = repo B selects repo B's occurrence only, because WSC-004 \
         authorizes it as a configured additional root"
    );
}

// ── RED-WITHIN2: one repository, two feature directories ─────────────

#[test]
fn red_within2_same_repository_different_feature_directories() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let billing = file(&repo, "src/Billing/Service.ts");
    let shipping = file(&repo, "src/Shipping/Service.ts");
    let state = state(&[]);
    seed_generic(
        &state,
        &billing,
        SemanticRelation::Injects,
        "Service",
        "BillingClock",
    );
    seed_generic(
        &state,
        &shipping,
        SemanticRelation::Injects,
        "Service",
        "ShippingClock",
    );

    assert_eq!(
        narrowed_entities(&state, "Service", Some(&repo.key()), None),
        vec![
            ("Service".to_string(), "Service.ts".to_string()),
            ("Service".to_string(), "Service.ts".to_string()),
        ],
        "one repository, two occurrences of one Model C identity"
    );
    assert_eq!(
        narrowed_entities(&state, "Service", Some(&repo.key()), Some("src/Billing")),
        vec![("Service".to_string(), "Service.ts".to_string())],
        "a feature-directory narrowing is finer than the repository root"
    );
    assert_eq!(
        narrowed_entities(&state, "Service", Some(&repo.key()), Some("src/Shipping")),
        vec![("Service".to_string(), "Service.ts".to_string())],
        "the other feature directory answers for its own occurrence"
    );

    // The two answers are distinguishable only by provenance, which is the whole
    // point: Model C identity is identical in both.
    let billing_entities = narrowed_entities(&state, "Service", Some(&repo.key()), Some(&billing));
    assert_eq!(
        billing_entities,
        vec![("Service".to_string(), "Service.ts".to_string())],
        "the exact-file spelling of the same narrowing agrees"
    );
}

// ── RED-WITHIN13: canonicalization parity ────────────────────────────

#[test]
fn red_within13_equivalent_canonical_spellings_select_the_same_occurrences() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let feature_a = file(&repo, "src/feature-a/a.ts");
    let feature_b = file(&repo, "src/feature-b/b.ts");
    let state = state(&[]);
    seed_call(&state, &feature_a, "LoadA", "save", 1);
    seed_call(&state, &feature_b, "LoadB", "save", 1);

    let expected = vec![call_fact("LoadA", "save", "a.ts", 1)];
    let spellings = [
        // relative to the declared workspace root (the documented rule)
        "src/feature-a".to_string(),
        // absolute, in the platform's canonical form
        repo.root
            .join("src")
            .join("feature-a")
            .to_string_lossy()
            .into_owned(),
        // absolute with a `..` segment that canonicalization removes
        repo.root
            .join("src")
            .join("feature-b")
            .join("..")
            .join("feature-a")
            .to_string_lossy()
            .into_owned(),
    ];
    for spelling in &spellings {
        assert_eq!(
            narrowed_edges(
                &state,
                "reverse_edges",
                "Method",
                "save",
                Some(&repo.key()),
                Some(spelling)
            ),
            expected,
            "every equivalent spelling of one path must select one occurrence set: {spelling}"
        );
    }
    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "save",
            Some(&repo.key()),
            None
        )
        .len(),
        2,
        "the three spellings were compared against a two-occurrence workspace"
    );
}

// ─ RED-WITHIN14: exact-file narrowing ───────────────────────────────

#[test]
fn red_within14_an_exact_file_narrows_to_that_file() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let service = file(&repo, "src/Billing/Service.ts");
    let other = file(&repo, "src/Billing/Other.ts");
    let state = state(&[]);
    seed_generic(
        &state,
        &service,
        SemanticRelation::Injects,
        "Service",
        "Clock",
    );
    seed_generic(&state, &other, SemanticRelation::Injects, "Helper", "Clock");
    seed_call(&state, &service, "Service", "save", 1);
    seed_call(&state, &other, "Helper", "save", 1);

    // The directory admits both files…
    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "save",
            Some(&repo.key()),
            Some("src/Billing")
        )
        .len(),
        2,
        "the directory narrowing admits every file beneath it"
    );
    // …and the exact file admits only the occurrences it asserted.
    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "save",
            Some(&repo.key()),
            Some("src/Billing/Service.ts")
        ),
        vec![call_fact("Service", "save", "Service.ts", 1)],
        "an exact-file narrowing keeps only that file's asserting occurrences"
    );
    assert!(
        narrowed_entities(
            &state,
            "Helper",
            Some(&repo.key()),
            Some("src/Billing/Service.ts")
        )
        .is_empty(),
        "an entity declared in another file of the same directory is excluded"
    );
    // The relative and absolute spellings of the same file agree.
    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "save",
            Some(&repo.key()),
            Some(&service)
        ),
        vec![call_fact("Service", "save", "Service.ts", 1)],
        "the absolute spelling of the exact file agrees with the relative one"
    );
}

// ── `entities_in_file` against the narrowing ─────────────────────────

#[test]
fn entities_in_file_is_validated_against_within_path() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let billing = file(&repo, "src/Billing/Service.ts");
    let shipping = file(&repo, "src/Shipping/Service.ts");
    let state = state(&[]);
    seed_generic(
        &state,
        &billing,
        SemanticRelation::Injects,
        "BillingService",
        "BillingClock",
    );
    seed_generic(
        &state,
        &shipping,
        SemanticRelation::Injects,
        "ShippingService",
        "ShippingClock",
    );

    // No narrowing: the explicit path is its own scope, exactly as before.
    assert_eq!(
        file_entities(&state, &billing, Some(&repo.key()), None),
        vec![
            ("BillingClock".to_string(), "Service.ts".to_string()),
            ("BillingService".to_string(), "Service.ts".to_string()),
        ],
        "the explicit file's own entities are unchanged without a narrowing"
    );

    // The file inside the narrowing answers…
    assert_eq!(
        file_entities(&state, &billing, Some(&repo.key()), Some("src/Billing")).len(),
        2,
        "a file inside the narrowing answers normally"
    );
    // …and the file outside it is answered with the minimal zero-result shape,
    // exactly like a file outside the workspace.
    assert!(
        file_entities(&state, &shipping, Some(&repo.key()), Some("src/Billing")).is_empty(),
        "a workspace file outside the narrowing must not be answerable through it"
    );
    assert_eq!(
        file_entities(&state, &shipping, Some(&repo.key()), Some("src/Shipping")).len(),
        2,
        "the same file answers when the narrowing contains it"
    );
}
