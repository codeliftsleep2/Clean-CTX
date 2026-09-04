// src/cbm/wire.rs
//
// CBM wire adapter — owns all CBM response parsing and shape detection.
//
// This module is the explicit adapter boundary between CBM's external wire
// contracts and Clean-CTX's internal representations. It isolates every
// CBM-specific shape assumption behind a single module so that CBM wire
// evolution does not leak into the semantic substrate.
//
// Shape detection (NOT version sniffing): the adapter recognizes response
// shapes by their actual structure (`cols`/`rows`/`groups` presence) and
// parses accordingly. The production target is CBM 0.10.8; legacy shapes are
// tolerated where they have been observed, but no compatibility for pre-0.10.8
// CBM is added.

use serde_json::Value;
use std::collections::HashMap;

use crate::cbm::bridge::GraphNode;

// ── Search ─────────────────────────────────────────────────────────────

/// Parse a CBM `search_graph` response into Clean-CTX `GraphNode`s.
///
/// CBM 0.10.8 (`format="json"`) emits the tree/table model:
/// ```text
/// { total, count, cols: ["name","label","lines","in","out",...],
///   groups: [{ qn_prefix, file, rows: [[name,label,lines,in,out,...]] }],
///   has_more }
/// ```
/// The full qualified name is reconstructed as `qn_prefix + "." + name`.
///
/// Legacy shape (tolerated): `{ results: [{ name, qualified_name, label, file_path, ... }] }`.
pub fn parse_search_results(body: &Value) -> Option<Vec<GraphNode>> {
    if let Some(groups) = body.get("groups").and_then(|g| g.as_array()) {
        let cols = body.get("cols").and_then(|c| c.as_array())?;
        let name_idx = col_index(cols, "name")?;
        let label_idx = col_index(cols, "label")?;
        let file_from_group =
            !groups.is_empty() && groups[0].get("file").and_then(|f| f.as_str()).is_some();

        let mut nodes = Vec::new();
        for group in groups {
            let qn_prefix = group
                .get("qn_prefix")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            // In the 0.10.8 model, `file` is a per-group field.
            let group_file = if file_from_group {
                group.get("file").and_then(|v| v.as_str()).unwrap_or("")
            } else {
                ""
            };
            let rows = group.get("rows").and_then(|r| r.as_array())?;
            for row in rows {
                let row_arr = row.as_array()?;
                let name = row_str(row_arr, name_idx);
                let label = row_str(row_arr, label_idx);
                let fqn = if qn_prefix.is_empty() {
                    name.to_string()
                } else {
                    format!("{}.{}", qn_prefix, name)
                };
                let file = if file_from_group {
                    group_file.to_string()
                } else {
                    col_index(cols, "file")
                        .map(|i| row_str(row_arr, i).to_string())
                        .unwrap_or_default()
                };
                nodes.push(GraphNode {
                    id: fqn,
                    label: label.to_string(),
                    name: name.to_string(),
                    file,
                    properties: HashMap::new(),
                });
            }
        }
        Some(nodes)
    } else if let Some(results) = body.get("results").and_then(|r| r.as_array()) {
        // Legacy shape (tolerated).
        let mut nodes = Vec::new();
        for n in results {
            let name = n.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            let fqn = n
                .get("qualified_name")
                .and_then(|v| v.as_str())
                .unwrap_or(name);
            let label = n.get("label").and_then(|v| v.as_str()).unwrap_or_default();
            let file = n
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            nodes.push(GraphNode {
                id: fqn.to_string(),
                label: label.to_string(),
                name: name.to_string(),
                file: file.to_string(),
                properties: HashMap::new(),
            });
        }
        Some(nodes)
    } else {
        None
    }
}
fn col_index(cols: &[Value], name: &str) -> Option<usize> {
    cols.iter().position(|c| c.as_str() == Some(name))
}

fn row_str(row: &[Value], idx: usize) -> &str {
    row.get(idx).and_then(|v| v.as_str()).unwrap_or_default()
}

// ── Trace ──────────────────────────────────────────────────────────────

use crate::cbm::bridge::GraphEdge;

/// Parse a CBM `trace_path` response into Clean-CTX `GraphEdge`s.
///
/// CBM 0.10.8 emits `callers`/`callees` as table models:
/// ```text
/// { function, direction, mode?,
///   callees_total, callees: { cols, rows },
///   callers_total, callers: { cols, rows },
///   truncated?, next_cursor? }
/// ```
/// Each row carries `[name, qualified_name, hop, ...]` keyed by `cols`.
///
/// Legacy shape (tolerated): `callers`/`callees` as arrays of
/// `{ name, qualified_name, hop }`.
pub fn parse_trace(body: &Value, function_qn: &str) -> Vec<GraphEdge> {
    let has_tables = body.get("callees").and_then(Value::as_object).is_some()
        || body.get("callers").and_then(Value::as_object).is_some();

    if !has_tables {
        // Legacy array shape (tolerated). Delegate to the verified legacy
        // parser (`CbmClient::extract_trace_edges`), which pins hop=1-direct
        // only, exact-duplicate dedup, and qualified→name fallback.
        return crate::cbm::client::extract_trace_edges(body, function_qn)
            .into_iter()
            .filter_map(|v| {
                Some(GraphEdge {
                    from: v.get("from")?.as_str()?.to_string(),
                    to: v.get("to")?.as_str()?.to_string(),
                    label: v
                        .get("label")
                        .and_then(Value::as_str)
                        .unwrap_or("calls")
                        .to_string(),
                    properties: HashMap::new(),
                })
            })
            .collect();
    }

    let mut edges = Vec::new();

    if let Some(callees) = body.get("callees") {
        if let Some(table) = callees.as_object() {
            edges.extend(parse_trace_table(table, function_qn, true));
        }
    }

    if let Some(callers) = body.get("callers") {
        if let Some(table) = callers.as_object() {
            edges.extend(parse_trace_table(table, function_qn, false));
        }
    }

    edges
}

/// Parse one `callers`/`callees` table (`{ cols, rows }`) into edges.
fn parse_trace_table(
    table: &serde_json::Map<String, Value>,
    function_qn: &str,
    caller_to_callee: bool,
) -> Vec<GraphEdge> {
    let cols = table.get("cols").and_then(|c| c.as_array());
    let rows = table.get("rows").and_then(|r| r.as_array());
    let (cols, rows) = match (cols, rows) {
        (Some(c), Some(r)) => (c, r),
        _ => return Vec::new(),
    };

    let qn_idx = col_index(cols, "qualified_name").or_else(|| col_index(cols, "name"));
    let mut edges = Vec::new();
    for row in rows {
        let row_arr = match row.as_array() {
            Some(a) => a,
            None => continue,
        };
        let qn = qn_idx.map(|i| row_str(row_arr, i)).unwrap_or_default();
        if qn.is_empty() {
            continue;
        }
        let (from, to) = if caller_to_callee {
            (function_qn.to_string(), qn.to_string())
        } else {
            (qn.to_string(), function_qn.to_string())
        };
        edges.push(GraphEdge {
            from,
            to,
            label: "calls".into(),
            properties: HashMap::new(),
        });
    }
    edges
}

// ── Architecture ───────────────────────────────────────────────────────

use crate::cbm::bridge::{ArchitectureDependency, ArchitectureModule, ArchitectureOverview};

/// Parse a CBM `get_architecture` response into Clean-CTX's architecture view.
///
/// CBM 0.10.8 emits each section as a `{ cols, rows }` tree model. The
/// `packages` and `boundaries` sections are only present when requested via
/// `aspects` (e.g. `aspects: ["packages","boundaries"]` or `["all"]`).
///
/// Legacy shape (tolerated): flat `packages[]` / `boundaries[]` arrays.
pub fn parse_architecture(body: &Value) -> Option<ArchitectureOverview> {
    let modules = parse_packages_section(body)?;
    // `boundaries` may be absent in small graphs / compact summaries — that
    // is an empty dependency set, not a parse failure (legacy-tolerance
    // contract, pinned by regression tests).
    let dependencies = parse_boundaries_section(body).unwrap_or_default();
    Some(ArchitectureOverview {
        modules,
        dependencies,
    })
}

/// Extract modules from the `packages` section (table or legacy array).
fn parse_packages_section(body: &Value) -> Option<Vec<ArchitectureModule>> {
    let section = body.get("packages")?;
    if let Some(table) = section.as_object() {
        let cols = table.get("cols").and_then(|c| c.as_array())?;
        let rows = table.get("rows").and_then(|r| r.as_array())?;
        let name_idx = col_index(cols, "name")?;
        let nodes_idx = col_index(cols, "nodes").or_else(|| col_index(cols, "node_count"));
        let mut modules = Vec::new();
        for row in rows {
            let row_arr = row.as_array()?;
            let name = row_str(row_arr, name_idx);
            let file_count = nodes_idx
                .and_then(|i| row_arr.get(i))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;
            modules.push(ArchitectureModule {
                name: name.to_string(),
                path: String::new(),
                file_count,
            });
        }
        Some(modules)
    } else if let Some(arr) = section.as_array() {
        // Legacy shape (tolerated).
        let mut modules = Vec::new();
        for p in arr {
            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            let file_count = p.get("node_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            modules.push(ArchitectureModule {
                name: name.to_string(),
                path: String::new(),
                file_count,
            });
        }
        Some(modules)
    } else {
        None
    }
}

/// Extract dependencies from the `boundaries` section (table or legacy array).
fn parse_boundaries_section(body: &Value) -> Option<Vec<ArchitectureDependency>> {
    let section = body.get("boundaries")?;
    if let Some(table) = section.as_object() {
        let cols = table.get("cols").and_then(|c| c.as_array())?;
        let rows = table.get("rows").and_then(|r| r.as_array())?;
        let from_idx = col_index(cols, "from")?;
        let to_idx = col_index(cols, "to")?;
        let mut deps = Vec::new();
        for row in rows {
            let row_arr = row.as_array()?;
            let from = row_str(row_arr, from_idx);
            let to = row_str(row_arr, to_idx);
            deps.push(ArchitectureDependency {
                from: from.to_string(),
                to: to.to_string(),
                kind: "calls".into(),
            });
        }
        Some(deps)
    } else if let Some(arr) = section.as_array() {
        // Legacy shape (tolerated).
        let mut deps = Vec::new();
        for b in arr {
            let from = b.get("from").and_then(|v| v.as_str()).unwrap_or_default();
            let to = b.get("to").and_then(|v| v.as_str()).unwrap_or_default();
            deps.push(ArchitectureDependency {
                from: from.to_string(),
                to: to.to_string(),
                kind: "calls".into(),
            });
        }
        Some(deps)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_search_results_0108_tree_model() {
        let body = serde_json::json!({
            "total": 1,
            "count": 1,
            "cols": ["name", "label", "lines", "in", "out"],
            "groups": [{
                "qn_prefix": "proj.src.app",
                "file": "src/app/Foo.java",
                "rows": [["Foo", "Class", "0", 0, 0]]
            }],
            "has_more": false
        });
        let nodes = parse_search_results(&body).expect("should parse 0.10.8 shape");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "proj.src.app.Foo");
        assert_eq!(nodes[0].name, "Foo");
        assert_eq!(nodes[0].file, "src/app/Foo.java");
    }

    #[test]
    fn parse_search_results_legacy_shape() {
        let body = serde_json::json!({
            "results": [{
                "name": "Foo",
                "qualified_name": "proj.Foo",
                "label": "Class",
                "file_path": "src/Foo.rs"
            }]
        });
        let nodes = parse_search_results(&body).expect("should parse legacy shape");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "proj.Foo");
        assert_eq!(nodes[0].file, "src/Foo.rs");
    }

    #[test]
    fn parse_search_results_unknown_shape_returns_none() {
        assert!(parse_search_results(&serde_json::json!({"foo": "bar"})).is_none());
    }

    #[test]
    fn parse_trace_0108_table_model() {
        let body = serde_json::json!({
            "function": "proj.main",
            "direction": "both",
            "callees": {
                "cols": ["name", "qualified_name", "hop"],
                "rows": [["helper", "proj.helper", 1]]
            },
            "callers": {
                "cols": ["name", "qualified_name", "hop"],
                "rows": [["entry", "proj.entry", 1]]
            }
        });
        let edges = parse_trace(&body, "proj.main");
        assert_eq!(edges.len(), 2);
        assert!(
            edges
                .iter()
                .any(|e| e.from == "proj.main" && e.to == "proj.helper")
        );
        assert!(
            edges
                .iter()
                .any(|e| e.from == "proj.entry" && e.to == "proj.main")
        );
    }

    #[test]
    fn parse_trace_legacy_arrays() {
        let body = serde_json::json!({
            "function": "proj.main",
            "direction": "both",
            "callees": [{ "name": "helper", "qualified_name": "proj.helper", "hop": 1 }],
            "callers": [{ "name": "entry", "qualified_name": "proj.entry", "hop": 1 }]
        });
        let edges = parse_trace(&body, "proj.main");
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn parse_architecture_0108_sections() {
        let body = serde_json::json!({
            "packages": {
                "cols": ["name", "nodes", "fan_in", "fan_out"],
                "rows": [["cbm", 81, 0, 0]]
            },
            "boundaries": {
                "cols": ["from", "to", "calls"],
                "rows": [["tests", "cbm", 42]]
            }
        });
        let ov = parse_architecture(&body).expect("should parse 0.10.8 sections");
        assert_eq!(ov.modules.len(), 1);
        assert_eq!(ov.modules[0].name, "cbm");
        assert_eq!(ov.modules[0].file_count, 81);
        assert_eq!(ov.dependencies.len(), 1);
        assert_eq!(ov.dependencies[0].from, "tests");
    }

    #[test]
    fn parse_architecture_legacy_arrays() {
        let body = serde_json::json!({
            "packages": [{ "name": "cbm", "node_count": 81, "fan_in": 0, "fan_out": 0 }],
            "boundaries": [{ "from": "tests", "to": "cbm", "call_count": 42 }]
        });
        let ov = parse_architecture(&body).expect("should parse legacy arrays");
        assert_eq!(ov.modules.len(), 1);
        assert_eq!(ov.dependencies.len(), 1);
    }
}
