// src/tests/mcp/workspace_query_within_path_traversal.rs
//
// `withinPath` on the GRAPH surfaces: `transitive_dependencies` and `has_cycle`.
//
// Reachability and cycle membership are properties of the WALK, so the narrowing
// must be applied DURING traversal, never to the returned list: an edge asserted
// outside the selected path may neither extend a path nor close a cycle. This is
// the same placement rule WSC-004 already established for the workspace roots,
// composed with the second layer.
//
//   RED-WITHIN7  — a narrowed walk cannot escape the selected path.
//   RED-WITHIN8  — a cycle cannot be assembled from two paths.
//   RED-WITHIN9  — a cycle that really exists inside the path is still detected.
//   RED-WITHIN10 — a subtree of a configured additional root is selectable, with
//                  no primary-root preference introduced.
//   plus the narrowed start-occurrence guard.

use super::workspace_query_scope::{DOMAIN, Repo, seed_generic, serialize, state};
use super::workspace_query_scope_entities::structured;
use super::workspace_query_within_path::{file, scoped};
use super::*;
use crate::layers::meta::semantic::SemanticRelation;
use crate::mcp::McpState;

/// `transitive_dependencies` → the dependency entity NAMES, sorted.
fn dependencies(
    state: &McpState,
    name: &str,
    depth: i32,
    root: Option<&str>,
    within: Option<&str>,
) -> Vec<String> {
    let arguments = scoped(
        json!({
            "type": "transitive_dependencies",
            "domain": DOMAIN,
            "entity_type": "Class",
            "name": name,
            "depth": depth,
        }),
        root,
        within,
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
fn cycle(state: &McpState, root: Option<&str>, within: Option<&str>) -> bool {
    let sc = structured(state, scoped(json!({ "type": "has_cycle" }), root, within));
    assert!(
        sc.get("discovery").is_none() && sc.get("hydration_attempted").is_none(),
        "has_cycle must stay non-hydration-eligible: {sc:?}"
    );
    sc["has_cycle"].as_bool().expect("has_cycle boolean")
}

// ── RED-WITHIN7: a narrowed walk cannot escape the selected path ─────

#[test]
fn red_within7_traversal_cannot_escape_the_selected_path() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let feature_a = file(&repo, "feature-a/a.ts");
    let feature_b = file(&repo, "feature-b/b.ts");
    let state = state(&[]);
    // feature-a: A -> X.  feature-b: X -> Y.
    seed_generic(&state, &feature_a, SemanticRelation::Injects, "A", "X");
    seed_generic(&state, &feature_b, SemanticRelation::Injects, "X", "Y");

    assert_eq!(
        dependencies(&state, "A", 0, Some(&repo.key()), None),
        vec!["X".to_string(), "Y".to_string()],
        "the whole authorized workspace may reach Y"
    );
    assert_eq!(
        dependencies(&state, "A", 0, Some(&repo.key()), Some("feature-a")),
        vec!["X".to_string()],
        "withinPath = feature-a stops the walk at X: the hop out of the path is \
         not part of this narrowing"
    );
    assert_eq!(
        dependencies(&state, "A", 0, Some(&repo.key()), Some("feature-b")),
        Vec::<String>::new(),
        "feature-b does not author the first hop at all"
    );
    assert_eq!(
        dependencies(&state, "X", 0, Some(&repo.key()), Some("feature-b")),
        vec!["Y".to_string()],
        "the second hop is traversable when the selected path authors it"
    );
    // The narrowed start-occurrence guard: `X` has an in-path occurrence (it is
    // the object of feature-a's edge) yet its only OUTGOING edge is asserted
    // elsewhere, so the walk from X inside feature-a is empty rather than
    // escaping the path.
    assert!(
        dependencies(&state, "X", 0, Some(&repo.key()), Some("feature-a")).is_empty(),
        "an in-path start with out-of-path edges must not escape the narrowing"
    );
    assert_eq!(
        dependencies(&state, "A", 0, None, None),
        vec!["X".to_string(), "Y".to_string()],
        "a root-less walk keeps the global view"
    );
}
// ── RED-WITHIN8: a cycle cannot be assembled from two paths ──────────

#[test]
fn red_within8_a_cycle_cannot_combine_two_paths() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let feature_a = file(&repo, "feature-a/a.ts");
    let feature_b = file(&repo, "feature-b/b.ts");
    let state = state(&[]);
    // Each path contributes one half of what would be a cycle.
    seed_generic(&state, &feature_a, SemanticRelation::Injects, "A", "B");
    seed_generic(&state, &feature_b, SemanticRelation::Injects, "B", "A");

    assert!(
        cycle(&state, Some(&repo.key()), None),
        "the workspace as a whole contains the cycle its own evidence forms"
    );
    assert!(
        !cycle(&state, Some(&repo.key()), Some("feature-a")),
        "feature-a alone is acyclic: the closing half belongs to another path"
    );
    assert!(
        !cycle(&state, Some(&repo.key()), Some("feature-b")),
        "feature-b alone is acyclic for the same reason"
    );
    assert!(
        cycle(&state, None, None),
        "the root-less view still reports the cycle the session holds"
    );
}

// ── RED-WITHIN9: a real cycle inside the path is preserved ───────────

#[test]
fn red_within9_a_cycle_inside_the_selected_path_is_still_detected() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let feature_a = file(&repo, "feature-a/a1.ts");
    let feature_a2 = file(&repo, "feature-a/a2.ts");
    let feature_b = file(&repo, "feature-b/b.ts");
    let state = state(&[]);
    seed_generic(&state, &feature_a, SemanticRelation::Injects, "A", "B");
    seed_generic(&state, &feature_a2, SemanticRelation::Injects, "B", "A");
    // An edge asserted elsewhere must not change the verdict for feature-a.
    seed_generic(&state, &feature_b, SemanticRelation::Injects, "B", "C");

    assert!(
        cycle(&state, Some(&repo.key()), Some("feature-a")),
        "both halves of this cycle are asserted inside the selected path"
    );
    assert!(
        cycle(&state, Some(&repo.key()), None),
        "and it is a cycle for the whole workspace too"
    );
    assert!(
        !cycle(&state, Some(&repo.key()), Some("feature-b")),
        "the neighbouring path reports no cycle of its own"
    );
}
// ── RED-WITHIN10: a subtree of an additional root is selectable ──────

#[test]
fn red_within10_a_subtree_of_an_additional_root_is_selectable() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_a2 = Repo::inside(base.path(), "repo-a-additional");
    let primary_file = file(&repo_a, "src/app/handler.ts");
    let additional_file = file(&repo_a2, "lib/core/service.ts");
    let state = state(&[&repo_a2]);
    seed_generic(
        &state,
        &primary_file,
        SemanticRelation::Injects,
        "Alpha",
        "Beta",
    );
    seed_generic(
        &state,
        &additional_file,
        SemanticRelation::Injects,
        "Beta",
        "Gamma",
    );

    // WSC-004 authorization alone: the primary root's and the configured
    // additional root's evidence are ONE workspace.
    assert_eq!(
        dependencies(&state, "Alpha", 0, Some(&repo_a.key()), None),
        vec!["Beta".to_string(), "Gamma".to_string()],
        "the additional root is authorized before any narrowing"
    );

    // A SUBTREE of the additional root can be selected with no primary-root
    // preference: the additional root is neither special-cased nor shadowed by
    // the primary root.
    let additional_subtree = repo_a2
        .root
        .join("lib")
        .join("core")
        .to_string_lossy()
        .into_owned();
    assert_eq!(
        dependencies(
            &state,
            "Beta",
            0,
            Some(&repo_a.key()),
            Some(&additional_subtree)
        ),
        vec!["Gamma".to_string()],
        "an authorized additional-root subtree answers for its own evidence"
    );
    assert_eq!(
        dependencies(&state, "Alpha", 0, Some(&repo_a.key()), Some("src/app")),
        vec!["Beta".to_string()],
        "a primary-root subtree does not reach into the additional root's edge"
    );
    assert_eq!(
        dependencies(&state, "Beta", 0, Some(&repo_a.key()), Some(&repo_a2.key())),
        vec!["Gamma".to_string()],
        "the additional root as a whole behaves like its subtree"
    );

    // An exact FILE under the additional root that asserts nothing admits
    // nothing: the narrowing's unit is ASSERTING provenance, not containment.
    file(&repo_a2, "lib/core/unused.ts");
    assert!(
        dependencies(
            &state,
            "Beta",
            0,
            Some(&repo_a.key()),
            Some("lib/core/unused.ts")
        )
        .is_empty(),
        "an exact-file narrowing admits only that file's own asserting occurrences"
    );
}
