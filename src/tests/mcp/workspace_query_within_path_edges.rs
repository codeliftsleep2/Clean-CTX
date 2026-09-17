// src/tests/mcp/workspace_query_within_path_edges.rs
//
// `withinPath` on the EDGE-occurrence surfaces — the sibling of
// `workspace_query_within_path.rs`, which owns the shared fixtures and the
// entity-surface regressions.
//
//   RED-WITHIN3  — reverse_edges narrows by ASSERTING file.
//   RED-WITHIN4  — forward_edges narrows by ASSERTING file.
//   RED-WITHIN11 — a path outside the authorized roots is refused, and no
//                  unrelated fact is exposed (on every surface).
//   RED-WITHIN12 — `feature` never admits `feature-old` (component ancestry).
//   RED-WITHIN15 — no `withinPath` ⇒ every WSC-004 behaviour is unchanged, and
//                  the option can only ever narrow, never widen.
//   plus the root-less rule: `withinPath` without an explicit `workspaceRoot` is
//   refused rather than becoming an authorization root.

use super::workspace_query_scope::{
    DOMAIN, Repo, call_fact, facts, seed_call, seed_generic, serialize, state,
};
use super::workspace_query_scope_entities::structured;
use super::workspace_query_within_path::{
    file, narrowed_edges, narrowed_entities, refused, scoped,
};
use super::*;
use crate::dictionary::path::canonical_identity_key;
use crate::layers::meta::semantic::SemanticRelation;

// ── RED-WITHIN3: reverse_edges narrows by ASSERTING file ─────────────

#[test]
fn red_within3_reverse_edges_narrows_by_asserting_file() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let billing = file(&repo, "src/Billing/BillingService.ts");
    let shipping = file(&repo, "src/Shipping/ShippingService.ts");
    let state = state(&[]);
    seed_call(&state, &billing, "BillingLoad", "OrderBy", 2);
    seed_call(&state, &shipping, "ShippingLoad", "OrderBy", 2);

    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo.key()),
            None
        ),
        vec![
            call_fact("BillingLoad", "OrderBy", "BillingService.ts", 2),
            call_fact("ShippingLoad", "OrderBy", "ShippingService.ts", 2),
        ],
        "both in-scope paths assert a call to the same target"
    );
    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo.key()),
            Some("src/Billing")
        ),
        vec![call_fact("BillingLoad", "OrderBy", "BillingService.ts", 2)],
        "narrowing keeps only the occurrences asserted inside the path"
    );
    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo.key()),
            Some("src/Shipping")
        ),
        vec![call_fact(
            "ShippingLoad",
            "OrderBy",
            "ShippingService.ts",
            2
        )],
        "the sibling path answers for its own asserting file"
    );

    // A narrowing whose directory is in scope but asserts nothing: the answer is
    // empty, which proves the boundary is the ASSERTING file (the callee `OrderBy`
    // has no local declaration at all, so no path could admit it as a caller).
    file(&repo, "src/App/Root.ts");
    assert!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo.key()),
            Some("src/App")
        )
        .is_empty(),
        "an in-scope path that asserts nothing answers with nothing"
    );
}

// ── RED-WITHIN4: forward_edges narrows by ASSERTING file ─────────────

#[test]
fn red_within4_forward_edges_narrows_by_asserting_file() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let billing = file(&repo, "src/Billing/Process.ts");
    let shipping = file(&repo, "src/Shipping/Process.ts");
    let state = state(&[]);
    seed_call(&state, &billing, "Process", "ChargeCard", 1);
    seed_call(&state, &shipping, "Process", "ShipParcel", 1);

    assert_eq!(
        narrowed_edges(
            &state,
            "forward_edges",
            "Method",
            "Process",
            Some(&repo.key()),
            None
        ),
        vec![
            call_fact("Process", "ChargeCard", "Process.ts", 1),
            call_fact("Process", "ShipParcel", "Process.ts", 1),
        ],
        "one semantic caller identity, two asserting files"
    );
    assert_eq!(
        narrowed_edges(
            &state,
            "forward_edges",
            "Method",
            "Process",
            Some(&repo.key()),
            Some("src/Billing")
        ),
        vec![call_fact("Process", "ChargeCard", "Process.ts", 1)],
        "forward narrowing returns only the edges asserted inside the path"
    );
    assert_eq!(
        narrowed_edges(
            &state,
            "forward_edges",
            "Method",
            "Process",
            Some(&repo.key()),
            Some("src/Shipping")
        ),
        vec![call_fact("Process", "ShipParcel", "Process.ts", 1)],
        "the sibling path answers for its own asserting file"
    );
}
// ── RED-WITHIN11: an unauthorized path is refused ────────────────────

#[test]
fn red_within11_a_path_outside_the_workspace_is_refused() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    // NOT configured as an additional root: it is another repository, not part of
    // this workspace.
    let repo_b = Repo::inside(base.path(), "repo-b");
    let unrelated = tempfile::TempDir::new().unwrap();
    let state = state(&[]);
    seed_call(&state, &repo_a.file("A.cs"), "Process", "OrderBy", 2);
    seed_call(&state, &repo_b.file("B.cs"), "SortData", "OrderBy", 2);
    seed_call(
        &state,
        &canonical_identity_key(&unrelated.path().join("C.cs").to_string_lossy()),
        "OtherData",
        "OrderBy",
        2,
    );

    // The unrelated repository's facts EXIST in the session (proved by a
    // root-less query), so the refusals below are authorization, not absence.
    assert_eq!(
        facts(
            &structured(
                &state,
                json!({
                    "type": "reverse_edges",
                    "domain": DOMAIN,
                    "entity_type": "Method",
                    "name": "OrderBy",
                })
            )["edges"]
        )
        .len(),
        3,
        "all three repositories' facts are retained in the session"
    );

    let message = refused(
        &state,
        scoped(
            json!({
                "type": "reverse_edges",
                "domain": DOMAIN,
                "entity_type": "Method",
                "name": "OrderBy",
            }),
            Some(&repo_a.key()),
            Some(&repo_b.key()),
        ),
    );
    assert!(
        message.contains("outside the active workspace scope"),
        "the refusal explains WHY the path cannot be used: {message}"
    );
    assert!(
        message.contains("repo-b"),
        "the refusal must name the RESOLVED path legibly — re-joining path \
         components would render a Windows root as `C:/\\/Users/...`: {message}"
    );

    // An absolute path that is not under any authorized root is refused the same
    // way, on every occurrence-bearing surface.
    let stray = unrelated.path().to_string_lossy().into_owned();
    refused(
        &state,
        scoped(
            json!({ "type": "find_entities", "name": "SortData" }),
            Some(&repo_a.key()),
            Some(&stray),
        ),
    );
    refused(
        &state,
        scoped(
            json!({
                "type": "forward_edges",
                "domain": DOMAIN,
                "entity_type": "Method",
                "name": "SortData",
            }),
            Some(&repo_a.key()),
            Some(&stray),
        ),
    );
    refused(
        &state,
        scoped(
            json!({ "type": "has_cycle" }),
            Some(&repo_a.key()),
            Some(&stray),
        ),
    );

    // And the authorized workspace still answers normally afterwards: a refusal
    // is a property of the argument, not of the session.
    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "OrderBy",
            Some(&repo_a.key()),
            None
        ),
        vec![call_fact("Process", "OrderBy", "A.cs", 2)],
        "the authorized workspace is unaffected by the refusals"
    );
}

// ── RED-WITHIN12: path-component ancestry, never a string prefix ─────

#[test]
fn red_within12_feature_never_admits_feature_old() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let feature = file(&repo, "feature/component.ts");
    let older = file(&repo, "feature-old/component.ts");
    let state = state(&[]);
    seed_call(&state, &feature, "FeatureLoad", "save", 1);
    seed_call(&state, &older, "OldLoad", "save", 1);

    // The trap is at the DIRECTORY level: `root\feature` is a raw string prefix of
    // `root\feature-old`. A prefix check would admit the sibling directory's facts;
    // component-wise ancestry does not.
    let feature_dir = canonical_identity_key(&repo.root.join("feature").to_string_lossy());
    let older_dir = canonical_identity_key(&repo.root.join("feature-old").to_string_lossy());
    assert!(
        older_dir.starts_with(&feature_dir),
        "the fixture must exercise the string-prefix trap: {older_dir} starts with {feature_dir}"
    );
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
        "both directory names' facts are in scope before narrowing"
    );
    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "save",
            Some(&repo.key()),
            Some("feature")
        ),
        vec![call_fact("FeatureLoad", "save", "component.ts", 1)],
        "withinPath = 'feature' must not admit 'feature-old'"
    );
    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "save",
            Some(&repo.key()),
            Some("feature-old")
        ),
        vec![call_fact("OldLoad", "save", "component.ts", 1)],
        "the sibling directory is selectable on its own path"
    );
}

// ── RED-WITHIN15: no `withinPath` ⇒ WSC-004 behaviour unchanged ─────

#[test]
fn red_within15_without_within_path_every_wsc004_behaviour_is_unchanged() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo_a = Repo::inside(base.path(), "repo-a");
    let repo_a2 = Repo::inside(base.path(), "repo-a-additional");
    let repo_b = Repo::inside(base.path(), "repo-b");
    // Two feature directories in the primary root, one file in the configured
    // additional root, and one UNCONFIGURED repository.
    let billing = file(&repo_a, "src/Billing/Service.ts");
    let shipping = file(&repo_a, "src/Shipping/Service.ts");
    let additional = file(&repo_a2, "lib/Service.ts");
    let outside = file(&repo_b, "src/Service.ts");
    let state = state(&[&repo_a2]);
    for (path, caller) in [
        (&billing, "BillingLoad"),
        (&shipping, "ShippingLoad"),
        (&additional, "AdditionalLoad"),
        (&outside, "OutsideLoad"),
    ] {
        seed_call(&state, path, caller, "save", 1);
    }
    seed_generic(
        &state,
        &billing,
        SemanticRelation::Injects,
        "Service",
        "Clock",
    );

    // (a) Root only: the authorized workspace is primary + additional root, and
    // the unconfigured repository is excluded — the pre-`withinPath` answer.
    assert_eq!(
        narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "save",
            Some(&repo_a.key()),
            None
        ),
        vec![
            call_fact("AdditionalLoad", "save", "Service.ts", 1),
            call_fact("BillingLoad", "save", "Service.ts", 1),
            call_fact("ShippingLoad", "save", "Service.ts", 1),
        ],
        "without withinPath the answer is the whole authorized workspace"
    );
    assert!(
        !narrowed_edges(
            &state,
            "reverse_edges",
            "Method",
            "save",
            Some(&repo_a.key()),
            None
        )
        .contains(&call_fact("OutsideLoad", "save", "Service.ts", 1)),
        "an unconfigured repository stays excluded, exactly as with WSC-004 alone"
    );

    // (b) The same query WITH a narrowing is strictly smaller: the option only
    // ever removes occurrences, never adds them.
    let narrowed = narrowed_edges(
        &state,
        "reverse_edges",
        "Method",
        "save",
        Some(&repo_a.key()),
        Some("src/Billing"),
    );
    assert_eq!(
        narrowed,
        vec![call_fact("BillingLoad", "save", "Service.ts", 1)],
        "the narrowed answer is a subset of the authorized one"
    );

    // (c) Root-less (no workspaceRoot, no withinPath): the global session view,
    // untouched.
    assert!(
        narrowed_edges(&state, "reverse_edges", "Method", "save", None, None).len() == 4,
        "a root-less query keeps its global view"
    );
    assert_eq!(
        narrowed_entities(&state, "Service", None, None),
        vec![("Service".to_string(), "Service.ts".to_string())],
        "a root-less find_entities keeps the global view"
    );

    // (d) A root-less query WITHOUT `withinPath` must not become narrowed by the
    // mere presence of the argument in the tool schema.
    assert!(
        narrowed_edges(&state, "reverse_edges", "Method", "save", None, None).contains(&call_fact(
            "OutsideLoad",
            "save",
            "Service.ts",
            1
        )),
        "no narrowing is applied when the argument is absent"
    );
}

// ── `withinPath` requires an explicit `workspaceRoot` ────────────────

#[test]
fn within_path_without_a_workspace_root_is_refused() {
    let _serial = serialize();
    let base = tempfile::TempDir::new().unwrap();
    let repo = Repo::inside(base.path(), "repo");
    let file_key = file(&repo, "src/Billing/Service.ts");
    let state = state(&[]);
    seed_call(&state, &file_key, "BillingLoad", "save", 1);

    // Without a declared workspace there is no authorized set, so the argument
    // cannot narrow one — and it is refused rather than promoted into a root.
    let message = refused(
        &state,
        scoped(
            json!({ "type": "find_entities", "name": "BillingLoad" }),
            None,
            Some(&repo.key()),
        ),
    );
    assert!(
        message.contains("workspaceRoot"),
        "the refusal must name the missing root: {message}"
    );
    refused(
        &state,
        scoped(
            json!({
                "type": "reverse_edges",
                "domain": DOMAIN,
                "entity_type": "Method",
                "name": "save",
            }),
            None,
            Some("src/Billing"),
        ),
    );
    // The same query with no scope arguments at all is untouched: the refusal is
    // about the combination, never about the session.
    assert_eq!(
        narrowed_edges(&state, "reverse_edges", "Method", "save", None, None),
        vec![call_fact("BillingLoad", "save", "Service.ts", 1)],
        "a root-less, narrowing-less query keeps its previous behaviour"
    );
}
