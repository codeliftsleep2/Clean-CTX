//! Typed graph queries against the CBM knowledge graph.
//!
//! `query_graph` (active project and project-explicit), `search`, `trace_path`,
//! and the architecture/intel projections. Split out of `bridge.rs` along the
//! boundary the file already had — no behavior changed.

use super::*;
use crate::cbm::client::CbmError;
use std::collections::HashMap;

impl GraphBridge {
    /// Per-symbol importance scores derived from CBM caller counts.
    ///
    /// Errors (CBM unavailable / failed / rejected the query) propagate as
    /// [`CbmError::Err`] — an `Ok(map)` is always a valid, complete result.
    pub fn get_symbol_importance_mut(
        &mut self,
    ) -> Result<HashMap<String, SymbolImportance>, CbmError> {
        let key = "symbol_importance".to_string();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .map_err(|e| CbmError::ParseError(format!("cached symbol_importance: {e}")));
        }
        let project = self.project_str();
        let symbols = self.query(move |c| c.get_symbol_importance(&project, Some(1)))?;
        let map: HashMap<_, _> = symbols
            .iter()
            .filter_map(|e| {
                Some((
                    e["name"].as_str()?.to_string(),
                    SymbolImportance {
                        symbol: e["name"].as_str()?.to_string(),
                        score: e["importance"].as_f64().unwrap_or(0.0),
                        file: e["file"].as_str().unwrap_or("").to_string(),
                    },
                ))
            })
            .collect();
        self.cache_insert(&key, &map);
        Ok(map)
    }

    /// Blast radius at depth 1: files containing direct callers of a symbol.
    ///
    /// H-01/M-04 fix: Replaced invalid `search_graph` name_pattern with valid Cypher
    /// query_graph call. Previous code passed `"depends_on:{sym}"` as a name_pattern
    /// regex, which CBM treated literally and returned zero matches.
    ///
    /// AUDIT FIX (F1): the WHERE clause previously referenced an undeclared
    /// variable (`m.name`) — live CBM fail-opens on invalid WHERE clauses and
    /// returned EVERY CALLS edge in the project as the "blast radius".
    ///
    /// `_depth` is accepted for API compatibility; CBM's single-hop CALLS
    /// match is depth 1.
    pub fn get_blast_radius(
        &mut self,
        symbol: &str,
        _depth: usize,
    ) -> Result<Vec<String>, CbmError> {
        let key = format!("blast:{symbol}");
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .map_err(|e| CbmError::ParseError(format!("cached blast radius: {e}")));
        }
        let escaped = symbol.replace('\'', "\\'");
        let cypher = format!(
            "MATCH (caller:Function)-[:CALLS]->(f:Function) WHERE f.name = '{escaped}' RETURN caller.name, caller.file_path"
        );
        let project = self.project_str();
        let table = self.query(move |c| c.query_graph(&cypher, &project))?;
        let files: Vec<_> = table
            .rows
            .iter()
            .filter_map(|r| r.get(1).and_then(|v| v.as_str().map(String::from)))
            .collect();
        self.cache_insert(&key, &files);
        Ok(files)
    }

    /// Dead-code candidates: `Function` and `Method` nodes with no callers
    /// that are not entry points.
    ///
    /// AUDIT FIX (F3): only `(f:Function)` was scanned, so dead class
    /// methods — the majority of TS/C#/Java symbols — were invisible.
    /// CBM 0.8.1's Cypher dialect has no verified UNION support here, so
    /// both labels are queried separately and merged (functions first).
    ///
    /// An `Ok(vec![])` means the graph genuinely has no dead candidates;
    /// failures propagate as [`CbmError::Err`].
    pub fn get_dead_code(&mut self) -> Result<Vec<DeadCodeEntry>, CbmError> {
        const DEAD_CODE_CYPHER: fn(&str) -> String = |label| {
            format!(
                "MATCH (n:{label}) WHERE n.in_degree = 0 AND n.is_entry_point = false \
                     RETURN n.name, n.file_path"
            )
        };
        let key = "dead_code".to_string();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .map_err(|e| CbmError::ParseError(format!("cached dead_code: {e}")));
        }
        let project = self.project_str();
        let mut entries = Vec::new();
        for label in ["Function", "Method"] {
            let table = self.query({
                let project = project.clone();
                move |c| c.query_graph(&DEAD_CODE_CYPHER(label), &project)
            })?;
            for row in &table.rows {
                if let Some(name) = row.first().and_then(|v| v.as_str()) {
                    entries.push(DeadCodeEntry {
                        symbol: name.to_string(),
                        file: row
                            .get(1)
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        reason: "no callers".to_string(),
                    });
                }
            }
        }
        self.cache_insert(&key, &entries);
        Ok(entries)
    }

    /// Get all CALLS edges from CBM's knowledge graph.
    /// Returns `(caller, callee)` pairs across all files.
    ///
    /// R-43b Phase 3: Consumed by `InferenceLayer::enrich_from_cbm()` to
    /// populate cross-file call edges (confidence = 0.75). Cached with TTL.
    pub fn get_call_edges(&mut self) -> Result<Vec<(String, String)>, CbmError> {
        let key = "call_edges".to_string();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .map_err(|e| CbmError::ParseError(format!("cached call_edges: {e}")));
        }
        let cypher = "MATCH (a:Function)-[:CALLS]->(b:Function) RETURN a.name, b.name".to_string();
        let project = self.project_str();
        let table = self.query(move |c| c.query_graph(&cypher, &project))?;
        let edges: Vec<(String, String)> = table
            .rows
            .iter()
            .filter_map(|r| {
                let from = r.first()?.as_str()?.to_string();
                let to = r.get(1)?.as_str()?.to_string();
                Some((from, to))
            })
            .collect();
        self.cache_insert(&key, &edges);
        Ok(edges)
    }

    // â”€â”€ CBM 0.8.1 LIMITATION (AUDIT F10) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    //
    // The former `get_dataflow_edges()` queried edge type `DATAFLOW`,
    // which does not exist in CBM 0.8.1's schema (edge types: USAGE,
    // DEFINES, CALLS, DECORATES, DEFINES_METHOD, TESTS, WRITES,
    // CONFIGURES, IMPORTS — verified via get_graph_schema on a live
    // index). It therefore always returned an empty result and was
    // removed rather than left as a silently-dead query path.
    //
    // Why USAGE/WRITES are NOT substitutes:
    //   - `USAGE` is reference tracking (`callee` property), not value
    //     flow — it has no read/write direction.
    //   - `WRITES` records field assignments only (e.g.
    //     `bridge.rs â†’ disk_cache`); there is no READS counterpart, so
    //     read-side dataflow can never be reconstructed from it.
    //
    // If a future CBM version introduces DATAFLOW (or directional
    // READS/WRITES dataflow edges), reintroduce the query and its
    // InferenceLayer consumption; the audit suite in
    // src/tests/cbm/graph_intel.rs pins the absence so the change is
    // noticed.

    /// Get the architecture overview from CBM (packages â†’ modules,
    /// boundaries â†’ dependencies).
    ///
    /// AUDIT FIX (F11): previously returned `None` for both "failed" and
    /// every other non-success path, conflating errors with missing data;
    /// failures now propagate as [`CbmError::Err`].
    pub fn get_architecture(&mut self) -> Result<ArchitectureOverview, CbmError> {
        let key = "architecture".to_string();
        if self.check_cache(&key) {
            // Cache hit is a successful query.
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .map_err(|e| CbmError::ParseError(format!("cached architecture: {e}")));
        }
        let project = self.project_str();
        let arch = self.query(move |c| c.get_architecture(&project))?;
        // CBM 0.8.1 emits packages/boundaries (not modules/
        // dependencies) — map through the verified wire-schema
        // parser instead of reading keys that never exist.
        let ov = parse_architecture_response(&arch);
        self.cache_insert(&key, &ov);
        Ok(ov)
    }

    /// Resolve a cross-language endpoint for a method name (Angular
    /// Ecosystem Deepening — NgRx effect â†’ .NET controller endpoint).
    ///
    /// Returns `None` when CBM is unavailable or no candidate matches.
    /// Uses the existing `query_graph` Cypher path with TTL in-memory +
    /// disk caching — **no new tool**.
    ///
    /// Returns the best-match as `"{ClassName}.{MethodName}"` (e.g.
    /// `"UserController.GetAll"`); empty/no-match â†’ `None`.
    ///
    /// The Cypher now joins the declaring `Class` node so the returned
    /// endpoint is **Controller-qualified** — the LLM can trace
    /// `Î¦effect:loadUsers$ â†’ UserController.GetAll` as a single semantic
    /// chain, not just a bare method name.
    pub fn resolve_cross_language_endpoint(&mut self, method_name: &str) -> Option<String> {
        // Graceful skip when CBM is unavailable — no error, no graph line.
        if !self.is_available() {
            return None;
        }
        let key = format!("endpoint:{method_name}");
        let project = self.project_str();
        if self.check_cache(&key) {
            return serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
        }

        let escaped = method_name.replace('\'', "\\'");
        // CBM 0.8.1 uses DEFINES_METHOD edges between Class and Method
        // result is `"{Class}.{Method}"`. Prefer Controller classes.
        // We call the client directly (via `self.query`) to get the raw
        // `{columns, rows}` — the generic `query_graph` flattens rows to
        // the first column, discarding the Controller class. We match on
        // the exact method name (case-sensitive) so we never return an
        // arbitrary fuzzy match.
        let cypher = format!(
            "MATCH (c:Class)-[:DEFINES_METHOD]->(m:Method) \
             WHERE m.name = '{escaped}' AND m.file_path =~ '.*\\.cs$' \
             RETURN m.name, c.name LIMIT 5"
        );
        let table = self.query(move |c| c.query_graph(&cypher, &project));
        let result: Option<String> = match table {
            Ok(table) => {
                // rows are Vec<Vec<Value>>: [m.name, c.name].
                // Prefer a row whose class name contains "Controller".
                let controller_hit = table.rows.iter().find_map(|row| {
                    let mname = row.first().and_then(|v| v.as_str())?;
                    let cname = row.get(1).and_then(|v| v.as_str())?;
                    if cname.contains("Controller") {
                        Some(format!("{cname}.{mname}"))
                    } else {
                        None
                    }
                });
                controller_hit.or_else(|| {
                    table.rows.first().and_then(|row| {
                        let mname = row.first().and_then(|v| v.as_str())?;
                        let cname = row.get(1).and_then(|v| v.as_str())?;
                        Some(format!("{cname}.{mname}"))
                    })
                })
            }
            Err(_) => None,
        };

        // Cache (write-through to disk when present).
        self.cache_insert(&key, &result);
        result
    }

    /// Execute a Cypher-like query against the ACTIVE project, served from the
    /// shared graph cache when a valid entry exists.
    pub fn query_graph(&mut self, cypher: &str) -> QueryResult {
        let key = format!("{QUERY_CACHE_KEY_NAMESPACE}:{cypher}");
        let project = self.project_str();
        self.query_graph_inner(&key, cypher, &project)
    }

    /// Execute a Cypher-like query against an EXPLICIT project, served from the
    /// same shared graph cache.
    ///
    /// The active-project key above (`cypher2:{cypher}`) is valid only while the
    /// bridge's active project IS the queried project: `set_project` /
    /// `set_workspace_root` clear the in-memory cache on every switch, and the
    /// disk store partitions by `(project_root, project_str)`. A caller that
    /// names a project explicitly — `cbm_proxy` resolves the request's project
    /// per call and deliberately does NOT promote it to the active project —
    /// cannot reuse that key, because two repositories that both declare the
    /// same symbol produce the same Cypher text, and one key would then serve
    /// one project's rows to the other.
    ///
    /// This entry point therefore carries the project in the key
    /// (`cypher2:{project}:{cypher}`) while reusing the *same* cache, TTL, disk
    /// write-through, and invalidation as every other graph query. Nothing is
    /// added: no second cache, no persistence, no lifecycle, and entries written
    /// here can never be read by the active-project key (or vice versa), so
    /// WSC-004's workspace isolation holds by construction.
    pub fn query_graph_scoped(&mut self, cypher: &str, project: &str) -> QueryResult {
        let key = format!("{QUERY_CACHE_KEY_NAMESPACE}:{project}:{cypher}");
        self.query_graph_inner(&key, cypher, project)
    }

    /// Shared body of the two `query_graph` entry points: one cache lookup, one
    /// typed transport call, one write-through.
    fn query_graph_inner(&mut self, key: &str, cypher: &str, project: &str) -> QueryResult {
        let q = cypher.to_string();
        let project = project.to_string();
        if self.check_cache(key) {
            // Cache hit is a successful query — clear any stale error.
            let result = serde_json::from_value(
                self.cache
                    .get(key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or(QueryResult {
                nodes: vec![],
                edges: vec![],
            });
            self.set_last_error(None);
            return result;
        }
        let result = self.query(move |c| c.query_graph(&q, &project));
        match result {
            Ok(table) => {
                self.set_last_error(None);
                // Wire contract (CBM-WIRE-002): column-shape conversion —
                // see `convert_query_rows`. The projection's echoed columns
                // decide the interpretation; row arity is never consulted
                // and arbitrary uniform triples are NEVER fabricated into
                // edges. (The original pre-fix implementation read only
                // column 0 while `edges` stayed a literal empty vec, so
                // every relationship-shaped Cypher collapsed to
                // "N node(s), 0 edge(s)" while the raw proxy path surfaced
                // the very same rows intact.)
                let r = convert_query_rows(&table.columns, &table.rows);
                self.cache_insert(key, &r);
                r
            }
            Err(e) => {
                self.set_last_error(Some(e));
                QueryResult {
                    nodes: vec![],
                    edges: vec![],
                }
            }
        }
    }

    pub fn search(&mut self, query: &str) -> Vec<GraphNode> {
        let key = format!("search:{query}");
        let project = self.project_str();
        let q = query.to_string();
        if self.check_cache(&key) {
            // Cache hit is a successful query — clear any stale error.
            let result = serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
            self.set_last_error(None);
            return result;
        }
        // H-01 fix: build a proper regex name_pattern from the query string.
        // If the query already contains regex metacharacters (., *, +, [, etc.)
        // use it as-is; otherwise wrap in .*...* for substring matching.
        let has_regex = q.chars().any(|c| {
            matches!(
                c,
                '.' | '*' | '+' | '[' | '(' | '\\' | '^' | '$' | '{' | '|'
            )
        });
        let name_pattern = if has_regex { q } else { format!(".*{q}.*") };
        // No label filter: CBM's name_pattern matches across ALL node labels.
        // Hardcoding `Some("Function")` here made every Class/Enum/Field/
        // Module invisible to graph_search (probe: the identical pattern
        // returns the GraphBridge Class unlabeled, zero hits labeled
        // Function). Explicit label overrides remain available on the raw
        // cbm_proxy path, which forwards CBM-native arguments unchanged.
        let result = self.query(move |c| c.search_graph(&name_pattern, &project, None));
        match result {
            Ok(nodes) => {
                self.set_last_error(None);
                let gn: Vec<GraphNode> = nodes.iter().filter_map(map_search_result).collect();
                self.cache_insert(&key, &gn);
                gn
            }
            Err(e) => {
                self.set_last_error(Some(e));
                vec![]
            }
        }
    }

    /// Trace a call path between two symbols. If `to` is empty, traces all
    /// direct edges around `from`. Otherwise post-filters to only include
    /// edges touching `to`.
    ///
    /// M-01 fix: previously ignored the `to` parameter — now properly filters
    /// results to only include edges touching the target symbol.
    ///
    /// Direction determination (wire-shape fix, 2026-08-24): with both
    /// endpoints supplied, OUTBOUND is attempted first — preserving pre-fix
    /// behavior exactly for outbound-reachable pairs. Only when the outbound
    /// attempt SUCCEEDS but yields no edge touching `to` do we fall back to
    /// INBOUND once, making inbound-only relationships (callee â† caller)
    /// discoverable. Errors are never swapped for the other direction: an
    /// `Err` means CBM failed (F11 invariant), not "no data".
    ///
    /// Wire-shape note: the typed client path previously parsed a phantom
    /// `edges` key and always collapsed to zero results; edges now come from
    /// CBM's real `callers` / `callees` arrays via
    /// `CbmClient::extract_trace_edges`.
    pub fn trace_path(&mut self, from: &str, to: &str) -> Vec<GraphEdge> {
        if from == to {
            // Trivially successful (no path needed) — clear stale error.
            self.set_last_error(None);
            return vec![];
        }
        let filter_target = if to.is_empty() {
            None
        } else {
            Some(to.to_string())
        };
        // No target â†’ single "both" sweep (pre-fix semantics preserved);
        // target present â†’ outbound-first with a single inbound fallback.
        let (first_direction, allow_fallback) = match filter_target {
            None => ("both", false),
            Some(_) => ("outbound", true),
        };
        let key = format!("trace:{from}:{to}");
        let project = self.project_str();
        if self.check_cache(&key) {
            // Cache hit is a successful query — clear any stale error.
            let result = serde_json::from_value(
                self.cache
                    .get(&key)
                    .expect("cache entry should exist after check_cache() returned true")
                    .value()
                    .data
                    .clone(),
            )
            .unwrap_or_default();
            self.set_last_error(None);
            return result;
        }

        // Attempt 1: preferred direction.
        let first = {
            let f = from.to_string();
            let project = project.clone();
            self.query(move |c| c.trace_path(&f, first_direction, &project, Some(3)))
        };
        let outcome = match first {
            Err(e) => Err(e),
            Ok(edges) => {
                let ge = filter_trace_edges(&edges, &filter_target);
                if !ge.is_empty() || !allow_fallback {
                    Ok(ge)
                } else {
                    // Attempt 2 (single fallback): outbound succeeded but no
                    // edge touches the target — the relationship may exist
                    // ONLY inbound (the target calls `from`). Errors here are
                    // propagated, never masked as empty data.
                    let f = from.to_string();
                    match self.query(move |c| c.trace_path(&f, "inbound", &project, Some(3))) {
                        Ok(edges) => Ok(filter_trace_edges(&edges, &filter_target)),
                        Err(e) => Err(e),
                    }
                }
            }
        };

        match outcome {
            Ok(ge) => {
                self.set_last_error(None);
                self.cache_insert(&key, &ge);
                ge
            }
            Err(e) => {
                self.set_last_error(Some(e));
                vec![]
            }
        }
    }
}
