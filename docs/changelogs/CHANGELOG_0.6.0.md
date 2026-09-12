---

## [0.6.0] - 2026-08-31

### Added

#### Semantic architecture (Phases 1–6)

* **Typed `SemanticEdge`, `EntityRef`, and `SemanticRelation` model** (`src/layers/meta/semantic.rs`) — structured semantic relationship representation with provenance tracking per meta-layer.
* **`extract_semantic_edges()` on `MetaLayer` trait** — meta-layers now produce structured semantic edges alongside existing Phi text output. Default implementation returns an empty vector.
* **`InferenceLayer.semantic_edges` field** — `InferenceLayer` now carries both `inferred_edges` (CBM/structural/heuristic) and `semantic_edges` (meta-layer facts). `all_edges_for()` returns both collections.
* **`PassContext.semantic_edges` field** (`pipeline.rs`) — semantic edges flow through the compression pipeline into the inference layer.
* **Angular semantic edge extraction** — Angular, NgRx, Signals, and Routing meta-layers emit `SemanticEdge` objects: Injects, UsesService, HasSelector, HandlesAction, Dispatches, TriggersReducer, ProducesAction, HasStore, RouteMapsTo, GuardedBy, ResolvedBy, Defines, and more.
* **.NET semantic edge extraction** (`src/dotnet_meta/semantic.rs`) — Controller, Minimal API, and AutoMapper semantic relationships represented as typed edges.
* **Spring Boot semantic edge extraction** (`src/spring_meta/semantic.rs`) — REST endpoint mappings, service injections, and configuration relationships represented as typed edges. Class-level `@RequestMapping` prefix correctly combined with method-level mapping annotations.

#### WorkspaceIndex (Phases 4a–4c)

* **`WorkspaceIndex`** (`src/workspace/`) — framework-agnostic cross-file semantic index. Provides entity/edge lookup, forward/reverse traversal, transitive dependency resolution, and deduplication by semantic identity.
* **Cross-file resolution** — `WorkspaceIndex` replaces the role previously served by the deleted legacy graph modules for semantic cross-file queries.
* **Graph traversal** — `transitive_dependencies()` and cycle detection for DI and routing relationships.

#### Workspace query interface (Task C)

* **`workspace_query` MCP tool** (`src/mcp/tool_handlers/query.rs`) — read-only cross-file semantic query API over `WorkspaceIndex`. Supports `find_entities`, `forward_edges`, `reverse_edges`, `entities_in_file`, `transitive_dependencies`, and `has_cycle`. Uses the same `(domain, entity_type, name)` identity model. No changes to the write lifecycle, CBM module, or WorkspaceIndex internals.
* **Handler registration** — registered through the standard handler registry in `src/mcp/tool_handlers/registry.rs`, exposed via `tool_list()` in `src/mcp/tools.rs`.
* **Docs** — `docs/agent/tooling.md` updated with `workspace_query` entry in the Workspace Tools section.

#### MCP token-economics correction

* **Post-compression token-economics gate** — after the compressed/hybrid representation is rendered, its actual token cost is compared against the raw source. If the candidate costs more tokens than raw, `raw_passthrough` is selected.
* **Uniform fidelity coverage** — the post-compression check applies to Low, Medium, High, and Edit fidelity compressed/hybrid output.
* **Pre-compression heuristic retained** — the existing BPE-based preflight heuristic for Edit fidelity remains as an optimization hint only.
* **Genuinely cheaper compression preserved** — compression is still selected when the candidate representation saves tokens.

### Fixed

- **Token-economics gate scope** — the pre-compression heuristic previously only applied to Edit fidelity. The new post-compression verification applies uniformly to all fidelity levels. See `src/mcp/tool_handlers/core.rs` (`maybe_economics_fallback`), `src/mcp/token_economics.rs`.

### Removed

- **Legacy meta-layer graph infrastructure** — removed after semantic coverage was verified through `extract_semantic_edges()` and `WorkspaceIndex`. The following deleted modules had no remaining imports after migration:
  - `src/angular_meta/graph.rs` — `AngularGraph`, `AngularGraphBuilder`, `ClassEntry`, `ClassKind`
  - `src/angular_meta/graph_state.rs` — `AngularGraphHandle`
  - `src/dotnet_meta/graph.rs` — `.NET DI graph`
  - `src/dotnet_meta/graph_state.rs` — `DotnetGraphHandle`
  - `src/spring_meta/graph.rs` — `SpringGraph`
  - `src/spring_meta/graph_state.rs` — `SpringGraphHandle`
- **`graph_state` re-exports and McpState integration fields** for all three legacy graph types.
- **`workspace.rs` graph-building pass** — the post-bundling `GraphCollector`/graph-build step no longer exists.
- **`compress_workspace` graph footer emission** — `§ΦGRAPH` and `Φgraph:` marker lines are no longer emitted. Phi text markers (`Φcmp:`, `Φsvc:`, etc.) remain.

### Changed

- `MetaLayer` trait now has `extract_semantic_edges()` with default empty implementation. All existing meta-layers (Angular, NgRx, Signals, Routing, .NET, Spring Boot) implement it.
- `Docs/developer_documentation.md`, `ARCHITECTURE_OVERVIEW.md`, `ANGULAR_META_LAYER.md`, `DOTNET_META_LAYER.md` updated to reflect the current architecture.
- `Cargo.toml` version bumped to `0.6.0`.

### Tests

| Area | Count |
|------|------:|
| `src/tests/angular_meta/semantic.rs` — Angular semantic edge extraction | 12 tests |
| `src/tests/dotnet_meta/semantic.rs` — .NET semantic edge extraction | ✓ |
| `src/tests/spring_meta/semantic.rs` — Spring semantic edge extraction | ✓ |
| `src/tests/workspace/index.rs` — WorkspaceIndex resolution, dedup, traversal | 20+ tests |
| `src/tests/mcp/tool_handlers.rs` — token-economics regression | 9 tests |
| **Total** | **2,200+ tests passing** |

---

