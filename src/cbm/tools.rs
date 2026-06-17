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
    ]
}