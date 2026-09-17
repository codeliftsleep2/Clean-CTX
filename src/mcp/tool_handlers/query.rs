// src/mcp/tool_handlers/query.rs
//
// workspace_query handler — read-only MCP API over WorkspaceIndex.
//
// This handler exposes the existing WorkspaceIndex query methods as an MCP
// tool. It is a thin read boundary over the already-wired write lifecycle
// established in Phases A/B.
//
// Semantic hydration: for eligible query types, after the initial WorkspaceIndex
// query, one exhaustive candidate-file discovery pass runs across the primary
// root plus configured additional roots. Healthy CBM is preferred; roots CBM
// cannot cover fall back to literal filesystem discovery. Both providers
// supply ONLY candidate file paths. Those paths flow through resolve_file_path_checked →
// compile_file_ir_focused → Clean-CTX semantic extraction → WorkspaceIndex.
// CBM graph semantics (edge counts, relationship types, etc.) never enter the
// WorkspaceIndex. The original query reruns exactly once after hydration.
//
// Scope of one query — the ONE rule, computed once per call and shared by the
// initial answer and the post-hydration rerun:
//
//   effective scope = workspaceRoot + configured additional_roots   (WSC-004)
//                     ∩ optional `withinPath` narrowing
//
// `withinPath` (optional string) narrows an ALREADY authorized workspace to a
// file or directory subtree: a relative path resolves against `workspaceRoot`, an
// absolute path is used as declared, and the resolved path is canonicalized once
// per query. It is validated against the authorized root set BEFORE the index is
// consulted, so a path outside that set — or a `withinPath` supplied without
// `workspaceRoot` — is refused with `-32602` and can never become an implicit
// authorization root. Occurrence PROVENANCE is the boundary on every surface
// (`EntityRef.file` for entity occurrences, `StoredEdge::asserting_file` for edge
// occurrences), reachability/cycle queries apply it DURING traversal, and Model C
// identity is untouched. Omitting `withinPath` leaves every query exactly as it
// was. The rule itself lives in one place — `workspace::scope::WorkspaceScope` —
// and no handler parses a path of its own.
//
// Module layout (handler groups are separate files, mirroring how the index
// splits its query families):
//   query.rs             — dispatch, the hydration cycle, the shared scope rule.
//   query/diagnostics.rs — the sparse LLM-facing discovery-diagnostic projection.
//   query/entities.rs    — find_entities, entities_in_file.
//   query/edges.rs       — forward_edges, reverse_edges.
//   query/graph.rs       — transitive_dependencies, has_cycle.
//
// Hydration diagnostics reach the caller through ONE shared projection
// (`diagnostics::discovery_field`): expected state is omitted and only
// decision-relevant deviation is serialized, as an optional `discovery` object.
// The internal `HydrationReport` stays complete; nothing about discovery,
// provider selection, or the semantic answer changes here.

use crate::mcp::McpState;
use crate::protocol::send_response;
use serde_json::Value;

mod diagnostics;
mod edges;
mod entities;
mod graph;

pub(super) use diagnostics::discovery_field;

/// Handle `workspace_query` — read-only cross-file semantic queries.
pub(crate) fn handle_workspace_query(id: &Value, params: &Value, state: &McpState) {
    let args = &params["arguments"];
    let query_type = match args["type"].as_str() {
        Some(t) => t,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": "Missing required argument: 'type'. Supported values: \
                     find_entities, forward_edges, reverse_edges, entities_in_file, \
                     transitive_dependencies, has_cycle.".to_string()
                }
            }));
            return;
        }
    };

    match query_type {
        "find_entities" => entities::handle_find_entities(id, args, state),
        "forward_edges" => edges::handle_forward_edges(id, args, state),
        "reverse_edges" => edges::handle_reverse_edges(id, args, state),
        "entities_in_file" => entities::handle_entities_in_file(id, args, state),
        "transitive_dependencies" => graph::handle_transitive_dependencies(id, args, state),
        "has_cycle" => graph::handle_has_cycle(id, args, state),
        _ => {
            // ... error handling unchanged
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32602,
                    "message": format!(
                        "Unknown query type: '{}'. Supported values: find_entities, \
                         forward_edges, reverse_edges, entities_in_file, \
                         transitive_dependencies, has_cycle.",
                        query_type
                    )
                }
            }));
        }
    }
}

/// Run a WorkspaceIndex query with optional one-cycle semantic hydration.
///
/// 1. Run the initial query against current WorkspaceIndex (authoritative for
///    what Clean-CTX currently knows).
/// 2. If the query is hydration-eligible, perform ONE exhaustive discovery hydration pass.
/// 3. Rerun the original query exactly once.
/// 4. Return final results + the complete internal hydration report. The
///    LLM-facing projection of that report is `diagnostics::discovery_field`,
///    applied by the handler that serializes the response.
fn run_query_with_hydration<F>(
    state: &McpState,
    query_type: &str,
    query_name: &str,
    workspace_root: Option<&str>,
    query_fn: F,
) -> (Value, usize, super::hydration::HydrationReport)
where
    F: Fn(&crate::workspace::index::WorkspaceIndex) -> (Value, usize),
{
    // Step 1: Initial authoritative query.
    let (initial_results, initial_count) = {
        let idx = state.workspace_index_read();
        query_fn(&idx)
    };

    // Step 2: Evaluate hydration eligibility (independent of result count).
    let eligible = super::hydration::is_hydration_eligible(
        query_type,
        &serde_json::json!({ "name": query_name }),
    );
    if !eligible {
        return (
            initial_results,
            initial_count,
            super::hydration::HydrationReport::default(),
        );
    }

    // Step 3: One exhaustive discovery hydration pass.
    let hydration =
        super::hydration::hydrate_workspace_index(state, query_type, query_name, workspace_root);

    // Step 4: Rerun original query exactly once.
    let (final_results, final_count) = {
        let idx = state.workspace_index_read();
        query_fn(&idx)
    };

    (final_results, final_count, hydration)
}

/// Extract a required string argument from the arguments object.
fn required_str<'a>(args: &'a Value, name: &str) -> Option<&'a str> {
    args[name].as_str().filter(|s| !s.is_empty())
}

/// Extract an optional integer argument; returns `default` if missing.
fn optional_i32(args: &Value, name: &str, default: i32) -> i32 {
    args[name].as_i64().map(|v| v as i32).unwrap_or(default)
}

/// The effective scope of one `workspace_query` call: the caller's
/// `workspaceRoot` plus the configured `additional_roots` (WSC-004), intersected
/// with the optional `withinPath` narrowing. No handler parses a root or a path
/// itself — this is the only place that reads either argument, and
/// `WorkspaceScope` is the only place that decides admission.
///
/// `Ok(None)` — the caller declared no workspace root and no `withinPath`: the
/// query stays unscoped (the global/session view), exactly as before, and no
/// fallback root is ever substituted.
///
/// `Err(message)` — an invalid `withinPath` argument: it was supplied without an
/// explicit `workspaceRoot`, or it resolves outside the authorized root set. The
/// query is refused before the index is consulted, so no unrelated fact is ever
/// exposed and the narrowing can never widen the authorized workspace.
fn query_scope(
    state: &McpState,
    args: &Value,
) -> Result<Option<crate::workspace::scope::WorkspaceScope>, String> {
    let within_path = args["withinPath"]
        .as_str()
        .map(str::trim)
        .filter(|path| !path.is_empty());
    match within_path {
        Some(within_path) => crate::workspace::scope::WorkspaceScope::narrowed(
            args["workspaceRoot"].as_str(),
            &state.config.additional_roots,
            within_path,
        )
        .map(Some),
        None => Ok(crate::workspace::scope::WorkspaceScope::new(
            args["workspaceRoot"].as_str(),
            &state.config.additional_roots,
        )),
    }
}

/// Refuse one query whose `withinPath` argument is not authorized.
///
/// `-32602` (invalid params) with the reason `WorkspaceScope` produced: the
/// argument is a parameter of the request that cannot be satisfied, not an empty
/// answer about the workspace.
fn send_scope_rejection(id: &Value, message: String) {
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "error": {
            "code": -32602,
            "message": format!("Invalid 'withinPath' argument: {message}")
        }
    }));
}

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query.rs"]
mod tests;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query_2.rs"]
mod tests_hydration;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query_3.rs"]
mod tests_multi_root_hydration;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query_4.rs"]
mod tests_reverse_hydration;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query_5.rs"]
mod tests_hydration_completeness;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query_6.rs"]
mod tests_filesystem_hydration;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query_7.rs"]
mod tests_filesystem_safety;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query_8.rs"]
mod tests_hydration_discovery_cache;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/workspace_query_9.rs"]
mod tests_filesystem_discovery_cache;

// Native call facts (`SemanticRelation::Calls`) end-to-end: cross-file,
// cross-project, and the repeated-query discovery cache.
#[cfg(all(test, feature = "rust", feature = "csharp"))]
#[path = "../../tests/mcp/workspace_query_calls.rs"]
mod tests_native_calls;

// Native call facts for the additional language producers (TypeScript, Java)
// end-to-end: cross-file `reverse_edges` returns Clean-CTX-authored callers for
// either language, without any CBM-supplied call relationship.
#[cfg(all(test, feature = "rust", feature = "typescript", feature = "java"))]
#[path = "../../tests/mcp/workspace_query_calls_languages.rs"]
mod tests_native_calls_languages;

// BOUND-ARROW callers (TypeScript) end-to-end: the arrow's binding name is the
// caller, the CALLER's file asserts the fact, and WSC-004 scope holds.
#[cfg(all(test, feature = "rust", feature = "typescript"))]
#[path = "../../tests/mcp/workspace_query_calls_arrows.rs"]
mod tests_native_calls_arrows;
