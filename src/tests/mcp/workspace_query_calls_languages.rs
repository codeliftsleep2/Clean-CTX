// src/tests/mcp/workspace_query_calls_languages.rs
//
// End-to-end `workspace_query` behaviour for the TypeScript and Java native
// call producers.
//
// This is the local analogue of `workspace_query_calls.rs` (which proves the
// same path for C#): the acceptance criterion for a native `Calls` producer is
// that `reverse_edges` for `domain = "builtin"`, `entity_type = "Method"`
// returns Clean-CTX-authored callers — never CBM-supplied call relationships.
//
// Every assertion is made against the real production path: filesystem
// candidate discovery → `resolve_file_path_checked` → `compile_file_ir_focused`
// → Clean-CTX semantic extraction → WorkspaceIndex → rerun of the original
// query. CBM is disabled, so the discovery provider can contribute candidate
// PATHS only and can never contribute a relationship.

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

/// `(caller name, asserting file NAME, argc)` for every returned call edge.
///
/// The asserting file is reduced to its final path component so the assertions
/// stay independent of the platform's canonical path form while still proving
/// WHICH file authored the fact.
fn call_facts(value: &serde_json::Value) -> Vec<(String, String, Option<u64>)> {
    let mut facts: Vec<(String, String, Option<u64>)> = value
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
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    facts.sort();
    facts
}

/// A TypeScript callee declaration file (the declaration need not be in the
/// caller's file for the caller's fact to be authored).
fn write_ts_callee(root: &Path) {
    std::fs::write(
        root.join("OrderService.ts"),
        concat!(
            "export class OrderService {\n",
            "  orderBy(a: number): void { }\n",
            "  orderByRange(a: number, b: number): void { }\n",
            "}\n",
        ),
    )
    .unwrap();
}

/// A TypeScript caller file: one method per observed arity.
fn write_ts_caller(root: &Path) {
    std::fs::write(
        root.join("CallerService.ts"),
        concat!(
            "export class CallerService {\n",
            "  Process(): void { this.orderBy(1); }\n",
            "  Project(): void { this.orderBy(1, 2); }\n",
            "}\n",
        ),
    )
    .unwrap();
}

/// A Java callee declaration file.
fn write_java_callee(root: &Path) {
    std::fs::write(
        root.join("OrderService.java"),
        concat!(
            "package demo;\n",
            "public class OrderService {\n",
            "    public void orderBy(int a) { }\n",
            "    public void orderByRange(int a, int b) { }\n",
            "}\n",
        ),
    )
    .unwrap();
}

/// A Java caller file: one method per observed arity.
fn write_java_caller(root: &Path) {
    std::fs::write(
        root.join("CallerService.java"),
        concat!(
            "package demo;\n",
            "public class CallerService {\n",
            "    public void Process() { this.orderBy(1); }\n",
            "    public void Project() { this.orderBy(1, 2); }\n",
            "}\n",
        ),
    )
    .unwrap();
}

// ── TypeScript: cross-file reverse_edges ────────────────────────────

#[test]
fn typescript_cross_file_reverse_edges_returns_the_native_caller() {
    let root = tempfile::TempDir::new().unwrap();
    write_ts_callee(root.path());
    write_ts_caller(root.path());
    let state = state(&[]);

    let (result, report) = reverse_edges(&state, "orderBy", root.path());

    assert!(report.hydration_attempted);
    assert!(report.discovery_completed);
    assert_eq!(report.discovery_provider, "filesystem");
    assert_eq!(
        call_facts(&result),
        vec![
            (
                "Process".to_string(),
                "CallerService.ts".to_string(),
                Some(1)
            ),
            (
                "Project".to_string(),
                "CallerService.ts".to_string(),
                Some(2)
            ),
        ],
        "the TypeScript caller's method-level identity and arity must come from \
         Clean-CTX compilation: {result}"
    );
}

// ── Java: cross-file reverse_edges ──────────────────────────────────

#[test]
fn java_cross_file_reverse_edges_returns_the_native_caller() {
    let root = tempfile::TempDir::new().unwrap();
    write_java_callee(root.path());
    write_java_caller(root.path());
    let state = state(&[]);

    let (result, report) = reverse_edges(&state, "orderBy", root.path());

    assert!(report.hydration_attempted);
    assert!(report.discovery_completed);
    assert_eq!(report.discovery_provider, "filesystem");
    assert_eq!(
        call_facts(&result),
        vec![
            (
                "Process".to_string(),
                "CallerService.java".to_string(),
                Some(1)
            ),
            (
                "Project".to_string(),
                "CallerService.java".to_string(),
                Some(2)
            ),
        ],
        "the Java caller's method-level identity and arity must come from \
         Clean-CTX compilation: {result}"
    );
}

/// A TypeScript caller that asserts one EXACT and one SPREAD call to the same
/// callee with the SAME written argument count — the two facts occurrence
/// identity must never collapse, and the two shapes the response must label
/// differently.
fn write_ts_spread_caller(root: &Path) {
    std::fs::write(
        root.join("SpreadService.ts"),
        concat!(
            "export class SpreadService {\n",
            "  Exact(): void { save(a); }\n",
            "  Expanded(): void { save(...args); }\n",
            "}\n",
        ),
    )
    .unwrap();
}

/// `(caller name, explicit_arg_count, the serialized spread qualifier)` for
/// every returned call edge — the evidence exactly as a client receives it.
///
/// `None` for the qualifier means the key was ABSENT, which is how an exact
/// call renders: the qualifier is published only where the written count is
/// not an arity.
fn call_evidence_shape(value: &serde_json::Value) -> Vec<(String, Option<u64>, Option<bool>)> {
    let mut shapes: Vec<(String, Option<u64>, Option<bool>)> = value
        .as_array()
        .map(|edges| {
            edges
                .iter()
                .filter(|edge| edge["relation"] == "Calls")
                .map(|edge| {
                    (
                        edge["subject"]["name"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        edge["call_evidence"]["explicit_arg_count"].as_u64(),
                        edge["call_evidence"]["has_spread"].as_bool(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    shapes.sort();
    shapes
}

// ── RED-SPREAD8: the response labels non-exact evidence ─────────────

#[test]
fn red_spread8_response_labels_spread_evidence() {
    let root = tempfile::TempDir::new().unwrap();
    write_ts_spread_caller(root.path());
    let state = state(&[]);

    let (result, report) = reverse_edges(&state, "save", root.path());
    assert!(report.discovery_completed);

    assert_eq!(
        call_evidence_shape(&result),
        vec![
            ("Exact".to_string(), Some(1), None),
            ("Expanded".to_string(), Some(1), Some(true)),
        ],
        "a spread call must publish the qualifier and an exact call must not, \
         so the written count can never be read as an exact arity: {result}"
    );

    // Both facts survive as separate occurrences authored by one file.
    assert_eq!(
        call_facts(&result),
        vec![
            ("Exact".to_string(), "SpreadService.ts".to_string(), Some(1)),
            (
                "Expanded".to_string(),
                "SpreadService.ts".to_string(),
                Some(1)
            ),
        ],
        "the same written count must remain two occurrences: {result}"
    );
}

// ── both languages in one workspace ─────────────────────────────────

#[test]
fn typescript_and_java_callers_coexist_in_one_workspace() {
    let root = tempfile::TempDir::new().unwrap();
    write_ts_callee(root.path());
    write_ts_caller(root.path());
    write_java_callee(root.path());
    write_java_caller(root.path());
    let state = state(&[]);

    let (result, _) = reverse_edges(&state, "orderBy", root.path());

    assert_eq!(
        call_facts(&result),
        vec![
            (
                "Process".to_string(),
                "CallerService.java".to_string(),
                Some(1)
            ),
            (
                "Process".to_string(),
                "CallerService.ts".to_string(),
                Some(1)
            ),
            (
                "Project".to_string(),
                "CallerService.java".to_string(),
                Some(2)
            ),
            (
                "Project".to_string(),
                "CallerService.ts".to_string(),
                Some(2)
            ),
        ],
        "both language producers must author independent facts for one callee: {result}"
    );
}
