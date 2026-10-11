# B1 current-wire scope and consumed contract inventory

Authority: approved Phase B continuation, implementation plan slices B0–B5.
A0–A5 are frozen. B0 is locally committed as `a62dbc1`.

## Actual consumed paths

- Typed search: CbmClient::search_graph → GraphBridge search/project search and
  trace candidate resolution. Pinned mcp.c:4999–5082 emits cols plus groups of
  qn_prefix/file/rows, with explicit qn_rule. Normalize by column names and that
  declared rule; missing tables cannot become successful empty searches.
- Typed query: CbmClient::query_graph → graph_query and active derived Cypher
  consumers. Keep projected column order, aliases, duplicate rows and cell values.
  Missing columns/rows and malformed row shapes must return ParseError rather
  than manufacturing empty columns or filtering away malformed rows.
- Typed trace: CbmClient::trace_path → GraphBridge graph_trace. Pinned
  mcp.c:9290–9393 and 10045–10098 emit grouped directional legs, not legacy arrays.
  Normalize declared qn_prefix/name identities. Only hop-1 evidence supports
  caller/callee edges; do not fabricate parents for deeper BFS discoveries.
- Typed architecture: CbmClient::get_architecture → bridge overview consumes
  packages/boundaries. Explicit JSON and exactly those aspects prevent relying
  on the newer compact default. Pinned mcp.c:8394–8406 and 8446–8457 emit tables
  with packages name/nodes/fan_in/fan_out and boundaries from/to/calls. Preserve
  existing overview mapping; do not add an architecture feature inventory.
- list_projects is presently consumed through raw proxy. Caller format/default
  remains intentional; do not introduce an unused typed wrapper.
- index_repository lifecycle, index_status, coverage, detect_changes and result
  limitation/continuation state belong to B2/B3. No new index policy in B1.
- Raw proxy: A1/A4 validated envelope remains untouched; typed format selection
  must not override the raw caller's tree/default output.

All typed machine consumers select supported format=json. Index invocation has
its own fixed format and remains unchanged. ToolError versus RpcError and plain
text versus JSON soft-error bodies remain distinct.

## Prepared deterministic evidence

Fourteen tracked cbm_current_wire_ cases are registered under client B1 tests.
They execute normal A3 admission and real typed CbmClient methods against local
fake frontends, using the existing process-fixture pattern. The added support
branch handles only b1-prefixed modes; existing A0–A4 modes are unchanged.
Fixtures are source-derived, not falsely labeled live captures. Existing A5
live grouped search output corroborates that shape; no new live run is claimed.

## Implementation and verification

**PROVEN BY CURRENT CODE:** client/wire.rs selects JSON for typed search,
query, trace and architecture; architecture requests only packages/boundaries.
client/decoder.rs validates required tables, row widths and consumed cell types.
Grouped identity follows the producer's explicit qn_rule, including the empty
prefix case. Search retains file, lines and degree cells in the normalized
client result. Query retains columns and every cell without coercion or row
filtering. Trace validates requested legs and exposes only hop-1 edges through
the existing edge normalizer. Zero-count architecture sections may be omitted
by the pinned producer; present sections must be valid tables. Raw calls and
index invocation are unchanged.

**PROVEN BY EXISTING TEST:** on the unchanged B0 production implementation,
14 cbm_current_wire_ cases executed: 11 behavioral failures and 3 baseline
passes (no compilation or fixture failure). Regression-only files and
registrations were stashed before implementation, then restored. GREEN:
14 passed, 0 failed. Assertions and fixture payloads were preserved; formatting
and the request-log iterator implementation were adjusted for Clippy.

Authorized preservation checks passed:

| Filter | Passed |
| --- | ---: |
| cbm_b0_ | 1 |
| cbm::tests::cache_ownership | 3 |
| cbm::tests::query_wire::deterministic | 16 |
| cbm::tests::integration | 8 |
| cbm_a4_ | 24 |

Package Clippy initially identified a test-helper iterator warning. Corrective
runs replaced last/filter-next_back with rfind; final package Clippy passed with --all-targets --all-features -j 4 -- -D warnings. No lint suppression was added. No full suite or live provider ran.

## Production path and acceptance boundary

**PROVEN BY CURRENT CODE:** registered MCP dispatch in
src/mcp/tool_dispatch.rs routes graph_search, graph_query, graph_trace and
get_architecture to existing cbm handlers. Those use the production GraphBridge
query methods, which call these CbmClient wrappers through the existing query
owner. Normalized nodes/edges/query tables flow into the unchanged bridge
mappers and current-epoch cache; architecture tables flow into the unchanged
overview mapper. ParseError follows the existing query failure/degraded response
path. No alternate test-only production pipeline was introduced.

**NOT ESTABLISHED:** new live end-to-end B1 acceptance against the pinned
provider, Windows acceptance, and model-host consumption. The existing A5
search capture corroborates grouped search only. Source-derived fake frontend
checks prove the client boundary, not full live MCP reachability. B1 local
implementation is green; live acceptance remains open for B5's bundled operator
verification. The existing bridge intentionally exposes only its existing node
fields; richer client cells are not claimed to reach the public MCP response.

B1 is not overall Phase B completion. B2 must preserve limits/state that vector
return types currently discard. B3 still requires explicit choice of collection
budget/deadline policy or truthful one-page outcomes before implementation.
