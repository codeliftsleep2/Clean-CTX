// src/tests/mcp/workspace_query_scope_traversal.rs
//
// WSC-004 — workspace-scoped GRAPH TRAVERSAL (`transitive_dependencies`,
// `has_cycle`).
//
// Sibling of `workspace_query_scope_entities.rs` (find_entities /
// entities_in_file), which owns the shared query helpers used below.
//
//   RED-WSC4-5  — a scoped transitive walk reflects only the workspace's own
//                 evidence, even when another repository shares the same names.
//   RED-WSC4-6  — a scoped walk never passes THROUGH an unrelated repository
//                 (the case that final-result filtering alone cannot fix).
//   RED-WSC4-7  — relationships across the primary and an additional root stay
//                 traversable.
//   RED-WSC4-8  — a cycle cannot be manufactured by combining two repositories.
//   RED-WSC4-9  — a cycle that really exists inside the workspace is detected.
//   RED-WSC4-10 — a cycle spanning primary + additional root is detected.
//   RED-WSC4-11 — a root-less query keeps the global view on every scoped
//                 surface (find_entities, transitive_dependencies, has_cycle).
//   RED-WSC4-12 — repository-name prefix safety (`repo` vs `repo-old`).
//   plus the start-occurrence guard: a scoped walk begins only at an entity the
//   active workspace actually contains.
//
// Scope is applied DURING traversal, never to the final result: reachability and
// cycles may only ever be built from one workspace's evidence.

use super::workspace_query_scope::{DOMAIN, Repo, insert, seed_generic, serialize, state};
use super::workspace_query_scope_entities::{cycle, dependencies, found_entities};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

/// One dependency edge `subject -> object`, with each endpoint's own provenance
/// set explicitly (so a test can express an endpoint declared in another
/// repository). The asserting file belongs to the caller, which passes the edge
/// to `insert`.
fn dependency_edge(
    subject_file: &str,
    subject: &str,
    object_file: &str,
    object: &str,
) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new(DOMAIN, "Class", subject).with_file(subject_file.to_string()),
        object: EntityRef::new(DOMAIN, "Class", object).with_file(object_file.to_string()),
        layer: DOMAIN,
        call_evidence: None,
    }
}

// ── RED-WSC4-5: scoped reachability uses one repository's evidence ───

#[test]
fn red_wsc4_5_scoped_traversal_reflects_only_the_workspaces_own_chain() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    // repo A: Alpha -> Beta -> Gamma
    seed_generic(
        &state,
        &repo_a.file("A1.cs"),
        SemanticRelation::Injects,
        "Alpha",
        "Beta",
    );
    seed_generic(
        &state,
        &repo_a.file("A2.cs"),
        SemanticRelation::Injects,
        "Beta",
        "Gamma",
    );
    // repo B reuses those names and would extend / alter the chain if the walk
    // were computed over the global graph.
    seed_generic(
        &state,
        &repo_b.file("B1.cs"),
        SemanticRelation::Injects,
        "Gamma",
        "Delta",
    );
    seed_generic(
        &state,
        &repo_b.file("B2.cs"),
        SemanticRelation::Injects,
        "Alpha",
        "Epsilon",
    );

    assert_eq!(
        dependencies(&state, "Alpha", 0, Some(&repo_a.key())),
        vec!["Beta".to_string(), "Gamma".to_string()],
        "A's scoped reachability is exactly A's own chain"
    );
    assert_eq!(
        dependencies(&state, "Alpha", 0, None),
        vec![
            "Beta".to_string(),
            "Delta".to_string(),
            "Epsilon".to_string(),
            "Gamma".to_string(),
        ],
        "a root-less query keeps the global session view"
    );
}

// ── RED-WSC4-6: no traversal THROUGH an unrelated repository ─────────

#[test]
fn red_wsc4_6_scoped_traversal_never_passes_through_an_unrelated_repository() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    // repo A: Alpha -> Xi;  repo B: Xi -> Ypsi.
    seed_generic(
        &state,
        &repo_a.file("A1.cs"),
        SemanticRelation::Injects,
        "Alpha",
        "Xi",
    );
    seed_generic(
        &state,
        &repo_b.file("B1.cs"),
        SemanticRelation::Injects,
        "Xi",
        "Ypsi",
    );

    assert_eq!(
        dependencies(&state, "Alpha", 0, Some(&repo_a.key())),
        vec!["Xi".to_string()],
        "filtering only the FINAL result would still have walked through Xi and \
         reported Ypsi, a dependency repo A never asserted"
    );
    assert_eq!(
        dependencies(&state, "Alpha", 0, None),
        vec!["Xi".to_string(), "Ypsi".to_string()],
        "the unscoped walk still reports the combined session reachability"
    );
}

// ── RED-WSC4-7: primary + additional root form one workspace ─────────

#[test]
fn red_wsc4_7_edges_across_primary_and_additional_root_stay_traversable() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_a2 = Repo::inside(base.path(), "repo-a-additional");
    let file_a = repo_a.file("A1.cs");
    let file_a2 = repo_a2.file("A2.cs");

    let state = state(&[&repo_a2]);
    seed_generic(&state, &file_a, SemanticRelation::Injects, "Alpha", "Svc");
    seed_generic(&state, &file_a2, SemanticRelation::Injects, "Svc", "Repo");

    assert_eq!(
        dependencies(&state, "Alpha", 0, Some(&repo_a.key())),
        vec!["Repo".to_string(), "Svc".to_string()],
        "the primary root's and the configured additional root's evidence are ONE workspace"
    );

    // Control: with the additional root NOT configured, the second hop is not
    // part of the workspace and the walk stops after the first hop. (`state` is
    // shadowed by the binding above, so the fixture constructor is called through
    // its module.)
    let isolated = super::workspace_query_scope::state(&[]);
    seed_generic(
        &isolated,
        &file_a,
        SemanticRelation::Injects,
        "Alpha",
        "Svc",
    );
    seed_generic(
        &isolated,
        &file_a2,
        SemanticRelation::Injects,
        "Svc",
        "Repo",
    );
    assert_eq!(
        dependencies(&isolated, "Alpha", 0, Some(&repo_a.key())),
        vec!["Svc".to_string()],
        "an unconfigured repository's edge must not be traversable"
    );
}

// ── RED-WSC4-8: no cycle manufactured from two repositories ──────────

#[test]
fn red_wsc4_8_a_cycle_cannot_be_manufactured_by_mixing_two_repositories() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    // Each repository contributes one half of what would be a cycle.
    seed_generic(
        &state,
        &repo_a.file("A1.cs"),
        SemanticRelation::Injects,
        "Alpha",
        "Beta",
    );
    seed_generic(
        &state,
        &repo_b.file("B1.cs"),
        SemanticRelation::Injects,
        "Beta",
        "Alpha",
    );

    assert!(
        !cycle(&state, Some(&repo_a.key())),
        "repo A alone is acyclic: the other half belongs to another workspace"
    );
    assert!(
        !cycle(&state, Some(&repo_b.key())),
        "repo B alone is acyclic for the same reason"
    );
    assert!(
        cycle(&state, None),
        "the unscoped session view still sees the cycle its combined evidence forms"
    );
}

// ── RED-WSC4-9: a real scoped cycle is preserved ─────────────────────

#[test]
fn red_wsc4_9_a_cycle_inside_the_workspace_is_still_detected() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let state = state(&[]);
    seed_generic(
        &state,
        &repo_a.file("A1.cs"),
        SemanticRelation::Injects,
        "Alpha",
        "Beta",
    );
    seed_generic(
        &state,
        &repo_a.file("A2.cs"),
        SemanticRelation::Injects,
        "Beta",
        "Alpha",
    );

    assert!(
        cycle(&state, Some(&repo_a.key())),
        "a cycle whose both halves are the workspace's own evidence must be reported"
    );
    assert!(cycle(&state, None), "and it is a cycle globally too");
}

// ── RED-WSC4-10: a cycle across primary + additional root ────────────

#[test]
fn red_wsc4_10_a_cycle_spanning_primary_and_additional_root_is_detected() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_a2 = Repo::inside(base.path(), "repo-a-additional");
    let file_a = repo_a.file("A1.cs");
    let file_a2 = repo_a2.file("A2.cs");

    let state = state(&[&repo_a2]);
    seed_generic(&state, &file_a, SemanticRelation::Injects, "Alpha", "Svc");
    seed_generic(&state, &file_a2, SemanticRelation::Injects, "Svc", "Alpha");
    assert!(
        cycle(&state, Some(&repo_a.key())),
        "both halves are the same workspace's evidence (primary + configured \
         additional root), so the cycle is real"
    );

    let isolated = super::workspace_query_scope::state(&[]);
    seed_generic(
        &isolated,
        &file_a,
        SemanticRelation::Injects,
        "Alpha",
        "Svc",
    );
    seed_generic(
        &isolated,
        &file_a2,
        SemanticRelation::Injects,
        "Svc",
        "Alpha",
    );
    assert!(
        !cycle(&isolated, Some(&repo_a.key())),
        "with the additional root unconfigured, its half is another repository's \
         evidence and cannot close a cycle here"
    );
}

// ── RED-WSC4-11: root-less non-regression on every scoped surface ────

#[test]
fn red_wsc4_11_root_less_queries_keep_the_global_view_on_every_scoped_surface() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    seed_generic(
        &state,
        &repo_a.file("A1.cs"),
        SemanticRelation::Injects,
        "Alpha",
        "Beta",
    );
    seed_generic(
        &state,
        &repo_b.file("B1.cs"),
        SemanticRelation::Injects,
        "Beta",
        "Gamma",
    );
    seed_generic(
        &state,
        &repo_b.file("B2.cs"),
        SemanticRelation::Injects,
        "Gamma",
        "Alpha",
    );

    // Root-less: the whole session's evidence, including the hop and the closing
    // edge that only repo B authored. (The BFS never re-reports the start identity,
    // even though the graph loops back to it.)
    assert_eq!(
        found_entities(&state, "Alpha", None),
        vec![
            ("Alpha".to_string(), "A1.cs".to_string()),
            ("Alpha".to_string(), "B2.cs".to_string()),
        ],
        "find_entities without a root answers with every occurrence in the session"
    );
    assert_eq!(
        dependencies(&state, "Alpha", 0, None),
        vec!["Beta".to_string(), "Gamma".to_string()],
        "transitive_dependencies without a root walks the whole session graph"
    );
    assert!(
        cycle(&state, None),
        "has_cycle without a root reports the global graph"
    );

    // Control: the same fixture, scoped, is constrained on all three surfaces —
    // so the root-less behaviour above is preservation, not scope leakage.
    assert_eq!(
        found_entities(&state, "Alpha", Some(&repo_a.key())),
        vec![("Alpha".to_string(), "A1.cs".to_string())],
        "find_entities scoped to A sees A's occurrence only"
    );
    assert_eq!(
        dependencies(&state, "Alpha", 0, Some(&repo_a.key())),
        vec!["Beta".to_string()],
        "the scoped walk stops where A's evidence stops"
    );
    assert!(
        !cycle(&state, Some(&repo_a.key())),
        "and no cycle is reported for A alone"
    );
}

// ── RED-WSC4-12: repository-name prefix safety ───────────────────────

#[test]
fn red_wsc4_12_repository_name_prefix_never_admits_a_sibling_repository() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let older = Repo::inside(base.path(), "repo-old");
    let state = state(&[]);
    seed_generic(
        &state,
        &older.file("Old1.cs"),
        SemanticRelation::Injects,
        "OldAlpha",
        "OldBeta",
    );
    seed_generic(
        &state,
        &older.file("Old2.cs"),
        SemanticRelation::Injects,
        "OldBeta",
        "OldAlpha",
    );

    assert!(
        older.key().starts_with(&repo.key()),
        "the fixture must exercise the string-prefix trap: {} starts with {}",
        older.key(),
        repo.key()
    );
    assert!(
        found_entities(&state, "OldAlpha", Some(&repo.key())).is_empty(),
        "entity occurrences of a sibling repository must not be admitted"
    );
    assert!(
        dependencies(&state, "OldAlpha", 0, Some(&repo.key())).is_empty(),
        "reachability of a sibling repository must not be admitted"
    );
    assert!(
        !cycle(&state, Some(&repo.key())),
        "a sibling repository's edges must not close a cycle here"
    );

    // The sibling repository still answers for itself: the boundary separates
    // repositories, it never hides them from their own queries.
    assert_eq!(
        found_entities(&state, "OldAlpha", Some(&older.key())),
        vec![
            ("OldAlpha".to_string(), "Old1.cs".to_string()),
            ("OldAlpha".to_string(), "Old2.cs".to_string()),
        ],
        "the sibling repository answers its own scoped query"
    );
    assert!(
        cycle(&state, Some(&older.key())),
        "and reports the cycle its own evidence forms"
    );
}

// ── Start-occurrence guard ───────────────────────────────────────────

#[test]
fn red_wsc4_gate_scoped_traversal_starts_only_at_a_workspace_entity() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let file_a = repo_a.file("A1.cs");
    let file_b = repo_b.file("B1.cs");
    let state = state(&[]);
    // repo A asserts `Xi -> Zeta`, while Xi's own occurrence is declared in repo B:
    // the FACT is A's evidence, but Xi itself is not an entity of A.
    insert(
        &state,
        &file_a,
        dependency_edge(&file_b, "Xi", &file_a, "Zeta"),
    );
    assert!(
        dependencies(&state, "Xi", 0, Some(&repo_a.key())).is_empty(),
        "a scoped walk must not begin at an entity the workspace does not contain"
    );
    assert_eq!(
        dependencies(&state, "Xi", 0, None),
        vec!["Zeta".to_string()],
        "the unscoped walk still follows the fact the session holds"
    );

    // Control: the same fact with an IN-SCOPE start occurrence is traversed, so
    // the guard constrains the start identity without over-restricting it.
    let in_scope = super::workspace_query_scope::state(&[]);
    insert(
        &in_scope,
        &file_a,
        dependency_edge(&file_a, "Xi", &file_a, "Zeta"),
    );
    assert_eq!(
        dependencies(&in_scope, "Xi", 0, Some(&repo_a.key())),
        vec!["Zeta".to_string()],
        "the guard must not block a start the workspace does contain"
    );
}
