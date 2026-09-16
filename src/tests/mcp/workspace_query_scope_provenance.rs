// src/tests/mcp/workspace_query_scope_provenance.rs
//
// Workspace-scope root identity, provenance and lifecycle regressions.
//
// Sibling of `workspace_query_scope.rs`, which owns the fixtures and the
// cross-repository isolation regressions (RED-SCOPE1–8). This file continues the
// RED-SCOPE series:
//   RED-SCOPE9  — equivalent root spellings select the same occurrences.
//   RED-SCOPE10 — removal/recompile never changes what a scope selects.
//   plus the object-provenance control (the CALLEE's file never gates admission)
//   and root-less-query preservation (an unscoped caller keeps its behaviour).
//
// The whole point of these four is the CONTRACT boundary of the scope: which root
// spellings denote the same workspace, which occurrences a workspace owns, and
// which facts the scope must never consult.

use super::workspace_query_scope::{
    DOMAIN, Repo, call_edge, call_fact, edges, facts, insert, seed_call, seed_generic, serialize,
    state,
};
use crate::layers::meta::semantic::SemanticRelation;
use crate::mcp::McpState;

// ── RED-SCOPE9: canonicalization parity ──────────────────────────────

#[test]
fn red_scope9_equivalent_root_spellings_select_the_same_occurrences() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    std::fs::create_dir_all(repo.root.join("sub")).unwrap();
    let state = state(&[]);
    seed_call(&state, &repo.file("A.cs"), "Process", "OrderBy", 2);

    let canonical = repo.key();
    let expected = facts(&edges(
        &state,
        "reverse_edges",
        "Method",
        "OrderBy",
        Some(&canonical),
    ));
    assert_eq!(expected, vec![call_fact("Process", "OrderBy", "A.cs", 2)]);

    // A spelling that traverses through a real subdirectory and back.
    let via_parent = repo
        .root
        .join("sub")
        .join("..")
        .to_string_lossy()
        .into_owned();
    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&via_parent)
        )),
        expected,
        "equivalent root spellings must select the same occurrence set"
    );

    // The Windows verbatim form is normalized away by the shared canonical
    // helper, so it must denote the same workspace.
    #[cfg(windows)]
    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&format!(r"\\?\{}", canonical))
        )),
        expected,
        "a verbatim root spelling must select the same occurrence set"
    );
}

// ── RED-SCOPE10: removal and recompile never change the scope ────────

#[test]
fn red_scope10_removal_and_recompile_leave_scoping_intact() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    let a_file = repo_a.file("A.cs");
    let b_file = repo_b.file("B.cs");
    seed_call(&state, &a_file, "Process", "OrderBy", 2);
    seed_call(&state, &b_file, "SortData", "OrderBy", 2);

    let query = |state: &McpState, repo: &Repo| {
        facts(&edges(
            state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo.key()),
        ))
    };
    assert_eq!(
        query(&state, &repo_a),
        vec![call_fact("Process", "OrderBy", "A.cs", 2)]
    );
    assert_eq!(
        query(&state, &repo_b),
        vec![call_fact("SortData", "OrderBy", "B.cs", 2)]
    );

    // Remove repository A's file from the index (deletion / recompilation).
    state.workspace_index_lock().remove_file(&a_file);

    assert!(
        query(&state, &repo_a).is_empty(),
        "A's occurrence is gone and B's must not surface in its place"
    );
    assert_eq!(
        query(&state, &repo_b),
        vec![call_fact("SortData", "OrderBy", "B.cs", 2)],
        "B's occurrence is untouched by A's removal"
    );
    assert_eq!(
        state
            .workspace_index_read()
            .reverse_edges_by_identity(DOMAIN, "Method", "OrderBy")
            .len(),
        1,
        "removal stays occurrence-exact"
    );

    // Recompile repository A's file: its evidence returns, still scoped to A.
    seed_call(&state, &a_file, "Process", "OrderBy", 2);
    assert_eq!(
        query(&state, &repo_a),
        vec![call_fact("Process", "OrderBy", "A.cs", 2)]
    );
    assert_eq!(
        query(&state, &repo_b),
        vec![call_fact("SortData", "OrderBy", "B.cs", 2)]
    );
}

// ── Object provenance never gates admission ──────────────────────────

#[test]
fn red_scope_object_provenance_is_not_the_boundary() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    let a_file = repo_a.file("A.cs");
    let b_file = repo_b.file("B.cs");
    // The callee's occurrence is declared in ANOTHER repository (or may have no
    // local declaration at all). The fact is asserted by the CALLER's file, so it
    // belongs to the caller's workspace and must be admitted there.
    insert(
        &state,
        &a_file,
        call_edge(&a_file, "Process", "OrderBy", 2, &b_file),
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
        "a cross-repository callee declaration must not hide the caller's fact"
    );
    assert!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo_b.key())
        ))
        .is_empty(),
        "repo B asserted nothing, so it returns nothing"
    );
}

// ── A root-less query keeps its previous (unscoped) behaviour ────────

#[test]
fn red_scope_root_less_query_remains_unscoped() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    seed_call(&state, &repo_a.file("A.cs"), "Process", "OrderBy", 2);
    seed_call(&state, &repo_b.file("B.cs"), "SortData", "OrderBy", 2);

    assert_eq!(
        facts(&edges(&state, "reverse_edges", "Method", "OrderBy", None)),
        vec![
            call_fact("Process", "OrderBy", "A.cs", 2),
            call_fact("SortData", "OrderBy", "B.cs", 2),
        ],
        "a caller that declared no workspace has no scope to constrain the answer to"
    );
}

// ── The scope is relation-agnostic at the shared layer ───────────────

#[test]
fn red_scope_generic_layer_is_shared_with_non_call_relations() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_b = Repo::inside(base.path(), "repo-b");
    let state = state(&[]);
    seed_generic(
        &state,
        &repo_a.file("A.cs"),
        SemanticRelation::Implements,
        "PaymentService",
        "IPaymentGateway",
    );
    seed_generic(
        &state,
        &repo_b.file("B.cs"),
        SemanticRelation::Implements,
        "PaymentService",
        "IPaymentGateway",
    );

    assert_eq!(
        facts(&edges(
            &state,
            "reverse_edges",
            "Class",
            "IPaymentGateway",
            Some(&repo_a.key())
        )),
        vec![(
            "PaymentService".to_string(),
            "IPaymentGateway".to_string(),
            "A.cs".to_string(),
            None
        )],
        "a reverse occurrence asserted in repo B must not appear in repo A"
    );
    assert_eq!(
        facts(&edges(
            &state,
            "forward_edges",
            "Class",
            "PaymentService",
            Some(&repo_b.key())
        )),
        vec![(
            "PaymentService".to_string(),
            "IPaymentGateway".to_string(),
            "B.cs".to_string(),
            None
        )],
        "the same rule applies to forward edges of a non-call relation"
    );
}
