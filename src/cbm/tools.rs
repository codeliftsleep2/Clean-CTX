// src/cbm/tools.rs
//
// MCP tool definitions for CBM integration.
// Self-contained — returns tool definition JSON that gets appended to Clean-CTX's tool list.

/// Return the list of CBM-specific MCP tool definitions.
pub fn cbm_tool_list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "graph_search",
            "description": "Search the CBM knowledge graph by name/pattern. Returns matching symbols with their file locations. Only available when codebase-memory-mcp is installed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name_pattern": { "type": "string", "description": "CBM-native search query (symbol name, pattern, or regex)." },
                    "query": { "type": "string", "description": "Clean-CTX shorthand search query (symbol name, pattern, or natural language). Accepted alongside name_pattern." },
                    "project": { "type": "string", "description": "Optional CBM project name. Defaults to workspace root. Supplying a project changes the bridge's active project for subsequent structured-wrapper calls that omit a project." }
                },
                "required": ["query"]
            }
        }),
        serde_json::json!({
            "name": "graph_query",
            "description": "Execute a Cypher-like query against the CBM knowledge graph. Returns matching nodes and edges. Only available when codebase-memory-mcp is installed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Cypher-like query (e.g., 'MATCH (c:Class)-[:CALLS]->(m:Method)')." },
                    "project": { "type": "string", "description": "Optional CBM project name. Defaults to workspace root. Supplying a project changes the bridge's active project for subsequent structured-wrapper calls that omit a project." }
                },
                "required": ["query"]
            }
        }),
        serde_json::json!({
            "name": "graph_trace",
            "description": "Trace a path between two symbols in the CBM knowledge graph. Only available when codebase-memory-mcp is installed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from": { "type": "string", "description": "Source symbol name." },
                    "to": { "type": "string", "description": "Target symbol name." },
                    "project": { "type": "string", "description": "Optional CBM project name. Defaults to workspace root. Supplying a project changes the bridge's active project for subsequent structured-wrapper calls that omit a project." }
                },
                "required": ["from", "to"]
            }
        }),
        serde_json::json!({
            "name": "get_architecture",
            "description": "Get the project architecture overview from CBM — modules, components, and dependencies. Only available when codebase-memory-mcp is installed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Optional CBM project name. Defaults to workspace root. Supplying a project changes the bridge's active project for subsequent structured-wrapper calls that omit a project." }
                }
            }
        }),
        serde_json::json!({
            "name": "get_cbm_status",
            "description": "Check whether codebase-memory-mcp (CBM) is available. Returns 'available', 'degraded', or 'unavailable' with details.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        // ── Project discovery ─────────────────────────────────────
        serde_json::json!({
            "name": "list_projects",
            "description": "List all CBM-indexed projects. Returns each project's CBM identity, path, and status. Project-independent — no project parameter required. Use this to discover the authoritative CBM project slug when needed.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        // ── Repository indexing ────────────────────────────────────
        serde_json::json!({
            "name": "index_repository",
            "description": "Trigger CBM to index (or reindex) a repository. After a successful apply_edit, Clean-CTX automatically performs a synchronous fast reindex — manual index_repository is normally not needed. Use this explicitly when external edits (host write tool, shell, git operation) have modified the repository outside Clean-CTX. fast mode is appropriate for normal post-edit refreshes; full mode is available for explicit rebuild/recovery scenarios. This operation updates CBM's graph/index; it does not modify the repository's files or Git state.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Path to the repository to index." },
                    "mode": { "type": "string", "description": "Indexing mode: 'fast' (normal refresh, default) or 'full' (rebuild/recovery)." }
                },
                "required": ["repo_path"]
            }
        }),
        // ── Phase 2: Pipe-Level Interception Proxy ───────────────────
        serde_json::json!({
            "name": "cbm_proxy",
            "description": "**Primary CBM integration point.** Forwards a query to CBM, intercepts the raw ~5000-token structural response at the pipe level, compresses it down to ~1100 tokens, and returns the compressed result. `cbm_tool` must be a real CBM tool name: 'search_graph', 'query_graph', 'trace_path', 'get_architecture', 'list_projects', or 'index_repository'. Use this instead of calling CBM directly. Only available when codebase-memory-mcp is installed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cbm_tool": { "type": "string", "description": "CBM tool to call (must be a real CBM tool): 'search_graph', 'query_graph', 'trace_path', 'get_architecture', 'list_projects', 'index_repository'. Default: 'search_graph'. NOTE: get_symbol_importance/get_dead_code are implemented internally via query_graph and are NOT CBM tools." },
                    "parameters": { "type": "object", "description": "Parameters to pass to CBM using CBM-native names. search_graph: { 'name_pattern': string, 'project': string }. query_graph: { 'query': string, 'project': string }. trace_path: { 'function_name': string, 'direction': string (inbound|outbound|both), 'depth': int, 'project': string }. get_architecture: { 'project': string }. list_projects: {} (no parameters required). index_repository: { 'repo_path': string (required), 'mode': string ('fast' | 'full', default 'fast') }." },
                    "query": { "type": "string", "description": "Shorthand: query text passed to CBM (mapped to name_pattern for search_graph, query for query_graph)." },
                    "project": { "type": "string", "description": "Shorthand: CBM project name (used when parameters is not set). Project resolution is scoped to this proxy call and does not change the bridge's active project." }
                },
                "required": []
            }
        }),
    ]
}
