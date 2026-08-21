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
                    "query": { "type": "string", "description": "Search query (symbol name, pattern, or natural language)." },
                    "project": { "type": "string", "description": "Optional CBM project name. Defaults to workspace root." }
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
                    "project": { "type": "string", "description": "Optional CBM project name. Defaults to workspace root." }
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
                    "project": { "type": "string", "description": "Optional CBM project name." }
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
                    "project": { "type": "string", "description": "Optional CBM project name." }
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
        // ── Phase 2: Pipe-Level Interception Proxy ───────────────────
        serde_json::json!({
            "name": "cbm_proxy",
            "description": "**Primary CBM integration point.** Forwards a query to CBM, intercepts the raw ~5000-token structural response at the pipe level, compresses it down to ~1100 tokens, and returns the compressed result. `cbm_tool` must be a real CBM tool name: 'search_graph', 'query_graph', 'trace_path', or 'get_architecture'. Use this instead of calling CBM directly. Only available when codebase-memory-mcp is installed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cbm_tool": { "type": "string", "description": "CBM tool to call (must be a real CBM tool): 'search_graph', 'query_graph', 'trace_path', 'get_architecture'. Default: 'search_graph'. NOTE: get_symbol_importance/get_dead_code are implemented internally via query_graph and are NOT CBM tools." },
                    "parameters": { "type": "object", "description": "Parameters to pass to CBM using CBM-native names. search_graph: { 'name_pattern': string, 'project': string }. query_graph: { 'query': string, 'project': string }. trace_path: { 'function_name': string, 'direction': string (inbound|outbound|both), 'depth': int, 'project': string }. get_architecture: { 'project': string }." },
                    "query": { "type": "string", "description": "Shorthand: query text passed to CBM (mapped to name_pattern for search_graph, query for query_graph)." },
                    "project": { "type": "string", "description": "Shorthand: CBM project name (used when parameters is not set)." }
                },
                "required": []
            }
        }),
    ]
}
