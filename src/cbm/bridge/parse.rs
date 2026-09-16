//! CBM wire-shape parsing.
//!
//! Maps CBM 0.8.1 payloads onto Clean-CTX concepts: `search_graph` results,
//! `trace_path` edges, `query_graph` row tables, and the architecture overview.
//! Split out of `bridge.rs`; pure functions, no behavior changed.

use super::*;
use serde_json::Value;
use std::collections::HashMap;

/// Parse CBM 0.8.1's actual `get_architecture` wire schema into the
/// Clean-CTX view.
///
/// CBM does **not** emit `modules`/`dependencies` keys. The equivalent
/// data lives in (verified against live captures in
/// `src/tests/cbm/fixtures/arch_*.json`):
///
/// - `packages[]`   `{name, node_count, fan_in, fan_out}` â†’ modules.
///   `node_count` lands in `ArchitectureModule::file_count`, the closest
///   existing field; CBM packages carry no filesystem path, so `path`
///   stays empty.
/// - `boundaries[]` `{from, to, call_count}`              â†’ dependencies
///   (cross-package call edges; `kind` is always `"calls"`).
///
/// Key sets vary between projects/modes (small graphs omit `boundaries`,
/// large ones may omit `entry_points`) — missing keys deserialize to
/// empty vecs rather than erroring.
/// Map one CBM 0.8.1 `search_graph` result object onto [`GraphNode`].
///
/// CBM emits `qualified_name` / `file_path` — there is **no `id`** and **no
/// `file`** key (verified against live captures). The previous mapping
/// required `n["id"]`, so `filter_map`'s `?` silently dropped EVERY result
/// and wrapper searches were always empty regardless of the pattern.
pub(crate) fn map_search_result(n: &Value) -> Option<GraphNode> {
    let name = n["name"].as_str()?;
    Some(GraphNode {
        id: n["qualified_name"].as_str().unwrap_or(name).to_string(),
        label: n["label"].as_str().unwrap_or("").to_string(),
        name: name.to_string(),
        file: n["file_path"].as_str().unwrap_or("").to_string(),
        properties: HashMap::new(),
    })
}

/// M-01 post-filter predicate: does `edge` touch the requested target?
///
/// Boundary normalization contract (2026-08-24 fix): CBM identifies symbols
/// by their QUALIFIED names (`...src.cbm.bridge.GraphBridge.query_graph`)
/// while the API accepts bare names (`query_graph`). A target therefore
/// matches a canonical endpoint when
///
///   1. the endpoint EQUALS the target (caller passed the fully qualified
///      form), or
///   2. the endpoint's FINAL DOT SEGMENT equals the target (bare-name
///      segment match — the dot is CBM's qualified-name separator,
///      verified against live captures).
///
/// Nothing looser: a bare `to` name must RETAIN edges whose wire endpoints
/// are qualified, but partial/multi-segment targets match nothing.
/// `__file__` module pseudo-nodes are genuine relationships and are never
/// filtered here.
pub(crate) fn edge_touches_target(edge: &GraphEdge, target: &str) -> bool {
    fn endpoint_matches(endpoint: &str, target: &str) -> bool {
        if endpoint == target {
            return true;
        }
        endpoint.rsplit('.').next() == Some(target)
    }
    endpoint_matches(&edge.from, target) || endpoint_matches(&edge.to, target)
}

/// Convert normalized wire edges into [`GraphEdge`]s, applying the M-01
/// post-filter when a target is specified. Preserves CBM emission order.
pub(crate) fn filter_trace_edges(
    edges: &[Value],
    filter_target: &Option<String>,
) -> Vec<GraphEdge> {
    edges
        .iter()
        .filter_map(|e| {
            let ge = GraphEdge {
                from: e["from"].as_str()?.to_string(),
                to: e["to"].as_str()?.to_string(),
                label: e["label"].as_str().unwrap_or("").into(),
                properties: HashMap::new(),
            };
            match filter_target {
                Some(target) if !edge_touches_target(&ge, target) => None,
                _ => Some(ge),
            }
        })
        .collect()
}

/// Convert CBM `{columns, rows}` query results into a [`QueryResult`] using
/// the COLUMN-SHAPE convention (wire contract CBM-WIRE-002, verified against
/// verbatim live captures 2026-08-24):
///
/// - A projection is relationship-shaped IFF it contains exactly ONE column
///   echoing a literal `type(...)` expression (whitespace-tolerant:
///   `"type( r )"` matches; aliased `type(r) AS kind` echoes as `"kind"`
///   and is intentionally UNDETECTABLE — CBM erases the marker).
/// - Relationship-shaped projections become one [`GraphEdge`] per row:
///   endpoints are the FIRST and LAST non-type projected columns, the type
///   cell becomes `label`, and every other projected column maps into
///   `properties` keyed by its echoed column text with the actual projected
///   value preserved. Scrambled 5/6/N-column orders work purely from the
///   column metadata — arity and position are never consulted.
/// - No unaliased `type(...)` column â‡’ node fallback REGARDLESS of column
///   count. Arbitrary uniform triples (e.g. `[name, in_degree, out_degree]`)
///   must NEVER fabricate edges — the previous arity rule produced a fake
///   edge labelled `"10"` for exactly that shape.
///
/// Node-shaped projections resolve from the echoed column metadata exactly
/// like the relationship-shaped path: column 0 keeps its legacy `id`/`name`
/// role, a `file_path` projection populates `GraphNode.file`, a clearly
/// recognizable `label` projection populates `GraphNode.label` (the legacy
/// empty default applies when no convincing `label` column exists), and every
/// other projected column is preserved in `GraphNode.properties` keyed by the
/// echoed column text with its projected JSON value verbatim. 0.5.1 fix: the
/// `f.file_path` value CBM actually returns is no longer silently discarded.
/// Deliberately NOT done here (tracked as separate findings): node
/// deduplication and endpoint normalization. Endpoint/label cells are
/// extracted as strings (non-strings become empty); property cells keep
/// their projected JSON value verbatim. Rows are never skipped or reordered
/// (rows without a first cell keep the legacy blanket skip).
pub(crate) fn convert_query_rows(columns: &[String], rows: &[Vec<Value>]) -> QueryResult {
    if columns.len() >= 3 {
        if let Some(type_idx) = single_type_column(columns) {
            // Defensive: every row must line up with the echoed columns,
            // otherwise the projection metadata cannot be trusted.
            if rows.iter().all(|row| row.len() == columns.len()) {
                let endpoints: Vec<usize> = (0..columns.len()).filter(|&i| i != type_idx).collect();
                let from_idx = endpoints[0];
                let to_idx = endpoints[endpoints.len() - 1];
                let edges = rows
                    .iter()
                    .map(|row| {
                        let mut properties = HashMap::new();
                        for &prop_idx in &endpoints[1..endpoints.len() - 1] {
                            // Preserve the projected JSON value verbatim
                            // (wire cells are strings; native values would
                            // pass through unharmed).
                            properties.insert(columns[prop_idx].clone(), row[prop_idx].clone());
                        }
                        GraphEdge {
                            from: row[from_idx].as_str().unwrap_or_default().to_string(),
                            to: row[to_idx].as_str().unwrap_or_default().to_string(),
                            label: row[type_idx].as_str().unwrap_or_default().into(),
                            properties,
                        }
                    })
                    .collect();
                return QueryResult {
                    nodes: vec![],
                    edges,
                };
            }
        }
    }
    // ── Node-shaped projection fallback ─────────────────────────────
    // Column 0 keeps its legacy `id`/`name` role. The echoed columns now
    // additionally drive `file`/`label` population and property preservation
    // so projected node data CBM actually returned (e.g. `f.file_path`) is no
    // longer silently discarded. Relationship-shaped projections returned
    // above; unaliased `type(...)` handling is untouched.
    let file_idx = find_projection_column(columns, "file_path");
    let label_idx = find_projection_column(columns, "label");
    let nodes = rows
        .iter()
        .enumerate()
        .filter_map(|(row_idx, row)| row.first().map(|first| (row_idx, row, first)))
        .map(|(_row_idx, row, first)| {
            let name = first.as_str().unwrap_or("");
            let file = file_idx
                .and_then(|idx| row.get(idx))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let label = label_idx
                .and_then(|idx| row.get(idx))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let mut properties = HashMap::new();
            for (idx, column) in columns.iter().enumerate() {
                if idx == 0 || Some(idx) == file_idx || Some(idx) == label_idx {
                    continue;
                }
                if let Some(value) = row.get(idx) {
                    // Preserve the projected JSON value verbatim (mirrors the
                    // relationship-shaped property rule above).
                    properties.insert(column.clone(), value.clone());
                }
            }
            GraphNode {
                id: name.to_string(),
                label,
                name: name.to_string(),
                file,
                properties,
            }
        })
        .collect();
    QueryResult {
        nodes,
        edges: vec![],
    }
}

/// Locate THE relationship-type column of an echoed projection.
///
/// Matches a literal `type(...)` expression case-insensitively with inner
/// whitespace tolerated (CBM echoes `"type( r )"` verbatim). Returns `Some`
/// only when EXACTLY ONE such column exists — zero means node-shaped, and
/// more than one means the projection's semantics are ambiguous, so we
/// refuse to guess. An `AS` alias replaces the whole expression in the echo
/// (`type(r) AS rel_kind` â‡’ `"rel_kind"`), which by design makes aliased
/// relationship projections indistinguishable from ordinary scalars here.
fn single_type_column(columns: &[String]) -> Option<usize> {
    let hits: Vec<usize> = columns
        .iter()
        .enumerate()
        .filter(|(_, col)| {
            let flat: String = col.chars().filter(|c| !c.is_whitespace()).collect();
            let lower = flat.to_lowercase();
            lower.starts_with("type(") && lower.ends_with(')')
        })
        .map(|(i, _)| i)
        .collect();
    if hits.len() == 1 { Some(hits[0]) } else { None }
}

/// Locate a node-property projection column (e.g. `file_path` or `label`) in
/// an echoed `query_graph` column list.
///
/// CBM echoes RETURN expressions verbatim (a `f.file_path` projection echoes
/// as `"f.file_path"`), mirroring the `type(...)` echo handled by
/// [`single_type_column`]. A property `P` is identified by a column whose
/// whitespace-stripped text is exactly `P` or ends with `.P`. Only a SINGLE
/// conclusive match maps; zero matches keep the legacy empty default, and
/// several matches would be ambiguous (which node's value?) so we refuse to
/// guess. An `AS` alias replaces the whole expression in the echo
/// (`f.file_path AS path` echoes as `"path"`), so aliased properties are
/// intentionally undetectable — exactly like the aliased `type()` pin.
/// Column 0 is unconditionally the legacy `id`/`name` identity column and is
/// never repurposed as a property.
fn find_projection_column(columns: &[String], property: &str) -> Option<usize> {
    let hits: Vec<usize> = columns
        .iter()
        .enumerate()
        .filter(|(idx, column)| {
            if *idx == 0 {
                return false;
            }
            let flat: String = column.chars().filter(|c| !c.is_whitespace()).collect();
            flat == property
                || flat
                    .strip_suffix(property)
                    .is_some_and(|prefix| prefix.ends_with('.'))
        })
        .map(|(idx, _)| idx)
        .collect();
    match hits.as_slice() {
        [idx] => Some(*idx),
        _ => None,
    }
}

/// Cache-key namespace for `query_graph` results.
///
/// Bumped `cypher:` -> `cypher2:` in 0.5.1 so results cached before the
/// node-mapping fix (whose `file` cells were always empty) can never be
/// served again. The SQLite schema is unchanged; pre-0.5.1 `cypher:` rows
/// stay in the database untouched (never read, never migrated).
pub(crate) const QUERY_CACHE_KEY_NAMESPACE: &str = "cypher2";

pub(crate) fn parse_architecture_response(arch: &Value) -> ArchitectureOverview {
    let modules = arch["packages"]
        .as_array()
        .map(|ms| {
            ms.iter()
                .filter_map(|m| {
                    Some(ArchitectureModule {
                        name: m["name"].as_str()?.to_string(),
                        path: String::new(),
                        file_count: m["node_count"].as_u64().unwrap_or(0) as usize,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let dependencies = arch["boundaries"]
        .as_array()
        .map(|ds| {
            ds.iter()
                .filter_map(|d| {
                    Some(ArchitectureDependency {
                        from: d["from"].as_str()?.to_string(),
                        to: d["to"].as_str()?.to_string(),
                        kind: "calls".to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    ArchitectureOverview {
        modules,
        dependencies,
    }
}
