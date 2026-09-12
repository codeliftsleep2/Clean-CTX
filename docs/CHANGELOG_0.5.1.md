---

## [0.5.1] - 2026-08-29

### Fixed

- **`graph_query` node-result data fidelity — projected `file_path` no longer erased.** `GraphBridge::convert_query_rows()` read only column 0 on node-shaped Cypher projections (`RETURN f.name, f.file_path`) and built `GraphNode { id, name, label: "", file: "", properties: {} }`, silently discarding the `f.file_path` cell CBM was returning. The 0.5.0 structured-output migration made the defect visible (empty `structuredContent.nodes[*].file`) against an `outputSchema` that already requires `file`. Node-shaped projections now resolve from the echoed column metadata exactly like the relationship-shaped path: column 0 keeps its legacy `id`/`name` role, a `file_path` projection populates `GraphNode.file`, a clearly recognizable `label` projection populates `GraphNode.label` (legacy empty default otherwise), and every other projected column is preserved in `GraphNode.properties` keyed by the echoed column text with its projected JSON value verbatim. The relationship/edge-shaped conversion path is unchanged. **Bug/conformance fix only — no MCP response schema change.** (`src/cbm/bridge.rs`, `src/tests/cbm/query_wire.rs`, `src/tests/cbm/handlers.rs`)

### Cache

- **Query-cache namespace bumped `cypher:` -> `cypher2:`** so results cached before this fix (always-empty `file` cells) can never be served after upgrade. SQLite schema unchanged; pre-0.5.1 `cypher:` rows remain in the database untouched (never read, never migrated). (`src/cbm/bridge.rs`)

