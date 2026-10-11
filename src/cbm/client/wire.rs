//! Typed requests and normalization for the pinned CBM JSON contract.

use super::{CbmClient, CbmError, QueryRows, decoder, observations};
use serde_json::Value;

impl CbmClient {
    // ── Response parsing ───────────────────────────────────────

    /// CBM wraps all tool responses in MCP content array format.
    /// The actual data is a JSON string inside `result.content[0].text`.
    /// This helper extracts and parses that inner JSON string.
    pub(super) fn parse_cbm_response(&self, response: &Value) -> Result<Value, CbmError> {
        let content = response["content"]
            .as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| CbmError::ParseError("missing content array".into()))?;
        let text = content["text"]
            .as_str()
            .ok_or_else(|| CbmError::ParseError("missing text field in content".into()))?;
        serde_json::from_str(text)
            .map_err(|e| CbmError::ParseError(format!("inner JSON parse: {e}")))
    }

    // ── Typed wrappers ──────────────────────────────────────────

    /// Search the CBM knowledge graph by name pattern and optional label filter.
    ///
    /// CBM params: `name_pattern`, `label`, `file_pattern`, `project`, `limit`, `offset`
    /// Build the `search_graph` tool arguments.
    ///
    /// Pure helper so tests can pin the exact request shape sent to CBM.
    /// `label` is optional — omitting it searches across all node labels
    /// (Class, Method, Function, Enum, ...).
    pub(crate) fn build_search_graph_args(
        name_pattern: &str,
        project: &str,
        label: Option<&str>,
    ) -> Value {
        let mut args =
            serde_json::json!({"name_pattern": name_pattern, "project": project, "format": "json"});
        if let Some(l) = label {
            args["label"] = serde_json::Value::String(l.to_string());
        }
        args
    }

    pub fn search_graph(
        &mut self,
        name_pattern: &str,
        project: &str,
        label: Option<&str>,
    ) -> Result<Vec<Value>, CbmError> {
        let args = Self::build_search_graph_args(name_pattern, project, label);
        let inner = self.collect("search_graph", args)?;
        let rows = decoder::search(&inner)?;
        observations::ProviderObservation::page(&inner).require_page("search_graph", rows.len())?;
        Ok(rows)
    }

    /// Trace call paths in the CBM knowledge graph.
    ///
    /// CBM params: `function_name`, `direction` (inbound|outbound|both), `depth`, `project`
    ///
    /// Grouped legs describe distance from the root, not deeper parent edges.
    pub fn trace_path(
        &mut self,
        function_name: &str,
        direction: &str,
        project: &str,
        depth: Option<usize>,
    ) -> Result<Vec<Value>, CbmError> {
        let mut args = serde_json::json!({"function_name": function_name, "direction": direction, "project": project, "format": "json"});
        if let Some(d) = depth {
            args["depth"] = serde_json::Value::Number(serde_json::Number::from(d));
        }
        let inner = self.collect("trace_path", args)?;
        let edges = decoder::trace(&inner, function_name, direction)?;
        let observation = observations::ProviderObservation::page(&inner);
        observation.require_page("trace_path", 0)?;
        for leg in ["callers", "callees"] {
            if let Some(groups) = inner[leg]["groups"].as_array() {
                let count: usize = groups
                    .iter()
                    .map(|g| g["rows"].as_array().map_or(0, Vec::len))
                    .sum();
                observations::ProviderObservation::page(&inner[leg])
                    .require_page("trace_leg", count)?;
                if inner[format!("{leg}_total")].as_u64() != Some(count as u64)
                    || inner[format!("{leg}_total_relation")] != "eq"
                {
                    return Err(CbmError::ToolError {
                        tool: "trace_path".into(),
                        message: "directional trace completeness unknown or partial".into(),
                    });
                }
            }
        }
        Ok(edges)
    }

    /// Get architecture overview from CBM.
    pub fn get_architecture(&mut self, project: &str) -> Result<Value, CbmError> {
        let r = self.call_tool(
            "get_architecture",
            serde_json::json!({
                "project": project, "format": "json", "aspects": ["packages", "boundaries"]
            }),
        )?;
        let inner = self.parse_cbm_response(&r)?;
        observations::ProviderObservation::page(&inner).require_page("get_architecture", 0)?;
        decoder::architecture(inner)
    }

    /// Execute a Cypher-like query against the CBM knowledge graph.
    ///
    /// Returns the full wire table as [`QueryRows`] — both `columns` and
    /// `rows`. CBM wraps results in `{columns, rows, total}`; the echoed
    /// `columns` carry the semantic projection (e.g. a verbatim `type(r)`
    /// marker) and MUST NOT be discarded by callers that interpret shapes.
    pub fn query_graph(&mut self, cypher: &str, project: &str) -> Result<QueryRows, CbmError> {
        let inner = self.collect(
            "query_graph",
            serde_json::json!({"query":cypher,"project":project,"format":"json"}),
        )?;
        let mut table = decoder::table(&inner, "columns")?;
        table.observation = observations::ProviderObservation::page(&inner);
        table
            .observation
            .require_page("query_graph", table.rows.len())?;
        Ok(table)
    }

    /// Get symbol importance (caller count) from CBM via Cypher query.
    ///
    /// CBM has no dedicated `get_symbol_importance` tool, but `in_degree`
    /// on function nodes provides the same information.
    pub fn get_symbol_importance(
        &mut self,
        project: &str,
        min_degree: Option<usize>,
    ) -> Result<Vec<Value>, CbmError> {
        let min = min_degree.unwrap_or(1);
        let cypher = format!(
            "MATCH (f:Function) WHERE f.in_degree >= {} RETURN f.name, f.file_path, f.in_degree, f.out_degree ORDER BY f.in_degree DESC",
            min
        );
        let table = self.query_graph(&cypher, project)?;
        Ok(table
            .rows
            .into_iter()
            .map(|row| {
                let name = row
                    .first()
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let file = row
                    .get(1)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let in_degree = row
                    .get(2)
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let out_degree = row
                    .get(3)
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                serde_json::json!({
                    "name": name,
                    "file": file,
                    "in_degree": in_degree as u64,
                    "out_degree": out_degree as u64,
                    // Normalized score for blending. Capped at 1.0: CBM graphs
                    // with in_degree > 100 (hot symbols) would exceed the
                    // documented [0.0, 1.0] importance contract.
                    "importance": (in_degree / 100.0).min(1.0),
                })
            })
            .collect())
    }

    /// Get dead code candidates from CBM via Cypher query.
    ///
    /// CBM has no dedicated `get_dead_code` tool, but functions with
    /// `in_degree = 0` and `is_entry_point = false` are dead code.
    pub fn get_dead_code(&mut self, project: &str) -> Result<Vec<Value>, CbmError> {
        let cypher = "MATCH (f:Function) WHERE f.in_degree = 0 AND f.is_entry_point = false RETURN f.name, f.file_path".to_string();
        let table = self.query_graph(&cypher, project)?;
        Ok(table
            .rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "name": row.first().and_then(|v| v.as_str()).unwrap_or(""),
                    "file": row.get(1).and_then(|v| v.as_str()).unwrap_or(""),
                    "reason": "no callers",
                })
            })
            .collect())
    }

    /// Trigger indexing of a repository in CBM 0.8.1+.
    /// Uses `index_repository` with `repo_path` and `mode` parameters.
    pub fn index_repository(&mut self, repo_path: &str, mode: &str) -> Result<Value, CbmError> {
        self.call_tool(
            "index_repository",
            serde_json::json!({"repo_path": repo_path, "mode": mode}),
        )
    }
}
