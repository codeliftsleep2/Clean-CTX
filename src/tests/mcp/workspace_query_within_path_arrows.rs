// src/tests/mcp/workspace_query_within_path_arrows.rs
//
// `withinPath` over REAL TypeScript call facts — the composition proof for
// same-named bound arrows inside one authorized workspace.
//
// The full production path is exercised in every assertion:
//
//   TypeScript source → candidate discovery → resolve_file_path_checked →
//   compile_file_ir_focused → bound-arrow callable scope → CoreOp::Call →
//   SemanticEdge::Calls → WorkspaceIndex → WSC-004 authorization → withinPath
//   narrowing → workspace_query response
//
//   RED-WITHIN5 — two paths declare the same Model C arrow identity
//                 (`load = () => …`); the narrowing selects exactly one
//                 provenance region while the identity is unchanged.
//   RED-WITHIN6 — a property arrow containing nested ANONYMOUS callbacks still
//                 narrows solely by the outer callable's asserting file (no path
//                 is ever assigned to the anonymous callback).
//   plus spread/arity evidence surviving the narrowing, and the asserting-file
//   boundary (the callee declaration's file authors nothing).
//
// CBM is disabled in every test (`config.cbm.enabled = false`), so the discovery
// provider can contribute candidate PATHS only and can never author a
// relationship.

use super::workspace_query_scope::{Repo, serialize, state};
use super::workspace_query_scope_entities::structured;
use super::workspace_query_within_path::scoped;
use super::*;
use crate::mcp::McpState;
use std::path::{Path, PathBuf};

/// A `Repo` view of a plain temporary workspace root, so the shared scope
/// fixtures (`state`, `Repo::key`) can be reused unchanged.
fn repo(path: &Path) -> Repo {
    Repo {
        root: path.to_path_buf(),
    }
}

/// Write one TypeScript file (creating parent directories) and return its path.
fn write_ts(root: &Path, name: &str, body: &str) -> PathBuf {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, body).unwrap();
    path
}

/// One `workspace_query` `reverse_edges` answer for `builtin` / `Method` /
/// `save`, through the real dispatch path (hydration included), scoped and
/// optionally narrowed.
fn edges(state: &McpState, root: &str, within: Option<&str>) -> serde_json::Value {
    let arguments = scoped(
        json!({
            "type": "reverse_edges",
            "domain": "builtin",
            "entity_type": "Method",
            "name": "save",
        }),
        Some(root),
        within,
    );
    structured(state, arguments)["edges"].clone()
}

/// `(caller name, asserting file NAME, argc, spread)` for every call edge.
///
/// The asserting file is reduced to its final path component so the assertions
/// stay independent of the platform's canonical spelling while still proving
/// WHICH file authored the fact. `None` for `spread` means the key is ABSENT,
/// which is how an exact call renders.
fn call_facts(value: &serde_json::Value) -> Vec<(String, String, Option<u64>, Option<bool>)> {
    let mut facts: Vec<(String, String, Option<u64>, Option<bool>)> = value
        .as_array()
        .map(|edges| {
            edges
                .iter()
                .filter(|edge| edge["relation"] == "Calls")
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
                        file_name,
                        edge["call_evidence"]["explicit_arg_count"].as_u64(),
                        edge["call_evidence"]["has_spread"].as_bool(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    facts.sort();
    facts
}
// ── RED-WITHIN5: same arrow identity in two paths ────────────────────

#[test]
fn red_within5_same_named_property_arrows_narrow_to_one_path() {
    let _serial = serialize();
    let root = tempfile::TempDir::new().unwrap();
    write_ts(
        root.path(),
        "feature-a/a.component.ts",
        r#"export class A {
    save(value: unknown): void { }

    load = () => { this.save('a'); };
}
"#,
    );
    write_ts(
        root.path(),
        "feature-b/b.component.ts",
        r#"export class B {
    save(value: unknown): void { }

    load = () => { this.save('b'); };
}
"#,
    );
    let state = state(&[]);
    let root_key = repo(root.path()).key();

    // The two arrows share one Model C identity (`builtin` / `Method` / `load`)
    // and one asserting pattern; only provenance separates them.
    assert_eq!(
        call_facts(&edges(&state, &root_key, None)),
        vec![
            (
                "load".to_string(),
                "a.component.ts".to_string(),
                Some(1),
                None
            ),
            (
                "load".to_string(),
                "b.component.ts".to_string(),
                Some(1),
                None
            ),
        ],
        "the authorized workspace holds both occurrences before narrowing"
    );

    assert_eq!(
        call_facts(&edges(&state, &root_key, Some("feature-a"))),
        vec![(
            "load".to_string(),
            "a.component.ts".to_string(),
            Some(1),
            None
        )],
        "withinPath = feature-a selects A's call occurrence only"
    );
    assert_eq!(
        call_facts(&edges(&state, &root_key, Some("feature-b"))),
        vec![(
            "load".to_string(),
            "b.component.ts".to_string(),
            Some(1),
            None
        )],
        "withinPath = feature-b selects B's call occurrence only"
    );

    // Identity is NOT path-qualified: the narrowed edge still reports the very
    // same Model C identity it reported unfiltered.
    let narrowed = edges(&state, &root_key, Some("feature-a"));
    let subject = &narrowed[0]["subject"];
    assert_eq!(subject["domain"].as_str(), Some("builtin"));
    assert_eq!(subject["entity_type"].as_str(), Some("Method"));
    assert_eq!(subject["name"].as_str(), Some("load"));
    assert!(
        subject["file"]
            .as_str()
            .unwrap_or_default()
            .ends_with("a.component.ts"),
        "provenance is the caller's file, never a synthetic path-qualified name: {subject}"
    );
}

// ── RED-WITHIN6: nested anonymous callbacks narrow by the OUTER arrow ─

#[test]
fn red_within6_nested_anonymous_callbacks_narrow_by_the_outer_arrow() {
    let _serial = serialize();
    let root = tempfile::TempDir::new().unwrap();
    let nested = r#"import { timer } from 'rxjs';

export class StreamStore {
    save(value: unknown): void { }

    // The anonymous `next`/`error` callbacks are NOT callable owners: their
    // invocations belong to the property arrow that contains them, and
    // provenance is that arrow's file.
    load = () => {
        timer(1).subscribe({
            next: () => { this.save('n'); },
            error: () => { this.save('e', 401); },
        });
    };
}
"#;
    write_ts(root.path(), "feature-a/a.stream.ts", nested);
    write_ts(root.path(), "feature-b/b.stream.ts", nested);
    let state = state(&[]);
    let root_key = repo(root.path()).key();

    assert_eq!(
        call_facts(&edges(&state, &root_key, None)),
        vec![
            ("load".to_string(), "a.stream.ts".to_string(), Some(1), None),
            ("load".to_string(), "a.stream.ts".to_string(), Some(2), None),
            ("load".to_string(), "b.stream.ts".to_string(), Some(1), None),
            ("load".to_string(), "b.stream.ts".to_string(), Some(2), None),
        ],
        "each nested invocation carries its own evidence, attributed to the arrow"
    );

    assert_eq!(
        call_facts(&edges(&state, &root_key, Some("feature-a"))),
        vec![
            ("load".to_string(), "a.stream.ts".to_string(), Some(1), None),
            ("load".to_string(), "a.stream.ts".to_string(), Some(2), None),
        ],
        "the narrowing keeps exactly the occurrences asserted by feature-a's file"
    );
    assert_eq!(
        call_facts(&edges(&state, &root_key, Some("feature-b"))),
        vec![
            ("load".to_string(), "b.stream.ts".to_string(), Some(1), None),
            ("load".to_string(), "b.stream.ts".to_string(), Some(2), None),
        ],
        "and the sibling path keeps its own"
    );

    // No occurrence is attributed to an anonymous callback name or to a path of
    // its own: the caller of every narrowed fact is still the outer arrow.
    for fact in call_facts(&edges(&state, &root_key, Some("feature-a"))) {
        assert_eq!(
            fact.0, "load",
            "an anonymous callback must never become the caller identity"
        );
    }
}
// ── Spread/arity evidence and the asserting-file boundary ────────────

#[test]
fn red_within_spread_and_arity_evidence_survive_the_narrowing() {
    let _serial = serialize();
    let root = tempfile::TempDir::new().unwrap();
    let callers = r#"export class SpreadStore {
    save(value: unknown): void { }

    exact = () => { this.save('x'); };
    expanded = () => { this.save(...['y']); };
}
"#;
    write_ts(root.path(), "feature-a/a-spread.store.ts", callers);
    write_ts(root.path(), "feature-b/b-spread.store.ts", callers);
    // A third path DECLARES the callee and asserts no call at all: a callee's
    // declaration file is never an asserting file.
    write_ts(
        root.path(),
        "feature-c/declaration.store.ts",
        r#"export class DeclarationStore {
    save(value: unknown): void { }
}
"#,
    );
    let state = state(&[]);
    let root_key = repo(root.path()).key();

    // Both paths assert BOTH shapes, and `call_facts` orders by
    // (caller, file, arity, spread): the two `exact` facts precede the two
    // `expanded` facts, because "exact" sorts before "expanded".
    assert_eq!(
        call_facts(&edges(&state, &root_key, None)),
        vec![
            (
                "exact".to_string(),
                "a-spread.store.ts".to_string(),
                Some(1),
                None
            ),
            (
                "exact".to_string(),
                "b-spread.store.ts".to_string(),
                Some(1),
                None
            ),
            (
                "expanded".to_string(),
                "a-spread.store.ts".to_string(),
                Some(1),
                Some(true)
            ),
            (
                "expanded".to_string(),
                "b-spread.store.ts".to_string(),
                Some(1),
                Some(true)
            ),
        ],
        "one written argument is either an exact arity or an expansion, in both paths"
    );

    assert_eq!(
        call_facts(&edges(&state, &root_key, Some("feature-a"))),
        vec![
            (
                "exact".to_string(),
                "a-spread.store.ts".to_string(),
                Some(1),
                None
            ),
            (
                "expanded".to_string(),
                "a-spread.store.ts".to_string(),
                Some(1),
                Some(true)
            ),
        ],
        "the exact call and the expansion both survive narrowing, qualifier intact"
    );

    assert!(
        call_facts(&edges(&state, &root_key, Some("feature-c"))).is_empty(),
        "the callee's declaration path authors no call fact of its own"
    );
}
