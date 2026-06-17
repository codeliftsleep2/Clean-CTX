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
            "description": "**Primary CBM integration point.** Forwards a query to CBM, intercepts the raw ~5000-token structural response at the pipe level, compresses it down to ~1100 tokens, and returns the compressed result. Use this instead of calling CBM directly. Only available when codebase-memory-mcp is installed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cbm_tool": { "type": "string", "description": "CBM tool to call: 'graph_search', 'graph_query', 'graph_trace', 'get_architecture', 'get_symbol_importance', 'get_dead_code'. Default: 'graph_search'." },
                    "parameters": { "type": "object", "description": "Parameters to pass to the CBM tool. For graph_search: { 'query': string, 'project': string }. For graph_trace: { 'from': string, 'to': string, 'project': string }. For others: { 'project': string }." },
                    "query": { "type": "string", "description": "Shorthand: query to pass to CBM (used when parameters is not set)." },
                    "project": { "type": "string", "description": "Shorthand: CBM project name (used when parameters is not set)." }
                },
                "required": []
            }
        }),
    ]
}