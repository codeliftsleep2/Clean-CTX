// src/tests/mcp/workspace_query_calls.rs
//
// End-to-end `workspace_query` behaviour for native call facts.
//
// Covers:
//   RED-CALL13 — cross-file: the caller is in file A, the callee declaration is
//                in file B; `reverse_edges` returns the caller.
//   RED-CALL14 — cross-project: the caller lives in a configured additional
//                root and the relationship is still hydrated and retained.
//   RED-CALL28 — a repeated identical native call query inherits the existing
//                discovery cache: candidates_discovered/compiled are 0 while the
//                previously authored call facts remain queryable (WSC-003).
//   the local analogue of the live OrderBy acceptance: argc 1 and argc 2 facts
//                for the same callee name stay distinguishable and no CBM call
//                edge is ever imported (WSC-002 — only Clean-CTX compilation
//                authors `Calls` edges).
//
// Every assertion is made against the real production path: filesystem
// candidate discovery → `resolve_file_path_checked` → `compile_file_ir_focused`
// → Clean-CTX semantic extraction → WorkspaceIndex → rerun of the original
// query.

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
    let (result, _, report) = super::run_query_with_hydration(
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
    )
    .expect("hydration succeeds");
    (result, report)
}

/// `(caller name, asserting file NAME, argc)` for every returned call edge.
///
/// The asserting file is reduced to its final path component so the
/// assertions stay independent of the platform's canonical path form while
/// still proving WHICH file authored the fact.
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

fn write_callee_declaration(root: &Path, file: &str) {
    std::fs::write(
        root.join(file),
        "namespace Demo { public class OrderService { public void OrderBy(int a) { } \
         public void OrderBy(int a, int b) { } } }\n",
    )
    .unwrap();
}

fn write_caller(root: &Path, file: &str, class_name: &str) {
    std::fs::write(
        root.join(file),
        format!(
            "namespace Demo {{ public class {class_name} {{ \
             public void Process() {{ OrderBy(1); }} \
             public void Project() {{ OrderBy(1, 2); }} }} }}\n"
        ),
    )
    .unwrap();
}

// ── RED-CALL13: cross-file ─────────────────────────────────────────

#[test]
fn red_call13_cross_file_reverse_edges_returns_the_native_caller() {
    let root = tempfile::TempDir::new().unwrap();
    write_callee_declaration(root.path(), "OrderService.cs");
    write_caller(root.path(), "Caller.cs", "CallerService");
    let state = state(&[]);

    let (result, report) = reverse_edges(&state, "OrderBy", root.path());

    assert!(report.hydration_attempted);
    assert_eq!(report.discovery_status, "completed");
    assert_eq!(report.discovery_provider, "filesystem");
    assert_eq!(
        call_facts(&result),
        vec![
            ("Process".to_string(), "Caller.cs".to_string(), Some(1)),
            ("Project".to_string(), "Caller.cs".to_string(), Some(2)),
        ],
        "the caller's method-level identity and arity must come from Clean-CTX compilation: {result}"
    );
}

// ── RED-CALL14: cross-project (additional root) ─────────────────────

#[test]
fn red_call14_additional_root_caller_is_hydrated_and_retained() {
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    write_callee_declaration(primary.path(), "OrderService.cs");
    write_caller(additional.path(), "RemoteCaller.cs", "RemoteCallerService");
    let state = state(&[additional.path().to_path_buf()]);

    let (result, report) = reverse_edges(&state, "OrderBy", primary.path());

    assert_eq!(report.discovery_status, "completed");
    assert_eq!(
        call_facts(&result),
        vec![
            (
                "Process".to_string(),
                "RemoteCaller.cs".to_string(),
                Some(1)
            ),
            (
                "Project".to_string(),
                "RemoteCaller.cs".to_string(),
                Some(2)
            ),
        ],
        "a caller outside the primary root must still be authored by Clean-CTX: {result}"
    );
}

// ── RED-CALL28: repeated query inherits the discovery cache ──────────

#[test]
fn red_call28_repeated_native_call_query_skips_discovery() {
    let root = tempfile::TempDir::new().unwrap();
    write_callee_declaration(root.path(), "OrderService.cs");
    write_caller(root.path(), "Caller.cs", "CallerService");
    let state = state(&[]);

    let (first, first_report) = reverse_edges(&state, "OrderBy", root.path());
    assert!(
        first_report.candidates_discovered > 0,
        "the first query must discover candidates (discovered={}, compiled={}, provider={})",
        first_report.candidates_discovered,
        first_report.candidates_compiled,
        first_report.discovery_provider
    );
    assert!(
        first_report.candidates_compiled > 0,
        "the first query must compile candidates (discovered={}, compiled={}, provider={})",
        first_report.candidates_discovered,
        first_report.candidates_compiled,
        first_report.discovery_provider
    );

    let (second, second_report) = reverse_edges(&state, "OrderBy", root.path());
    assert_eq!(
        second_report.candidates_discovered, 0,
        "a repeated identical query must not rediscover (compiled={}, provider={})",
        second_report.candidates_compiled, second_report.discovery_provider
    );
    assert_eq!(
        second_report.candidates_compiled, 0,
        "a repeated identical query must not recompile (discovered={}, provider={})",
        second_report.candidates_discovered, second_report.discovery_provider
    );
    assert_eq!(
        call_facts(&second),
        call_facts(&first),
        "previously authored call facts must remain queryable"
    );
}
