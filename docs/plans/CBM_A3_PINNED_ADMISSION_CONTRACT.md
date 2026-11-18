# A3 pinned frontend admission contract

Authority: CBM `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`.
This record precedes A3 production implementation. No live provider was run.

## Source-established behavior

**PROVEN BY CURRENT CODE:** `src/main.c:3194–3379` obtains the secure endpoint,
acquires client exact-build cohort ownership, bootstraps/HELLO-connects the
account daemon, sets MCP session context, and only then enters the stdio frontend.
`src/daemon/service.c` validates build/version/ABI/cache compatibility internally.
A successful MCP response therefore traverses that already-established session;
Clean-CTX must not attempt a second account-level admission protocol.

**PROVEN BY CURRENT CODE:** `src/mcp/mcp.c:19915` handles `initialize`, calls
`detect_session`, and may start provider-owned auto-indexing. Direct `tools/call`
is accepted without an initialized-state guard, but does not perform this
initialize branch. Initialization is appropriate for the supported session path,
not a claim that the provider rejects every pre-initialize tool call.

**PROVEN BY CURRENT CODE / EXISTING TEST:** initialize accepts
`protocolVersion: "2024-11-05"`, echoing it when supported. The result contains
`protocolVersion`, `serverInfo` (name `codebase-memory-mcp`, packaging version),
`capabilities.tools` and `capabilities.prompts`, and optional instructions.
See `mcp.c:1402–1482` and `tests/test_mcp.c:mcp_initialize_response`.
Clean-CTX sends capabilities `{}` and clientInfo identifying Clean-CTX. It
requires the requested supported version, server identity/version and a tools
capability. It does not infer exact artifact provenance from packaging version.

**PROVEN BY CURRENT CODE:** notifications without IDs have no response; only
cancellation has special handling (`mcp.c:19893`). `notifications/initialized`
is accepted/ignored, not a readiness acknowledgement or admission gate.
Clean-CTX sends it for interoperability and does not wait for a reply.

**PROVEN BY CURRENT CODE:** `tools/list` with `{}` returns the entire tool
registry without `nextCursor`. Pagination applies only when a cursor member is
supplied (`mcp.c:1175–1219`). A3 uses the complete no-cursor form, not Phase B
query/page collection. An unexpected partial listing cannot establish this gate.

**PROVEN BY CURRENT CODE:** startup refusal can be a null-ID JSON-RPC error
`-32001` with a provider message, plus stderr (`main.c:1737–1771`). Some failures,
such as rejected session context, are stderr plus exit only. Correlated RPC
errors, malformed protocol, startup refusal and transport exit remain distinct.
The null-ID error does not identify a precise build/cache conflict kind; preserve
its cause without substring classification or automatic account repair.

## Minimum integration surface

| Required tool | Existing consumer |
| --- | --- |
| `index_repository` | Root/background/explicit/lazy indexing |
| `search_graph` | Typed search and proxy |
| `query_graph` | Typed graph queries and internal intelligence helpers |
| `trace_path` | Typed call tracing and proxy |
| `get_architecture` | Architecture projection and proxy |
| `list_projects` | Registered project discovery and proxy |
| `detect_changes` | Existing graph-cache invalidation/change detection |

These tools are in the pinned default registry. No newer optional tool becomes
mandatory. Tool presence/basic definition validation is not Phase B schema or
query-decoder compatibility proof.

## Intended adapter sequence and evidence boundaries

Transport constructed (not Available) → correlated initialize → validate result
→ send initialized notification → correlated full tools/list → validate minimum
surface → mark usable → permit bridge indexing and provider operations.
Failure retires the direct frontend through A2; no replacement/re-admission or
account mutation. Native optional-provider fallback remains available.
All startup responses share A1 envelope/ID/bounded-frame validation.

Exact artifact provenance remains an external A5/live acceptance requirement:
MCP initialize does not expose the daemon HELLO build fingerprint, ABI or cache
identity as structured fields. No runtime SHA equality can be invented here.
