// src/tests/mcp/workspace_query_calls_arrows.rs
//
// End-to-end `workspace_query` behaviour for BOUND-ARROW callers.
//
// This is the query-surface acceptance of the arrow callable-identity feature:
// the caller is the arrow's written binding name (never the enclosing class,
// never a synthetic anonymous id), the asserting file is the CALLER's file, and
// the fact reaches `reverse_edges` for `domain = "builtin"`,
// `entity_type = "Method"` through the REAL production path:
//
//   TypeScript source -> candidate discovery -> resolve_file_path_checked ->
//   compile_file_ir_focused -> arrow callable scope -> CoreOp::Call ->
//   semantic projection -> WorkspaceIndex -> WSC-004 scope -> rerun of the
//   original query
//
// CBM is disabled in every test (`config.cbm.enabled = false`), so the
// discovery provider can contribute candidate PATHS only and can never author a
// relationship.
//
//   RED-ARROW27 — property-arrow caller is queryable (same-file target).
//   RED-ARROW28 — cross-file: caller in file A, target declaration in file B.
//   RED-ARROW29 — caller inside a configured additional root stays in scope.
//   RED-ARROW30 — same-name arrow callers in several in-scope paths keep
//                 distinct provenance, and an out-of-scope repository is
//                 excluded by the WSC-004 root scope (this revision has no
//                 `withinPath` argument: root scope + occurrence provenance is
//                 the narrowing surface it offers).
//   plus the spread qualifier on an ARROW caller at the response surface.

use std::path::{Path, PathBuf};

/// A state with CBM disabled so discovery falls back to the deterministic
/// filesystem provider (candidate PATHS only — never relationships).
fn state(additional: &[PathBuf]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}

/// Run a `reverse_edges` query for a `builtin / Method` name through the
/// production hydration cycle.
fn reverse_edges(
    state: &crate::mcp::McpState,
    name: &str,
    root: &Path,
) -> (serde_json::Value, super::super::hydration::HydrationReport) {
    let (result, _, _, report) = super::run_query_with_hydration(
        state,
        "reverse_edges",
        name,
        Some(&root.to_string_lossy()),
        |index| {
            let value =
                serde_json::to_value(index.reverse_edges_by_identity("builtin", "Method", name))
                    .unwrap_or_default();
            let count = value.as_array().map_or(0, Vec::len);
            (value, count)
        },
    );
    (result, report)
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

/// `(caller name, asserting file NAME, argc, spread)` for every call edge.
///
/// The asserting file is reduced to its final path component so the assertions
/// stay independent of the platform's canonical path form while still proving
/// WHICH file authored the fact. `None` for `spread` means the qualifier key was
/// ABSENT, which is how an exact call renders.
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

/// `(caller name, asserting file NAME)` only — the identity/provenance view.
fn callers(value: &serde_json::Value) -> Vec<(String, String)> {
    call_facts(value)
        .into_iter()
        .map(|(caller, file, _, _)| (caller, file))
        .collect()
}

// ── RED-ARROW27: a property arrow is a real caller ───────────────────

#[test]
fn red_arrow27_property_arrow_caller_is_queryable() {
    let root = tempfile::TempDir::new().unwrap();
    write_ts(
        root.path(),
        "handler.store.ts",
        r#"export class HandlerStore {
    save(value: unknown): void { }
    load = () => { this.save('x'); };
}
"#,
    );
    let state = state(&[]);

    let (result, report) = reverse_edges(&state, "save", root.path());

    assert!(report.discovery_completed);
    assert_eq!(
        report.discovery_provider, "filesystem",
        "no CBM relationship may author this fact"
    );
    assert_eq!(
        callers(&result),
        vec![("load".to_string(), "handler.store.ts".to_string())],
        "the arrow's binding name is the caller and its own file asserts it: {result}"
    );
    assert_eq!(
        call_facts(&result),
        vec![(
            "load".to_string(),
            "handler.store.ts".to_string(),
            Some(1),
            None
        )],
        "the written arity travels with the fact: {result}"
    );
}

// ── RED-ARROW28: cross-file arrow caller ────────────────────────────

#[test]
fn red_arrow28_cross_file_arrow_caller_is_attributed_to_its_own_file() {
    let root = tempfile::TempDir::new().unwrap();
    write_ts(
        root.path(),
        "feature/order.service.ts",
        r#"export class OrderService {
    save(value: unknown): void { }
}
"#,
    );
    write_ts(
        root.path(),
        "feature/consumer.component.ts",
        r#"export class ConsumerComponent {
    load = () => { save('x'); };
}
"#,
    );
    let state = state(&[]);

    let (result, report) = reverse_edges(&state, "save", root.path());

    assert!(report.discovery_completed);
    assert!(
        report.candidates_compiled > 0,
        "hydration must compile the discovered candidates (discovered={}, compiled={})",
        report.candidates_discovered,
        report.candidates_compiled
    );
    assert_eq!(
        callers(&result),
        vec![("load".to_string(), "consumer.component.ts".to_string())],
        "the CALLER's file asserts the fact, never the callee's file: {result}"
    );
}

// ── RED-ARROW29: additional root ────────────────────────────────────

#[test]
fn red_arrow29_arrow_caller_in_an_additional_root_stays_in_scope() {
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    write_ts(
        primary.path(),
        "order.service.ts",
        r#"export class OrderService {
    save(value: unknown): void { }
}
"#,
    );
    write_ts(
        additional.path(),
        "remote.component.ts",
        r#"export class RemoteComponent {
    load = () => { save('x'); };
}
"#,
    );
    let state = state(&[additional.path().to_path_buf()]);

    let (result, report) = reverse_edges(&state, "save", primary.path());

    assert!(report.discovery_completed);
    assert_eq!(
        callers(&result),
        vec![("load".to_string(), "remote.component.ts".to_string())],
        "a configured additional root belongs to the queried workspace: {result}"
    );
}

// ── RED-ARROW30: same name, several paths, scoped provenance ────────

#[test]
fn red_arrow30_same_name_arrow_callers_keep_distinct_scoped_provenance() {
    let primary = tempfile::TempDir::new().unwrap();
    let outside = tempfile::TempDir::new().unwrap();
    write_ts(
        primary.path(),
        "feature-a/a.component.ts",
        r#"export class AComponent {
    load = () => { save('a'); };
}
"#,
    );
    write_ts(
        primary.path(),
        "feature-b/b.component.ts",
        r#"export class BComponent {
    load = () => { save('b'); };
}
"#,
    );
    // A repository that is NOT the queried root and is NOT a configured
    // additional root. Its fact is authored first, so the scoped query proves
    // FILTERING rather than absence.
    write_ts(
        outside.path(),
        "other.component.ts",
        r#"export class OtherComponent {
    load = () => { save('c'); };
}
"#,
    );

    let state = state(&[]);
    let (authored, _) = reverse_edges(&state, "save", outside.path());
    assert_eq!(
        callers(&authored).len(),
        1,
        "the out-of-scope repository's fact must exist before the scoped query: {authored}"
    );

    let (scoped, _) = reverse_edges(&state, "save", primary.path());

    assert_eq!(
        callers(&scoped),
        vec![
            ("load".to_string(), "a.component.ts".to_string()),
            ("load".to_string(), "b.component.ts".to_string()),
        ],
        "both in-scope occurrences survive as the SAME identity with distinct \
         provenance, and the out-of-scope repository is excluded: {scoped}"
    );
    assert_eq!(
        call_facts(&scoped).len(),
        2,
        "identical name+arity facts from different files are distinct occurrences"
    );
}

// ── The spread qualifier on an ARROW caller at the response surface ──

#[test]
fn arrow_caller_spread_evidence_reaches_the_query_surface() {
    let root = tempfile::TempDir::new().unwrap();
    write_ts(
        root.path(),
        "spread.store.ts",
        r#"export class SpreadStore {
    save(value: unknown): void { }
    exact = () => { this.save('x'); };
    expanded = () => { this.save(...['x']); };
}
"#,
    );
    let state = state(&[]);

    let (result, _) = reverse_edges(&state, "save", root.path());

    assert_eq!(
        call_facts(&result),
        vec![
            (
                "exact".to_string(),
                "spread.store.ts".to_string(),
                Some(1),
                None
            ),
            (
                "expanded".to_string(),
                "spread.store.ts".to_string(),
                Some(1),
                Some(true)
            ),
        ],
        "one written argument can be an exact arity or an expansion: the \
         qualifier must survive on an arrow caller: {result}"
    );
}
